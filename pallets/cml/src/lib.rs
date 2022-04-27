#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

mod functions;
pub mod generator;
mod rpc;
mod types;

pub use cml::*;
pub use param::*;
pub use types::*;

use frame_support::{pallet_prelude::*, traits::Currency};
use frame_system::pallet_prelude::*;
use pallet_utils::{CommonUtils, CurrencyOperations};
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_std::prelude::*;

/// The balance type of this module.
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod cml {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
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

	/// Used to allocate CML ID of new created DAO CML.
	#[pallet::storage]
	#[pallet::getter(fn last_cml_id)]
	pub type LastCmlId<T: Config> = StorageValue<_, CmlId, ValueQuery>;

	/// Storage of all valid CMLs, invalid CMLs (dead CML or fresh seed that over the fresh duration) will be
	/// cleaned every staking period begins.
	#[pallet::storage]
	#[pallet::getter(fn cml_store)]
	pub type CmlStore<T: Config> =
		StorageMap<_, Twox64Concat, CmlId, CML<T::AccountId, T::BlockNumber>, ValueQuery>;

	/// Double map about user and related cml ID of him.
	#[pallet::storage]
	#[pallet::getter(fn user_cml_store)]
	pub type UserCmlStore<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, CmlId, ()>;

	#[pallet::storage]
	#[pallet::getter(fn npc_account)]
	pub type NPCAccount<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub npc_account: T::AccountId,
		pub genesis_seeds: GenesisSeeds,
	}
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				npc_account: Default::default(),
				genesis_seeds: GenesisSeeds::default(),
			}
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			NPCAccount::<T>::set(self.npc_account.clone());
			crate::functions::init_from_genesis_seeds::<T>(
				&self.genesis_seeds,
				NPCAccount::<T>::get(),
			);
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Params:
		/// 1. cml id
		/// 2. from account
		/// 3. to account
		CmlTransfered(CmlId, T::AccountId, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Could not find CML in the cml store, indicates that the specified CML not existed.
		NotFoundCML,
		/// Trying to operate a CML not belongs to the user.
		CMLOwnerInvalid,
		/// Only NPC account can generate cml
		OnlyNPCAccountCanGenerateCml,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(_n: BlockNumberFor<T>) {}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(195_000_000)]
		pub fn generate_cml(sender: OriginFor<T>, a_amount: u32, b_amount: u32) -> DispatchResult {
			let who = ensure_signed(sender)?;

			pallet_utils::extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						who.eq(&NPCAccount::<T>::get()),
						Error::<T>::OnlyNPCAccountCanGenerateCml
					);
					Ok(())
				},
				|who| {
					let mut salt = vec![];
					salt.append(&mut a_amount.to_le_bytes().to_vec());
					salt.append(&mut b_amount.to_le_bytes().to_vec());

					let rand_value = sp_core::U256::from(
						T::CommonUtils::generate_random(who.clone(), &salt).as_bytes(),
					);
					let seeds = generator::construct_seeds(
						LastCmlId::<T>::get(),
						frame_support::Hashable::twox_256(&rand_value),
						a_amount as u64,
						b_amount as u64,
						0,
					);
					crate::functions::init_from_genesis_seeds::<T>(&seeds, NPCAccount::<T>::get());
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn transfer(
			sender: OriginFor<T>,
			cml_id: CmlId,
			to_account: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			pallet_utils::extrinsic_procedure(
				&who,
				|who| {
					ensure!(CmlStore::<T>::contains_key(cml_id), Error::<T>::NotFoundCML);
					ensure!(
						CmlStore::<T>::get(cml_id).owner().eq(who),
						Error::<T>::CMLOwnerInvalid
					);
					Ok(())
				},
				|who| {
					CmlStore::<T>::mutate(cml_id, |cml| {
						cml.set_owner(to_account.clone());
					});
					UserCmlStore::<T>::insert(to_account.clone(), cml_id, ());

					Self::deposit_event(Event::CmlTransfered(cml_id, who.clone(), to_account));
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn approve(
			sender: OriginFor<T>,
			_cml_id: CmlId,
			_proxy_account: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			pallet_utils::extrinsic_procedure(
				&who,
				|_who| Ok(()),
				|_who| {
					// todo complete me
				},
			)
		}
	}
}
