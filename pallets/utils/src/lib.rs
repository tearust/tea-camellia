#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use utils::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod functions;
mod types;

use frame_support::{pallet_prelude::*, traits::Randomness};
use frame_system::pallet_prelude::*;
use sp_core::U256;
use sp_io::hashing::blake2_256;
use sp_std::prelude::*;
use types::*;

#[frame_support::pallet]
pub mod utils {
    use super::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
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
    impl<T: Config> Pallet<T> {}
}
