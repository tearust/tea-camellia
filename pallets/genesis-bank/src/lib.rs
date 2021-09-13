#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use genesis_bank::*;
pub use types::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod functions;
mod rpc;
mod types;

use auction_interface::AuctionOperation;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement},
};
use frame_system::pallet_prelude::*;
use genesis_bank_interface::GenesisBankOperation;
use pallet_cml::{CmlId, CmlOperation, CmlType, SeedProperties};
use pallet_utils::{extrinsic_procedure, CurrencyOperations};
use sp_runtime::traits::{AtLeast32BitUnsigned, Saturating, Zero};
use sp_std::prelude::*;

/// The balance type of this module.
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod genesis_bank {
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
		/// Auction operation trait defined in auction interface.
		type AuctionOperation: AuctionOperation<
			AccountId = Self::AccountId,
			Balance = BalanceOf<Self>,
			BlockNumber = Self::BlockNumber,
		>;
		/// The loan's contract length, before this time the loan has to be paid off or in default
		#[pallet::constant]
		type LoanTermDuration: Get<Self::BlockNumber>;
		/// Billing cycle of bank to calculate bill.
		/// How frequent the bank review all the loan
		#[pallet::constant]
		type BillingCycle: Get<Self::BlockNumber>;

		#[pallet::constant]
		type CmlALoanAmount: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type CmlBLoanAmount: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type CmlCLoanAmount: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn operation_account)]
	pub type OperationAccount<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

	#[pallet::storage]
	pub type CloseHeight<T: Config> = StorageValue<_, T::BlockNumber, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn lien_store)]
	pub type CollateralStore<T: Config> = StorageMap<
		_,
		Twox64Concat,
		AssetUniqueId,
		Loan<T::AccountId, T::BlockNumber, BalanceOf<T>>,
		ValueQuery,
	>;

	/// Interest rates of one loan period in ten thousand units(‱).
	/// This number need to be an integer
	#[pallet::storage]
	#[pallet::getter(fn interest_rate)]
	pub type InterestRate<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// AMM curve coefficient k: `x * y = k`, k initialized when genesis build.
	#[pallet::storage]
	#[pallet::getter(fn amm_curve_k_coefficient)]
	pub type AMMCurveKCoefficient<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn user_lien_store)]
	pub type UserCollateralStore<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		AssetUniqueId,
		(),
		ValueQuery,
	>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub operation_account: T::AccountId,
		pub bank_initial_balance: BalanceOf<T>,
		/// Interest rates of one loan period in ten thousand units(‱).
		/// This number need to be an integer
		pub bank_initial_interest_rate: BalanceOf<T>,
	}
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				operation_account: Default::default(),
				bank_initial_balance: Default::default(),
				bank_initial_interest_rate: Default::default(),
			}
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			OperationAccount::<T>::set(self.operation_account.clone());

			InterestRate::<T>::set(self.bank_initial_interest_rate);
			AMMCurveKCoefficient::<T>::set(
				self.bank_initial_interest_rate * self.bank_initial_balance,
			);
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub fn deposit_event)]
	pub enum Event<T: Config> {
		BurnedCmlList(Vec<CmlId>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Loan already exists that cannot be pawn again.
		LoanAlreadyExists,
		/// The given asset id not exist in collateral store.
		LoanNotExists,
		/// Collateral not belongs to user.
		InvalidBorrower,
		/// Loan in default
		LoanInDefault,
		/// User have not enough free balance to pay off loan.
		InsufficientRepayBalance,
		/// Close height should larger equal than current height.
		InvalidCloseHeight,
		/// Only frozen seeds are allowed to be collateral
		ShouldPawnFrozenSeed,
		/// Only genesis seeds are allowed to be collateral
		ShouldPawnGenesisSeed,
		/// Not allowed to spawn a cml if it is in auction.
		CannotPawnWhenCmlIsInAuction,
		/// Collateral store is not empty and bank cannot shutdown.
		CollateralStoreNotEmpty,
		/// User collateral store not empty cannot shutdown.
		UserCollateralStoreNotEmpty,
		/// Loan id convert to cml id with invalid length.
		ConvertToCmlIdLengthMismatch,
		/// Con not apply loan after current height larger equal than the close height.
		/// Close height is a preset block height that the Genesis Bank will stop operation
		/// We have such a close time because Genesis bank is supposed to be temporary cold-start
		/// helper. When newer Defi service tApps are ready, the Genesis Bank should be retired
		CannotApplyLoanAfterClosed,
		GenesisBankInsufficientFreeBalance,
		NoNeedToRepayInterest,
		RepayAmountCanNotBeZero,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: BlockNumberFor<T>) {
			if Self::is_time_for_reset_interest_rate(n) {
				Self::try_clean_default_loan();

				Self::reset_all_loan_amounts();
				Self::reset_interest_rate();
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(195_000_000)]
		pub fn update_interest_rate_amm_k(
			sender: OriginFor<T>,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let root = ensure_root(sender)?;

			extrinsic_procedure(
				&root,
				|_root| Ok(()),
				|_root| {
					AMMCurveKCoefficient::<T>::set(amount);
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn close_bank(sender: OriginFor<T>, height: T::BlockNumber) -> DispatchResult {
			let root = ensure_root(sender)?;

			extrinsic_procedure(
				&root,
				|_root| {
					ensure!(
						frame_system::Pallet::<T>::block_number() <= height,
						Error::<T>::InvalidCloseHeight
					);
					Ok(())
				},
				|_root| {
					CloseHeight::<T>::set(Some(height));
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn apply_loan_genesis_bank(
			sender: OriginFor<T>,
			id: AssetId,
			asset_type: AssetType,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;
			let unique_id = AssetUniqueId {
				asset_type,
				inner_id: id,
			};

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						!CollateralStore::<T>::contains_key(&unique_id),
						Error::<T>::LoanAlreadyExists
					);
					Self::check_before_collateral(&unique_id, who)
				},
				|who| {
					Self::create_new_collateral(&unique_id, who);
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn payoff_loan(
			sender: OriginFor<T>,
			id: AssetId,
			asset_type: AssetType,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;
			let unique_id = AssetUniqueId {
				asset_type,
				inner_id: id,
			};

			extrinsic_procedure(
				&who,
				|who| Self::check_before_payoff_loan(&unique_id, who, amount),
				|who| Self::payoff_loan_inner(&unique_id, who, amount),
			)
		}
	}

	impl<T: Config> From<BankError> for Error<T> {
		fn from(e: BankError) -> Self {
			match e {
				BankError::ConvertToCmlIdLengthMismatch => Error::<T>::ConvertToCmlIdLengthMismatch,
			}
		}
	}
}
