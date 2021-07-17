#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

mod cml_operation;
mod functions;
pub mod generator;
mod rpc;
mod staking;
mod types;
mod weights;

pub use cml::*;
pub use types::*;
pub use weights::WeightInfo;

use frame_support::{
	dispatch::DispatchResult,
	ensure,
	pallet_prelude::*,
	traits::{Currency, Get},
};
use frame_system::pallet_prelude::*;
use pallet_utils::{
	extrinsic_procedure, extrinsic_procedure_with_weight, CommonUtils, CurrencyOperations,
};
use sp_runtime::traits::{AtLeast32BitUnsigned, Saturating, Zero};
use sp_std::prelude::*;

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

		/// The latest block height to draw seeds use coupon, after this block height the left
		/// seeds shall be destroyed.
		#[pallet::constant]
		type CouponTimoutHeight: Get<Self::BlockNumber>;

		#[pallet::constant]
		type SeedFreshDuration: Get<Self::BlockNumber>;

		#[pallet::constant]
		type StakingPeriodLength: Get<Self::BlockNumber>;

		/// Common utils trait
		type CommonUtils: CommonUtils<AccountId = Self::AccountId>;

		type CurrencyOperations: CurrencyOperations<
			AccountId = Self::AccountId,
			Balance = BalanceOf<Self>,
		>;

		type StakingEconomics: StakingEconomics<BalanceOf<Self>, Self::AccountId>;

		type WeightInfo: WeightInfo;

		#[pallet::constant]
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
	pub type InvestorCouponStore<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, CmlType, Coupon>;

	#[pallet::storage]
	#[pallet::getter(fn team_user_store)]
	pub type TeamCouponStore<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, CmlType, Coupon>;

	#[pallet::storage]
	#[pallet::getter(fn miner_item_store)]
	pub type MinerItemStore<T: Config> =
		StorageMap<_, Twox64Concat, MachineId, MinerItem, ValueQuery>;

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
	#[pallet::getter(fn mining_cml_task_points)]
	pub type MiningCmlTaskPoints<T: Config> =
		StorageMap<_, Twox64Concat, CmlId, ServiceTaskPoint, ValueQuery>;

	#[pallet::storage]
	pub type AccountRewards<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, BalanceOf<T>, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub genesis_coupons: GenesisCoupons<T::AccountId>,
		pub genesis_seeds: GenesisSeeds,
	}
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				genesis_coupons: GenesisCoupons::default(),
				genesis_seeds: GenesisSeeds::default(),
			}
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			crate::functions::init_from_genesis_coupons::<T>(&self.genesis_coupons);
			crate::functions::init_from_genesis_seeds::<T>(&self.genesis_seeds);
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
		WithoutCoupon,
		NotEnoughCoupon,
		InvalidCouponAmount,

		NotEnoughDrawSeeds,
		SeedsNotOutdatedYet,
		CouponsHasOutdated,
		NoNeedToCleanOutdatedSeeds,

		NotFoundCML,
		CMLOwnerInvalid,
		CmlIsNotSeed,

		InsufficientFreeBalance,
		InsufficientReservedBalance,
		MinerAlreadyExist,
		NotFoundMiner,
		InvalidCreditAmount,
		CannotTransferCmlWithCredit,
		InvalidMiner,
		InvalidMinerIp,

		InvalidStakingIndex,
		InvalidStakingOwner,
		NotFoundRewardAccount,
		StakingSlotsOverTheMaxLength,
		StakingSlotsOverAcceptableIndex,

		/// There is no credit of user, no need to pay for it.
		CmlNoNeedToPayOff,

		/// Defrost time should have value when defrost.
		CmlDefrostTimeIsNone,
		/// Cml should be frozen seed.
		CmlShouldBeFrozenSeed,
		/// Cml is still in frozen locked period that cannot be defrosted.
		CmlStillInFrozenLockedPeriod,
		/// Cml should be fresh seed.
		CmlShouldBeFreshSeed,
		/// Cml in fresh seed state and have expired the fresh duration.
		CmlFreshSeedExpired,
		/// Cml is tree means that can't be frozen seed or fresh seed.
		CmlShouldBeTree,
		/// Cml has over the lifespan
		CmlShouldDead,
		/// Cml is mining that can start mining again.
		CmlIsMiningAlready,
		/// Cml is staking that can't staking again or start mining.
		CmlIsStaking,
		/// Before start mining staking slot should be empty.
		CmlStakingSlotNotEmpty,
		/// Means we cannot decide staking type from given params.
		ConfusedStakingType,
		/// Cml is not mining that can't stake to.
		CmlIsNotMining,
		/// Cml is not staking to current miner that can't unstake.
		CmlIsNotStakingToCurrentMiner,
		/// Cml staking index over than staking slot length, that means point to not exist staking.
		CmlStakingIndexOverflow,
		/// Cml staking item owner is none, that can't identify staking belongs.
		CmlOwnerIsNone,
		/// Cml staking item owner and the owner field of cml item not match.
		CmlOwnerMismatch,
		/// Cml is not staking that can't unstake.
		CmlIsNotStaking,
		/// Some status that can't convert to another status.
		CmlInvalidStatusConversion,
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
		#[pallet::weight(195_000_000)]
		pub fn clean_outdated_seeds(sender: OriginFor<T>) -> DispatchResult {
			let root = ensure_root(sender)?;
			let current_block = frame_system::Pallet::<T>::block_number();

			extrinsic_procedure(
				&root,
				|_| {
					ensure!(
						Self::is_coupon_outdated(current_block),
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
					Self::try_clean_outdated_coupons(current_block);
				},
			)
		}

		#[pallet::weight(T::WeightInfo::transfer_coupon())]
		pub fn transfer_coupon(
			sender: OriginFor<T>,
			target: T::AccountId,
			cml_type: CmlType,
			schedule_type: DefrostScheduleType,
			#[pallet::compact] amount: u32,
		) -> DispatchResult {
			let sender = ensure_signed(sender)?;
			let sender_coupon = match schedule_type {
				DefrostScheduleType::Investor => InvestorCouponStore::<T>::get(&sender, cml_type),
				DefrostScheduleType::Team => TeamCouponStore::<T>::get(&sender, cml_type),
			};
			let target_coupon = match schedule_type {
				DefrostScheduleType::Investor => InvestorCouponStore::<T>::get(&target, cml_type),
				DefrostScheduleType::Team => TeamCouponStore::<T>::get(&target, cml_type),
			};

			extrinsic_procedure(
				&sender,
				|_| {
					ensure!(
						!Self::is_coupon_outdated(frame_system::Pallet::<T>::block_number()),
						Error::<T>::CouponsHasOutdated
					);
					ensure!(sender_coupon.is_some(), Error::<T>::NotEnoughCoupon);
					let sender_coupon = sender_coupon.as_ref().unwrap();
					ensure!(sender_coupon.amount >= amount, Error::<T>::NotEnoughCoupon);
					sender_coupon
						.amount
						.checked_sub(amount)
						.ok_or(Error::<T>::InvalidCouponAmount)?;
					if let Some(target_coupon) = target_coupon.as_ref() {
						target_coupon
							.amount
							.checked_add(amount)
							.ok_or(Error::<T>::InvalidCouponAmount)?;
					}

					Ok(())
				},
				|_| {
					if sender_coupon.is_none() {
						return;
					}
					let from_amount = sender_coupon
						.as_ref()
						.unwrap()
						.amount
						.saturating_sub(amount);

					match target_coupon.as_ref() {
						Some(target_coupon) => {
							let to_amount = target_coupon.amount.saturating_add(amount);
							Self::add_or_create_coupon(&target, cml_type, schedule_type, to_amount);
						}
						None => {
							Self::add_or_create_coupon(&target, cml_type, schedule_type, amount)
						}
					}

					Self::add_or_create_coupon(&sender, cml_type, schedule_type, from_amount);
				},
			)
		}

		#[pallet::weight(T::WeightInfo::draw_investor_cmls_from_coupon())]
		pub fn draw_cmls_from_coupon(
			sender: OriginFor<T>,
			schedule_type: DefrostScheduleType,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(sender)?;
			let (a_coupon, b_coupon, c_coupon) = Self::take_coupons(&sender, schedule_type);

			extrinsic_procedure_with_weight(
				&sender,
				|_| {
					ensure!(
						!Self::is_coupon_outdated(frame_system::Pallet::<T>::block_number()),
						Error::<T>::CouponsHasOutdated
					);
					ensure!(
						a_coupon + b_coupon + c_coupon > 0,
						Error::<T>::WithoutCoupon
					);
					Self::check_luck_draw(a_coupon, b_coupon, c_coupon, schedule_type)?;
					Ok(())
				},
				|sender| {
					let weight = match schedule_type {
						DefrostScheduleType::Investor => None,
						DefrostScheduleType::Team => {
							Some(T::WeightInfo::draw_team_cmls_from_coupon())
						}
					};
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
					weight
				},
			)
		}

		#[pallet::weight(T::WeightInfo::active_cml())]
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

		#[pallet::weight(T::WeightInfo::start_mining())]
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
					cml.check_start_mining(&current_block_number)
						.map_err(|e| Error::<T>::from(e))?;
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

		#[pallet::weight(T::WeightInfo::stop_mining())]
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

		#[pallet::weight(T::WeightInfo::start_balance_staking())]
		pub fn start_staking(
			sender: OriginFor<T>,
			staking_to: CmlId,
			staking_cml: Option<CmlId>,
			acceptable_slot_index: Option<StakingIndex>,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(sender)?;
			let current_block_number = frame_system::Pallet::<T>::block_number();

			extrinsic_procedure_with_weight(
				&who,
				|who| {
					ensure!(
						CmlStore::<T>::contains_key(staking_to),
						Error::<T>::NotFoundCML
					);

					let amount: Result<Option<BalanceOf<T>>, DispatchError> = match staking_cml {
						Some(cml_id) => {
							Self::check_belongs(&cml_id, &who)?;
							let cml = CmlStore::<T>::get(cml_id);
							cml.check_can_stake_to(&current_block_number)
								.map_err(|e| Error::<T>::from(e))?;
							Ok(None)
						}
						None => {
							Self::check_balance_staking(&who)?;
							Ok(Some(T::StakingPrice::get()))
						}
					};

					let cml = CmlStore::<T>::get(staking_to);
					if let Some(acceptable_slot_index) = acceptable_slot_index {
						ensure!(
							cml.staking_slots().len() <= acceptable_slot_index as usize,
							Error::<T>::StakingSlotsOverAcceptableIndex
						);
					}
					ensure!(
						cml.staking_slots().len() <= T::StakingSlotsMaxLength::get() as usize,
						Error::<T>::StakingSlotsOverTheMaxLength
					);
					cml.check_can_be_stake(&current_block_number, &amount?, &staking_cml)
						.map_err(|e| Error::<T>::from(e))?;
					Ok(())
				},
				|who| {
					let weight = staking_cml
						.as_ref()
						.map(|_| T::WeightInfo::start_cml_staking());

					let staking_index: Option<StakingIndex> =
						CmlStore::<T>::mutate(staking_to, |cml| {
							Self::stake(who, cml, &staking_cml, &current_block_number)
						});

					Self::deposit_event(Event::Staked(
						who.clone(),
						staking_to,
						staking_index.unwrap_or(u32::MAX),
					));
					weight
				},
			)
		}

		#[pallet::weight(T::WeightInfo::stop_balance_staking(*staking_index))]
		pub fn stop_staking(
			sender: OriginFor<T>,
			staking_to: CmlId,
			staking_index: StakingIndex,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(sender)?;

			extrinsic_procedure_with_weight(
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
					cml.check_unstake(&index, &staking_cml.as_ref())
						.map_err(|e| Error::<T>::from(e))?;

					Ok(())
				},
				|who| match CmlStore::<T>::mutate(staking_to, |cml| {
					Self::unstake(who, cml, staking_index)
				}) {
					true => None,
					false => Some(T::WeightInfo::stop_cml_staking(staking_index)),
				},
			)
		}

		#[pallet::weight(T::WeightInfo::withdraw_staking_reward())]
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

		#[pallet::weight(T::WeightInfo::pay_off_mining_credit())]
		pub fn pay_off_mining_credit(sender: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						GenesisMinerCreditStore::<T>::contains_key(who),
						Error::<T>::CmlNoNeedToPayOff
					);
					ensure!(
						T::CurrencyOperations::free_balance(who)
							>= GenesisMinerCreditStore::<T>::get(who),
						Error::<T>::InsufficientFreeBalance
					);
					Ok(())
				},
				|who| {
					let pay_off_balance = GenesisMinerCreditStore::<T>::get(who);
					// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
					if T::CurrencyOperations::reserve(who, pay_off_balance).is_err() {
						return;
					}
					GenesisMinerCreditStore::<T>::remove(who);
				},
			)
		}

		#[pallet::weight(T::WeightInfo::dummy_ra_task())]
		pub fn dummy_ra_task(sender: OriginFor<T>, machine_id: MachineId) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|_who| {
					ensure!(
						MinerItemStore::<T>::contains_key(machine_id),
						Error::<T>::NotFoundMiner
					);
					let machine_item = MinerItemStore::<T>::get(machine_id);

					ensure!(
						CmlStore::<T>::contains_key(machine_item.cml_id),
						Error::<T>::NotFoundCML
					);
					// Self::check_belongs(&machine_item.cml_id, who)
					Ok(())
				},
				|_who| {
					Self::complete_ra_task(machine_id);
				},
			)
		}
	}

	impl<T: Config> From<CmlError> for Error<T> {
		fn from(e: CmlError) -> Self {
			match e {
				CmlError::CmlDefrostTimeIsNone => Error::<T>::CmlDefrostTimeIsNone,
				CmlError::CmlShouldBeFrozenSeed => Error::<T>::CmlShouldBeFrozenSeed,
				CmlError::CmlStillInFrozenLockedPeriod => Error::<T>::CmlStillInFrozenLockedPeriod,
				CmlError::CmlShouldBeFreshSeed => Error::<T>::CmlShouldBeFreshSeed,
				CmlError::CmlFreshSeedExpired => Error::<T>::CmlFreshSeedExpired,
				CmlError::CmlShouldBeTree => Error::<T>::CmlShouldBeTree,
				CmlError::CmlShouldDead => Error::<T>::CmlShouldDead,
				CmlError::CmlIsMiningAlready => Error::<T>::CmlIsMiningAlready,
				CmlError::CmlIsStaking => Error::<T>::CmlIsStaking,
				CmlError::CmlStakingSlotNotEmpty => Error::<T>::CmlStakingSlotNotEmpty,
				CmlError::ConfusedStakingType => Error::<T>::ConfusedStakingType,
				CmlError::CmlIsNotMining => Error::<T>::CmlIsNotMining,
				CmlError::CmlIsNotStakingToCurrentMiner => {
					Error::<T>::CmlIsNotStakingToCurrentMiner
				}
				CmlError::CmlStakingIndexOverflow => Error::<T>::CmlStakingIndexOverflow,
				CmlError::CmlOwnerIsNone => Error::<T>::CmlOwnerIsNone,
				CmlError::CmlOwnerMismatch => Error::<T>::CmlOwnerMismatch,
				CmlError::CmlIsNotStaking => Error::<T>::CmlIsNotStaking,
				CmlError::CmlInvalidStatusConversion => Error::<T>::CmlInvalidStatusConversion,
			}
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

	fn add_cml(
		who: &Self::AccountId,
		cml: CML<Self::AccountId, Self::BlockNumber, Self::Balance, Self::FreshDuration>,
	);
}

pub trait StakingEconomics<Balance, AccountId> {
	fn increase_issuance(total_point: ServiceTaskPoint) -> Balance;

	fn total_staking_rewards_of_miner(
		miner_point: ServiceTaskPoint,
		total_point: ServiceTaskPoint,
	) -> Balance;

	fn miner_total_staking_price(snapshots: &Vec<StakingSnapshotItem<AccountId>>) -> Balance;

	fn single_staking_reward(
		miner_total_rewards: Balance,
		total_staking_point: Balance,
		snapshot_item: &StakingSnapshotItem<AccountId>,
	) -> Balance;
}

pub trait Task {
	fn complete_ra_task(machine_id: MachineId);
}
