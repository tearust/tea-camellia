#![cfg_attr(not(feature = "std"), no_std)]

pub use traits::{CommonUtils, CurrencyOperations};
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use utils::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod functions;
pub mod traits;
mod types;
use frame_support::{
	dispatch::DispatchResultWithPostInfo,
	pallet_prelude::*,
	traits::{
		BalanceStatus, Currency, ExistenceRequirement, OnUnbalanced, Randomness, ReservableCurrency,
	},
	weights::Weight,
};
use frame_system::pallet_prelude::*;
use sp_core::H256;
use sp_io::hashing::blake2_256;
use sp_std::prelude::*;
use types::*;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod utils {
	use super::*;

	type PositiveImbalanceOf<T> = <<T as Config>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::PositiveImbalance;
	type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::NegativeImbalance;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The lockable currency type
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// Handler for the unbalanced increment when rewarding
		type Reward: OnUnbalanced<PositiveImbalanceOf<Self>>;
		/// Handler for the unbalanced decrement when slashing
		type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;

		type RandomnessSource: Randomness<H256, Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	pub enum Event<T: Config> {}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		InsufficientReservedBalance,
		InsufficientRepatriateBalance,
		MismatchedRepatriateBatchList,
		/// Generally this error should never happen, otherwise should check logic error.
		UnexpectedBalanceResult,
		AccountNotExist,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}

pub fn extrinsic_procedure<AccountId, CheckFn, SetFn>(
	who: &AccountId,
	ck_fn: CheckFn,
	set_fn: SetFn,
) -> DispatchResult
where
	CheckFn: Fn(&AccountId) -> DispatchResult,
	SetFn: FnOnce(&AccountId),
{
	ck_fn(who)?;
	set_fn(who);
	Ok(())
}

pub fn extrinsic_procedure_with_weight<AccountId, CheckFn, SetFn>(
	who: &AccountId,
	ck_fn: CheckFn,
	set_fn: SetFn,
) -> DispatchResultWithPostInfo
where
	CheckFn: Fn(&AccountId) -> DispatchResult,
	SetFn: Fn(&AccountId) -> Option<Weight>,
{
	ck_fn(who)?;
	Ok(set_fn(who).into())
}
