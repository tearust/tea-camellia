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
mod mining;
mod rpc;

use bonding_curve_interface::BondingCurveOperation;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement},
};
use frame_system::pallet_prelude::*;
use genesis_bank_interface::GenesisBankOperation;
use genesis_exchange_interface::MiningOperation;
use log::error;
use pallet_cml::{CmlOperation, CmlType, SeedProperties};
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

		type BondingCurveOperation: BondingCurveOperation<
			AccountId = Self::AccountId,
			Balance = BalanceOf<Self>,
		>;

		/// Price-to-Earning Ratio
		#[pallet::constant]
		type PER: Get<BalanceOf<Self>>;

		/// Length of a USD interest calculation.
		#[pallet::constant]
		type InterestPeriodLength: Get<Self::BlockNumber>;

		#[pallet::constant]
		type CmlAMiningMachineCost: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type CmlBMiningMachineCost: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type CmlCMiningMachineCost: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type CmlARedeemCouponCost: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type CmlBRedeemCouponCost: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type CmlCRedeemCouponCost: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type BorrowAllowance: Get<BalanceOf<Self>>;

		/// Ratio cap ten thousand units(‱).
		#[pallet::constant]
		type BorrowDebtRatioCap: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn operation_account)]
	pub type OperationAccount<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn npc_account)]
	pub type NPCAccount<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

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
		StorageMap<_, Twox64Concat, T::AccountId, (Vec<u8>, Vec<u8>), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn usd_debt)]
	pub type USDDebt<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub operation_account: T::AccountId,
		pub npc_account: T::AccountId,
		pub operation_usd_amount: BalanceOf<T>,
		pub operation_tea_amount: BalanceOf<T>,
		pub competition_users: Vec<(T::AccountId, BalanceOf<T>)>,
		pub bonding_curve_npc: (T::AccountId, BalanceOf<T>),
		pub initial_usd_interest_rate: BalanceOf<T>,
	}
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				operation_account: Default::default(),
				npc_account: Default::default(),
				operation_usd_amount: Default::default(),
				operation_tea_amount: Default::default(),
				competition_users: Default::default(),
				bonding_curve_npc: Default::default(),
				initial_usd_interest_rate: Default::default(),
			}
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			OperationAccount::<T>::set(self.operation_account.clone());
			NPCAccount::<T>::set(self.npc_account.clone());
			AMMCurveKCoefficient::<T>::set(self.operation_usd_amount * self.operation_tea_amount);

			USDStore::<T>::insert(&self.operation_account, &self.operation_usd_amount);
			self.competition_users.iter().for_each(|(user, balance)| {
				USDStore::<T>::insert(user, balance);
				let value: (Vec<u8>, Vec<u8>) = (vec![], vec![]);
				CompetitionUsers::<T>::insert(user, value);
			});
			USDStore::<T>::insert(&self.bonding_curve_npc.0, &self.bonding_curve_npc.1);

			// initialize USD interest rate
			USDInterestRate::<T>::set(self.initial_usd_interest_rate);
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
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
		/// Exchange have not enough USD
		ExchangeInsufficientUSD,
		/// Exchange have not enough TEA
		ExchangeInsufficientTEA,
		/// User have not enough USD
		UserInsufficientUSD,
		/// User have not enough TEA
		UserInsufficientTEA,
		/// Given amount is invalid
		InvalidCalculationAmount,
		/// Transfer amount is more than user really holding
		InvalidTransferUSDAmount,
		/// Currency amount should not be 0
		AmountShouldNotBeZero,
		/// Buy or sell amount parameters should not both have values
		BuyAndSellAmountShouldNotBothExist,
		/// Buy or sell amount parameters should not both be none
		BuyOrSellAmountShouldExist,
		/// USD interest rate should larger than competitions count, otherwise the rate of each one
		/// will be zero.
		USDInterestRateShouldLargerThanCompetitionsCount,
		/// Not enough USD to pay the mining machine cost
		InsufficientUSDToPayMiningMachineCost,
		/// Borrowed amount should not be 0
		BorrowAmountShouldNotBeZero,
		/// Borrowed USD debt amount has over the maximum amount currency unit can hold
		BorrowDebtHasOverflow,
		/// Borrowed USD amount has over the maximum amount currency unit can hold
		BorrowAmountHasOverflow,
		/// Given USD repay debt amount is more than user USD amount
		InsufficientUSDToRepayDebts,
		/// Use have not USD debt so no need to repay it
		NoNeedToRepayUSDDebts,
		/// Repay USD amount should not be 0
		RepayUSDAmountShouldNotBeZero,
		/// Repay usd amount is more than debt amount really required
		RepayUSDAmountMoreThanDebtAmount,
		/// Not enough USD amount to redeem coupons when draw lucky box
		InsufficientUSDToRedeemCoupons,
		/// User debt utilization needs to be below the debt / asset ratio of `BorrowDebtRatioCap`,
		/// before any more COFFEE loans can be issued
		BorrowedDebtAmountHasOverThanMaxAllowed,
		/// User asset amount should larger than borrow borrow allowance
		UsdDebtReferenceAssetAmountIsLowerThanBorrowAllowance,
		/// If user asset amount is less than `BorrowAllowance` debt amount should less than borrow allowance
		InitialBorrowAmountShouldLessThanBorrowAllowance,
		/// The competition user has registered already
		CompetitionUserAlreadyRegistered,
		/// Only allowed NPC account to register new competition user
		OnlyAllowedNpcAccountToRegister,
		/// Only allowed competition account to borrow USD
		OnlyAllowedCompetitionUserBorrowUSD,
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
				|_root| Ok(()),
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

		#[pallet::weight(195_000_000)]
		pub fn borrow_usd(sender: OriginFor<T>, amount: BalanceOf<T>) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(!amount.is_zero(), Error::<T>::BorrowAmountShouldNotBeZero);
					ensure!(
						USDDebt::<T>::get(who).checked_add(&amount).is_some(),
						Error::<T>::BorrowDebtHasOverflow
					);
					ensure!(
						USDStore::<T>::get(who).checked_add(&amount).is_some(),
						Error::<T>::BorrowAmountHasOverflow,
					);
					ensure!(
						CompetitionUsers::<T>::contains_key(who),
						Error::<T>::OnlyAllowedCompetitionUserBorrowUSD
					);
					Self::check_borrowed_amount(who, &amount)?;

					Ok(())
				},
				|who| {
					USDDebt::<T>::mutate(who, |balance| {
						*balance = balance.saturating_add(amount);
					});
					USDStore::<T>::mutate(who, |balance| {
						*balance = balance.saturating_add(amount);
					});
				},
			)
		}

		/// repay debts buy given specified amount, if `amount` is none will repay all debts by
		/// default.
		#[pallet::weight(195_000_000)]
		pub fn repay_usd_debts(
			sender: OriginFor<T>,
			amount: Option<BalanceOf<T>>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					let repay_amount = match amount {
						Some(amount) => amount,
						None => USDDebt::<T>::get(who),
					};
					ensure!(
						!USDDebt::<T>::get(who).is_zero(),
						Error::<T>::NoNeedToRepayUSDDebts
					);
					ensure!(
						!repay_amount.is_zero(),
						Error::<T>::RepayUSDAmountShouldNotBeZero
					);
					ensure!(
						repay_amount <= USDDebt::<T>::get(who),
						Error::<T>::RepayUSDAmountMoreThanDebtAmount
					);
					ensure!(
						USDStore::<T>::get(who) >= repay_amount,
						Error::<T>::InsufficientUSDToRepayDebts
					);
					Ok(())
				},
				|who| {
					USDDebt::<T>::mutate(who, |debt| {
						let repay_amount = match amount {
							Some(amount) => amount,
							None => USDDebt::<T>::get(who),
						};
						*debt = debt.saturating_sub(repay_amount);
						USDStore::<T>::mutate(who, |a| {
							*a = a.saturating_sub(repay_amount);
						});
					});

					if USDDebt::<T>::get(who).is_zero() {
						USDDebt::<T>::remove(who);
					}
					if USDStore::<T>::get(who).is_zero() {
						USDStore::<T>::remove(who);
					}
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn register_for_competition(
			sender: OriginFor<T>,
			user: T::AccountId,
			erc20_address: Vec<u8>,
			email_address: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						!CompetitionUsers::<T>::contains_key(&user),
						Error::<T>::CompetitionUserAlreadyRegistered
					);
					ensure!(
						who.eq(&NPCAccount::<T>::get()),
						Error::<T>::OnlyAllowedNpcAccountToRegister
					);
					Ok(())
				},
				|_who| {
					CompetitionUsers::<T>::insert(
						&user,
						(erc20_address.clone(), email_address.clone()),
					);
				},
			)
		}
	}
}
