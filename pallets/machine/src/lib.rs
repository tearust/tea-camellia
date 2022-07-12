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
mod weights;

use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::Currency};
use frame_system::pallet_prelude::*;
use pallet_utils::{extrinsic_procedure, CommonUtils, CurrencyOperations};
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

		#[pallet::constant]
		type ConnIdLength: Get<u32>;

		#[pallet::constant]
		type IpAddressLength: Get<u32>;

		#[pallet::constant]
		type StartupMachineBindingsLength: Get<u32>;

		#[pallet::constant]
		type StartupTappBindingsLength: Get<u32>;

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
	#[pallet::getter(fn last_issuer_id)]
	pub type LastIssuerId<T: Config> = StorageValue<_, IssuerId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn issuers)]
	pub(super) type Issuers<T: Config> =
		StorageMap<_, Twox64Concat, IssuerId, Issuer<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn issuer_owners)]
	pub(super) type IssuerOwners<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, IssuerId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn machines)]
	pub(super) type Machines<T: Config> =
		StorageMap<_, Twox64Concat, TeaPubKey, Machine<T::AccountId>>;

	#[pallet::storage]
	#[pallet::getter(fn machine_bindings)]
	pub(super) type MachineBindings<T: Config> =
		StorageMap<_, Twox64Concat, TeaPubKey, CmlId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn startup_machine_bindings)]
	pub(super) type StartupMachineBindings<T: Config> = StorageValue<
		_,
		BoundedVec<
			(TeaPubKey, CmlId, BoundedVec<u8, T::ConnIdLength>),
			T::StartupMachineBindingsLength,
		>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn startup_bonding_bindings)]
	pub(super) type StartupTappBindings<T: Config> = StorageValue<
		_,
		BoundedVec<
			(TeaPubKey, CmlId, BoundedVec<u8, T::IpAddressLength>),
			T::StartupTappBindingsLength,
		>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn startup_owner)]
	pub(super) type StartupOwner<T: Config> = StorageValue<_, T::AccountId>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Params:
		/// 1. tea_id
		/// 2. from account
		/// 3. to account
		MachineTransfered(TeaPubKey, T::AccountId, T::AccountId),

		/// Params:
		/// 1. tea_id
		/// 2. cml id
		/// 3. owner
		Layer2InfoBinded(TeaPubKey, CmlId, T::AccountId),

		/// Params:
		/// 1. tea_id
		/// 2. cml_id
		/// 3. conn id
		/// 4. old tea id
		/// 5. old cml id
		/// 6. at height
		MachineStartupReset(
			Vec<TeaPubKey>,
			Vec<CmlId>,
			Vec<Vec<u8>>,
			Vec<TeaPubKey>,
			Vec<CmlId>,
			T::BlockNumber,
		),

		/// Params:
		/// 1. tea_id
		/// 2. cml_id
		/// 3. ip address
		/// 4. old tea id
		/// 5. old cml id
		/// 6. at height
		TappStartupReset(
			Vec<TeaPubKey>,
			Vec<CmlId>,
			Vec<Vec<u8>>,
			Vec<TeaPubKey>,
			Vec<CmlId>,
			T::BlockNumber,
		),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The account has been registered that can't be used to register again
		IssuerOwnerRegistered,
		/// The given issuer not exist
		IssuerNotExist,
		/// The given issuer owner is invalid
		InvalidIssuerOwner,
		/// The given machine id is already exist
		MachineAlreadyExist,
		/// The given machine id is not exist
		MachineNotExist,
		/// Machine owner is not valid
		InvalidMachineOwner,
		/// Length of given lists not the same
		BindingItemsLengthMismatch,
		ConnIdLengthToLong,
		IpAddressLengthToLong,
		StartupMachineBindingsLengthToLong,
		StartupTappBindingsLengthToLong,
		StartupOwnerIsNone,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub startup_owner: Option<T::AccountId>,
		pub startup_machine_bindings: Vec<(TeaPubKey, CmlId, Vec<u8>)>,
		pub startup_tapp_bindings: Vec<(TeaPubKey, CmlId, Vec<u8>)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				startup_owner: Default::default(),
				startup_machine_bindings: Default::default(),
				startup_tapp_bindings: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			StartupOwner::<T>::set(self.startup_owner.clone());

			let owner = self.startup_owner.clone().unwrap();
			self.startup_machine_bindings
				.iter()
				.for_each(|(tea_id, cml_id, _)| {
					Machines::<T>::insert(
						tea_id,
						Machine {
							tea_id: *tea_id,
							issuer_id: BUILTIN_ISSURE,
							owner: owner.clone(),
						},
					);
					MachineBindings::<T>::insert(tea_id, cml_id);
				});
			StartupMachineBindings::<T>::set(
				self.startup_machine_bindings
					.clone()
					.into_iter()
					.map(|(tea_id, cml_id, conn_id)| (tea_id, cml_id, conn_id.try_into().unwrap()))
					.collect::<Vec<(TeaPubKey, CmlId, BoundedVec<u8, _>)>>()
					.try_into()
					.unwrap(),
			);

			self.startup_tapp_bindings
				.iter()
				.for_each(|(tea_id, cml_id, _)| {
					Machines::<T>::insert(
						tea_id,
						Machine {
							tea_id: *tea_id,
							issuer_id: BUILTIN_ISSURE,
							owner: owner.clone(),
						},
					);
					MachineBindings::<T>::insert(tea_id, cml_id);
				});
			StartupTappBindings::<T>::set(
				self.startup_tapp_bindings
					.clone()
					.into_iter()
					.map(|(tea_id, cml_id, ip)| (tea_id, cml_id, ip.try_into().unwrap()))
					.collect::<Vec<(TeaPubKey, CmlId, BoundedVec<u8, _>)>>()
					.try_into()
					.unwrap(),
			);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(195_000_000)]
		pub fn register_issuer(
			sender: OriginFor<T>,
			owner: T::AccountId,
			_url: Vec<u8>,
		) -> DispatchResult {
			let root = ensure_root(sender)?;

			extrinsic_procedure(
				&root,
				|_| {
					ensure!(
						!IssuerOwners::<T>::contains_key(&owner),
						Error::<T>::IssuerOwnerRegistered
					);
					Ok(())
				},
				|_| {
					let new_id = Self::next_id();
					Issuers::<T>::insert(
						new_id,
						Issuer {
							id: new_id,
							owner: owner.clone(),
						},
					);
					IssuerOwners::<T>::insert(owner.clone(), new_id);
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn register_machine(
			sender: OriginFor<T>,
			tea_id: TeaPubKey,
			owner: T::AccountId,
			issuer_id: IssuerId,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						Issuers::<T>::contains_key(issuer_id),
						Error::<T>::IssuerNotExist
					);
					ensure!(
						Issuers::<T>::get(issuer_id).unwrap().owner.eq(who),
						Error::<T>::InvalidIssuerOwner
					);
					ensure!(
						!Machines::<T>::contains_key(tea_id),
						Error::<T>::MachineAlreadyExist
					);
					Ok(())
				},
				|_| {
					Machines::<T>::insert(
						tea_id,
						Machine {
							tea_id,
							issuer_id,
							owner,
						},
					)
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn transfer_machine(
			sender: OriginFor<T>,
			tea_id: TeaPubKey,
			to_account: T::AccountId,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						Machines::<T>::contains_key(tea_id),
						Error::<T>::MachineNotExist
					);
					ensure!(
						Machines::<T>::get(tea_id).unwrap().owner.eq(who),
						Error::<T>::InvalidMachineOwner
					);
					Ok(())
				},
				|who| {
					Machines::<T>::mutate(tea_id, |machine| {
						if let Some(machine) = machine {
							machine.owner = to_account.clone();
						}
					});

					Self::deposit_event(Event::MachineTransfered(tea_id, who.clone(), to_account));
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn register_for_layer2(
			sender: OriginFor<T>,
			tea_id: TeaPubKey,
			cml_id: u64,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						Machines::<T>::contains_key(tea_id),
						Error::<T>::MachineNotExist
					);
					ensure!(
						Machines::<T>::get(tea_id).unwrap().owner.eq(who),
						Error::<T>::InvalidMachineOwner
					);
					Ok(())
				},
				|who| {
					MachineBindings::<T>::insert(tea_id, cml_id);
					Self::deposit_event(Event::Layer2InfoBinded(tea_id, cml_id, who.clone()));
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn reset_mining_startup(
			sender: OriginFor<T>,
			tea_ids: Vec<TeaPubKey>,
			cml_ids: Vec<u64>,
			conn_ids: Vec<Vec<u8>>,
		) -> DispatchResult {
			let root = ensure_root(sender)?;

			let tea_ids_len = tea_ids.len();
			let cml_ids_len = cml_ids.len();
			let conn_ids_len = conn_ids.len();
			let conn_ids_lens: Vec<u32> = conn_ids.iter().map(|id| id.len() as u32).collect();
			extrinsic_procedure(
				&root,
				|_| {
					ensure!(
						StartupOwner::<T>::get().is_some(),
						Error::<T>::StartupOwnerIsNone,
					);
					ensure!(
						tea_ids_len == cml_ids_len,
						Error::<T>::BindingItemsLengthMismatch
					);
					ensure!(
						tea_ids_len == conn_ids_len,
						Error::<T>::BindingItemsLengthMismatch
					);
					ensure!(
						tea_ids_len as u32 <= T::StartupMachineBindingsLength::get(),
						Error::<T>::StartupMachineBindingsLengthToLong
					);
					for len in conn_ids_lens {
						ensure!(
							len <= T::ConnIdLength::get(),
							Error::<T>::ConnIdLengthToLong
						);
					}
					Ok(())
				},
				move |_| {
					StartupMachineBindings::<T>::get()
						.iter()
						.for_each(|(tea_id, _, _)| {
							Machines::<T>::remove(tea_id);
							MachineBindings::<T>::remove(tea_id);
						});

					let owner = StartupOwner::<T>::get().unwrap();
					let mut startups = Vec::new();
					for i in 0..tea_ids.len() {
						Machines::<T>::insert(
							tea_ids[i],
							Machine {
								tea_id: tea_ids[i],
								issuer_id: BUILTIN_ISSURE,
								owner: owner.clone(),
							},
						);
						MachineBindings::<T>::insert(tea_ids[i], cml_ids[i]);
						startups.push((
							tea_ids[i],
							cml_ids[i],
							conn_ids[i].clone().try_into().unwrap(),
						));
					}
					let old_bindings = StartupMachineBindings::<T>::get();
					StartupMachineBindings::<T>::set(startups.try_into().unwrap());

					let mut old_tea_ids = vec![];
					let mut old_cml_ids = vec![];
					for (tea_id, cml_id, _) in old_bindings {
						old_tea_ids.push(tea_id);
						old_cml_ids.push(cml_id);
					}

					let current_block = frame_system::Pallet::<T>::block_number();
					Self::deposit_event(Event::MachineStartupReset(
						tea_ids,
						cml_ids,
						conn_ids,
						old_tea_ids,
						old_cml_ids,
						current_block,
					));
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn reset_tapp_startup(
			sender: OriginFor<T>,
			tea_ids: Vec<TeaPubKey>,
			cml_ids: Vec<u64>,
			ip_list: Vec<Vec<u8>>,
		) -> DispatchResult {
			let root = ensure_root(sender)?;

			let tea_ids_len = tea_ids.len();
			let cml_ids_len = cml_ids.len();
			let ip_list_len = ip_list.len();
			let ip_address_len: Vec<u32> = ip_list.iter().map(|ip| ip.len() as u32).collect();
			extrinsic_procedure(
				&root,
				|_| {
					ensure!(
						StartupOwner::<T>::get().is_some(),
						Error::<T>::StartupOwnerIsNone,
					);
					ensure!(
						tea_ids_len == cml_ids_len,
						Error::<T>::BindingItemsLengthMismatch
					);
					ensure!(
						tea_ids_len == ip_list_len,
						Error::<T>::BindingItemsLengthMismatch
					);
					ensure!(
						tea_ids_len as u32 <= T::StartupTappBindingsLength::get(),
						Error::<T>::StartupTappBindingsLengthToLong
					);
					for ip_len in ip_address_len {
						ensure!(
							ip_len <= T::IpAddressLength::get(),
							Error::<T>::IpAddressLengthToLong
						);
					}
					Ok(())
				},
				move |_| {
					StartupTappBindings::<T>::get()
						.iter()
						.for_each(|(tea_id, _, _)| {
							Machines::<T>::remove(tea_id);
							MachineBindings::<T>::remove(tea_id);
						});

					let owner = StartupOwner::<T>::get().unwrap();
					let mut startups = Vec::new();
					for i in 0..tea_ids.len() {
						Machines::<T>::insert(
							tea_ids[i],
							Machine {
								tea_id: tea_ids[i],
								issuer_id: BUILTIN_ISSURE,
								owner: owner.clone(),
							},
						);
						MachineBindings::<T>::insert(tea_ids[i], cml_ids[i]);
						startups.push((
							tea_ids[i],
							cml_ids[i],
							ip_list[i].clone().try_into().unwrap(),
						));
					}
					let old_bindings = StartupTappBindings::<T>::get();
					StartupTappBindings::<T>::set(startups.try_into().unwrap());

					let mut old_tea_ids = vec![];
					let mut old_cml_ids = vec![];
					for (tea_id, cml_id, _) in old_bindings {
						old_tea_ids.push(tea_id);
						old_cml_ids.push(cml_id);
					}

					let current_block = frame_system::Pallet::<T>::block_number();
					Self::deposit_event(Event::TappStartupReset(
						tea_ids,
						cml_ids,
						ip_list,
						old_tea_ids,
						old_cml_ids,
						current_block,
					));
				},
			)
		}
	}
}
