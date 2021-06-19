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

use frame_support::{
	dispatch::DispatchResult,
	ensure,
	pallet_prelude::*,
	traits::{Currency, Get},
};
use frame_system::pallet_prelude::*;
use log::info;
use node_primitives::BlockNumber;
use pallet_utils::CommonUtils;
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
		/// Common utils trait
		type CommonUtils: CommonUtils<AccountId = Self::AccountId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

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
	pub type MinerItemStore<T: Config> = StorageMap<_, Twox64Concat, MachineId, MinerItem>;

	#[pallet::storage]
	#[pallet::getter(fn type_a_lucky_draw_box)]
	pub type TypeALuckyDrawBox<T: Config> = StorageValue<_, Vec<CmlId>>;

	#[pallet::storage]
	#[pallet::getter(fn type_b_lucky_draw_box)]
	pub type TypeBLuckyDrawBox<T: Config> = StorageValue<_, Vec<CmlId>>;

	#[pallet::storage]
	#[pallet::getter(fn type_c_lucky_draw_box)]
	pub type TypeCLuckyDrawBox<T: Config> = StorageValue<_, Vec<CmlId>>;

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
					cml_type: *cml_type,
					lock: *lock,
					unlock_type: *unlock_type,
				};
				UserVoucherStore::<T>::insert(&account, cml_type, voucher);

				SeedsCleaned::<T>::set(Some(false));

				let mut a_draw_box = Vec::new();
				self.genesis_seeds.a_seeds.iter().for_each(|seed| {
					CmlStore::<T>::insert(seed.id, CML::new(seed.clone()));
					a_draw_box.push(seed.id);
				});
				TypeALuckyDrawBox::<T>::set(Some(a_draw_box));

				let mut b_draw_box = Vec::new();
				self.genesis_seeds.b_seeds.iter().for_each(|seed| {
					CmlStore::<T>::insert(seed.id, CML::new(seed.clone()));
					b_draw_box.push(seed.id);
				});
				TypeBLuckyDrawBox::<T>::set(Some(b_draw_box));

				let mut c_draw_box = Vec::new();
				self.genesis_seeds.c_seeds.iter().for_each(|seed| {
					CmlStore::<T>::insert(seed.id, CML::new(seed.clone()));
					c_draw_box.push(seed.id);
				});
				TypeCLuckyDrawBox::<T>::set(Some(c_draw_box));
			}
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId", CmlId = "CmlId")]
	pub enum Event<T: Config> {
		DrawCmls(T::AccountId, u64),
		ActiveCml(T::AccountId, CmlId),
	}

	#[pallet::error]
	pub enum Error<T> {
		WithoutVoucher,
		NotEnoughVoucher,
		InvalidVoucherAmount,
		NotFoundCML,
		CMLNotLive,
		NotEnoughTeaToStaking,
		MinerAlreadyExist,
		CMLOwnerInvalid,
		NotEnoughDrawSeeds,
		DrawBoxNotInitialized,
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
		pub fn draw_cmls_from_voucher(sender: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(sender)?;

			let (a_coupon, b_coupon, c_coupon) = Self::take_vouchers(&sender);
			ensure!(
				a_coupon + b_coupon + c_coupon > 0,
				Error::<T>::WithoutVoucher
			);

			let seed_ids = Self::lucky_draw(&sender, a_coupon, b_coupon, c_coupon)?;
			let seeds_count = seed_ids.len() as u64;
			UserCmlStore::<T>::insert(&sender, seed_ids);

			Self::deposit_event(Event::DrawCmls(sender, seeds_count));
			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn active_cml_for_nitro(
			sender: OriginFor<T>,
			cml_id: CmlId,
			machine_id: MachineId,
			miner_ip: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(sender)?;

			let _ = Self::get_cml_by_id(&cml_id)?;
			Self::check_belongs(&cml_id, &sender)?;

			let miner_item = MinerItem {
				id: machine_id.clone(),
				ip: miner_ip,
				status: MinerStatus::Active,
			};

			ensure!(
				!<MinerItemStore<T>>::contains_key(&machine_id),
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

			let current_block_number = frame_system::Pallet::<T>::block_number();
			Self::update_cml_to_active(
				&cml_id,
				machine_id.clone(),
				staking_item,
				current_block_number,
			)?;
			<MinerItemStore<T>>::insert(&machine_id, miner_item);

			info!("TODO ---- lock balance");

			Self::deposit_event(Event::ActiveCml(sender.clone(), cml_id));
			Ok(())
		}
	}
}
