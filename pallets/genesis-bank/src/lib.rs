#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use genesis_bank::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod functions;
mod rpc;
mod types;

use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement},
};
use frame_system::pallet_prelude::*;
use pallet_cml::{CmlOperation, SeedProperties};
use pallet_utils::{extrinsic_procedure, CurrencyOperations};
use sp_runtime::traits::Zero;
use sp_std::convert::TryInto;
use sp_std::prelude::*;
use types::*;

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
		/// The max time lien should be pay off.
		#[pallet::constant]
		type LienTermDuration: Get<Self::BlockNumber>;
		/// The unit amount that a genesis cml can be paid.
		#[pallet::constant]
		type GenesisCmlLienAmount: Get<BalanceOf<Self>>;
		/// Lending rates of one lien period in ten thousand units(‱).
		#[pallet::constant]
		type LendingRates: Get<BalanceOf<Self>>;
		/// Billing cycle of bank to calculate bill.
		#[pallet::constant]
		type LienBillingPeriod: Get<Self::BlockNumber>;
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
	pub type LienStore<T: Config> =
		StorageMap<_, Twox64Concat, AssetUniqueId, Lien<T::AccountId, T::BlockNumber>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn user_lien_store)]
	pub type UserLienStore<T: Config> = StorageDoubleMap<
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
	}
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				operation_account: Default::default(),
			}
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			OperationAccount::<T>::set(self.operation_account.clone());
		}
	}

	#[pallet::event]
	pub enum Event<T: Config> {}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Lien already exists that cannot be pawn again.
		LienAlreadyExists,
		/// Bank have not enough free balance to pay the user.
		InsufficientBalanceToPay,
		/// The given asset id not exist in asset store.
		AssetNotExists,
		/// Asset not belongs to user.
		InvalidAssetUser,
		/// Lien has expired.
		LienHasExpired,
		/// User have not enough free balance to redeem asset.
		InsufficientRedeemBalance,
		/// Close height should larger equal than current height.
		InvalidCloseHeight,
		/// Should pawn cml with frozen seed status.
		ShouldPawnFrozenSeed,
		/// Only allowed pawn genesis seed .
		ShouldPawnGenesisSeed,
		/// Lien store not empty cannot shutdown.
		LienStoreNotEmpty,
		/// User lien store not empty cannot shutdown.
		UserLienStoreNotEmpty,
		/// Asset id convert to cml id with invalid length.
		ConvertToCmlIdLengthMismatch,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: BlockNumberFor<T>) {
			if Self::is_lien_billing_period_end(n) {
				Self::try_clean_expired_lien();
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
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
		pub fn shutdown_all(sender: OriginFor<T>) -> DispatchResult {
			let root = ensure_root(sender)?;

			extrinsic_procedure(
				&root,
				|_root| {
					ensure!(
						LienStore::<T>::iter().count() == 0,
						Error::<T>::LienStoreNotEmpty
					);
					ensure!(
						UserLienStore::<T>::iter().count() == 0,
						Error::<T>::UserLienStoreNotEmpty
					);
					Ok(())
				},
				|_root| {
					let balance =
						T::CurrencyOperations::free_balance(&OperationAccount::<T>::get());
					T::CurrencyOperations::slash(&OperationAccount::<T>::get(), balance);
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn pawn_asset_to_genesis_bank(
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
						LienStore::<T>::contains_key(&unique_id),
						Error::<T>::LienAlreadyExists
					);
					Self::check_pawn_asset(&unique_id, who)
				},
				|who| {
					Self::create_new_lien(&unique_id, who);
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn redeem_asset(
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
				|who| Self::check_redeem_asset(&unique_id, who),
				|who| Self::redeem_asset_inner(&unique_id, who),
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
