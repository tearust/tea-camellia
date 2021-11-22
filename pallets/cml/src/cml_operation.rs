use super::*;

impl<T: cml::Config> CmlOperation for cml::Pallet<T> {
	type AccountId = T::AccountId;
	type Balance = BalanceOf<T>;
	type BlockNumber = T::BlockNumber;
	type FreshDuration = T::SeedFreshDuration;

	/// Get cml with given cml ID, if not exist will throw the `NotFoundCML` error.
	fn cml_by_id(
		cml_id: &CmlId,
	) -> Result<
		CML<Self::AccountId, Self::BlockNumber, Self::Balance, Self::FreshDuration>,
		DispatchError,
	> {
		ensure!(CmlStore::<T>::contains_key(cml_id), Error::<T>::NotFoundCML);
		Ok(CmlStore::<T>::get(cml_id))
	}

	fn cml_by_machine_id(
		machine_id: &MachineId,
	) -> Option<CML<Self::AccountId, Self::BlockNumber, Self::Balance, Self::FreshDuration>> {
		match Self::miner_item_by_machine_id(machine_id) {
			Some(miner_item) => {
				if !CmlStore::<T>::contains_key(miner_item.cml_id) {
					return None;
				}

				Some(CmlStore::<T>::get(miner_item.cml_id))
			}
			None => None,
		}
	}

	fn miner_item_by_machine_id(
		machine_id: &MachineId,
	) -> Option<MinerItem<Self::BlockNumber, Self::AccountId>> {
		if !MinerItemStore::<T>::contains_key(machine_id) {
			return None;
		}
		Some(MinerItemStore::<T>::get(machine_id))
	}

	/// Check if the given CML not belongs to specified account.
	fn check_belongs(cml_id: &u64, who: &Self::AccountId) -> DispatchResult {
		ensure!(CmlStore::<T>::contains_key(cml_id), Error::<T>::NotFoundCML);
		ensure!(
			UserCmlStore::<T>::contains_key(who, cml_id),
			Error::<T>::CMLOwnerInvalid
		);
		Ok(())
	}

	/// Check if `from_account` can transfer the specifying CML to `target_account`.
	fn check_transfer_cml_to_other(
		from_account: &Self::AccountId,
		cml_id: &CmlId,
		target_account: &Self::AccountId,
	) -> DispatchResult {
		Self::check_belongs(cml_id, from_account)?;

		let cml = CmlStore::<T>::get(cml_id);
		if cml.is_mining() {
			ensure!(
				T::CurrencyOperations::can_reserve(target_account, T::StakingPrice::get()),
				Error::<T>::InsufficientFreeBalance
			);
		}
		Ok(())
	}

	/// Transfer `from_account` the specifying CML to `target_account`.
	fn transfer_cml_to_other(
		from_account: &Self::AccountId,
		cml_id: &CmlId,
		target_account: &Self::AccountId,
	) {
		if Self::check_transfer_cml_to_other(from_account, cml_id, target_account).is_err() {
			return;
		}

		let success = CmlStore::<T>::mutate(&cml_id, |cml| {
			cml.set_owner(target_account);
			if cml.is_mining() {
				let amount = match cml.staking_slots().get(0) {
					Some(item) => item.amount.clone(),
					None => None,
				};
				if amount.is_none() {
					return false;
				}

				return if let Ok(staking_item) =
					Self::create_balance_staking(target_account, amount.unwrap())
				{
					T::CurrencyOperations::unreserve(from_account, amount.unwrap());
					cml.swap_first_slot(staking_item);
					true
				} else {
					false
				};
			}
			true
		});
		// see https://github.com/tearust/tea-camellia/issues/13
		if !success {
			return;
		}

		// remove from from UserCmlStore
		UserCmlStore::<T>::remove(from_account, cml_id);
		UserCmlStore::<T>::insert(target_account, cml_id, ());
	}

	/// Get the deposit price if CML is mining, or `None` otherwise.
	fn cml_deposit_price(cml_id: &CmlId) -> Option<Self::Balance> {
		if !CmlStore::<T>::contains_key(cml_id) {
			return None;
		}

		let cml = CmlStore::<T>::get(cml_id);
		if cml.is_mining() {
			if let Some(staking_item) = cml.staking_slots().get(0) {
				return staking_item.amount.clone();
			}
		}
		None
	}

	/// Add a cml into `CmlStore` and bind the CML with the given user.
	fn add_cml(
		who: &Self::AccountId,
		cml: CML<Self::AccountId, Self::BlockNumber, Self::Balance, Self::FreshDuration>,
	) {
		let cml_id = cml.id();
		CmlStore::<T>::insert(cml_id, cml);
		UserCmlStore::<T>::insert(who, cml_id, ());
	}

	/// Remove cml from `CmlStore` and unbind cml with its owner.
	fn remove_cml(cml_id: CmlId) {
		let cml = CmlStore::<T>::take(cml_id);
		if let Some(owner) = cml.owner() {
			UserCmlStore::<T>::remove(owner, cml_id);
		}
	}

	/// Get all user owned cml list.
	fn user_owned_cmls(who: &Self::AccountId) -> Vec<CmlId> {
		Self::user_cml_list(who)
	}

	/// Estimate reward according to total task point and miner task point
	///
	/// Both `total_point` and `miner_task_point` returns points in milli-unit, that means
	///  each 1000 milli-points will be treated as 1 task point in reward calculation.
	fn estimate_reward_statements<X, Y>(
		total_point: X,
		miner_task_point: Y,
	) -> Vec<(Self::AccountId, CmlId, Self::Balance)>
	where
		X: FnOnce() -> ServiceTaskPoint,
		Y: Fn(CmlId) -> ServiceTaskPoint,
	{
		let current_height = frame_system::Pallet::<T>::block_number();
		let total_task_point = total_point();

		let mut miner_total_rewards = BTreeMap::new();
		ActiveStakingSnapshot::<T>::iter_keys().for_each(|cml_id| {
			let miner_task_point = miner_task_point(cml_id);
			let (performance, _) = Self::miner_performance(cml_id, &current_height);
			let miner_total_reward = T::StakingEconomics::total_staking_rewards_of_miner(
				miner_task_point,
				total_task_point,
				performance.unwrap_or(0),
			);
			let mining_reward = Self::calculate_mining_reward(cml_id, &miner_total_reward);

			miner_total_rewards.insert(
				cml_id,
				(
					mining_reward.clone(),
					miner_total_reward.saturating_sub(mining_reward),
				),
			);
		});

		let mut reward_statements = Vec::new();
		miner_total_rewards
			.iter()
			.for_each(|(cml_id, (mining_reward, _))| {
				if mining_reward.is_zero() {
					return;
				}

				let cml = CmlStore::<T>::get(cml_id);
				if let Some(owner) = cml.owner() {
					reward_statements.push((owner.clone(), *cml_id, mining_reward.clone()));
				}
			});

		let snapshots: Vec<(CmlId, Vec<StakingSnapshotItem<T::AccountId>>)> =
			ActiveStakingSnapshot::<T>::iter().collect();
		for (cml_id, snapshot_items) in snapshots {
			if let Some((_, rest_reward)) = miner_total_rewards.get(&cml_id) {
				reward_statements.append(&mut Self::single_cml_staking_reward_statements(
					cml_id,
					&snapshot_items,
					rest_reward.clone(),
				));
			}
		}

		reward_statements
	}

	fn calculate_mining_reward(cml_id: CmlId, miner_total_reward: &Self::Balance) -> Self::Balance {
		let cml = CmlStore::<T>::get(cml_id);
		let mining_reward_ratio = Self::mining_reward_rate_by_type(cml.cml_type());
		miner_total_reward.saturating_mul(mining_reward_ratio) / 10000u32.into()
	}

	fn cml_staking_snapshots(cml_id: CmlId) -> Vec<StakingSnapshotItem<Self::AccountId>> {
		ActiveStakingSnapshot::<T>::get(cml_id)
	}

	fn single_cml_staking_reward_statements(
		cml_id: CmlId,
		snapshot_items: &Vec<StakingSnapshotItem<Self::AccountId>>,
		miner_total_reward: Self::Balance,
	) -> Vec<(Self::AccountId, CmlId, Self::Balance)> {
		let total_staking_point = T::StakingEconomics::miner_total_staking_weight(&snapshot_items);

		let mut reward_statements = Vec::new();
		for item in snapshot_items.iter() {
			let reward = T::StakingEconomics::single_staking_reward(
				miner_total_reward,
				total_staking_point,
				item,
			);
			reward_statements.push((item.owner.clone(), cml_id, reward));
		}
		reward_statements
	}

	fn current_mining_cmls() -> Vec<(CmlId, MachineId)> {
		MinerItemStore::<T>::iter()
			.map(|(_, miner_item)| (miner_item.cml_id, miner_item.id))
			.collect()
	}

	/// return a pair of values, first is current performance calculated by given block height,
	/// the second is the peak performance.
	fn miner_performance(
		cml_id: CmlId,
		block_height: &Self::BlockNumber,
	) -> (Option<Performance>, Performance) {
		let cml = CmlStore::<T>::get(cml_id);
		let peak_performance = cml.get_peak_performance();
		if cml.lifespan().is_zero() {
			return (None, peak_performance);
		} else {
			if let Some(plant_at_block) = cml.get_plant_at() {
				let age_percentage =
					(*block_height - *plant_at_block) * 100u32.into() / cml.lifespan();
				if let Ok(age_percentage) = age_percentage.try_into() {
					return (
						Some(cml.calculate_performance(age_percentage)),
						peak_performance,
					);
				}
			}
		};

		(None, peak_performance)
	}

	fn user_coupon_list(who: &Self::AccountId, schedule_type: DefrostScheduleType) -> Vec<Coupon> {
		match schedule_type {
			DefrostScheduleType::Team => TeamCouponStore::<T>::iter_prefix(who)
				.map(|(_, coupon)| coupon)
				.collect(),
			DefrostScheduleType::Investor => InvestorCouponStore::<T>::iter_prefix(who)
				.map(|(_, coupon)| coupon)
				.collect(),
		}
	}

	fn task_point_base() -> ServiceTaskPoint {
		TaskPointBase::<T>::get()
	}

	fn mining_status(cml_id: CmlId) -> (bool, MinerStatus) {
		let cml = CmlStore::<T>::get(cml_id);
		let status = match cml.machine_id() {
			Some(machine_id) => MinerItemStore::<T>::get(machine_id).status,
			None => MinerStatus::Offline,
		};

		(cml.is_mining(), status)
	}

	fn is_cml_over_max_suspend_height(cml_id: CmlId, block_height: &Self::BlockNumber) -> bool {
		let cml = CmlStore::<T>::get(cml_id);
		if let Some(machine_id) = cml.machine_id() {
			if let Some(height) = MinerItemStore::<T>::get(machine_id).suspend_height {
				return *block_height > height + T::MaxAllowedSuspendHeight::get();
			}
		}

		false
	}

	fn check_miner(machine_id: MachineId, miner_account: &Self::AccountId) -> bool {
		let miner_item = MinerItemStore::<T>::get(machine_id);
		miner_item.controller_account.eq(miner_account)
	}

	fn suspend_mining(machine_id: MachineId) {
		MinerItemStore::<T>::mutate(&machine_id, |item| {
			item.status = MinerStatus::Offline;
			item.suspend_height = Some(frame_system::Pallet::<T>::block_number());

			let cml_id = item.cml_id;
			T::BondingCurveOperation::cml_host_tapps(cml_id)
				.iter()
				.for_each(|tapp_id| {
					T::BondingCurveOperation::pay_hosting_penalty(*tapp_id, cml_id);
					T::BondingCurveOperation::try_deactive_tapp(*tapp_id);
				});
		});
	}

	fn append_reward(account: &Self::AccountId, amount: Self::Balance) {
		AccountRewards::<T>::mutate(account, |balance| {
			*balance = balance.saturating_add(amount);
		});
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		mock::*, tests::seed_from_lifespan, ActiveStakingSnapshot, CmlOperation, CmlStore, CmlType,
		Config, MachineId, MinerItem, MinerItemStore, MinerStatus, MiningProperties,
		StakingProperties, TreeProperties, UserCmlStore, CML,
	};
	use frame_support::{assert_ok, traits::Currency};
	use pallet_utils::CurrencyOperations;

	#[test]
	fn mining_status_works() {
		new_test_ext().execute_with(|| {
			let cml_id = 1;
			let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
			assert!(cml.machine_id().is_none());
			CmlStore::<Test>::insert(cml_id, cml.clone());
			assert_eq!(Cml::mining_status(cml_id), (false, MinerStatus::Offline));

			let machine_id: MachineId = [1u8; 32];
			let mut miner_item: MinerItem<u64, u64> = Default::default();
			miner_item.status = MinerStatus::Active;
			MinerItemStore::<Test>::insert(machine_id, miner_item.clone());
			cml.start_mining(machine_id, Default::default(), &0);
			CmlStore::<Test>::insert(cml_id, cml.clone());
			assert_eq!(Cml::mining_status(cml_id), (true, MinerStatus::Active));

			miner_item.status = MinerStatus::Offline;
			MinerItemStore::<Test>::insert(machine_id, miner_item.clone());
			assert_eq!(Cml::mining_status(cml_id), (true, MinerStatus::Offline));
		})
	}

	#[test]
	fn is_cml_over_max_suspend_height_works() {
		new_test_ext().execute_with(|| {
			let cml_id = 1;
			let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
			assert!(cml.machine_id().is_none());
			CmlStore::<Test>::insert(cml_id, cml.clone());

			assert!(!Cml::is_cml_over_max_suspend_height(cml_id, &u64::MAX));
			let machine_id: MachineId = [1u8; 32];
			let mut miner_item: MinerItem<u64, u64> = Default::default();
			MinerItemStore::<Test>::insert(machine_id, miner_item.clone());
			cml.start_mining(machine_id, Default::default(), &0);
			CmlStore::<Test>::insert(cml_id, cml.clone());
			assert!(!Cml::is_cml_over_max_suspend_height(cml_id, &u64::MAX));

			miner_item.suspend_height = Some(100);
			MinerItemStore::<Test>::insert(machine_id, miner_item.clone());

			assert!(!Cml::is_cml_over_max_suspend_height(
				cml_id,
				&(100 + MAX_ALLOWED_SUSPEND_HEIGHT as u64)
			));

			assert!(Cml::is_cml_over_max_suspend_height(
				cml_id,
				&(100 + MAX_ALLOWED_SUSPEND_HEIGHT as u64 + 1)
			));
		})
	}

	#[test]
	fn transfer_cml_to_other_works() {
		new_test_ext().execute_with(|| {
			let user_id = 11;
			let owner = 22;
			let user_origin_balance = 100 * 1000;
			let owner_origin_balance = 100 * 1000;
			<Test as Config>::Currency::make_free_balance_be(&user_id, user_origin_balance);
			<Test as Config>::Currency::make_free_balance_be(&owner, owner_origin_balance);

			let cml_id = 1;
			let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
			UserCmlStore::<Test>::insert(owner, cml_id, ());
			CmlStore::<Test>::insert(cml_id, cml);

			assert_ok!(Cml::start_mining(
				Origin::signed(owner),
				cml_id,
				[1u8; 32],
				owner,
				b"miner_ip".to_vec(),
				None,
			));
			assert!(UserCmlStore::<Test>::contains_key(owner, cml_id));
			let cml = CmlStore::<Test>::get(cml_id);
			assert_eq!(cml.staking_slots()[0].owner, owner);

			assert_ok!(Cml::check_transfer_cml_to_other(&owner, &cml_id, &user_id));
			Cml::transfer_cml_to_other(&owner, &cml_id, &user_id);

			assert!(UserCmlStore::<Test>::contains_key(user_id, cml_id));
			let cml = CmlStore::<Test>::get(cml_id);
			assert_eq!(cml.owner(), Some(&user_id));
			assert_eq!(cml.staking_slots()[0].owner, user_id);

			assert_eq!(
				<Test as Config>::Currency::free_balance(&user_id),
				user_origin_balance - STAKING_PRICE
			);
			assert_eq!(
				<Test as Config>::Currency::reserved_balance(&user_id),
				STAKING_PRICE
			);

			assert_eq!(
				<Test as Config>::Currency::free_balance(&owner),
				user_origin_balance
			);
			assert_eq!(<Test as Config>::Currency::reserved_balance(&owner), 0);
		})
	}

	#[test]
	fn transfer_cml_to_other_works_if_sender_reserved_balance_has_been_slashed() {
		new_test_ext().execute_with(|| {
			let user_id = 11;
			let owner = 22;
			let user_origin_balance = 100 * 1000;
			let owner_origin_balance = 100 * 1000;
			<Test as Config>::Currency::make_free_balance_be(&user_id, user_origin_balance);
			<Test as Config>::Currency::make_free_balance_be(&owner, owner_origin_balance);

			let cml_id = 1;
			let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
			UserCmlStore::<Test>::insert(owner, cml_id, ());
			CmlStore::<Test>::insert(cml_id, cml);
			assert_ok!(Cml::start_mining(
				Origin::signed(owner),
				cml_id,
				[1u8; 32],
				owner,
				b"miner_ip".to_vec(),
				None,
			));

			assert_eq!(
				<Test as Config>::Currency::free_balance(&owner),
				user_origin_balance - STAKING_PRICE
			);
			assert_eq!(
				<Test as Config>::Currency::reserved_balance(&owner),
				STAKING_PRICE
			);

			let slashed_amount = STAKING_PRICE / 2;
			Utils::slash_reserved(&owner, slashed_amount);

			assert_ok!(Cml::check_transfer_cml_to_other(&owner, &cml_id, &user_id));
			Cml::transfer_cml_to_other(&owner, &cml_id, &user_id);

			assert_eq!(
				<Test as Config>::Currency::free_balance(&owner),
				user_origin_balance - slashed_amount
			);
			assert_eq!(<Test as Config>::Currency::reserved_balance(&owner), 0);
		})
	}

	#[test]
	fn estimate_reward_statements_works() {
		new_test_ext().execute_with(|| {
			let user_id1 = 1;
			let user_id2 = 2;
			let owner1 = 11;
			let owner2 = 22;
			let user_origin_balance = 100 * 1000;
			let owner_origin_balance = 100 * 1000;
			<Test as Config>::Currency::make_free_balance_be(&user_id1, user_origin_balance);
			<Test as Config>::Currency::make_free_balance_be(&user_id2, user_origin_balance);
			<Test as Config>::Currency::make_free_balance_be(&owner1, owner_origin_balance);
			<Test as Config>::Currency::make_free_balance_be(&owner2, owner_origin_balance);

			let cml_id1 = 1;
			let mut seed1 = seed_from_lifespan(cml_id1, 100);
			seed1.cml_type = CmlType::A;
			seed1.performance = 10000;
			let mut cml = CML::from_genesis_seed(seed1);
			cml.set_owner(&owner1);
			UserCmlStore::<Test>::insert(owner1, cml_id1, ());
			CmlStore::<Test>::insert(cml_id1, cml);
			assert_ok!(Cml::start_mining(
				Origin::signed(owner1),
				cml_id1,
				[1u8; 32],
				owner1,
				b"miner_ip".to_vec(),
				None,
			));

			let cml_id2 = 2;
			let mut seed2 = seed_from_lifespan(cml_id2, 100);
			seed2.cml_type = CmlType::B;
			seed2.performance = 10000;
			let mut cml2 = CML::from_genesis_seed(seed2);
			cml2.set_owner(&owner2);
			UserCmlStore::<Test>::insert(owner2, cml_id2, ());
			CmlStore::<Test>::insert(cml_id2, cml2);
			assert_ok!(Cml::start_mining(
				Origin::signed(owner2),
				cml_id2,
				[2u8; 32],
				owner2,
				b"miner_ip2".to_vec(),
				Some(b"orbit_id".to_vec()),
			));

			assert_ok!(Cml::start_staking(
				Origin::signed(user_id1),
				cml_id1,
				None,
				None
			));
			assert_ok!(Cml::start_staking(
				Origin::signed(user_id2),
				cml_id2,
				None,
				None
			));

			frame_system::Pallet::<Test>::set_block_number(40);
			Cml::collect_staking_info();
			assert_eq!(ActiveStakingSnapshot::<Test>::get(cml_id1).len(), 2);
			assert_eq!(ActiveStakingSnapshot::<Test>::get(cml_id2).len(), 2);

			let statements = Cml::estimate_reward_statements(|| 2, |_id| 1); // task point is 1
			assert_eq!(statements.len(), 5);
			const DUMMY_DOLLARS: u128 = 100000;
			// first reward is miner of cml2
			assert_eq!(statements[0], (owner2, cml_id2, DUMMY_DOLLARS / 2));
			assert_eq!(statements[1], (owner1, cml_id1, DUMMY_DOLLARS));
			assert_eq!(statements[2], (user_id1, cml_id1, DUMMY_DOLLARS));
			assert_eq!(statements[3], (owner2, cml_id2, DUMMY_DOLLARS));
			assert_eq!(statements[4], (user_id2, cml_id2, DUMMY_DOLLARS));
		})
	}
}
