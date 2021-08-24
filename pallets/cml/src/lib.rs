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
pub use param::*;
pub use types::*;
pub use weights::WeightInfo;

use frame_support::{
	dispatch::DispatchResult,
	ensure,
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement, Get},
};
use frame_system::pallet_prelude::*;
use genesis_exchange_interface::MiningOperation;
use pallet_utils::{
	extrinsic_procedure, extrinsic_procedure_with_weight, CommonUtils, CurrencyOperations,
};
use sp_runtime::traits::{AtLeast32BitUnsigned, Saturating, Zero};
use sp_std::{convert::TryInto, prelude::*};

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

		/// The fixed (free balance) amount to staking to a CML with balance.
		#[pallet::constant]
		type StakingPrice: Get<BalanceOf<Self>>;

		/// The latest block height to draw seeds use coupon, after this block height the left
		/// seeds shall be destroyed.
		#[pallet::constant]
		type CouponTimoutHeight: Get<Self::BlockNumber>;

		/// Fresh seed duration, if a fresh seed stays over than the duration can't active (including planting
		///	and mining) any more.
		#[pallet::constant]
		type SeedFreshDuration: Get<Self::BlockNumber>;

		/// Length of a staking window, staking rewards will be dispathed at the end of the staking period.
		#[pallet::constant]
		type StakingPeriodLength: Get<Self::BlockNumber>;

		/// Common utils trait
		type CommonUtils: CommonUtils<AccountId = Self::AccountId>;

		/// Operations about currency that used in Tea Camellia.
		type CurrencyOperations: CurrencyOperations<
			AccountId = Self::AccountId,
			Balance = BalanceOf<Self>,
		>;

		/// Operations to calculate staking rewards.
		type StakingEconomics: StakingEconomics<BalanceOf<Self>, Self::AccountId>;

		type MiningOperation: MiningOperation<AccountId = Self::AccountId>;

		/// Weight definition about all user related extrinsics.
		type WeightInfo: WeightInfo;

		/// Max length about staking slots of a mining CML, note this is max length of the staking slot array
		/// not the max staking index, due to a CML can stake into the mining CML the actual staking index may
		/// larger than `StakingSlotsMaxLength`.
		#[pallet::constant]
		type StakingSlotsMaxLength: Get<StakingIndex>;

		/// Punishment amount need to pay for each staking account when stop mining.
		#[pallet::constant]
		type StopMiningPunishment: Get<BalanceOf<Self>>;
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
	pub type CmlStore<T: Config> = StorageMap<
		_,
		Twox64Concat,
		CmlId,
		CML<T::AccountId, T::BlockNumber, BalanceOf<T>, T::SeedFreshDuration>,
		ValueQuery,
	>;

	/// Double map about user and related cml ID of him.
	#[pallet::storage]
	#[pallet::getter(fn user_cml_store)]
	pub type UserCmlStore<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, CmlId, ()>;

	/// Double map about investor user and `CmlType` of his coupon, value is the information of the coupon.
	#[pallet::storage]
	#[pallet::getter(fn investor_user_store)]
	pub type InvestorCouponStore<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, CmlType, Coupon>;

	/// Double map about team user and `CmlType` of his coupon, value is the information of the coupon.
	#[pallet::storage]
	#[pallet::getter(fn team_user_store)]
	pub type TeamCouponStore<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, CmlType, Coupon>;

	/// Storage of miner.
	#[pallet::storage]
	#[pallet::getter(fn miner_item_store)]
	pub type MinerItemStore<T: Config> =
		StorageMap<_, Twox64Concat, MachineId, MinerItem, ValueQuery>;

	/// Double map about `CmlType` and `DefrostScheduleType`, value is the rest of CMLs in lucky draw box.
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

	/// Storage of the snapshot about current active staking related information.
	#[pallet::storage]
	#[pallet::getter(fn active_staking_snapshot)]
	pub type ActiveStakingSnapshot<T: Config> =
		StorageMap<_, Twox64Concat, CmlId, Vec<StakingSnapshotItem<T::AccountId>>, ValueQuery>;

	/// Storage of the accumulating stask point about a miner.
	#[pallet::storage]
	#[pallet::getter(fn mining_cml_task_points)]
	pub type MiningCmlTaskPoints<T: Config> =
		StorageMap<_, Twox64Concat, CmlId, ServiceTaskPoint, ValueQuery>;

	/// Staking rewards of all participating staking users.
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
		/// Event fired after user drawed CMLs from lucky draw box successfully.
		///
		/// First paramter is account id of the user, and the second parameter is the total drawed CML counts.
		DrawCmls(T::AccountId, u64),
		/// Event fired after user converted a seed to tree successfully.
		///
		/// First paramter is account id of the user, and the second parameter is the CML ID.
		ActiveCml(T::AccountId, CmlId),
		/// Event fired after user staked into a CML successfully.
		///
		/// First paramter is account id of the user, and the second parameter is the staking CML ID,
		/// the third paramter is current staking index.
		Staked(T::AccountId, CmlId, StakingIndex),
		/// Event fired when end of staking window.
		RewardStatements(Vec<(T::AccountId, CmlId, BalanceOf<T>)>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// User have no coupons to draw cmls.
		WithoutCoupon,
		/// User have not enough coupons to transfer.
		NotEnoughCoupon,
		/// User transfer the coupons amount is over than he really has.
		InvalidCouponAmount,

		/// There is not enough draw seeds left in luck draw box (this error indicate the genesis draw seeds count
		/// and coupons amount not matched).
		NotEnoughDrawSeeds,
		/// Seeds is not outdated that can't be clean out with `clean_outdated_seeds` extrinsic.
		SeedsNotOutdatedYet,
		/// Coupons has outdated that can't transfer or be used to transfer to others.
		CouponsHasOutdated,
		/// Lucky draw box is already empty so there is no need to clean out.
		NoNeedToCleanOutdatedSeeds,

		/// Could not find CML in the cml store, indicates that the specified CML not existed.
		NotFoundCML,
		/// Trying to operate a CML not belongs to the user.
		CMLOwnerInvalid,
		/// Cml is not seed so there is no need to active (to tree) or do something else.
		CmlIsNotSeed,

		/// User account free balance is not enoungh.
		InsufficientFreeBalance,
		/// The specified machine ID is already mining, that should not be used to start mining again.
		MinerAlreadyExist,
		/// The specified machine ID is not found in the machine store, indicates that the specified machin
		/// ID not existed.
		NotFoundMiner,
		/// Specified CML in not valid to operate as a mining tree.
		InvalidMiner,
		/// Sepcified miner IP is not a valid format of IPv4.
		InvalidMinerIp,

		/// Specified staking index is over than the max length of current staking slots.
		InvalidStakingIndex,
		/// The first staking slot cannot be unstake, if do want to unstake it please stop mining instead.
		CannotUnstakeTheFirstSlot,
		/// User is not the owner of specified CML.
		InvalidStakingOwner,
		/// User has no reward of staking that can't to withdraw the reward.
		NotFoundRewardAccount,
		/// Specified CML has been staked by too much users that can't be append staking anymore.
		StakingSlotsOverTheMaxLength,
		/// User specfied max acceptable slot length and current staking index has over than that.
		StakingSlotsOverAcceptableIndex,

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
		/// Not enough free balance to pay for each of the staking accounts.
		InsufficientFreeBalanceToPayForPunishment,
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
		/// Called by sudo user to clean up outdated seeds after the coupon has outdated.
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

		/// Transfer coupon from `sender` to `target`.
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

		/// Draw cmls with coupon in lucky draw box.
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

		/// Convert a valid seed into tree.
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

		/// Start mining with binding a `MachineId` with given CML.
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
					// check if miner is ready to prepare a mining machine.
					T::MiningOperation::check_buying_mining_machine(sender, cml_id)?;

					Self::check_belongs(&cml_id, &sender)?;
					ensure!(
						!<MinerItemStore<T>>::contains_key(&machine_id),
						Error::<T>::MinerAlreadyExist
					);
					Self::check_miner_ip_validity(&miner_ip)?;

					let cml = CmlStore::<T>::get(cml_id);
					cml.check_start_mining(&current_block_number)
						.map_err(|e| Error::<T>::from(e))?;
					Self::check_miner_first_staking(&sender)?;

					Ok(())
				},
				|sender| {
					// Prepare the mining machine at first.
					T::MiningOperation::buy_mining_machine(sender, cml_id);

					let ip = miner_ip.clone();
					CmlStore::<T>::mutate(cml_id, |cml| {
						let staking_item =
							Self::create_balance_staking(&sender, T::StakingPrice::get());
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

		/// Stop mining cml, and or slots staking to the CML will be canceled.
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
					ensure!(
						T::CurrencyOperations::free_balance(sender)
							>= T::StopMiningPunishment::get()
								* Self::customer_staking_length(sender, &cml).into(),
						Error::<T>::InsufficientFreeBalanceToPayForPunishment,
					);
					Ok(())
				},
				|sender| {
					Self::pay_for_miner_customer(sender, cml_id);
					Self::stop_mining_inner(sender, cml_id, &machine_id);
				},
			)
		}

		/// Staking to a CML with other free balance or another CML belongs to user.
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

		/// Stop staking to CML.
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
						!staking_index.is_zero(),
						Error::<T>::CannotUnstakeTheFirstSlot
					);
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
					Self::unstake(who, cml, staking_index, T::StakingPrice::get())
				}) {
					true => None,
					false => Some(T::WeightInfo::stop_cml_staking(staking_index)),
				},
			)
		}

		/// Withdraw staking rewards of given user.
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

		/// Called after a miner has complete a RA task, note this is dummy extrinsic to simulate the
		/// task submit, the real RA task should initiate at an Enclave enviroment and validate by
		/// other nodes.
		#[pallet::weight(T::WeightInfo::dummy_ra_task())]
		pub fn dummy_ra_task(
			sender: OriginFor<T>,
			machine_id: MachineId,
			task_point: ServiceTaskPoint,
		) -> DispatchResult {
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
					Self::complete_ra_task(machine_id, task_point);
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

/// Operations about CML that called by other pallets to interact with.
pub trait CmlOperation {
	type AccountId: PartialEq + Clone;
	type Balance: Clone;
	type BlockNumber: Default + AtLeast32BitUnsigned + Clone;
	type FreshDuration: Get<Self::BlockNumber>;

	/// Get cml with given cml ID, if not exist will throw the `NotFoundCML` error.
	fn cml_by_id(
		cml_id: &CmlId,
	) -> Result<
		CML<Self::AccountId, Self::BlockNumber, Self::Balance, Self::FreshDuration>,
		DispatchError,
	>;

	/// Check if the given CML not belongs to specified account.
	fn check_belongs(cml_id: &CmlId, who: &Self::AccountId) -> Result<(), DispatchError>;

	/// Check if `from_account` can transfer the specifying CML to `target_account`.
	fn check_transfer_cml_to_other(
		from_account: &Self::AccountId,
		cml_id: &CmlId,
		target_account: &Self::AccountId,
	) -> DispatchResult;

	/// Transfer `from_account` the specifying CML to `target_account`.
	fn transfer_cml_to_other(
		from_account: &Self::AccountId,
		cml_id: &CmlId,
		target_account: &Self::AccountId,
	);

	/// Get the deposit price if CML is mining, or `None` otherwise.
	fn cml_deposit_price(cml_id: &CmlId) -> Option<Self::Balance>;

	/// Add a cml into `CmlStore` and bind the CML with the given user.
	fn add_cml(
		who: &Self::AccountId,
		cml: CML<Self::AccountId, Self::BlockNumber, Self::Balance, Self::FreshDuration>,
	);

	/// Remove cml from `CmlStore` and unbind cml with its owner.
	/// **Note** this can only be used with cml in seed state, or a cml tree not mining or staking,
	/// otherwise will lead to some error states.
	fn remove_cml(cml_id: CmlId);

	/// Get all user owned cml list.
	fn user_owned_cmls(who: &Self::AccountId) -> Vec<CmlId>;

	/// Estimate reward statements by given total task point and each miner task point calculation
	/// methods.
	///
	/// Both `total_point` and `miner_task_point` returns points in milli-unit, that means
	///  each 1000 milli-points will be treated as 1 task point in reward calculation.
	fn estimate_reward_statements<X, Y>(
		total_point: X,
		miner_task_point: Y,
	) -> Vec<(Self::AccountId, CmlId, Self::Balance)>
	where
		X: FnOnce() -> ServiceTaskPoint,
		Y: Fn(CmlId) -> ServiceTaskPoint;

	/// Get current mining cml list;
	fn current_mining_cmls() -> Vec<CmlId>;
}

/// Operations to calculate staking rewards.
pub trait StakingEconomics<Balance, AccountId> {
	/// Calculate issuance balance with given total task point of current staking window.
	fn increase_issuance(total_point: ServiceTaskPoint) -> Balance;

	/// Calculate total staking rewards of the given miner, the staking rewards should split to all staking
	/// users.
	fn total_staking_rewards_of_miner(
		miner_point: ServiceTaskPoint,
		total_point: ServiceTaskPoint,
		performance: Performance,
	) -> Balance;

	/// Calculate all staking weight about the given miner.
	fn miner_total_staking_weight(snapshots: &Vec<StakingSnapshotItem<AccountId>>) -> Balance;

	/// Calculate a single staking reward.
	fn single_staking_reward(
		miner_total_rewards: Balance,
		total_staking_point: Balance,
		snapshot_item: &StakingSnapshotItem<AccountId>,
	) -> Balance;
}

/// Operations about task, tasks usually initiated at an enclave environment.
pub trait Task {
	/// Called after a miner has complete a RA task.
	fn complete_ra_task(machine_id: MachineId, task_point: ServiceTaskPoint);
}
