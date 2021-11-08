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
mod group;
mod rpc;
mod types;
mod utils;
mod weights;

use frame_support::{
	dispatch::DispatchResult, pallet_prelude::*, sp_runtime::traits::Verify, traits::Currency,
};
use frame_system::pallet_prelude::*;
use pallet_cml::{CmlOperation, CmlType, MinerStatus, SeedProperties, Task, TreeProperties};
use pallet_utils::{extrinsic_procedure, CommonUtils, CurrencyOperations};
use sp_core::{ed25519, H256};
use sp_io::hashing::blake2_256;
use sp_runtime::traits::{One, Saturating};
use sp_std::{
	collections::{btree_map::BTreeMap, btree_set::BTreeSet},
	prelude::*,
};
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

		#[pallet::constant]
		type PerRaTaskPoint: Get<u32>;

		#[pallet::constant]
		type UpdateValidatorsDuration: Get<Self::BlockNumber>;

		#[pallet::constant]
		type MaxGroupMemberCount: Get<u32>;

		#[pallet::constant]
		type MinGroupMemberCount: Get<u32>;

		#[pallet::constant]
		type MaxAllowedRaCommitDuration: Get<Self::BlockNumber>;

		#[pallet::constant]
		type PhishingAllowedDuration: Get<Self::BlockNumber>;

		#[pallet::constant]
		type TipsAllowedDuration: Get<Self::BlockNumber>;

		/// How long a offline evidence can be used to suspend a cml
		#[pallet::constant]
		type OfflineValidDuration: Get<Self::BlockNumber>;

		/// How many offline evidences can suspend a cml
		#[pallet::constant]
		type OfflineEffectThreshold: Get<u32>;

		#[pallet::constant]
		type ReportRawardDuration: Get<Self::BlockNumber>;

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

		/// Operations about currency that used in Tea Camellia.
		type CurrencyOperations: CurrencyOperations<
			AccountId = Self::AccountId,
			Balance = BalanceOf<Self>,
		>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// TEA node storage, key is TEA ID with type of `TeaPubKey`, value is a struct holding
	/// information about a TEA node.
	#[pallet::storage]
	#[pallet::getter(fn nodes)]
	pub(super) type Nodes<T: Config> =
		StorageMap<_, Twox64Concat, TeaPubKey, Node<T::BlockNumber>, ValueQuery>;

	/// Ephemeral ID map, key is Ephemeral ID with type of `TeaPubKey`, value is TEA ID with
	/// type of `TeaPubKey`.
	#[pallet::storage]
	#[pallet::getter(fn ephemeral_ids)]
	pub(super) type EphemeralIds<T: Config> =
		StorageMap<_, Twox64Concat, TeaPubKey, TeaPubKey, ValueQuery>;

	/// PeerId ID map, key is Peer ID with type of `PeerId`, value is TEA ID with type of
	/// `TeaPubKey`.
	#[pallet::storage]
	#[pallet::getter(fn peer_ids)]
	pub(super) type PeerIds<T: Config> = StorageMap<_, Twox64Concat, PeerId, TeaPubKey, ValueQuery>;

	/// Builtin nodes used to startup the RA process, key is TEA ID with type of `TeaPubKey`,
	/// value is an empty place holder.
	#[pallet::storage]
	#[pallet::getter(fn builtin_nodes)]
	pub(super) type BuiltinNodes<T: Config> =
		StorageMap<_, Twox64Concat, TeaPubKey, (), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn builtin_miners)]
	pub(super) type BuiltinMiners<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, (), ValueQuery>;

	/// Runtime activities of registered TEA nodes.
	#[pallet::storage]
	#[pallet::getter(fn runtime_activities)]
	pub(super) type RuntimeActivities<T: Config> =
		StorageMap<_, Twox64Concat, TeaPubKey, RuntimeActivity<T::BlockNumber>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn validators_collection)]
	pub(super) type ValidatorsCollection<T: Config> = StorageValue<_, Vec<TeaPubKey>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn validators_count)]
	pub(super) type ValidatorGroupsCount<T: Config> =
		StorageMap<_, Twox64Concat, u32, u32, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn report_evidences)]
	pub(super) type ReportEvidences<T: Config> =
		StorageMap<_, Twox64Concat, TeaPubKey, ReportEvidence<T::BlockNumber>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tips_evidences)]
	pub(super) type TipsEvidences<T: Config> =
		StorageMap<_, Twox64Concat, TeaPubKey, TipsEvidence<T::BlockNumber>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn offline_evidences)]
	pub(super) type OfflineEvidences<T: Config> =
		StorageMap<_, Twox64Concat, TeaPubKey, Vec<OfflineEvidence<T::BlockNumber>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn report_reward_amount)]
	pub(super) type ReportRawardAmount<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tips_reward_amount)]
	pub(super) type TipsRawardAmount<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn allowed_pcr_values)]
	pub(super) type AllowedPcrValues<T: Config> =
		StorageMap<_, Twox64Concat, H256, PcrSlots, ValueQuery>;

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

		/// Fired after RA validators changed.
		RaValidatorsChanged(Vec<TeaPubKey>),

		/// Statements items fields:
		/// - Reporter (reward owner)
		/// - Phisher
		/// - Reward amount
		ReportEvidencesStatements(Vec<(TeaPubKey, TeaPubKey, BalanceOf<T>)>),
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
		/// Node is not in RA validators list, so it is invalid to commit a RA result.
		NotRaValidator,
		/// Signature length not matched, that means signature is invalid.
		InvalidSignatureLength,
		/// Signature verify failed.
		InvalidSignature,
		/// User is not owner of the Tea ID.
		InvalidTeaIdOwner,
		/// User is not the built-in miner
		InvalidBuiltinMiner,
		/// Ra commit has expired.
		RaCommitExpired,
		/// Report node is not exist
		ReportNodeNotExist,
		/// Only B type cml can commit report
		OnlyBTypeCmlCanCommitReport,
		/// Only C type cml can report evidence
		OnlyCTypeCmlCanReport,
		/// Phishing report has been committedd multiple times
		RedundantReport,
		/// Phishing node not exist
		PhishingNodeNotExist,
		/// Phishing node is not in active state can't report again
		PhishingNodeNotActive,
		/// Report offline node is not exist
		OfflineNodeNotExist,
		/// Offline node is not in active state can't report again
		OfflineNodeNotActive,
		/// Phishing node can not commit report himself
		PhishingNodeCannotCommitReport,
		/// Type C cml is not allowed to phishing
		PhishingNodeCannotBeTypeC,
		/// Offline node can't be type C cml
		OfflineNodeCannotBeTypeC,
		/// Can not commit offline evidence multi time in short time
		CanNotCommitOfflineEvidenceMultiTimes,
		/// Tips has been committedd multiple times
		RedundantTips,
		/// The pcr has registered already
		PcrAlreadyExists,
		/// The pcr not registered so cannot unregister
		PcrNotExists,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: BlockNumberFor<T>) {
			if Self::should_update_validators(&n) {
				Self::update_runtime_status(n);
				Self::update_validators();
				group::update_validator_groups_count::<T>();
			}

			if Self::should_pay_report_reward(&n) {
				Self::pay_report_reward();
			}
		}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub builtin_nodes: Vec<TeaPubKey>,
		pub builtin_miners: Vec<T::AccountId>,
		pub report_reward_amount: BalanceOf<T>,
		pub tips_reward_amount: BalanceOf<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				builtin_nodes: Default::default(),
				builtin_miners: Default::default(),
				report_reward_amount: Default::default(),
				tips_reward_amount: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			ReportRawardAmount::<T>::set(self.report_reward_amount);
			TipsRawardAmount::<T>::set(self.tips_reward_amount);

			// we must ensure sufficient RA builtin nodes to start up.
			if self.builtin_nodes.len() < T::MinGroupMemberCount::get() as usize {
				panic!("insufficient builtin RA nodes");
			}

			for tea_id in self.builtin_nodes.iter() {
				let mut node = Node::default();
				node.tea_id = tea_id.clone();
				Nodes::<T>::insert(tea_id, node);
				BuiltinNodes::<T>::insert(tea_id, ());
			}

			self.builtin_miners
				.iter()
				.for_each(|account| BuiltinMiners::<T>::insert(account, ()));

			ValidatorsCollection::<T>::set(self.builtin_nodes.clone());
			group::update_validator_groups_count::<T>();
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(195_000_000)]
		pub fn set_reward_amount(
			sender: OriginFor<T>,
			report_reward: BalanceOf<T>,
			tips_reward: BalanceOf<T>,
		) -> DispatchResult {
			let root = ensure_root(sender)?;

			extrinsic_procedure(
				&root,
				|_| Ok(()),
				|_| {
					ReportRawardAmount::<T>::set(report_reward);
					TipsRawardAmount::<T>::set(tips_reward);
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn register_pcr(
			sender: OriginFor<T>,
			slots: Vec<PcrValue>,
			description: Vec<u8>,
		) -> DispatchResult {
			let root = ensure_root(sender)?;
			let hash = Self::pcr_slots_hash(&slots);

			extrinsic_procedure(
				&root,
				|_| {
					ensure!(
						!AllowedPcrValues::<T>::contains_key(&hash),
						Error::<T>::PcrAlreadyExists,
					);

					Ok(())
				},
				move |_| {
					AllowedPcrValues::<T>::insert(hash, PcrSlots { slots, description });
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn unregister_pcr(sender: OriginFor<T>, hash: H256) -> DispatchResult {
			let root = ensure_root(sender)?;

			extrinsic_procedure(
				&root,
				|_| {
					ensure!(
						AllowedPcrValues::<T>::contains_key(&hash),
						Error::<T>::PcrNotExists,
					);

					Ok(())
				},
				move |_| {
					AllowedPcrValues::<T>::remove(hash);
				},
			)
		}

		#[pallet::weight(T::WeightInfo::update_node_profile())]
		pub fn update_node_profile(
			origin: OriginFor<T>,
			tea_id: TeaPubKey,
			ephemeral_id: TeaPubKey,
			profile_cid: Cid,
			peer_id: PeerId,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			extrinsic_procedure(
				&sender,
				|sender| {
					ensure!(Nodes::<T>::contains_key(&tea_id), Error::<T>::NodeNotExist);
					ensure!(!peer_id.is_empty(), Error::<T>::InvalidPeerId);
					Self::check_tea_id_belongs(sender, &tea_id)?;
					Ok(())
				},
				|sender| {
					let old_node = Self::pop_existing_node(&tea_id);

					let current_block_number = frame_system::Pallet::<T>::block_number();
					let status = Self::initial_node_status(&tea_id);
					let node = Node {
						tea_id,
						ephemeral_id,
						profile_cid: profile_cid.clone(),
						ra_nodes: vec![],
						status,
						peer_id: peer_id.clone(),
						create_time: old_node.create_time,
						update_time: current_block_number,
					};
					Nodes::<T>::insert(&tea_id, &node);
					EphemeralIds::<T>::insert(ephemeral_id, &tea_id);
					PeerIds::<T>::insert(&peer_id, &tea_id);

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
				|sender| {
					ensure!(Nodes::<T>::contains_key(&tea_id), Error::<T>::NodeNotExist);
					ensure!(
						Nodes::<T>::contains_key(&target_tea_id),
						Error::<T>::ApplyNodeNotExist
					);
					Self::check_tea_id_belongs(sender, &tea_id)?;

					let target_node = Nodes::<T>::get(&target_tea_id);
					let current_block = frame_system::Pallet::<T>::block_number();
					ensure!(
						current_block
							<= target_node
								.update_time
								.saturating_add(T::MaxAllowedRaCommitDuration::get()),
						Error::<T>::RaCommitExpired
					);
					ensure!(
						Self::is_ra_validator(&tea_id, &target_tea_id, &target_node.update_time),
						Error::<T>::NotRaValidator
					);

					let my_node = Nodes::<T>::get(&tea_id);
					let content =
						crate::utils::encode_ra_request_content(&tea_id, &target_tea_id, is_pass);
					Self::verify_ed25519_signature(&my_node.ephemeral_id, &content, &signature)?;
					Ok(())
				},
				|sender| {
					let target_status = Self::update_node_status(&tea_id, &target_tea_id, is_pass);
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

		#[pallet::weight(195_000_000)]
		pub fn commit_tips_evidence(
			sender: OriginFor<T>,
			tea_id: TeaPubKey,
			report_tea_id: TeaPubKey,
			phishing_tea_id: TeaPubKey,
			signature: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;
			let current_height = frame_system::Pallet::<T>::block_number();

			extrinsic_procedure(
				&who,
				|who| {
					Self::check_type_c_evidence(
						who,
						&tea_id,
						&report_tea_id,
						&phishing_tea_id,
						&signature,
					)?;

					if TipsEvidences::<T>::contains_key(&report_tea_id) {
						ensure!(
							TipsEvidences::<T>::get(&report_tea_id)
								.height
								.saturating_add(T::TipsAllowedDuration::get())
								>= current_height.clone(),
							Error::<T>::RedundantTips,
						);
					}

					Ok(())
				},
				|_who| {
					TipsEvidences::<T>::insert(
						&report_tea_id,
						TipsEvidence {
							height: current_height,
							target: phishing_tea_id,
						},
					);
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn commit_report_evidence(
			sender: OriginFor<T>,
			tea_id: TeaPubKey,
			report_tea_id: TeaPubKey,
			phishing_tea_id: TeaPubKey,
			signature: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;
			let current_height = frame_system::Pallet::<T>::block_number();

			extrinsic_procedure(
				&who,
				|who| {
					Self::check_type_c_evidence(
						who,
						&tea_id,
						&report_tea_id,
						&phishing_tea_id,
						&signature,
					)?;

					if ReportEvidences::<T>::contains_key(&phishing_tea_id) {
						ensure!(
							ReportEvidences::<T>::get(&phishing_tea_id)
								.height
								.saturating_add(T::PhishingAllowedDuration::get())
								>= current_height.clone(),
							Error::<T>::RedundantReport
						);
					}
					let phishing_miner =
						T::CmlOperation::miner_item_by_machine_id(&phishing_tea_id);
					ensure!(
						phishing_miner.is_some()
							&& phishing_miner.unwrap().status == MinerStatus::Active,
						Error::<T>::PhishingNodeNotActive
					);

					Ok(())
				},
				|_who| {
					ReportEvidences::<T>::insert(
						&phishing_tea_id,
						ReportEvidence {
							height: current_height,
							reporter: report_tea_id,
						},
					);
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn commit_offline_evidence(
			sender: OriginFor<T>,
			tea_id: TeaPubKey,
			offline_tea_id: TeaPubKey,
			_signature: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;
			let current_height = frame_system::Pallet::<T>::block_number();

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(Nodes::<T>::contains_key(&tea_id), Error::<T>::NodeNotExist);
					ensure!(
						Nodes::<T>::contains_key(&offline_tea_id),
						Error::<T>::OfflineNodeNotExist
					);
					Self::check_tea_id_belongs(who, &tea_id)?;

					let offline_cml = T::CmlOperation::cml_by_machine_id(&offline_tea_id);
					ensure!(
						offline_cml.is_some() && offline_cml.unwrap().cml_type() != CmlType::C,
						Error::<T>::OfflineNodeCannotBeTypeC
					);

					let current_cml = T::CmlOperation::cml_by_machine_id(&tea_id);
					ensure!(
						current_cml.is_some() && current_cml.unwrap().cml_type() == CmlType::B,
						Error::<T>::OnlyBTypeCmlCanCommitReport
					);

					let offline_miner = T::CmlOperation::miner_item_by_machine_id(&offline_tea_id);
					ensure!(
						offline_miner.is_some()
							&& offline_miner.unwrap().status == MinerStatus::Active,
						Error::<T>::OfflineNodeNotActive
					);

					ensure!(
						!OfflineEvidences::<T>::get(&offline_tea_id)
							.iter()
							.any(|ev| {
								ev.tea_id.eq(&tea_id)
									&& ev.height.saturating_add(T::OfflineValidDuration::get())
										> current_height
							}),
						Error::<T>::CanNotCommitOfflineEvidenceMultiTimes
					);

					// todo check signature is signed by ephemeral key of tea_id

					Ok(())
				},
				|_who| {
					OfflineEvidences::<T>::mutate(&offline_tea_id, |evidences| {
						evidences.retain(|ev| {
							ev.height.saturating_add(T::OfflineValidDuration::get())
								> current_height
						});

						evidences.push(OfflineEvidence {
							height: current_height,
							tea_id: tea_id,
						});
					});
					Self::try_suspend_node(&offline_tea_id);
				},
			)
		}
	}
}
