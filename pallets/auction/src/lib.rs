#![cfg_attr(not(feature = "std"), no_std)]

// Disable the following two lints since they originate from an external macro (namely decl_storage)
#![allow(clippy::string_lit_as_bytes)]
#![allow(clippy::unused_unit)]

use sp_std::{cmp::Ordering, prelude::*};
use frame_support::pallet_prelude::*;
use frame_support::{ensure};
use frame_support::traits::{
	Currency, LockableCurrency, LockIdentifier, WithdrawReasons,
	Get,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use sp_runtime::{
  SaturatedConversion,
	traits::{Saturating, AtLeast32BitUnsigned, Bounded, MaybeSerializeDeserialize, Member, One, Zero},
	DispatchError, DispatchResult,
};
// use node_primitives::Balance;
use log::{info};

mod mock;
mod tests;
mod weights;
mod types;
mod functions;
pub use types::*;

// pub use weights::WeightInfo;

pub use auction::*;
use pallet_cml as cml;

const AUCTION_ID: LockIdentifier = *b"_auction";

pub type BalanceOf<T> = 
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod auction {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + cml::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

    type Currency: LockableCurrency<Self::AccountId>;

		/// The auction ID type.
		type AuctionId: Parameter
			+ Member
			+ AtLeast32BitUnsigned
			+ Default
			+ Copy
			+ MaybeSerializeDeserialize
			+ Bounded
			+ codec::FullCodec;

    #[pallet::constant]
    type AuctionDeposit: Get<BalanceOf<Self>>;
		// type WeightInfo: WeightInfo;
	}

	#[pallet::error]
	pub enum Error<T> {
    CmlIdInvalid,
    NotEnoughBalance,
    AuctionNotExist,
    InvalidBidPrice,
    NoNeedBid,

		// AuctionNotStarted,
		// BidNotAccepted,
		// NoAvailableAuctionId,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
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
    AuctionItem<T::AuctionId, T::AccountId, T::AssetId, BalanceOf<T>, T::BlockNumber>, 
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

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		// fn on_initialize(now: T::BlockNumber) -> Weight {
		// 	T::WeightInfo::on_finalize(AuctionEndTime::<T>::iter_prefix(&now).count() as u32)
		// }

		// fn on_finalize(now: T::BlockNumber) {
		// 	for (auction_id, _) in AuctionEndTime::<T>::drain_prefix(&now) {
		// 		if let Some(auction) = Auctions::<T>::take(&auction_id) {
		// 			T::Handler::on_auction_ended(auction_id, auction.bid);
		// 		}
		// 	}
		// }
	}

	#[pallet::call]
  impl<T: Config> Pallet<T> {

    #[pallet::weight(10_000)]
    pub fn put_to_store(
      origin: OriginFor<T>,
      cml_id: T::AssetId,
      starting_price: BalanceOf<T>,
      buy_now_price: Option<BalanceOf<T>>,
    ) -> DispatchResult {
      let sender = ensure_signed(origin)?;

      let cml = cml::Pallet::<T>::get_cml_by_id(&cml_id)?;

      let auction_item = Self::new_auction_item(cml_id, sender.clone(), starting_price, buy_now_price);
      
      UserAuctionStore::<T>::mutate(&sender, |maybe_list| {	
        if let Some(ref mut list) = maybe_list {
          list.push(auction_item.id);
        }
        else {
          *maybe_list = Some(vec![auction_item.id]);
        }
      });
      
      AuctionStore::<T>::insert(auction_item.id, auction_item);

      // TODO not work
      let reason = WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE;
      <T as auction::Config>::Currency::set_lock(AUCTION_ID, &sender, T::AuctionDeposit::get(), reason);

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
      let min_price = Self::get_min_bid_price(&auction_item, &sender);
      info!("1111 => {:?}", auction_item);
      info!("2222 => {:?}", min_price);

      if let Some(bid_user) = &auction_item.bid_user {
        ensure!(&bid_user.cmp(&sender) != &Ordering::Equal, Error::<T>::NoNeedBid);
      }

      ensure!(min_price < price, Error::<T>::InvalidBidPrice);
      ensure!(&auction_item.cml_owner.cmp(&sender) != &Ordering::Equal, Error::<T>::NoNeedBid);

      // TODO complete auction
      // if price >= auction_item.buy_now_price {}

      let current_block = frame_system::Pallet::<T>::block_number();
      let maybe_bid_item = BidStore::<T>::get(&sender, &auction_id);
      if let Some(bid_item) = maybe_bid_item {
        // increase price
        let new_price = bid_item.price.saturating_add(price);
        BidStore::<T>::mutate(&sender, &auction_id, |maybe_item| {
          if let Some(ref mut item) = maybe_item {
            item.price = new_price;
            item.updated_at = current_block;

            info!("3333 => {:?}", item);
          }
        });
      }
      else {
        // new bid
        let item = Self::new_bid_item(auction_item.id, sender.clone(), price);
        info!("4444 => {:?}", item);
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

      // TODO deposit price to lock balance

      Ok(())
    }
		
	
	}
}

