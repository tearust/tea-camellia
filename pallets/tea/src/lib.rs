#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use tea::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;

#[frame_support::pallet]
pub mod tea {
    use super::types::*;
    use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_std::prelude::*;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn nodes)]
    pub(super) type Nodes<T: Config> =
        StorageMap<_, Twox64Concat, TeaPubKey, Option<Node<T::BlockNumber>>>;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId", T::BlockNumber = "BlockNumber")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        NewNodeJoined(T::AccountId, Node<T::BlockNumber>),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Node already registered.
        NodeAlreadyExist,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(100)]
        pub fn add_new_node(origin: OriginFor<T>, tea_id: TeaPubKey) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            ensure!(
                !Nodes::<T>::contains_key(&tea_id),
                Error::<T>::NodeAlreadyExist
            );
            let current_block_number = <frame_system::Module<T>>::block_number();

            let new_node = Node {
                tea_id,
                ephemeral_id: [0u8; 32],
                profile_cid: Vec::new(),
                urls: Vec::new(),
                peer_id: Vec::new(),
                create_time: current_block_number,
                update_time: current_block_number,
                ra_nodes: Vec::new(),
                status: NodeStatus::Pending,
            };

            Nodes::<T>::insert(tea_id, Some(new_node.clone()));
            Self::deposit_event(Event::NewNodeJoined(sender, new_node));

            Ok(())
        }
    }
}
