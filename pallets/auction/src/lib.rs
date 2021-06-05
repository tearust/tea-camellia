#![cfg_attr(not(feature = "std"), no_std)]

// Disable the following two lints since they originate from an external macro (namely decl_storage)
#![allow(clippy::string_lit_as_bytes)]
#![allow(clippy::unused_unit)]

use sp_std::{cmp::Ordering, prelude::*};
use frame_support::pallet_prelude::*;
use frame_support::{ensure};
use frame_support::traits::{
  Currency, ReservableCurrency,
  ExistenceRequirement::AllowDeath,
  // LockIdentifier, WithdrawReasons,
	Get,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use sp_runtime::{
  SaturatedConversion,
	traits::{Saturating, AtLeast32BitUnsigned, Bounded, MaybeSerializeDeserialize, Member, One},
	DispatchResult,
};

use log::{info};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod weights;
mod types;
mod functions;
pub use types::*;

// pub use weights::WeightInfo;

pub use auction::*;
use pallet_cml as cml;

// pub const AUCTION_ID: LockIdentifier = *b"_auction";

pub type BalanceOf<T> = 
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod auction {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + cml::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

    type Currency: ReservableCurrency<Self::AccountId>;

		/// The auction ID type.
		type AuctionId: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ Bounded
			+ codec::FullCodec;

    // type WeightInfo: WeightInfo;
    
    #[pallet::constant]
    type AuctionDealWindowBLock: Get<Self::BlockNumber>;

    #[pallet::constant]
    type BidDeposit: Get<BalanceOf<Self>>;

    #[pallet::constant]
    type MinPriceForBid: Get<BalanceOf<Self>>;

    #[pallet::constant]
    type AuctionOwnerPenaltyForEachBid: Get<BalanceOf<Self>>;
	}

	#[pallet::error]
	pub enum Error<T> {
    NotEnoughBalance,
    AuctionNotExist,
    InvalidBidPrice,
    NoNeedBid,
    /// The bid auction item belongs to extrinsic sender self
    BidSelfBelongs,
    AuctionOwnerInvalid,
    NotFoundBid,
    NotAllowQuitBid,
    NotInWindowBlock,

    LockableInvalid,

    NotAllowToAuction,
    BalanceReserveOrUnreserveError,
    BalanceTransferError,

    NotEnoughBalanceForPenalty,
	}

	#[pallet::event]
	// #[pallet::generate_deposit(fn deposit_event)]
	pub enum Event<T: Config> {
		/// A bid is placed. [auction_id, bidder, bidding_amount]
		Bid(T::AuctionId, T::AccountId, BalanceOf<T>),
	}

	#[pallet::storage]
	#[pallet::getter(fn auction_store)]
  pub type AuctionStore<T: Config> = StorageMap<
    _, 
    Twox64Concat, 
    T::AuctionId, 
    AuctionItem<T::AuctionId, T::AccountId, T::CmlId, BalanceOf<T>, T::BlockNumber>, 
    OptionQuery
  >;

	#[pallet::storage]
  #[pallet::getter(fn user_auction_store)]
  pub type UserAuctionStore<T: Config> = StorageMap<
    _,
    // Blake2_128Concat,
    Twox64Concat,
    T::AccountId,
    Vec<T::AuctionId>,
    OptionQuery
  >;

  #[pallet::type_value]
	pub fn DefaultAuctionId<T: Config>() -> T::AuctionId { <T::AuctionId>::saturated_from(1_u32) }
	#[pallet::storage]
	#[pallet::getter(fn auctions_index)]
  pub type LastAuctionId<T: Config> = StorageValue<
    _, 
    T::AuctionId, 
    ValueQuery,
    DefaultAuctionId<T>,
  >;

  #[pallet::storage]
  #[pallet::getter(fn bid_store)]
  pub type BidStore<T: Config> = StorageDoubleMap<
    _,
    Twox64Concat, T::AccountId,
    Twox64Concat, T::AuctionId,
    BidItem<T::AuctionId, T::AccountId, BalanceOf<T>, T::BlockNumber>,
    OptionQuery,
  >;

  #[pallet::storage]
  #[pallet::getter(fn auction_bid_store)]
  pub type AuctionBidStore<T: Config> = StorageMap<
    _,
    Twox64Concat, T::AuctionId,
    Vec<T::AccountId>,
  >;

  #[pallet::storage]
  #[pallet::getter(fn user_bid_store)]
  pub type UserBidStore<T: Config> = StorageMap<
    _,
    Twox64Concat, T::AccountId,
    Vec<T::AuctionId>,
  >;

  #[pallet::storage]
  #[pallet::getter(fn endblock_auction_store)]
  pub type EndblockAuctionStore<T: Config> = StorageMap<
    _,
    Twox64Concat, T::BlockNumber,
    Vec<T::AuctionId>,
  >;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		// fn on_initialize(now: T::BlockNumber) -> Weight {
			
		// }

		fn on_finalize(now: T::BlockNumber) {
      let b = now % T::AuctionDealWindowBLock::get();

      if b < <T::BlockNumber>::saturated_from(1_u64) {
        info!("[check_auction_in_block_window] start");
        let f = match Self::check_auction_in_block_window() {
          Ok(_) => true,
          Err(e) => {
            info!("on_finalize error => {:?}", e);
            false
          }
        };
        info!("[check_auction_in_block_window] => {:?}", f);
      }

		}
	}

	#[pallet::call]
  impl<T: Config> Pallet<T> {

    #[pallet::weight(10_000)]
    pub fn put_to_store(
      origin: OriginFor<T>,
      cml_id: T::CmlId,
      starting_price: BalanceOf<T>,
      buy_now_price: Option<BalanceOf<T>>,
    ) -> DispatchResult {
      let sender = ensure_signed(origin)?;

      let cml_item = cml::Pallet::<T>::get_cml_by_id(&cml_id)?;
      cml::Pallet::<T>::check_belongs(&cml_id, &sender)?;

      // check cml status
      ensure!(cml_item.status != cml::CmlStatus::Dead, Error::<T>::NotAllowToAuction);

      let auction_item = Self::new_auction_item(cml_id, sender.clone(), starting_price, buy_now_price);
      Self::add_auction_to_storage(auction_item, &sender);

      Ok(())
    }

    #[pallet::weight(10_000)]
    pub fn bid_for_auction(
      origin: OriginFor<T>,
      auction_id: T::AuctionId,
      price: BalanceOf<T>,
    ) -> DispatchResult {
      let sender = ensure_signed(origin)?;

      // validate balance
      let balance = <T as auction::Config>::Currency::free_balance(&sender);
      ensure!(balance >= price, Error::<T>::NotEnoughBalance);
      
      // check auction item
      let auction_item = AuctionStore::<T>::get(&auction_id).ok_or(Error::<T>::AuctionNotExist)?;
      let min_price = Self::get_min_bid_price(&auction_item, &sender)?;

      if let Some(bid_user) = &auction_item.bid_user {
        ensure!(&bid_user.cmp(&sender) != &Ordering::Equal, Error::<T>::NoNeedBid);
      }

      ensure!(min_price <= price, Error::<T>::InvalidBidPrice);
      ensure!(&auction_item.cml_owner.cmp(&sender) != &Ordering::Equal, Error::<T>::BidSelfBelongs);

      let cml_item = cml::Pallet::<T>::get_cml_by_id(&auction_item.cml_id)?;
      let deposit_bid_price = match cml_item.status {
        cml::CmlStatus::CmlLive => {
          let total_price = price.saturating_add(T::BidDeposit::get());
          ensure!(balance > total_price, Error::<T>::NotEnoughBalance);
          Some(T::BidDeposit::get())
        },
        _ => None,
      };
      
      let current_block = frame_system::Pallet::<T>::block_number();
      let maybe_bid_item = BidStore::<T>::get(&sender, &auction_id);
      if let Some(bid_item) = maybe_bid_item {
        // increase price
        let new_price = bid_item.price.saturating_add(price);
        BidStore::<T>::mutate(&sender, &auction_id, |maybe_item| {
          if let Some(ref mut item) = maybe_item {
            item.price = new_price;
            item.updated_at = current_block;

          }
        });
      }
      else {
        // new bid
        if let Some(deposit_price) = deposit_bid_price.clone() {
          Self::reserve(&sender, deposit_price)?;
        }
        let item = Self::new_bid_item(auction_item.id, sender.clone(), price, deposit_bid_price);
        BidStore::<T>::insert(sender.clone(), auction_item.id, item);
        AuctionBidStore::<T>::mutate(auction_item.id, |maybe_list| {
          if let Some(ref mut list) = maybe_list {
            list.push(sender.clone());
          }
          else {
            *maybe_list = Some(vec![sender.clone()]);
          }
        });

        UserBidStore::<T>::mutate(&sender, |maybe_list| {
          if let Some(ref mut list) = maybe_list {
            list.push(auction_item.id);
          }
          else {
            *maybe_list = Some(vec![auction_item.id]);
          }
        });

      }
      Self::update_bid_price_for_auction_item(&auction_id, sender.clone());

      // lock bid price
      Self::reserve(&sender, price)?;

      if let Some(buy_now_price) = auction_item.buy_now_price {
        if price >= buy_now_price {    
          Self::complete_auction(&auction_item, &sender)?;
        }
      }

      Ok(())
    }

    #[pallet::weight(100_000)]
    pub fn remove_from_store(
      origin: OriginFor<T>,
      auction_id: T::AuctionId,
    ) -> DispatchResult {
      let sender = ensure_signed(origin)?;

      let auction_item = AuctionStore::<T>::get(&auction_id).ok_or(Error::<T>::AuctionNotExist)?;
      ensure!(&sender.cmp(&auction_item.cml_owner) == &Ordering::Equal, Error::<T>::AuctionOwnerInvalid);

      let maybe_list = AuctionBidStore::<T>::get(&auction_id);
      if let Some(list) = maybe_list {
        let len = list.len();
        let penalty = T::AuctionOwnerPenaltyForEachBid::get().saturating_mul(<BalanceOf<T>>::saturated_from(len as u128));
        ensure!(
          penalty < <T as auction::Config>::Currency::free_balance(&sender), 
          Error::<T>::NotEnoughBalanceForPenalty
        );

        for user in list.into_iter() {
          Self::transfer_balance(&sender, &user, T::AuctionOwnerPenaltyForEachBid::get())?;
        }
      }
      Self::delete_auction(&auction_id)?;

      Ok(())
    }
    
    #[pallet::weight(100_000)]
    pub fn remove_bid_for_auction(
      origin: OriginFor<T>,
      auction_id: T::AuctionId,
    ) -> DispatchResult {
      let sender = ensure_signed(origin)?;

      let auction_item = AuctionStore::<T>::get(&auction_id).ok_or(Error::<T>::AuctionNotExist)?;
      
      if let Some(bid_user) = auction_item.bid_user {
        ensure!(&sender.cmp(&bid_user) != &Ordering::Equal, Error::<T>::NotAllowQuitBid);
      }

      
      Self::delete_bid(&sender, &auction_id)?;
      

      Ok(())
    }
	
	}
}

