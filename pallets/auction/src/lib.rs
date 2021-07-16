#![cfg_attr(not(feature = "std"), no_std)]
// Disable the following two lints since they originate from an external macro (namely decl_storage)
#![allow(clippy::string_lit_as_bytes)]
#![allow(clippy::unused_unit)]

use frame_support::ensure;
use frame_support::pallet_prelude::*;
use frame_support::traits::{Currency, ExistenceRequirement::AllowDeath, Get, ReservableCurrency};
use frame_system::{ensure_signed, pallet_prelude::*};
use log::{info, warn};
use pallet_cml::{CmlId, CmlOperation, SeedProperties, TreeProperties};
use pallet_utils::{extrinsic_procedure, CurrencyOperations};
use sp_runtime::{
	traits::{Saturating, Zero},
	DispatchResult, SaturatedConversion,
};
use sp_std::{cmp::Ordering, convert::TryInto, prelude::*};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod functions;
mod rpc;
mod types;
mod weights;

pub use auction::*;
pub use types::*;
// pub use weights::WeightInfo;

type BalanceOf<T> =
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

		type AuctionOperation: AuctionOperation<
			AccountId = Self::AccountId,
			Balance = BalanceOf<Self>,
			BlockNumber = Self::BlockNumber,
		>;

		// type WeightInfo: WeightInfo;

		#[pallet::constant]
		type AuctionDealWindowBLock: Get<Self::BlockNumber>;

		#[pallet::constant]
		type MinPriceForBid: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type AuctionOwnerPenaltyForEachBid: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type AuctionPledgeAmount: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type MaxUsersPerAuction: Get<u64>;

		#[pallet::constant]
		type AuctionFeePerWindow: Get<BalanceOf<Self>>;
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
		AuctionOwnerHasCredit,
		NotFoundBid,
		NotAllowQuitBid,

		LockableInvalid,
		NotAllowToAuction,
		NotEnoughBalanceForPenalty,
		OverTheMaxUsersPerAuctionLimit,
		BuyNowPriceShouldHigherThanStartingPrice,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub fn deposit_event)]
	pub enum Event<T: Config> {
		BidAuction(AuctionId, T::AccountId, BalanceOf<T>),
		NewAuctionToStore(AuctionId, T::AccountId),
		AuctionSuccess(AuctionId, T::AccountId, BalanceOf<T>),
	}

	#[pallet::storage]
	#[pallet::getter(fn auction_store)]
	pub type AuctionStore<T: Config> = StorageMap<
		_,
		Twox64Concat,
		AuctionId,
		AuctionItem<T::AccountId, BalanceOf<T>, T::BlockNumber>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn last_auction_id)]
	pub type LastAuctionId<T: Config> = StorageValue<_, AuctionId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn bid_store)]
	pub type BidStore<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		AuctionId,
		BidItem<T::AccountId, BalanceOf<T>, T::BlockNumber>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn auction_bid_store)]
	pub type AuctionBidStore<T: Config> = StorageMap<_, Twox64Concat, AuctionId, Vec<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn endblock_auction_store)]
	pub type EndBlockAuctionStore<T: Config> =
		StorageMap<_, Twox64Concat, T::BlockNumber, Vec<AuctionId>>;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_finalize(now: T::BlockNumber) {
			if !Self::is_end_of_auction_window(&now) {
				return;
			}

			Self::try_complete_auctions();
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
			auto_renew: bool,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			extrinsic_procedure(
				&sender,
				|sender| {
					let cml_item = T::CmlOperation::cml_by_id(&cml_id)?;
					T::CmlOperation::check_belongs(&cml_id, &sender)?;
					ensure!(
						T::CurrencyOperations::free_balance(&sender)
							>= T::AuctionPledgeAmount::get() + T::AuctionFeePerWindow::get(),
						Error::<T>::NotEnoughBalance
					);
					if let Some(buy_now_price) = buy_now_price.as_ref() {
						ensure!(
							*buy_now_price > starting_price,
							Error::<T>::BuyNowPriceShouldHigherThanStartingPrice
						);
					}

					let current_height = frame_system::Pallet::<T>::block_number();
					// check cml status
					ensure!(
						cml_item.is_frozen_seed()
							|| (cml_item.is_fresh_seed() && !cml_item.has_expired(&current_height))
							|| cml_item.check_tree_validity(&current_height).is_ok(),
						Error::<T>::NotAllowToAuction
					);
					ensure!(
						T::CmlOperation::user_credit_amount(sender).is_zero(),
						Error::<T>::AuctionOwnerHasCredit
					);
					Ok(())
				},
				|sender| {
					// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
					if T::CurrencyOperations::reserve(sender, T::AuctionPledgeAmount::get())
						.is_err()
					{
						return;
					}
					let _ = T::CurrencyOperations::slash(sender, T::AuctionFeePerWindow::get());

					let auction_item = Self::new_auction_item(
						cml_id,
						sender.clone(),
						starting_price,
						buy_now_price,
						auto_renew,
					);
					Self::add_auction_to_storage(auction_item.clone());

					Self::deposit_event(Event::NewAuctionToStore(auction_item.id, sender.clone()));
				},
			)
		}

		#[pallet::weight(10_000)]
		pub fn bid_for_auction(
			origin: OriginFor<T>,
			auction_id: AuctionId,
			price: BalanceOf<T>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			extrinsic_procedure(
				&sender,
				|sender| {
					Self::check_bid_for_auction(&sender, &auction_id, price)?;

					let auction_item = AuctionStore::<T>::get(auction_id);
					if Self::can_by_now(&auction_item, price) {
						T::CmlOperation::check_transfer_cml_to_other(
							&auction_item.cml_owner,
							&auction_item.cml_id,
							sender,
						)?;
					}
					Ok(())
				},
				|sender| {
					if BidStore::<T>::contains_key(&sender, &auction_id) {
						Self::increase_bid_price(&sender, &auction_id, price);
					} else {
						Self::create_new_bid(&sender, &auction_id, price);
					}
					Self::update_current_winner(&auction_id, &sender);
					Self::deposit_event(Event::BidAuction(auction_id, sender.clone(), price));

					Self::try_complete_auction(&sender, &auction_id);
				},
			)
		}

		#[pallet::weight(100_000)]
		pub fn remove_from_store(origin: OriginFor<T>, auction_id: AuctionId) -> DispatchResult {
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
						sender.cmp(&auction_item.cml_owner) == Ordering::Equal,
						Error::<T>::AuctionOwnerInvalid
					);
					Ok(())
				},
				|sender| {
					Self::clear_auction_pledge(sender, &auction_id);
					Self::delete_auction(&auction_id, None);
				},
			)
		}

		#[pallet::weight(100_000)]
		pub fn remove_bid_for_auction(
			origin: OriginFor<T>,
			auction_id: AuctionId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			extrinsic_procedure(
				&sender,
				|sender| {
					Self::check_delete_bid(sender, &auction_id)?;

					Ok(())
				},
				|sender| {
					let bid_item = Self::delete_bid(sender, &auction_id);
					Self::return_for_bid(sender, &bid_item);
				},
			)
		}
	}
}

pub trait AuctionOperation {
	type AccountId: Default;
	type Balance: Default;
	type BlockNumber: Default;

	fn add_auction_to_storage(
		auction_item: AuctionItem<Self::AccountId, Self::Balance, Self::BlockNumber>,
	);

	fn create_new_bid(sender: &Self::AccountId, auction_id: &AuctionId, price: Self::Balance);

	// return current block window number and next.
	fn get_window_block() -> (Self::BlockNumber, Self::BlockNumber);
}
