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
use pallet_utils::{extrinsic_procedure, CommonUtils, CurrencyOperations};
use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedSub, Saturating, Zero};
use sp_std::{convert::TryInto, prelude::*};

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
		type VoucherTimoutHeight: Get<Self::BlockNumber>;

		type SeedFreshDuration: Get<Self::BlockNumber>;

		type StakingPeriodLength: Get<Self::BlockNumber>;

		/// Common utils trait
		type CommonUtils: CommonUtils<AccountId = Self::AccountId>;

		type CurrencyOperations: CurrencyOperations<
			AccountId = Self::AccountId,
			Balance = BalanceOf<Self>,
		>;

		type StakingEconomics: StakingEconomics<BalanceOf<Self>, AccountId = Self::AccountId>;

		type StakingSlotsMaxLength: Get<StakingIndex>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn last_cml_id)]
	pub type LastCmlId<T: Config> = StorageValue<_, CmlId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn cml_store)]
	pub type CmlStore<T: Config> = StorageMap<
		_,
		Twox64Concat,
		CmlId,
		CML<T::AccountId, T::BlockNumber, BalanceOf<T>, T::SeedFreshDuration>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn user_cml_store)]
	pub type UserCmlStore<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, CmlId, ()>;

	#[pallet::storage]
	#[pallet::getter(fn investor_user_store)]
	pub type InvestorVoucherStore<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, CmlType, Voucher>;

	#[pallet::storage]
	#[pallet::getter(fn team_user_store)]
	pub type TeamVoucherStore<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, CmlType, Voucher>;

	#[pallet::storage]
	#[pallet::getter(fn miner_item_store)]
	pub type MinerItemStore<T: Config> = StorageMap<_, Twox64Concat, MachineId, MinerItem>;

	#[pallet::storage]
	#[pallet::getter(fn miner_credit_store)]
	pub type GenesisMinerCreditStore<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn lucky_draw_box)]
	pub type LuckyDrawBox<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		CmlType,
		Twox64Concat,
		DefrostScheduleType,
		Vec<CmlId>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn active_staking_snapshot)]
	pub type ActiveStakingSnapshot<T: Config> =
		StorageMap<_, Twox64Concat, CmlId, Vec<StakingSnapshotItem<T::AccountId>>, ValueQuery>;

	#[pallet::storage]
	pub type AccountRewards<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

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
			use crate::functions::convert_genesis_seeds_to_cmls;

			self.genesis_vouchers
				.vouchers
				.iter()
				.for_each(|voucher_config| {
					let voucher: Voucher = voucher_config.clone().into();
					match voucher_config.schedule_type {
						DefrostScheduleType::Investor => InvestorVoucherStore::<T>::insert(
							&voucher_config.account,
							voucher_config.cml_type,
							voucher,
						),
						DefrostScheduleType::Team => TeamVoucherStore::<T>::insert(
							&voucher_config.account,
							voucher_config.cml_type,
							voucher,
						),
					}
				});

			let (a_cml_list, investor_a_draw_box, team_a_draw_box) = convert_genesis_seeds_to_cmls::<
				T::AccountId,
				T::BlockNumber,
				BalanceOf<T>,
				T::SeedFreshDuration,
			>(&self.genesis_seeds.a_seeds);
			let (b_cml_list, investor_b_draw_box, team_b_draw_box) = convert_genesis_seeds_to_cmls::<
				T::AccountId,
				T::BlockNumber,
				BalanceOf<T>,
				T::SeedFreshDuration,
			>(&self.genesis_seeds.b_seeds);
			let (c_cml_list, investor_c_draw_box, team_c_draw_box) = convert_genesis_seeds_to_cmls::<
				T::AccountId,
				T::BlockNumber,
				BalanceOf<T>,
				T::SeedFreshDuration,
			>(&self.genesis_seeds.c_seeds);
			LuckyDrawBox::<T>::insert(
				CmlType::A,
				DefrostScheduleType::Investor,
				investor_a_draw_box,
			);
			LuckyDrawBox::<T>::insert(CmlType::A, DefrostScheduleType::Team, team_a_draw_box);
			LuckyDrawBox::<T>::insert(
				CmlType::B,
				DefrostScheduleType::Investor,
				investor_b_draw_box,
			);
			LuckyDrawBox::<T>::insert(CmlType::B, DefrostScheduleType::Team, team_b_draw_box);
			LuckyDrawBox::<T>::insert(
				CmlType::C,
				DefrostScheduleType::Investor,
				investor_c_draw_box,
			);
			LuckyDrawBox::<T>::insert(CmlType::C, DefrostScheduleType::Team, team_c_draw_box);

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
		Staked(T::AccountId, CmlId, StakingIndex),
	}

	#[pallet::error]
	pub enum Error<T> {
		WithoutVoucher,
		NotEnoughVoucher,
		InvalidVoucherAmount,

		NotEnoughDrawSeeds,
		SeedsNotOutdatedYet,
		VouchersHasOutdated,
		NoNeedToCleanOutdatedSeeds,

		NotFoundCML,
		CMLOwnerInvalid,
		CmlIsNotSeed,
		SeedNotValid,

		InsufficientFreeBalance,
		InsufficientReservedBalance,
		MinerAlreadyExist,
		NotFoundMiner,
		InvalidCreditAmount,
		CannotTransferCmlWithCredit,
		InvalidMiner,
		InvalidMinerIp,

		InvalidStaker,
		InvalidStakee,
		InvalidStakingIndex,
		InvalidStakingOwner,
		InvalidUnstaking,
		NotFoundRewardAccount,
		StakingSlotsOverTheMaxLength,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: BlockNumberFor<T>) {
			if Self::is_staking_period_start(n) {
				Self::try_kill_cml(n);
				// initialize staking related
				Self::collect_staking_info();
			} else if Self::is_staking_period_end(n) {
				// calculate staking rewards
				Self::calculate_staking();
				Self::clear_staking_info();
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)]
		pub fn clean_outdated_seeds(sender: OriginFor<T>) -> DispatchResult {
			let root = ensure_root(sender)?;
			let current_block = frame_system::Pallet::<T>::block_number();

			extrinsic_procedure(
				&root,
				|_| {
					ensure!(
						Self::is_voucher_outdated(current_block),
						Error::<T>::SeedsNotOutdatedYet
					);
					ensure!(
						!Self::lucky_draw_box_all_empty(vec![
							DefrostScheduleType::Investor,
							DefrostScheduleType::Team
						]),
						Error::<T>::NoNeedToCleanOutdatedSeeds,
					);
					Ok(())
				},
				|_| {
					Self::try_clean_outdated_vouchers(current_block);
				},
			)
		}

		#[pallet::weight(1_000)]
		pub fn transfer_voucher(
			sender: OriginFor<T>,
			target: T::AccountId,
			cml_type: CmlType,
			schedule_type: DefrostScheduleType,
			#[pallet::compact] amount: u32,
		) -> DispatchResult {
			let sender = ensure_signed(sender)?;
			let sender_voucher = match schedule_type {
				DefrostScheduleType::Investor => InvestorVoucherStore::<T>::get(&sender, cml_type),
				DefrostScheduleType::Team => TeamVoucherStore::<T>::get(&sender, cml_type),
			};
			let target_voucher = match schedule_type {
				DefrostScheduleType::Investor => InvestorVoucherStore::<T>::get(&target, cml_type),
				DefrostScheduleType::Team => TeamVoucherStore::<T>::get(&target, cml_type),
			};

			extrinsic_procedure(
				&sender,
				|_| {
					ensure!(
						!Self::is_voucher_outdated(frame_system::Pallet::<T>::block_number()),
						Error::<T>::VouchersHasOutdated
					);
					ensure!(sender_voucher.is_some(), Error::<T>::NotEnoughVoucher);
					let sender_voucher = sender_voucher.as_ref().unwrap();
					ensure!(
						sender_voucher.amount >= amount,
						Error::<T>::NotEnoughVoucher
					);
					sender_voucher
						.amount
						.checked_sub(amount)
						.ok_or(Error::<T>::InvalidVoucherAmount)?;
					if let Some(target_voucher) = target_voucher.as_ref() {
						target_voucher
							.amount
							.checked_add(amount)
							.ok_or(Error::<T>::InvalidVoucherAmount)?;
					}

					Ok(())
				},
				|_| {
					if sender_voucher.is_none() {
						return;
					}
					let from_amount = sender_voucher
						.as_ref()
						.unwrap()
						.amount
						.saturating_sub(amount);

					match target_voucher.as_ref() {
						Some(target_voucher) => {
							let to_amount = target_voucher.amount.saturating_add(amount);
							Self::set_voucher(&target, cml_type, schedule_type, to_amount);
						}
						None => Self::set_voucher(&target, cml_type, schedule_type, amount),
					}

					Self::set_voucher(&sender, cml_type, schedule_type, from_amount);
				},
			)
		}

		#[pallet::weight(10_000)]
		pub fn draw_cmls_from_voucher(
			sender: OriginFor<T>,
			schedule_type: DefrostScheduleType,
		) -> DispatchResult {
			let sender = ensure_signed(sender)?;
			let (a_coupon, b_coupon, c_coupon) = Self::take_vouchers(&sender, schedule_type);

			extrinsic_procedure(
				&sender,
				|_| {
					ensure!(
						!Self::is_voucher_outdated(frame_system::Pallet::<T>::block_number()),
						Error::<T>::VouchersHasOutdated
					);
					ensure!(
						a_coupon + b_coupon + c_coupon > 0,
						Error::<T>::WithoutVoucher
					);
					Self::check_luck_draw(a_coupon, b_coupon, c_coupon, schedule_type)?;
					Ok(())
				},
				|sender| {
					let seed_ids =
						Self::lucky_draw(&sender, a_coupon, b_coupon, c_coupon, schedule_type);
					let seeds_count = seed_ids.len() as u64;
					seed_ids.iter().for_each(|id| {
						CmlStore::<T>::mutate(id, |cml| {
							cml.set_owner(&sender);
						});
						UserCmlStore::<T>::insert(&sender, id, ());
					});

					Self::deposit_event(Event::DrawCmls(sender.clone(), seeds_count));
				},
			)
		}

		#[pallet::weight(10_000)]
		pub fn active_cml(sender: OriginFor<T>, cml_id: CmlId) -> DispatchResult {
			let sender = ensure_signed(sender)?;
			let current_block_number = frame_system::Pallet::<T>::block_number();

			extrinsic_procedure(
				&sender,
				|sender| {
					Self::check_belongs(&cml_id, sender)?;
					Self::check_seed_validity(cml_id, &current_block_number)?;
					Ok(())
				},
				|sender| {
					CmlStore::<T>::mutate(cml_id, |cml| {
						cml.try_convert_to_tree(&current_block_number);
					});

					Self::deposit_event(Event::ActiveCml(sender.clone(), cml_id));
				},
			)
		}

		#[pallet::weight(10_000)]
		pub fn start_mining(
			sender: OriginFor<T>,
			cml_id: CmlId,
			machine_id: MachineId,
			miner_ip: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(sender)?;
			let current_block_number = frame_system::Pallet::<T>::block_number();

			extrinsic_procedure(
				&sender,
				|sender| {
					Self::check_belongs(&cml_id, &sender)?;
					ensure!(
						!<MinerItemStore<T>>::contains_key(&machine_id),
						Error::<T>::MinerAlreadyExist
					);
					Self::check_miner_ip_validity(&miner_ip)?;

					let cml = CmlStore::<T>::get(cml_id);
					ensure!(
						cml.can_start_mining(&current_block_number),
						Error::<T>::InvalidMiner
					);
					Self::check_miner_first_staking(&sender, &cml)?;

					Ok(())
				},
				|sender| {
					let ip = miner_ip.clone();
					CmlStore::<T>::mutate(cml_id, |cml| {
						let staking_item = if cml.is_from_genesis() {
							Self::create_genesis_miner_balance_staking(&sender)
						} else {
							Self::create_balance_staking(&sender, T::StakingPrice::get())
						};
						// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
						if staking_item.is_err() {
							return;
						}

						cml.start_mining(machine_id, staking_item.unwrap(), &current_block_number);
						MinerItemStore::<T>::insert(
							&machine_id,
							MinerItem {
								cml_id,
								ip,
								id: machine_id.clone(),
								status: MinerStatus::Active,
							},
						);
					});
				},
			)
		}

		#[pallet::weight(10_000)]
		pub fn stop_mining(
			sender: OriginFor<T>,
			cml_id: CmlId,
			machine_id: MachineId,
		) -> DispatchResult {
			let sender = ensure_signed(sender)?;

			extrinsic_procedure(
				&sender,
				|sender| {
					Self::check_belongs(&cml_id, &sender)?;
					let cml = CmlStore::<T>::get(&cml_id);
					ensure!(cml.is_mining(), Error::<T>::InvalidMiner);
					ensure!(
						MinerItemStore::<T>::contains_key(machine_id),
						Error::<T>::NotFoundMiner
					);
					Ok(())
				},
				|_sender| {
					CmlStore::<T>::mutate(cml_id, |cml| {
						cml.stop_mining();
					});
					MinerItemStore::<T>::remove(machine_id);
				},
			)
		}

		#[pallet::weight(10_000)]
		pub fn start_staking(
			sender: OriginFor<T>,
			staking_to: CmlId,
			staking_cml: Option<CmlId>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;
			let current_block_number = frame_system::Pallet::<T>::block_number();

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						CmlStore::<T>::contains_key(staking_to),
						Error::<T>::NotFoundCML
					);

					let amount = match staking_cml {
						Some(cml_id) => {
							Self::check_belongs(&cml_id, &who)?;
							let cml = CmlStore::<T>::get(cml_id);
							ensure!(
								cml.can_stake_to(&current_block_number),
								Error::<T>::InvalidStaker
							);
							None
						}
						None => {
							Self::check_balance_staking(&who)?;
							Some(T::StakingPrice::get())
						}
					};

					let cml = CmlStore::<T>::get(staking_to);
					ensure!(
						cml.staking_slots().len() <= T::StakingSlotsMaxLength::get() as usize,
						Error::<T>::StakingSlotsOverTheMaxLength
					);
					ensure!(
						cml.can_be_stake(&current_block_number, &amount, &staking_cml),
						Error::<T>::InvalidStakee
					);
					Ok(())
				},
				|who| {
					let staking_index: Option<StakingIndex> =
						CmlStore::<T>::mutate(staking_to, |cml| {
							Self::stake(who, cml, &staking_cml, &current_block_number)
						});

					Self::deposit_event(Event::Staked(
						who.clone(),
						staking_to,
						staking_index.unwrap_or(u64::MAX),
					));
				},
			)
		}

		#[pallet::weight(10_000)]
		pub fn stop_staking(
			sender: OriginFor<T>,
			staking_to: CmlId,
			staking_index: StakingIndex,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						CmlStore::<T>::contains_key(staking_to),
						Error::<T>::NotFoundCML
					);
					let cml = CmlStore::<T>::get(staking_to);
					ensure!(
						cml.staking_slots().len() > staking_index as usize,
						Error::<T>::InvalidStakingIndex,
					);
					let staking_item: &StakingItem<T::AccountId, BalanceOf<T>> = cml
						.staking_slots()
						.get(staking_index as usize)
						.ok_or(Error::<T>::InvalidStakingIndex)?;
					ensure!(staking_item.owner == *who, Error::<T>::InvalidStakingOwner);
					if let Some(cml_id) = staking_item.cml {
						Self::check_belongs(&cml_id, who)?;
					} else {
						ensure!(
							T::CurrencyOperations::reserved_balance(who) >= T::StakingPrice::get(),
							Error::<T>::InsufficientReservedBalance
						);
					}

					let (index, staking_cml) = match staking_item.cml {
						Some(cml_id) => (None, Some(CmlStore::<T>::get(cml_id))),
						None => (Some(staking_index), None),
					};
					ensure!(
						cml.can_unstake(&index, &staking_cml.as_ref()),
						Error::<T>::InvalidUnstaking
					);

					Ok(())
				},
				|who| {
					CmlStore::<T>::mutate(staking_to, |cml| {
						Self::unstake(who, cml, staking_index);
					});
				},
			)
		}

		#[pallet::weight(10_000)]
		pub fn withdraw_staking_reward(sender: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						AccountRewards::<T>::contains_key(who),
						Error::<T>::NotFoundRewardAccount
					);
					Ok(())
				},
				|who| {
					let balance = AccountRewards::<T>::get(who);
					T::CurrencyOperations::deposit_creating(&who, balance);
					AccountRewards::<T>::remove(who);
				},
			)
		}
	}
}

pub trait CmlOperation {
	type AccountId: PartialEq + Clone;
	type Balance: Clone;
	type BlockNumber: Default + AtLeast32BitUnsigned + Clone;
	type FreshDuration: Get<Self::BlockNumber>;

	fn cml_by_id(
		cml_id: &CmlId,
	) -> Result<
		CML<Self::AccountId, Self::BlockNumber, Self::Balance, Self::FreshDuration>,
		DispatchError,
	>;

	fn check_belongs(cml_id: &CmlId, who: &Self::AccountId) -> Result<(), DispatchError>;

	fn check_transfer_cml_to_other(
		from_account: &Self::AccountId,
		cml_id: &CmlId,
		target_account: &Self::AccountId,
	) -> DispatchResult;

	fn transfer_cml_to_other(
		from_account: &Self::AccountId,
		cml_id: &CmlId,
		target_account: &Self::AccountId,
	);

	fn cml_deposit_price(cml_id: &CmlId) -> Option<Self::Balance>;

	fn user_credit_amount(account_id: &Self::AccountId) -> Self::Balance;
}

pub trait StakingEconomics<Balance> {
	type AccountId: PartialEq + Clone;

	fn increase_issuance(total_point: ServiceTaskPoint) -> Balance;

	fn total_staking_rewards_of_miner(
		miner_point: ServiceTaskPoint,
		total_point: ServiceTaskPoint,
	) -> Balance;

	fn miner_staking_point(
		snapshots: &Vec<StakingSnapshotItem<Self::AccountId>>,
	) -> MinerStakingPoint;

	fn single_staking_reward(
		miner_total_rewards: Balance,
		total_staking_point: MinerStakingPoint,
		snapshot_item: &StakingSnapshotItem<Self::AccountId>,
	) -> Balance;
}
