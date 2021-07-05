#![cfg_attr(not(feature = "std"), no_std)]
// Disable the following two lints since they originate from an external macro (namely decl_storage)
#![allow(clippy::string_lit_as_bytes)]
#![allow(clippy::unused_unit)]

use frame_support::ensure;
use frame_support::pallet_prelude::*;
use frame_support::traits::{
	Currency,
	ExistenceRequirement::AllowDeath,
	// LockIdentifier, WithdrawReasons,
	Get,
	ReservableCurrency,
};
use frame_system::{ensure_signed, pallet_prelude::*};
use log::{info, warn};
use pallet_cml::{CmlId, CmlOperation, SeedProperties, TreeProperties};
use pallet_utils::{extrinsic_procedure, CurrencyOperations};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, Bounded, MaybeSerializeDeserialize, Member, One, Saturating},
	DispatchResult, SaturatedConversion,
};
use sp_std::{cmp::Ordering, prelude::*};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod functions;
mod types;
mod weights;
pub use types::*;

// pub use weights::WeightInfo;

pub use auction::*;

// pub const AUCTION_ID: LockIdentifier = *b"_auction";

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod auction {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;

		type CurrencyOperations: CurrencyOperations<
			AccountId = Self::AccountId,
			Balance = BalanceOf<Self>,
		>;

		type CmlOperation: CmlOperation<
			AccountId = Self::AccountId,
			Balance = BalanceOf<Self>,
			BlockNumber = Self::BlockNumber,
		>;

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
		type MinPriceForBid: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type AuctionOwnerPenaltyForEachBid: Get<BalanceOf<Self>>;
	}

	#[pallet::error]
	pub enum Error<T> {
		NotEnoughBalance,
		NotEnoughReserveBalance,
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

		NotEnoughBalanceForPenalty,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub fn deposit_event)]
	pub enum Event<T: Config> {
		BidAuction(T::AuctionId, T::AccountId, BalanceOf<T>),
		NewAuctionToStore(T::AuctionId, T::AccountId),
		AuctionSuccess(T::AuctionId, T::AccountId, BalanceOf<T>),
	}

	#[pallet::storage]
	#[pallet::getter(fn auction_store)]
	pub type AuctionStore<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AuctionId,
		AuctionItem<T::AuctionId, T::AccountId, BalanceOf<T>, T::BlockNumber>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn user_auction_store)]
	pub type UserAuctionStore<T: Config> = StorageMap<
		_,
		// Blake2_128Concat,
		Twox64Concat,
		T::AccountId,
		Vec<T::AuctionId>,
		OptionQuery,
	>;

	#[pallet::type_value]
	pub fn DefaultAuctionId<T: Config>() -> T::AuctionId {
		<T::AuctionId>::saturated_from(1_u32)
	}
	#[pallet::storage]
	#[pallet::getter(fn auctions_index)]
	pub type LastAuctionId<T: Config> =
		StorageValue<_, T::AuctionId, ValueQuery, DefaultAuctionId<T>>;

	#[pallet::storage]
	#[pallet::getter(fn bid_store)]
	pub type BidStore<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		T::AuctionId,
		BidItem<T::AuctionId, T::AccountId, BalanceOf<T>, T::BlockNumber>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn auction_bid_store)]
	pub type AuctionBidStore<T: Config> =
		StorageMap<_, Twox64Concat, T::AuctionId, Vec<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn user_bid_store)]
	pub type UserBidStore<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<T::AuctionId>>;

	#[pallet::storage]
	#[pallet::getter(fn endblock_auction_store)]
	pub type EndblockAuctionStore<T: Config> =
		StorageMap<_, Twox64Concat, T::BlockNumber, Vec<T::AuctionId>>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
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
			cml_id: CmlId,
			starting_price: BalanceOf<T>,
			buy_now_price: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			extrinsic_procedure(
				&sender,
				|sender| {
					let cml_item = T::CmlOperation::get_cml_by_id(&cml_id)?;
					T::CmlOperation::check_belongs(&cml_id, &sender)?;

					let current_height = frame_system::Pallet::<T>::block_number();
					// check cml status
					ensure!(
						cml_item.is_frozen_seed()
							|| (cml_item.is_fresh_seed() && !cml_item.has_expired(&current_height))
							|| cml_item.tree_valid(&current_height),
						Error::<T>::NotAllowToAuction
					);
					Ok(())
				},
				|sender| {
					let auction_item = Self::new_auction_item(
						cml_id,
						sender.clone(),
						starting_price,
						buy_now_price,
					);
					Self::add_auction_to_storage(auction_item.clone(), &sender);

					Self::deposit_event(Event::NewAuctionToStore(auction_item.id, sender.clone()));
				},
			)
		}

		#[pallet::weight(10_000)]
		pub fn bid_for_auction(
			origin: OriginFor<T>,
			auction_id: T::AuctionId,
			price: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			extrinsic_procedure(
				&sender,
				|sender| {
					Self::check_bid_for_auction(&sender, &auction_id, price)?;
					Ok(())
				},
				|sender| {
					if BidStore::<T>::contains_key(&sender, &auction_id) {
						Self::increase_bid_price(&sender, &auction_id, price);
					} else {
						Self::create_new_bid(&sender, &auction_id, price);
					}
					Self::update_bid_price_for_auction_item(&auction_id, &sender);
					Self::deposit_event(Event::BidAuction(auction_id, sender.clone(), price));

					Self::try_complete_auction(&sender, &auction_id);
				},
			)
		}

		#[pallet::weight(100_000)]
		pub fn remove_from_store(origin: OriginFor<T>, auction_id: T::AuctionId) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			extrinsic_procedure(
				&sender,
				|sender| {
					ensure!(
						AuctionStore::<T>::contains_key(&auction_id),
						Error::<T>::AuctionNotExist
					);
					let auction_item = AuctionStore::<T>::get(&auction_id);
					ensure!(
						&sender.cmp(&auction_item.cml_owner) == &Ordering::Equal,
						Error::<T>::AuctionOwnerInvalid
					);

					let maybe_list = AuctionBidStore::<T>::get(&auction_id);
					if let Some(list) = maybe_list {
						let len = list.len();
						let penalty = T::AuctionOwnerPenaltyForEachBid::get()
							.saturating_mul(<BalanceOf<T>>::saturated_from(len as u128));
						ensure!(
							penalty < <T as auction::Config>::Currency::free_balance(&sender),
							Error::<T>::NotEnoughBalanceForPenalty
						);
					}
					Ok(())
				},
				|sender| {
					let maybe_list = AuctionBidStore::<T>::get(&auction_id);
					if let Some(list) = maybe_list {
						for user in list.into_iter() {
							if let Err(e) = T::CurrencyOperations::transfer(
								&sender,
								&user,
								T::AuctionOwnerPenaltyForEachBid::get(),
								AllowDeath,
							) {
								// should never happen, print here just in case
								warn!("transfer failed: {:?}", e);
							}
						}
					}
					Self::delete_auction(&auction_id);
				},
			)
		}

		#[pallet::weight(100_000)]
		pub fn remove_bid_for_auction(
			origin: OriginFor<T>,
			auction_id: T::AuctionId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			extrinsic_procedure(
				&sender,
				|sender| {
					ensure!(
						AuctionStore::<T>::contains_key(&auction_id),
						Error::<T>::AuctionNotExist
					);

					let auction_item = AuctionStore::<T>::get(&auction_id);
					if let Some(bid_user) = auction_item.bid_user {
						ensure!(
							&sender.cmp(&bid_user) != &Ordering::Equal,
							Error::<T>::NotAllowQuitBid
						);
					}
					Self::check_delete_bid(sender, &auction_id)?;

					Ok(())
				},
				|sender| {
					Self::delete_bid(&sender, &auction_id);
				},
			)
		}
	}
}
