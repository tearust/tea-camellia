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
mod rpc;
mod types;
mod utils;
mod weights;

use frame_support::{
	dispatch::DispatchResult, pallet_prelude::*, sp_runtime::traits::Verify, traits::Currency,
};
use frame_system::pallet_prelude::*;
use pallet_utils::{extrinsic_procedure, CommonUtils, CurrencyOperations};
use sp_core::{ed25519, H256};
use sp_io::hashing::blake2_256;
use sp_std::prelude::*;

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

		/// Common utils trait
		type CommonUtils: CommonUtils<AccountId = Self::AccountId>;

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

	/// Builtin nodes used to startup the RA process, key is TEA ID with type of `TeaPubKey`,
	/// value is an empty place holder.
	#[pallet::storage]
	#[pallet::getter(fn builtin_nodes)]
	pub(super) type BuiltinNodes<T: Config> =
		StorageMap<_, Twox64Concat, TeaPubKey, (), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tapp_store_startup_nodes)]
	pub(crate) type TappStoreStartupNodes<T: Config> = StorageValue<_, Vec<TeaPubKey>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Fired after register node successfully.
		NewNodeJoined(T::AccountId, Node<T::BlockNumber>),

		/// Event fields:
		/// - Sender
		/// - Node object
		/// - Old tea id
		/// - Event block height
		NodeIdChanged(
			T::AccountId,
			Node<T::BlockNumber>,
			TeaPubKey,
			T::BlockNumber,
		),

		/// Fired after update node profile successfully.
		UpdateNodeProfile(T::AccountId, Node<T::BlockNumber>),
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
		/// The pcr hash not in registered pcr list
		InvalidPcrHash,
		/// The versions has registered already
		VersionsAlreadyExists,
		/// The versions not registered so cannot unregister
		VersionsNotExist,
		/// The given ephemeral id not matched the ephemeral id registered
		/// when update node profile
		NodeEphemeralIdNotMatch,
		/// The size of version keys and values not match
		VersionKvpSizeNotMatch,
		/// Version expired height should larger than current height
		InvalidVersionExpireHeight,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: BlockNumberFor<T>) {}
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub builtin_nodes: Vec<TeaPubKey>,
		pub phantom: PhantomData<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				builtin_nodes: Default::default(),
				phantom: PhantomData,
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
			TappStoreStartupNodes::<T>::set(self.builtin_nodes.clone());
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(195_000_000)]
		pub fn reset_tapp_store_startup_nodes(
			sender: OriginFor<T>,
			nodes: Vec<TeaPubKey>,
		) -> DispatchResult {
			let root = ensure_root(sender)?;

			extrinsic_procedure(
				&root,
				|_| Ok(()),
				|_| {
					TappStoreStartupNodes::<T>::set(nodes);
				},
			)
		}
	}
}
