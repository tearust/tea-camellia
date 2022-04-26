#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.  /// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use bonding_curve::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod functions;

use codec::Encode;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement},
};
use frame_system::pallet_prelude::*;
use pallet_utils::{extrinsic_procedure, CurrencyOperations};
use sp_core::H256;
use sp_io::hashing::blake2_256;
use sp_std::prelude::*;

/// The balance type of this module.
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod bonding_curve {
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
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn withdraw_storage)]
	pub(crate) type WithdrawStorage<T: Config> =
		StorageMap<_, Twox64Concat, H256, T::BlockNumber, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn consume_storage)]
	pub(crate) type ConsumeStorage<T: Config> =
		StorageMap<_, Twox64Concat, H256, T::BlockNumber, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub reserved_balance_account: T::AccountId,
		pub npc_account: T::AccountId,
		pub user_create_tapp: bool,
	}
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				reserved_balance_account: Default::default(),
				npc_account: Default::default(),
				user_create_tapp: false,
			}
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub fn deposit_event)]
	pub enum Event<T: Config> {
		/// Fired after topuped successfully, event parameters:
		/// 2. From account
		/// 3. To account
		/// 4. Topup amount
		/// 5. Curent block number
		TAppTopup(T::AccountId, T::AccountId, BalanceOf<T>, T::BlockNumber),

		/// Fired after topuped successfully, event parameters:
		/// 2. From account
		/// 3. To account
		/// 4. Topup amount
		/// 5. Curent block number
		/// 6. Tsid
		TAppWithdraw(
			T::AccountId,
			T::AccountId,
			BalanceOf<T>,
			T::BlockNumber,
			Vec<u8>,
		),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// TEA free balance is not enough
		InsufficientFreeBalance,
		/// Withdraw tsid already exist
		WithdrawTsidAlreadyExist,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(_n: BlockNumberFor<T>) {}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// This is basically a normal transfer balance extrinsic, except emit a topup event
		#[pallet::weight(195_000_000)]
		pub fn topup(
			sender: OriginFor<T>,
			tapp_operation_account: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						T::CurrencyOperations::free_balance(who) >= amount,
						Error::<T>::InsufficientFreeBalance
					);
					Ok(())
				},
				|who| {
					if let Err(e) = T::CurrencyOperations::transfer(
						who,
						&tapp_operation_account,
						amount,
						ExistenceRequirement::AllowDeath,
					) {
						// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
						log::error!("tapp topup transfer free balance failed: {:?}", e);
						return;
					}

					let current_height = frame_system::Pallet::<T>::block_number();
					Self::deposit_event(Event::TAppTopup(
						who.clone(),
						tapp_operation_account.clone(),
						amount,
						current_height,
					));
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn withdraw(
			sender: OriginFor<T>,
			to_account: T::AccountId,
			amount: BalanceOf<T>,
			tsid: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			let withdraw_hash = Self::tsid_hash(&tsid);
			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						!WithdrawStorage::<T>::contains_key(&withdraw_hash),
						Error::<T>::WithdrawTsidAlreadyExist
					);
					ensure!(
						T::CurrencyOperations::free_balance(who) >= amount,
						Error::<T>::InsufficientFreeBalance
					);
					Ok(())
				},
				|who| {
					if let Err(e) = T::CurrencyOperations::transfer(
						who,
						&to_account,
						amount,
						ExistenceRequirement::AllowDeath,
					) {
						// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
						log::error!("tapp withdraw transfer free balance failed: {:?}", e);
						return;
					}

					let current_block = frame_system::Pallet::<T>::block_number();
					WithdrawStorage::<T>::insert(&withdraw_hash, current_block);

					Self::deposit_event(Event::TAppWithdraw(
						who.clone(),
						to_account,
						amount,
						current_block,
						tsid,
					));
				},
			)
		}
	}
}
