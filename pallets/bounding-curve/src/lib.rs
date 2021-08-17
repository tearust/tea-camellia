#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use bounding_curve::*;
pub use types::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod functions;
mod rpc;
mod types;

use bounding_curve_interface::{BuyBoundingCurve, SellBoundingCurve};
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement},
};
use frame_system::pallet_prelude::*;
use pallet_utils::{extrinsic_procedure, CurrencyOperations};
use sp_runtime::traits::{CheckedSub, Saturating, Zero};
use sp_std::convert::TryInto;
use sp_std::prelude::*;

/// The balance type of this module.
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod bounding_curve {
	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The lockable currency type
		type Currency: Currency<Self::AccountId>;
		/// Currency operations trait defined in utils trait.
		type CurrencyOperations: CurrencyOperations<
			AccountId = Self::AccountId,
			Balance = BalanceOf<Self>,
		>;
		#[pallet::constant]
		type TAppNameMaxLength: Get<u32>;

		type LinearBuyCurve: BuyBoundingCurve<BalanceOf<Self>>;
		type LinearSellCurve: SellBoundingCurve<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn account_table)]
	pub type AccountTable<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		TAppId,
		BalanceOf<T>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn total_supply_table)]
	pub type TotalSupplyTable<T: Config> =
		StorageMap<_, Twox64Concat, TAppId, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tapp_bounding_curve)]
	pub type TAppBoundingCurve<T: Config> =
		StorageMap<_, Twox64Concat, TAppId, TAppItem, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tapp_names)]
	pub type TAppNames<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, TAppId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn last_cml_id)]
	pub type LastTAppId<T: Config> = StorageValue<_, TAppId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn operation_account)]
	pub type OperationAccount<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub fn deposit_event)]
	pub enum Event<T: Config> {}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		TAppNameIsTooLong,
		TAppNameAlreadyExist,
		InsufficientFreeBalance,
		InsufficientTAppToken,
		TAppIdNotExist,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: BlockNumberFor<T>) {}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(195_000_000)]
		pub fn create_new_tapp(
			sender: OriginFor<T>,
			tapp_name: Vec<u8>,
			init_fund: BalanceOf<T>,
			buy_curve: BuyCurveType,
			sell_curve: SellCurveType,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						tapp_name.len() <= T::TAppNameMaxLength::get() as usize,
						Error::<T>::TAppNameIsTooLong
					);
					ensure!(
						!TAppNames::<T>::contains_key(&tapp_name),
						Error::<T>::TAppNameAlreadyExist
					);
					ensure!(
						T::CurrencyOperations::free_balance(who) >= init_fund,
						Error::<T>::InsufficientFreeBalance,
					);
					Ok(())
				},
				|who| {
					let id = Self::next_id();
					TAppNames::<T>::insert(&tapp_name, id);
					TAppBoundingCurve::<T>::insert(
						id,
						TAppItem {
							id,
							name: tapp_name.clone(),
							buy_curve: buy_curve.clone(),
							sell_curve: sell_curve.clone(),
						},
					);
					Self::buy_token_inner(who, id, init_fund)
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn buy_token(
			sender: OriginFor<T>,
			tapp_id: TAppId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						TAppBoundingCurve::<T>::contains_key(tapp_id),
						Error::<T>::TAppIdNotExist
					);
					ensure!(
						T::CurrencyOperations::free_balance(who) >= amount,
						Error::<T>::InsufficientFreeBalance,
					);
					Ok(())
				},
				|who| {
					Self::buy_token_inner(who, tapp_id, amount);
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn sell_token(
			sender: OriginFor<T>,
			tapp_id: TAppId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						TAppBoundingCurve::<T>::contains_key(tapp_id),
						Error::<T>::TAppIdNotExist
					);
					ensure!(
						AccountTable::<T>::get(who, tapp_id) >= amount,
						Error::<T>::InsufficientTAppToken,
					);
					Ok(())
				},
				|who| {
					Self::sell_token_inner(who, tapp_id, amount);
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn consume(
			sender: OriginFor<T>,
			tapp_id: TAppId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| Ok(()),
				|who| {
					// todo implement me
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn expense(
			sender: OriginFor<T>,
			tapp_id: TAppId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| Ok(()),
				|who| {
					// todo implement me
				},
			)
		}
	}
}
