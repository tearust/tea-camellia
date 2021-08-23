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

use bounding_curve_interface::BoundingCurveInterface;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement},
};
use frame_system::pallet_prelude::*;
use pallet_utils::{extrinsic_procedure, CurrencyOperations};
use sp_runtime::traits::{CheckedAdd, CheckedSub, Saturating, Zero};
use sp_std::{collections::btree_set::BTreeSet, prelude::*};

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

		#[pallet::constant]
		type TAppTickerMaxLength: Get<u32>;

		#[pallet::constant]
		type TAppTickerMinLength: Get<u32>;

		#[pallet::constant]
		type TAppDetailMaxLength: Get<u32>;

		#[pallet::constant]
		type TAppLinkMaxLength: Get<u32>;

		#[pallet::constant]
		type PoolBalanceReversePrecision: Get<BalanceOf<Self>>;

		type LinearCurve: BoundingCurveInterface<BalanceOf<Self>>;

		#[allow(non_camel_case_types)]
		type UnsignedSquareRoot_10: BoundingCurveInterface<BalanceOf<Self>>;

		#[allow(non_camel_case_types)]
		type UnsignedSquareRoot_7: BoundingCurveInterface<BalanceOf<Self>>;
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
		StorageMap<_, Twox64Concat, TAppId, TAppItem<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tapp_names)]
	pub type TAppNames<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, TAppId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tapp_tickers)]
	pub type TAppTickers<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, TAppId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn last_cml_id)]
	pub type LastTAppId<T: Config> = StorageValue<_, TAppId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn operation_account)]
	pub type OperationAccount<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub fn deposit_event)]
	pub enum Event<T: Config> {
		/// Fired after Tapp created successfully, event parameters:
		/// 1. TApp Id
		/// 2. TApp name array encoded with UTF8
		/// 3. TApp owner Account Id
		TAppCreated(TAppId, Vec<u8>, T::AccountId),

		/// Fired after TApp token bought successfully, event parameters:
		/// 1. TApp Id
		/// 2. Bought Account Id
		/// 3. Bought TEA amount
		TokenBought(TAppId, T::AccountId, BalanceOf<T>),

		/// Fired after TApp token sold successfully, event parameters:
		/// 1. TApp Id
		/// 2. Sold Account Id
		/// 3. Sold TEA amount
		TokenSold(TAppId, T::AccountId, BalanceOf<T>),

		/// Fired after TApp expensed successfully, event parameters:
		/// 1. TApp Id
		/// 2. Payed Account Id
		/// 3. Payed TEA amount
		TAppExpense(TAppId, T::AccountId, BalanceOf<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		TAppNameIsTooLong,
		TAppTickerIsTooLong,
		TAppTickerIsTooShort,
		TAppDetailIsTooLong,
		TAppLinkIsTooLong,
		TAppNameAlreadyExist,
		TAppTickerAlreadyExist,
		InsufficientFreeBalance,
		InsufficientTAppToken,
		/// Sell amount more than total supply
		InsufficientTotalSupply,
		TAppIdNotExist,
		/// Sell amount more than total reserved pool tea token
		TAppInsufficientFreeBalance,
		OperationAmountCanNotBeZero,
		BuyTeaAmountCanNotBeZero,
		SellTeaAmountCanNotBeZero,
		SubtractionOverflow,
		AddOverflow,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(_n: BlockNumberFor<T>) {}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(195_000_000)]
		pub fn create_new_tapp(
			sender: OriginFor<T>,
			tapp_name: Vec<u8>,
			ticker: Vec<u8>,
			init_fund: BalanceOf<T>,
			detail: Vec<u8>,
			link: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;
			let buy_curve = CurveType::UnsignedSquareRoot_10;
			let sell_curve = CurveType::UnsignedSquareRoot_7;

			extrinsic_procedure(
				&who,
				|who| {
					Self::check_tapp_fields_length(&tapp_name, &ticker, &detail, &link)?;
					ensure!(
						!TAppNames::<T>::contains_key(&tapp_name),
						Error::<T>::TAppNameAlreadyExist
					);
					ensure!(
						!TAppTickers::<T>::contains_key(&ticker),
						Error::<T>::TAppTickerAlreadyExist
					);
					ensure!(
						!init_fund.is_zero(),
						Error::<T>::OperationAmountCanNotBeZero
					);
					let deposit_tea_amount =
						Self::calculate_increase_amount_from_curve_total_supply(
							buy_curve,
							0u32.into(),
							init_fund,
						)?;
					ensure!(
						!deposit_tea_amount.is_zero(),
						Error::<T>::BuyTeaAmountCanNotBeZero
					);
					ensure!(
						T::CurrencyOperations::free_balance(who) >= deposit_tea_amount,
						Error::<T>::InsufficientFreeBalance,
					);
					Ok(())
				},
				|who| {
					let id = Self::next_id();
					TAppNames::<T>::insert(&tapp_name, id);
					TAppTickers::<T>::insert(&ticker, id);
					TAppBoundingCurve::<T>::insert(
						id,
						TAppItem {
							id,
							name: tapp_name.clone(),
							ticker: ticker.clone(),
							owner: who.clone(),
							buy_curve,
							sell_curve,
							detail: detail.clone(),
							link: link.clone(),
						},
					);
					Self::buy_token_inner(who, id, init_fund);

					Self::deposit_event(Event::TAppCreated(id, tapp_name.clone(), who.clone()));
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn buy_token(
			sender: OriginFor<T>,
			tapp_id: TAppId,
			tapp_amount: BalanceOf<T>,
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
						!tapp_amount.is_zero(),
						Error::<T>::OperationAmountCanNotBeZero
					);
					let deposit_tea_amount = Self::calculate_buy_amount(tapp_id, tapp_amount)?;
					ensure!(
						!deposit_tea_amount.is_zero(),
						Error::<T>::BuyTeaAmountCanNotBeZero
					);
					ensure!(
						T::CurrencyOperations::free_balance(who) >= deposit_tea_amount,
						Error::<T>::InsufficientFreeBalance,
					);
					Ok(())
				},
				|who| {
					let deposit_tea_amount = Self::buy_token_inner(who, tapp_id, tapp_amount);
					Self::deposit_event(Event::TokenBought(
						tapp_id,
						who.clone(),
						deposit_tea_amount,
					));
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn sell_token(
			sender: OriginFor<T>,
			tapp_id: TAppId,
			tapp_amount: BalanceOf<T>,
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
						AccountTable::<T>::get(who, tapp_id) >= tapp_amount,
						Error::<T>::InsufficientTAppToken,
					);
					ensure!(
						!tapp_amount.is_zero(),
						Error::<T>::OperationAmountCanNotBeZero
					);
					let tea_amount = Self::calculate_sell_amount(tapp_id, tapp_amount)?;
					ensure!(!tea_amount.is_zero(), Error::<T>::SellTeaAmountCanNotBeZero);
					Ok(())
				},
				|who| {
					let sold_amount = Self::sell_token_inner(who, tapp_id, tapp_amount);
					Self::deposit_event(Event::TokenSold(tapp_id, who.clone(), sold_amount));
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn consume(
			sender: OriginFor<T>,
			tapp_id: TAppId,
			tea_amount: BalanceOf<T>,
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
						!tea_amount.is_zero(),
						Error::<T>::OperationAmountCanNotBeZero
					);
					ensure!(
						T::CurrencyOperations::free_balance(who) >= tea_amount,
						Error::<T>::InsufficientFreeBalance,
					);
					Ok(())
				},
				|who| {
					match Self::calculate_given_increase_tea_how_much_token_mint(
						tapp_id, tea_amount,
					) {
						Ok(deposit_tapp_amount) => {
							if let Err(e) =
								Self::allocate_buy_tea_amount(who, tapp_id, deposit_tapp_amount)
							{
								// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
								log::error!("allocate buy tea amount failed: {:?}", e);
								return;
							}
							Self::distribute_to_investors(tapp_id, deposit_tapp_amount);
						}
						Err(e) => {
							// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
							log::error!(
								"calculate given increase tea how much token mint failed: {:?}",
								e
							);
							return;
						}
					}
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn expense(
			sender: OriginFor<T>,
			tapp_id: TAppId,
			tea_amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|_who| {
					ensure!(
						TAppBoundingCurve::<T>::contains_key(tapp_id),
						Error::<T>::TAppIdNotExist
					);
					ensure!(
						!tea_amount.is_zero(),
						Error::<T>::OperationAmountCanNotBeZero
					);

					let withdraw_tapp_amount =
						Self::calculate_given_received_tea_how_much_seller_give_away(
							tapp_id, tea_amount,
						)?;
					ensure!(
						TotalSupplyTable::<T>::get(tapp_id) > withdraw_tapp_amount,
						Error::<T>::InsufficientTotalSupply
					);
					Ok(())
				},
				|who| {
					match Self::calculate_given_received_tea_how_much_seller_give_away(
						tapp_id, tea_amount,
					) {
						Ok(withdraw_tapp_amount) => {
							if let Err(e) = T::CurrencyOperations::transfer(
								&OperationAccount::<T>::get(),
								who,
								tea_amount,
								ExistenceRequirement::AllowDeath,
							) {
								// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
								log::error!("transfer free balance failed: {:?}", e);
								return;
							}
							Self::collect_with_investors(tapp_id, withdraw_tapp_amount);

							Self::deposit_event(Event::TAppExpense(
								tapp_id,
								who.clone(),
								tea_amount,
							));
						}
						Err(e) => {
							// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
							log::error!("calculation failed: {:?}", e);
						}
					}
				},
			)
		}
	}
}
