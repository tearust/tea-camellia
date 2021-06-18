#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

mod functions;
mod impl_stored_map;
mod types;
pub use types::*;

use log::info;
use sp_runtime::SaturatedConversion;
use sp_std::prelude::*;

use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
use frame_system::pallet_prelude::*;
// use codec::{HasCompact};
use frame_support::ensure;
use frame_support::traits::{Currency, Get};

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
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

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
	}
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				voucher_list: vec![],
			}
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (account, group, amount, lock, unlock_type) in self.voucher_list.iter() {
				let voucher = Voucher {
					amount: *amount,
					group: *group,
					lock: *lock,
					unlock_type: *unlock_type,
				};
				UserVoucherStore::<T>::insert(&account, group, voucher);
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
		// TODO
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
