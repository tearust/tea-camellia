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

mod functions;
mod types;
mod utils;
mod weights;

use frame_support::{
	dispatch::DispatchResult, pallet_prelude::*, sp_runtime::traits::Verify, traits::Currency,
};
use frame_system::pallet_prelude::*;
use pallet_cml::{CmlOperation, Task};
use pallet_utils::{extrinsic_procedure, CommonUtils};
use sp_core::{ed25519, H256};
use sp_std::prelude::*;
use tea_interface::TeaOperation;

pub use types::*;
pub use weights::WeightInfo;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod tea {
	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId>;

		/// If node dot not update runtime activity within the given block heights, status of the
		/// node should become Inactive.
		#[pallet::constant]
		type RuntimeActivityThreshold: Get<u32>;
		/// The minimum number of RA result commit to let the candidate node status become active.
		#[pallet::constant]
		type MinRaPassedThreshold: Get<u32>;
		#[pallet::constant]
		type PerRaTaskPoint: Get<u32>;
		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
		/// Common utils trait
		type CommonUtils: CommonUtils<AccountId = Self::AccountId>;
		/// Operations type about task execution
		type TaskService: Task;

		type CmlOperation: CmlOperation<
			AccountId = Self::AccountId,
			Balance = BalanceOf<Self>,
			BlockNumber = Self::BlockNumber,
		>;
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

	#[pallet::storage]
	#[pallet::getter(fn builtin_miners)]
	pub(super) type BuiltinMiners<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, ()>;

	/// Runtime activities of registered TEA nodes.
	#[pallet::storage]
	#[pallet::getter(fn runtime_activities)]
	pub(super) type RuntimeActivities<T: Config> =
		StorageMap<_, Twox64Concat, TeaPubKey, RuntimeActivity<T::BlockNumber>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Fired after register node successfully.
		NewNodeJoined(T::AccountId, Node<T::BlockNumber>),

		/// Fired after update node profile successfully.
		UpdateNodeProfile(T::AccountId, Node<T::BlockNumber>),

		/// Fired after a RA node commit RA result successfully.
		CommitRaResult(T::AccountId, RaResult),

		/// Fired after a TEA node update runtime activity successfully.
		UpdateRuntimeActivity(T::AccountId, RuntimeActivity<T::BlockNumber>),
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
		/// Signature length not matched, that means signature is invalid.
		InvalidSignatureLength,
		/// Signature verify failed.
		InvalidSignature,
		/// User is not owner of the Tea ID.
		InvalidTeaIdOwner,
		/// User is not the built-in miner
		InvalidBuiltinMiner,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: BlockNumberFor<T>) {
			Self::update_runtime_status(n);
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub builtin_nodes: Vec<TeaPubKey>,
		pub builtin_miners: Vec<T::AccountId>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				builtin_nodes: Default::default(),
				builtin_miners: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for tea_id in self.builtin_nodes.iter() {
				let mut node = Node::default();
				node.tea_id = tea_id.clone();
				Nodes::<T>::insert(tea_id, node);
				BuiltinNodes::<T>::insert(tea_id, ());
			}

			self.builtin_miners
				.iter()
				.for_each(|account| BuiltinMiners::<T>::insert(account, ()));
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::WeightInfo::update_node_profile())]
		pub fn update_node_profile(
			origin: OriginFor<T>,
			tea_id: TeaPubKey,
			ephemeral_id: TeaPubKey,
			profile_cid: Cid,
			urls: Vec<Url>,
			peer_id: PeerId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			extrinsic_procedure(
				&sender,
				|sender| {
					ensure!(Nodes::<T>::contains_key(&tea_id), Error::<T>::NodeNotExist);
					ensure!(!peer_id.is_empty(), Error::<T>::InvalidPeerId);
					if !BuiltinNodes::<T>::contains_key(&tea_id) {
						ensure!(
							T::CmlOperation::check_miner(tea_id, sender),
							Error::<T>::InvalidTeaIdOwner
						);
					} else {
						ensure!(
							BuiltinMiners::<T>::contains_key(sender),
							Error::<T>::InvalidBuiltinMiner
						);
					}
					Ok(())
				},
				|sender| {
					let old_node = Self::pop_existing_node(&tea_id);
					let seed = T::CommonUtils::generate_random(sender.clone(), &tea_id.to_vec());

					let current_block_number = frame_system::Pallet::<T>::block_number();
					let urls_count = urls.len();
					let ra_nodes = Self::select_ra_nodes(&tea_id, seed);
					let status = Self::get_initial_node_status(&tea_id);
					let node = Node {
						tea_id,
						ephemeral_id,
						profile_cid: profile_cid.clone(),
						urls: urls.clone(),
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

					Self::deposit_event(Event::UpdateNodeProfile(sender.clone(), node));
				},
			)
		}

		#[pallet::weight(T::WeightInfo::remote_attestation())]
		pub fn remote_attestation(
			origin: OriginFor<T>,
			tea_id: TeaPubKey,
			target_tea_id: TeaPubKey,
			is_pass: bool,
			signature: Signature,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			extrinsic_procedure(
				&sender,
				|_sender| {
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

					let my_node = Nodes::<T>::get(&tea_id).unwrap();
					let content =
						crate::utils::encode_ra_request_content(&tea_id, &target_tea_id, is_pass);
					Self::verify_ed25519_signature(&my_node.ephemeral_id, &content, &signature)?;
					Ok(())
				},
				|sender| {
					let index = Self::get_index_in_ra_nodes(&tea_id, &target_tea_id);
					let target_status =
						Self::update_node_status(&target_tea_id, index.unwrap(), is_pass);
					T::TaskService::complete_ra_task(tea_id, T::PerRaTaskPoint::get());
					Self::deposit_event(Event::CommitRaResult(
						sender.clone(),
						RaResult {
							tea_id,
							target_tea_id,
							is_pass,
							target_status,
						},
					));
				},
			)
		}

		#[pallet::weight(T::WeightInfo::update_runtime_activity())]
		pub fn update_runtime_activity(
			origin: OriginFor<T>,
			tea_id: TeaPubKey,
			cid: Option<Cid>,
			ephemeral_id: TeaPubKey,
			signature: Signature,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			extrinsic_procedure(
				&sender,
				|_sender| {
					ensure!(Nodes::<T>::contains_key(&tea_id), Error::<T>::NodeNotExist);
					Self::verify_ed25519_signature(&ephemeral_id, &tea_id, &signature)?;
					Ok(())
				},
				|sender| {
					let runtime_activity = RuntimeActivity {
						tea_id,
						cid: cid.clone(),
						ephemeral_id,
						update_height: frame_system::Pallet::<T>::block_number(),
					};
					RuntimeActivities::<T>::insert(&tea_id, &runtime_activity);

					Self::deposit_event(Event::UpdateRuntimeActivity(
						sender.clone(),
						runtime_activity,
					));
				},
			)
		}
	}
}
