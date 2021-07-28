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
mod types;

use frame_support::{pallet_prelude::*, traits::Currency};
use frame_system::pallet_prelude::*;
use types::*;

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
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	pub enum Event<T: Config> {}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(195_000_000)]
		pub fn pawn_asset_to_genesis_bank(
			_sender: OriginFor<T>,
			_id: AssetId,
			_asset_type: AssetType,
		) -> DispatchResult {
			// todo implement me
			Ok(())
		}

		#[pallet::weight(195_000_000)]
		pub fn pay_off_for_asset(
			_sender: OriginFor<T>,
			_id: AssetId,
			_asset_type: AssetType,
		) -> DispatchResult {
			// todo implement me
			Ok(())
		}
	}
}
