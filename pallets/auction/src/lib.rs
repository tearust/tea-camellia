#![cfg_attr(not(feature = "std"), no_std)]

// Disable the following two lints since they originate from an external macro (namely decl_storage)
#![allow(clippy::string_lit_as_bytes)]
#![allow(clippy::unused_unit)]

use sp_std::prelude::*;
use frame_support::pallet_prelude::*;
use frame_support::{ensure};
use frame_support::traits::{
	Currency, LockableCurrency, LockIdentifier, WithdrawReasons,
	Get,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use sp_runtime::{
  SaturatedConversion,
	traits::{AtLeast32BitUnsigned, Bounded, MaybeSerializeDeserialize, Member, One, Zero},
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
    InvalidPrice,
		// AuctionNotExist,
		// AuctionNotStarted,
		// BidNotAccepted,
		// InvalidBidPrice,
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

	/// Index auctions by end time.
	// #[pallet::storage]
	// #[pallet::getter(fn auction_end_time)]
	// pub type AuctionEndTime<T: Config> =
	// 	StorageDoubleMap<_, Twox64Concat, T::BlockNumber, Blake2_128Concat, T::AuctionId, (), OptionQuery>;

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

      let (list, index) = cml::Pallet::<T>::find_cml_index(&sender, &cml_id);

      //TODO check cml status

      ensure!(index >= 0, Error::<T>::CmlIdInvalid);

      // let balance = T::Currency::free_balance(&sender);
      // ensure!(buy_now_price >= starting_price, Error::<T>::NotEnoughBalance);
      // if let Some(buy_now_price) = buy_now_price {
      //   ensure!(buy_now_price > starting_price, Error::<T>::InvalidPrice);
      //   ensure!(balance >= buy_now_price, Error::<T>::NotEnoughBalance);
      // }

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

      // let reason = WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE;
      // <T as auction::Config>::Currency::set_lock(AUCTION_ID, &sender, T::AuctionDeposit::get(), reason);

      Ok(())
    }
		
	// 	#[pallet::weight(10_000)]
	// 	pub fn bid(
	// 		origin: OriginFor<T>,
	// 		id: T::AuctionId,
	// 		#[pallet::compact] value: T::Balance,
	// 	) -> DispatchResultWithPostInfo {
	// 		let from = ensure_signed(origin)?;

	// 		Auctions::<T>::try_mutate_exists(id, |auction| -> DispatchResult {
	// 			let mut auction = auction.as_mut().ok_or(Error::<T>::AuctionNotExist)?;

	// 			let block_number = <frame_system::Pallet<T>>::block_number();

	// 			// make sure auction is started
	// 			ensure!(block_number >= auction.start, Error::<T>::AuctionNotStarted);

	// 			if let Some(ref current_bid) = auction.bid {
	// 				ensure!(value > current_bid.1, Error::<T>::InvalidBidPrice);
	// 			} else {
	// 				ensure!(!value.is_zero(), Error::<T>::InvalidBidPrice);
	// 			}
	// 			let bid_result = T::Handler::on_new_bid(block_number, id, (from.clone(), value), auction.bid.clone());

	// 			ensure!(bid_result.accept_bid, Error::<T>::BidNotAccepted);
	// 			match bid_result.auction_end_change {
	// 				Change::NewValue(new_end) => {
	// 					if let Some(old_end_block) = auction.end {
	// 						AuctionEndTime::<T>::remove(&old_end_block, id);
	// 					}
	// 					if let Some(new_end_block) = new_end {
	// 						AuctionEndTime::<T>::insert(&new_end_block, id, ());
	// 					}
	// 					auction.end = new_end;
	// 				}
	// 				Change::NoChange => {}
	// 			}
	// 			auction.bid = Some((from.clone(), value));

	// 			Ok(())
	// 		})?;

	// 		Self::deposit_event(Event::Bid(id, from, value));
	// 		Ok(().into())
	// 	}
	}
}

