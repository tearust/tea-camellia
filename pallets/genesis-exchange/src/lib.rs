#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use genesis_exchange::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod functions;
mod rpc;

use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement},
};
use frame_system::pallet_prelude::*;
use genesis_bank_interface::GenesisBankOperation;
use log::error;
use pallet_cml::CmlOperation;
use pallet_utils::{extrinsic_procedure, CurrencyOperations};
use sp_runtime::traits::{CheckedAdd, CheckedSub, Saturating, Zero};
use sp_std::{cmp::max, collections::btree_map::BTreeMap, convert::TryInto, prelude::*};

/// The balance type of this module.
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod genesis_exchange {
	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The lockable currency type
		type Currency: Currency<Self::AccountId>;
		/// Cml operation trait defined in cml trait.
		type CmlOperation: CmlOperation<
			AccountId = Self::AccountId,
			Balance = BalanceOf<Self>,
			BlockNumber = Self::BlockNumber,
		>;
		/// Currency operations trait defined in utils trait.
		type CurrencyOperations: CurrencyOperations<
			AccountId = Self::AccountId,
			Balance = BalanceOf<Self>,
		>;

		type GenesisBankOperation: GenesisBankOperation<
			AccountId = Self::AccountId,
			Balance = BalanceOf<Self>,
			BlockNumber = Self::BlockNumber,
		>;

		/// Price-to-Earning Ratio
		#[pallet::constant]
		type PER: Get<BalanceOf<Self>>;

		/// Length of a USD interest calculation.
		#[pallet::constant]
		type InterestPeriodLength: Get<Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn operation_account)]
	pub type OperationAccount<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

	/// AMM curve coefficient k: `x * y = k`, k initialized when genesis build.
	#[pallet::storage]
	#[pallet::getter(fn amm_curve_k_coefficient)]
	pub type AMMCurveKCoefficient<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// Interest rates of one interest period in ten thousand units(‱).
	/// This number need to be an integer
	#[pallet::storage]
	#[pallet::getter(fn usd_interest_rate)]
	pub type USDInterestRate<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn usd_store)]
	pub type USDStore<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn competition_users)]
	pub type CompetitionUsers<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, (), ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub operation_account: T::AccountId,
		pub operation_usd_amount: BalanceOf<T>,
		pub operation_tea_amount: BalanceOf<T>,
		pub competition_users: Vec<(T::AccountId, BalanceOf<T>)>,
	}
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				operation_account: Default::default(),
				operation_usd_amount: Default::default(),
				operation_tea_amount: Default::default(),
				competition_users: Default::default(),
			}
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			OperationAccount::<T>::set(self.operation_account.clone());
			AMMCurveKCoefficient::<T>::set(self.operation_usd_amount * self.operation_tea_amount);

			USDStore::<T>::insert(self.operation_account.clone(), &self.operation_usd_amount);
			self.competition_users
				.iter()
				.for_each(|(user, balance)| USDStore::<T>::insert(user, balance));

			self.competition_users
				.iter()
				.for_each(|(user, _)| CompetitionUsers::<T>::insert(user, ()));

			// initialize USD interest rate
			USDInterestRate::<T>::set(1500u32.into());
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId")]
	pub enum Event<T: Config> {
		/// Event fired after buy tea (from usd) successfully.
		///
		/// Event parameters:
		/// 1. Account id
		/// 2. Exchange TEA amount
		/// 3. Exchange USD amount
		/// 4. current 1TEA equals how many USD amount
		/// 5. current 1USD equals how many TEA amount
		ExchangeSuccess(
			T::AccountId,
			BalanceOf<T>,
			BalanceOf<T>,
			BalanceOf<T>,
			BalanceOf<T>,
		),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		ExchangeInsufficientUSD,
		ExchangeInsufficientTEA,
		UserInsufficientUSD,
		UserInsufficientTEA,
		InvalidCalculationAmount,
		InvalidTransferUSDAmount,
		AmountShouldNotBeZero,
		BuyAndSellAmountShouldNotBothExist,
		BuyOrSellAmountShouldExist,
		/// USD interest rate should larger than competitions count, otherwise the rate of each one
		/// will be zero.
		USDInterestRateShouldLargerThanCompetitionsCount,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: BlockNumberFor<T>) {
			if Self::is_interest_period_end(n) {
				Self::accumulate_usd_interest();
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(195_000_000)]
		pub fn update_usd_interest_rate(
			sender: OriginFor<T>,
			rate: BalanceOf<T>,
		) -> DispatchResult {
			let root = ensure_root(sender)?;
			extrinsic_procedure(
				&root,
				|_root| {
					ensure!(
						rate >= ((USDStore::<T>::iter().count() - 1) as u32).into(),
						Error::<T>::USDInterestRateShouldLargerThanCompetitionsCount
					);
					Ok(())
				},
				|_root| USDInterestRate::<T>::set(rate),
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn tea_to_usd(
			sender: OriginFor<T>,
			buy_usd_amount: Option<BalanceOf<T>>,
			sell_tea_amount: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;
			let exchange_remains_usd = USDStore::<T>::get(OperationAccount::<T>::get());
			let exchange_remains_tea =
				T::CurrencyOperations::free_balance(&OperationAccount::<T>::get());

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						!(buy_usd_amount.is_some() && sell_tea_amount.is_some()),
						Error::<T>::BuyAndSellAmountShouldNotBothExist
					);

					if let Some(buy_usd_amount) = buy_usd_amount.as_ref() {
						Self::check_buy_tea_to_usd(
							who,
							buy_usd_amount,
							&exchange_remains_usd,
							&exchange_remains_tea,
						)
					} else if let Some(sell_tea_amount) = sell_tea_amount.as_ref() {
						Self::check_sell_tea_to_usd(
							who,
							sell_tea_amount,
							&exchange_remains_usd,
							&exchange_remains_tea,
						)
					} else {
						ensure!(false, Error::<T>::BuyOrSellAmountShouldExist);
						Ok(())
					}
				},
				|who| {
					if let Some(buy_usd_amount) = buy_usd_amount.as_ref() {
						Self::exchange_buy_tea_to_usd(
							who,
							buy_usd_amount,
							&exchange_remains_usd,
							&exchange_remains_tea,
						)
					} else if let Some(sell_tea_amount) = sell_tea_amount.as_ref() {
						Self::exchange_sell_tea_to_usd(
							who,
							sell_tea_amount,
							&exchange_remains_usd,
							&exchange_remains_tea,
						)
					}
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn usd_to_tea(
			sender: OriginFor<T>,
			buy_tea_amount: Option<BalanceOf<T>>,
			sell_usd_amount: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;
			let exchange_remains_usd = USDStore::<T>::get(OperationAccount::<T>::get());
			let exchange_remains_tea =
				T::CurrencyOperations::free_balance(&OperationAccount::<T>::get());

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						!(buy_tea_amount.is_some() && sell_usd_amount.is_some()),
						Error::<T>::BuyAndSellAmountShouldNotBothExist
					);

					if let Some(buy_tea_amount) = buy_tea_amount.as_ref() {
						Self::check_buy_usd_to_tea(
							who,
							buy_tea_amount,
							&exchange_remains_usd,
							&exchange_remains_tea,
						)
					} else if let Some(sell_usd_amount) = sell_usd_amount.as_ref() {
						Self::check_sell_usd_to_tea(
							who,
							sell_usd_amount,
							&exchange_remains_usd,
							&exchange_remains_tea,
						)
					} else {
						ensure!(false, Error::<T>::BuyOrSellAmountShouldExist);
						Ok(())
					}
				},
				|who| {
					if let Some(buy_tea_amount) = buy_tea_amount.as_ref() {
						Self::exchange_buy_usd_to_tea(
							who,
							buy_tea_amount,
							&exchange_remains_usd,
							&exchange_remains_tea,
						)
					} else if let Some(sell_usd_amount) = sell_usd_amount.as_ref() {
						Self::exchange_sell_usd_to_tea(
							who,
							sell_usd_amount,
							&exchange_remains_usd,
							&exchange_remains_tea,
						)
					}
				},
			)
		}
	}
}
