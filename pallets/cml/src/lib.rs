#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

mod functions;
pub mod generator;
mod impl_stored_map;
mod types;
pub use types::*;

use frame_support::ensure;
use frame_support::traits::{Currency, Get};
use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_system::pallet_prelude::*;
use log::info;
use node_primitives::BlockNumber;
use sp_runtime::SaturatedConversion;
use sp_std::prelude::*;

pub use cml::*;

pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod cml {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId>;

		#[pallet::constant]
		type StakingPrice: Get<BalanceOf<Self>>;

		/// The latest block height to draw seeds use voucher, after this block height the left
		/// seeds shall be destroyed.
		type TimoutHeight: Get<BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn seeds)]
	pub(super) type Seeds<T: Config> = StorageMap<_, Twox64Concat, CmlId, Seed>;

	#[pallet::storage]
	#[pallet::getter(fn owner_seeds)]
	pub(super) type OwnerSeedsMap<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Vec<CmlId>>;

	#[pallet::storage]
	#[pallet::getter(fn seeds_cleaned)]
	pub(super) type SeedsCleaned<T: Config> = StorageValue<_, bool>;

	#[pallet::type_value]
	pub fn DefaultAssetId<T: Config>() -> CmlId {
		<CmlId>::saturated_from(10000_u32)
	}
	#[pallet::storage]
	pub type LastCmlId<T: Config> = StorageValue<_, CmlId, ValueQuery, DefaultAssetId<T>>;

	#[pallet::storage]
	#[pallet::getter(fn cml_store)]
	pub type CmlStore<T: Config> =
		StorageMap<_, Twox64Concat, CmlId, CML<T::AccountId, T::BlockNumber, BalanceOf<T>>>;

	#[pallet::storage]
	#[pallet::getter(fn user_cml_store)]
	pub type UserCmlStore<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, Vec<CmlId>>;

	#[pallet::storage]
	#[pallet::getter(fn voucher_user_store)]
	pub type UserVoucherStore<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, CmlType, Voucher>;

	#[pallet::storage]
	pub type MinerItemStore<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, MinerItem>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub voucher_list: Vec<(
			T::AccountId,
			CmlType,
			u32,
			Option<u32>,
			Option<VoucherUnlockType>,
		)>,
		pub genesis_seeds: GenesisSeeds,
	}
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				voucher_list: vec![],
				genesis_seeds: GenesisSeeds::default(),
			}
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (account, cml_type, amount, lock, unlock_type) in self.voucher_list.iter() {
				let voucher = Voucher {
					amount: *amount,
					group: *cml_type,
					lock: *lock,
					unlock_type: *unlock_type,
				};
				UserVoucherStore::<T>::insert(&account, cml_type, voucher);

				SeedsCleaned::<T>::set(Some(false));

				self.genesis_seeds.a_seeds.iter().for_each(|seed| {
					Seeds::<T>::insert(seed.id, seed.clone());
				});
				self.genesis_seeds.b_seeds.iter().for_each(|seed| {
					Seeds::<T>::insert(seed.id, seed.clone());
				});
				self.genesis_seeds.c_seeds.iter().for_each(|seed| {
					Seeds::<T>::insert(seed.id, seed.clone());
				});
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId", CmlId = "CmlId")]
	pub enum Event<T: Config> {
		ActiveCml(T::AccountId, CmlId),
	}

	#[pallet::error]
	pub enum Error<T> {
		NotEnoughVoucher,
		InvalidVoucherAmount,
		NotFoundCML,
		CMLNotLive,
		NotEnoughTeaToStaking,
		MinerAlreadyExist,
		CMLOwnerInvalid,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: BlockNumberFor<T>) {
			Self::try_clean_outdated_seeds(n);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)]
		pub fn transfer_voucher(
			sender: OriginFor<T>,
			target: T::AccountId,
			group: CmlType,
			#[pallet::compact] amount: u32,
		) -> DispatchResult {
			let sender = ensure_signed(sender)?;

			let sender_voucher =
				UserVoucherStore::<T>::get(&sender, group).ok_or(Error::<T>::NotEnoughVoucher)?;
			ensure!(
				sender_voucher.amount >= amount,
				Error::<T>::NotEnoughVoucher
			);

			let from_amount = sender_voucher
				.amount
				.checked_sub(amount)
				.ok_or(Error::<T>::InvalidVoucherAmount)?;

			if let Some(target_voucher) = UserVoucherStore::<T>::get(&target, group) {
				let to_amount = target_voucher
					.amount
					.checked_add(amount)
					.ok_or(Error::<T>::InvalidVoucherAmount)?;
				Self::set_voucher(&target, group, to_amount);
			} else {
				Self::set_voucher(&target, group, amount);
			}

			Self::set_voucher(&sender, group, from_amount);

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn convert_cml_from_voucher(
			sender: OriginFor<T>,
			group: CmlType,
			count: u32,
		) -> DispatchResult {
			let sender = ensure_signed(sender)?;

			let sender_voucher =
				UserVoucherStore::<T>::get(&sender, group).ok_or(Error::<T>::NotEnoughVoucher)?;
			ensure!(sender_voucher.amount >= count, Error::<T>::NotEnoughVoucher);

			let from_amount = sender_voucher
				.amount
				.checked_sub(count)
				.ok_or(Error::<T>::InvalidVoucherAmount)?;

			let list = Self::new_cml_from_voucher(CmlGroup::Nitro, count, group);
			Self::set_voucher(&sender, group, from_amount);

			for cml in list.iter() {
				Self::add_cml(&sender, cml.clone());
			}

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn active_cml_for_nitro(
			sender: OriginFor<T>,
			cml_id: CmlId,
			miner_id: Vec<u8>,
			miner_ip: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(sender)?;

			let _ = Self::get_cml_by_id(&cml_id)?;
			Self::check_belongs(&cml_id, &sender)?;

			let miner_item = MinerItem {
				id: miner_id.clone(),
				ip: miner_ip,
				status: MinerStatus::Active,
			};

			ensure!(
				!<MinerItemStore<T>>::contains_key(&miner_id),
				Error::<T>::MinerAlreadyExist
			);

			let balance = T::Currency::free_balance(&sender);

			let max_price: BalanceOf<T> = T::StakingPrice::get();
			ensure!(balance > max_price, Error::<T>::NotEnoughTeaToStaking);

			let staking_item = StakingItem {
				owner: sender.clone(),
				category: StakingCategory::Tea,
				amount: Some(T::StakingPrice::get()),
				cml: None,
			};

			Self::update_cml_to_active(&cml_id, miner_id.clone(), staking_item)?;
			<MinerItemStore<T>>::insert(&miner_id, miner_item);

			info!("TODO ---- lock balance");

			Self::deposit_event(Event::ActiveCml(sender.clone(), cml_id));
			Ok(())
		}
	}
}
