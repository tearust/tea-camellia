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
mod staking;
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
use pallet_utils::{CommonUtils, CurrencyOperations};
use sp_runtime::traits::{AtLeast32BitUnsigned, Zero};
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

		type CurrencyOperations: CurrencyOperations<
			AccountId = Self::AccountId,
			Balance = BalanceOf<Self>,
		>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn last_cml_id)]
	pub type LastCmlId<T: Config> = StorageValue<_, CmlId, ValueQuery>;

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
	#[pallet::getter(fn lucky_draw_box)]
	pub type LuckyDrawBox<T: Config> = StorageMap<_, Twox64Concat, CmlType, Vec<CmlId>, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub genesis_vouchers: GenesisVouchers<T::AccountId>,
		pub genesis_seeds: GenesisSeeds,
	}
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				genesis_vouchers: GenesisVouchers::default(),
				genesis_seeds: GenesisSeeds::default(),
			}
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			use crate::functions::convert_seeds_to_cmls;

			self.genesis_vouchers
				.vouchers
				.iter()
				.for_each(|voucher_config| {
					let voucher: Voucher = voucher_config.clone().into();
					UserVoucherStore::<T>::insert(
						&voucher_config.account,
						voucher_config.cml_type,
						voucher,
					);
				});

			let (a_cml_list, a_draw_box) =
				convert_seeds_to_cmls::<T::AccountId, T::BlockNumber, BalanceOf<T>>(
					&self.genesis_seeds.a_seeds,
				);
			let (b_cml_list, b_draw_box) =
				convert_seeds_to_cmls::<T::AccountId, T::BlockNumber, BalanceOf<T>>(
					&self.genesis_seeds.b_seeds,
				);
			let (c_cml_list, c_draw_box) =
				convert_seeds_to_cmls::<T::AccountId, T::BlockNumber, BalanceOf<T>>(
					&self.genesis_seeds.c_seeds,
				);
			LuckyDrawBox::<T>::insert(CmlType::A, a_draw_box);
			LuckyDrawBox::<T>::insert(CmlType::B, b_draw_box);
			LuckyDrawBox::<T>::insert(CmlType::C, c_draw_box);

			a_cml_list
				.iter()
				.chain(b_cml_list.iter())
				.chain(c_cml_list.iter())
				.for_each(|cml| CmlStore::<T>::insert(cml.id(), cml.clone()));

			LastCmlId::<T>::set(
				(self.genesis_seeds.a_seeds.len()
					+ self.genesis_seeds.b_seeds.len()
					+ self.genesis_seeds.c_seeds.len()) as CmlId,
			)
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

		DrawBoxNotInitialized,
		NotEnoughDrawSeeds,
		SeedsNotOutdatedYet,
		NoNeedToCleanOutdatedSeeds,

		NotFoundCML,
		CMLNotLive,
		CMLOwnerInvalid,
		ShouldStakingLiveSeed,

		MinerAlreadyExist,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: BlockNumberFor<T>) {
			Self::try_kill_cml(n);
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)]
		pub fn clean_outdated_seeds(sender: OriginFor<T>) -> DispatchResult {
			let _root = ensure_root(sender)?;

			let current_block = frame_system::Pallet::<T>::block_number();
			ensure!(
				current_block > T::TimoutHeight::get().into(),
				Error::<T>::SeedsNotOutdatedYet
			);
			ensure!(
				!Self::lucky_draw_box_all_empty(),
				Error::<T>::NoNeedToCleanOutdatedSeeds,
			);

			Self::try_clean_outdated_seeds(current_block);
			Ok(())
		}

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

			let mut seed_ids = Self::lucky_draw(&sender, a_coupon, b_coupon, c_coupon)?;
			let seeds_count = seed_ids.len() as u64;
			UserCmlStore::<T>::mutate(&sender, |ids| match ids {
				Some(ids) => ids.append(&mut seed_ids),
				None => *ids = Some(seed_ids),
			});

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

			ensure!(CmlStore::<T>::contains_key(cml_id), Error::<T>::NotFoundCML);
			Self::check_belongs(&cml_id, &sender)?;

			Self::init_miner_item(machine_id, miner_ip)?;

			let current_block_number = frame_system::Pallet::<T>::block_number();
			let staking_item = Self::create_balance_staking(&sender)?;
			Self::update_cml_to_active(
				&cml_id,
				machine_id.clone(),
				staking_item,
				current_block_number,
			)?;

			Self::deposit_event(Event::ActiveCml(sender.clone(), cml_id));
			Ok(())
		}
	}
}

pub trait CmlOperation {
	type AccountId: Clone;
	type Balance;
	type BlockNumber: Default + AtLeast32BitUnsigned + Clone;

	fn get_cml_by_id(
		cml_id: &CmlId,
	) -> Result<CML<Self::AccountId, Self::BlockNumber, Self::Balance>, DispatchError>;

	fn check_belongs(cml_id: &CmlId, who: &Self::AccountId) -> Result<(), DispatchError>;

	fn transfer_cml_other(
		from_account: &Self::AccountId,
		cml_id: &CmlId,
		target_account: &Self::AccountId,
	) -> Result<(), DispatchError>;
}
