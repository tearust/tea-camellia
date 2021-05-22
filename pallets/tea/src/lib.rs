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

use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Randomness};
use frame_system::pallet_prelude::*;
use sp_core::U256;
use sp_io::hashing::blake2_256;
use sp_std::prelude::*;
use types::*;

#[frame_support::pallet]
pub mod tea {
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

    /// TEA node storage, key is TEA ID with type of `TeaPubKey`, value is a struct holding
    /// information about a TEA node.
    #[pallet::storage]
    #[pallet::getter(fn nodes)]
    pub(super) type Nodes<T: Config> = StorageMap<_, Twox64Concat, TeaPubKey, Node<T::BlockNumber>>;

    /// Bootstrap nodes set, key is TEA ID with type of `TeaPubKey`, value is an empty place holder.
    ///
    /// Bootstrap node must have public IP address, and url list should record how to access it.
    #[pallet::storage]
    #[pallet::getter(fn boot_nodes)]
    pub(super) type BootNodes<T: Config> = StorageMap<_, Twox64Concat, TeaPubKey, ()>;

    /// Ephemeral ID map, key is Ephemeral ID with type of `TeaPubKey`, value is TEA ID with
    /// type of `TeaPubKey`.
    #[pallet::storage]
    #[pallet::getter(fn ephemeral_ids)]
    pub(super) type EphemeralIds<T: Config> = StorageMap<_, Twox64Concat, TeaPubKey, TeaPubKey>;

    /// PeerId ID map, key is Peer ID with type of `PeerId`, value is TEA ID with type of
    /// `TeaPubKey`.
    #[pallet::storage]
    #[pallet::getter(fn peer_ids)]
    pub(super) type PeerIds<T: Config> = StorageMap<_, Twox64Concat, PeerId, TeaPubKey>;

    /// Builtin nodes used to startup the RA process, key is TEA ID with type of `TeaPubKey`,
    /// value is an empty place holder.
    #[pallet::storage]
    #[pallet::getter(fn builtin_nodes)]
    pub(super) type BuiltinNodes<T: Config> = StorageMap<_, Twox64Concat, TeaPubKey, ()>;

    #[pallet::event]
    #[pallet::metadata(T::AccountId = "AccountId", T::BlockNumber = "BlockNumber")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Fired after register node successfully.
        NewNodeJoined(T::AccountId, Node<T::BlockNumber>),

        /// Fired after update node profile successfully.
        UpdateNodeProfile(T::AccountId, Node<T::BlockNumber>),

        /// Fired after a RA node commit RA result successfully.
        CommitRaResult(T::AccountId, RaResult),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Node already registered.
        NodeAlreadyExist,
        /// Did not registered the node yet, should register node first.
        NodeNotExist,
        /// When commit RA result the apply node not registered yet, should register first.
        ApplyNodeNotExist,
        /// Peer ID should be a valid address about IPFS node.
        InvalidPeerId,
        /// Node is already activated. Because node will be activated after 3/4 RA nodes agreed,
        /// so the rest 1/4 node commit RA results later shall fail.
        NodeAlreadyActive,
        /// Node is not in RA nodes list, so it is invalid to commit a RA result.
        NotInRaNodes,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// add new node is an expensive operation to prevent abuse
        #[pallet::weight(10_000)]
        pub fn add_new_node(origin: OriginFor<T>, tea_id: TeaPubKey) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            ensure!(
                !Nodes::<T>::contains_key(&tea_id),
                Error::<T>::NodeAlreadyExist
            );
            let current_block_number = frame_system::Pallet::<T>::block_number();

            let mut new_node = Node::default();
            new_node.tea_id = tea_id.clone();
            new_node.create_time = current_block_number;
            new_node.update_time = current_block_number;
            Nodes::<T>::insert(tea_id, new_node.clone());

            Self::deposit_event(Event::NewNodeJoined(sender, new_node));
            Ok(())
        }

        /// update node profile is an expensive operation to prevent abuse
        #[pallet::weight(10_000)]
        pub fn update_node_profile(
            origin: OriginFor<T>,
            tea_id: TeaPubKey,
            ephemeral_id: TeaPubKey,
            profile_cid: Cid,
            urls: Vec<Url>,
            peer_id: PeerId,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            ensure!(Nodes::<T>::contains_key(&tea_id), Error::<T>::NodeNotExist);
            ensure!(!peer_id.is_empty(), Error::<T>::InvalidPeerId);

            let old_node = Self::pop_existing_node(&tea_id);
            let seed = Self::generate_random(sender.clone(), &tea_id);

            let current_block_number = frame_system::Pallet::<T>::block_number();
            let urls_count = urls.len();
            let ra_nodes = Self::select_ra_nodes(&tea_id, seed);
            let status = Self::get_initial_node_status(&tea_id);
            let node = Node {
                tea_id,
                ephemeral_id,
                profile_cid,
                urls,
                ra_nodes,
                status,
                peer_id: peer_id.clone(),
                create_time: old_node.create_time,
                update_time: current_block_number,
            };
            Nodes::<T>::insert(&tea_id, &node);
            EphemeralIds::<T>::insert(ephemeral_id, &tea_id);
            PeerIds::<T>::insert(&peer_id, &tea_id);
            if urls_count > 0 {
                BootNodes::<T>::insert(&tea_id, ());
            }

            Self::deposit_event(Event::UpdateNodeProfile(sender, node));
            Ok(())
        }

        #[pallet::weight(100)]
        pub fn remote_attestation(
            origin: OriginFor<T>,
            tea_id: TeaPubKey,
            target_tea_id: TeaPubKey,
            is_pass: bool,
            _signature: Signature,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // todo: verify signature
            ensure!(Nodes::<T>::contains_key(&tea_id), Error::<T>::NodeNotExist);
            ensure!(
                Nodes::<T>::contains_key(&target_tea_id),
                Error::<T>::ApplyNodeNotExist
            );
            let target_node = Nodes::<T>::get(&target_tea_id).unwrap();
            ensure!(
                target_node.status != NodeStatus::Active,
                Error::<T>::NodeAlreadyActive
            );

            let index = Self::get_index_in_ra_nodes(&tea_id, &target_tea_id);
            ensure!(index.is_some(), Error::<T>::NotInRaNodes);

            let target_status = Self::update_node_status(&target_tea_id, index.unwrap(), is_pass);
            Self::deposit_event(Event::CommitRaResult(
                sender,
                RaResult {
                    tea_id,
                    target_tea_id,
                    is_pass,
                    target_status,
                },
            ));
            Ok(())
        }
    }
}

impl<T: tea::Config> tea::Pallet<T> {
    fn pop_existing_node(tea_id: &TeaPubKey) -> Node<T::BlockNumber> {
        let old_node = Nodes::<T>::get(tea_id).unwrap();
        BootNodes::<T>::remove(&old_node.tea_id);
        EphemeralIds::<T>::remove(&old_node.ephemeral_id);
        PeerIds::<T>::remove(&old_node.peer_id);
        old_node
    }

    fn generate_random(sender: T::AccountId, tea_id: &TeaPubKey) -> U256 {
        let random_seed = <pallet_randomness_collective_flip::Module<T>>::random_seed();
        //pallet_randomness_collective_flip::Pallet::<T>::random_seed();
        let payload = (
            random_seed,
            sender.clone(),
            tea_id,
            frame_system::Pallet::<T>::block_number(),
        );
        payload.using_encoded(blake2_256).into()
    }

    pub(crate) fn is_builtin_node(tea_id: &TeaPubKey) -> bool {
        BuiltinNodes::<T>::get(tea_id).is_some()
    }

    fn get_initial_node_status(tea_id: &TeaPubKey) -> NodeStatus {
        match Self::is_builtin_node(tea_id) {
            true => NodeStatus::Active,
            false => NodeStatus::Pending,
        }
    }

    fn select_ra_nodes(tea_id: &TeaPubKey, _seed: U256) -> Vec<(TeaPubKey, bool)> {
        if Self::is_builtin_node(tea_id) {
            return Vec::new();
        }

        let mut ra_nodes = Vec::new();
        // todo: select 4 active nodes(calculate with `seed`) as ra nodes.
        for (tea_id, _) in BuiltinNodes::<T>::iter() {
            ra_nodes.push((tea_id, false));
        }
        ra_nodes
    }

    fn get_index_in_ra_nodes(tea_id: &TeaPubKey, target_tea_id: &TeaPubKey) -> Option<usize> {
        let target_node = Nodes::<T>::get(target_tea_id).unwrap();
        for i in 0..target_node.ra_nodes.len() {
            let (ra_tea_id, _) = target_node.ra_nodes[i];
            if ra_tea_id.eq(tea_id) {
                return Some(i);
            }
        }
        None
    }

    pub(crate) fn update_node_status(
        tea_id: &TeaPubKey,
        index: usize,
        is_pass: bool,
    ) -> NodeStatus {
        let mut target_node = Nodes::<T>::get(tea_id).unwrap();
        target_node.ra_nodes[index] = (tea_id.clone(), is_pass);
        let status = if is_pass {
            let approved_count = target_node
                .ra_nodes
                .iter()
                .filter(|(_, is_pass)| *is_pass)
                .count() as u32;
            // need 3/4 vote at least for now.
            if approved_count >= MIN_RA_PASSED_THRESHOLD {
                NodeStatus::Active
            } else {
                NodeStatus::Pending
            }
        } else {
            NodeStatus::Invalid
        };
        target_node.status = status.clone();
        Nodes::<T>::insert(tea_id, &target_node);

        status
    }
}
