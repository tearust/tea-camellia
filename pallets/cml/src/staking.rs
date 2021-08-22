use super::*;

const TASK_POINT_BASE: ServiceTaskPoint = 1000;

impl<T: cml::Config> cml::Pallet<T> {
	pub(crate) fn is_staking_period_start(height: T::BlockNumber) -> bool {
		height % T::StakingPeriodLength::get() == 1u32.into()
	}

	pub(crate) fn is_staking_period_end(height: T::BlockNumber) -> bool {
		height % T::StakingPeriodLength::get() == 0u32.into()
	}

	pub(crate) fn check_balance_staking(who: &T::AccountId) -> DispatchResult {
		ensure!(
			T::CurrencyOperations::free_balance(who) >= T::StakingPrice::get(),
			Error::<T>::InsufficientFreeBalance,
		);
		Ok(())
	}

	pub(crate) fn collect_staking_info() {
		MinerItemStore::<T>::iter().for_each(|(_, miner_item)| {
			let cml = CmlStore::<T>::get(miner_item.cml_id);
			let mut snapshot_items = Vec::new();
			let mut current_index = 0;
			for slot in cml.staking_slots() {
				let weight = match slot.cml {
					Some(cml_id) => {
						let cml = CmlStore::<T>::get(cml_id);
						cml.staking_weight()
					}
					None => 1,
				};
				snapshot_items.push(StakingSnapshotItem {
					owner: slot.owner.clone(),
					staking_at: current_index,
					weight,
				});

				current_index += weight;
			}

			ActiveStakingSnapshot::<T>::insert(cml.id(), snapshot_items);
		});
	}

	pub(crate) fn clear_staking_info() {
		ActiveStakingSnapshot::<T>::remove_all();
		MiningCmlTaskPoints::<T>::remove_all();
	}

	pub(crate) fn calculate_staking() {
		let reward_statements = Self::estimate_reward_statements(
			Self::service_task_point_total,
			Self::miner_task_point,
		);

		for (owner, _cml_id, initial_reward) in reward_statements.iter() {
			let reward = Self::try_return_left_staking_reward(owner, *initial_reward);
			if reward.is_zero() {
				continue;
			}

			if AccountRewards::<T>::contains_key(owner) {
				AccountRewards::<T>::mutate(owner, |balance| {
					*balance = balance.saturating_add(reward);
				});
			} else {
				AccountRewards::<T>::insert(owner, reward);
			}
		}

		Self::deposit_event(Event::RewardStatements(reward_statements));
	}

	/// return a pair of values, first is current performance calculated by given block height,
	/// the second is the peak performance.
	pub(crate) fn miner_performance(
		cml_id: CmlId,
		block_height: &T::BlockNumber,
	) -> (Performance, Performance) {
		let cml = CmlStore::<T>::get(cml_id);
		let age_percentage = if cml.lifespan().is_zero() {
			100u32.into()
		} else {
			if let Some(plant_at_block) = cml.get_plant_at() {
				(*block_height - *plant_at_block) * 100u32.into()
					/ cml.lifespan()
			}
			else{
				0u32.into()
			}
		};

		(
			cml.calculate_performance(age_percentage.try_into().unwrap_or(0)),
			cml.get_peak_performance(),
		)
	}

	pub(crate) fn try_return_left_staking_reward(
		owner: &T::AccountId,
		initial_reward: BalanceOf<T>,
	) -> BalanceOf<T> {
		let mut reward = initial_reward;
		while let Some(cml_id) = Self::first_credit_cml(owner) {
			reward = Self::pay_single_mining_credit(owner, cml_id, reward);
			if reward.is_zero() {
				break;
			}
		}
		reward
	}

	pub(crate) fn pay_single_mining_credit(
		owner: &T::AccountId,
		cml_id: CmlId,
		initial_reward: BalanceOf<T>,
	) -> BalanceOf<T> {
		let mut reward = initial_reward;

		let credit = GenesisMinerCreditStore::<T>::get(&owner, cml_id);
		let should_reserved = if credit > reward {
			GenesisMinerCreditStore::<T>::insert(&owner, cml_id, credit.saturating_sub(reward));
			let reserved = reward;
			reward = BalanceOf::<T>::zero();
			reserved
		} else {
			GenesisMinerCreditStore::<T>::remove(&owner, cml_id);
			reward = reward.saturating_sub(credit);
			credit
		};

		T::CurrencyOperations::deposit_creating(&owner, should_reserved);
		if T::CurrencyOperations::reserve(&owner, should_reserved).is_err() {
			// should never happen, set reward to zero just in case
			reward = BalanceOf::<T>::zero();
		}

		reward
	}

	pub(crate) fn first_credit_cml(owner: &T::AccountId) -> Option<CmlId> {
		GenesisMinerCreditStore::<T>::iter_prefix(owner)
			.take(1)
			.next()
			.map(|(id, _)| id)
	}

	pub(crate) fn service_task_point_total() -> ServiceTaskPoint {
		let mut total: ServiceTaskPoint = 0;
		for (_, point) in MiningCmlTaskPoints::<T>::iter() {
			total = total.saturating_add(point);
		}
		total * TASK_POINT_BASE
	}

	pub(crate) fn miner_task_point(cml_id: CmlId) -> ServiceTaskPoint {
		MiningCmlTaskPoints::<T>::get(cml_id) * TASK_POINT_BASE
	}

	pub(crate) fn check_miner_first_staking(
		who: &T::AccountId,
		cml: &CML<T::AccountId, T::BlockNumber, BalanceOf<T>, T::SeedFreshDuration>,
	) -> DispatchResult {
		if !cml.is_from_genesis() {
			Self::check_balance_staking(who)?;
		}
		Ok(())
	}

	pub(crate) fn create_genesis_miner_balance_staking(
		who: &T::AccountId,
		cml_id: CmlId,
	) -> Result<StakingItem<T::AccountId, BalanceOf<T>>, DispatchError> {
		let free_balance = T::CurrencyOperations::free_balance(who);
		let (actual_staking, credit_amount) = if free_balance < T::StakingPrice::get() {
			(Zero::zero(), Some(T::StakingPrice::get()))
		} else {
			(T::StakingPrice::get(), None)
		};

		let result = Self::create_balance_staking(who, actual_staking)?;
		if let Some(credit_amount) = credit_amount {
			if !GenesisMinerCreditStore::<T>::contains_key(who, cml_id) {
				GenesisMinerCreditStore::<T>::insert(who, cml_id, credit_amount);
			} else {
				GenesisMinerCreditStore::<T>::mutate(who, cml_id, |amount| {
					*amount = amount.saturating_add(credit_amount);
				});
			}
		}
		Ok(result)
	}

	pub(crate) fn create_balance_staking(
		who: &T::AccountId,
		actual_staking: BalanceOf<T>,
	) -> Result<StakingItem<T::AccountId, BalanceOf<T>>, DispatchError> {
		T::CurrencyOperations::reserve(who, actual_staking)?;
		Ok(StakingItem {
			owner: who.clone(),
			category: StakingCategory::Tea,
			amount: Some(T::StakingPrice::get()),
			cml: None,
		})
	}
}

const DOLLARS: u32 = 100000;
/// simple implementation of staking economics, this should only be used in unit tests,
/// and can not be used in production environment.
impl<T: cml::Config> StakingEconomics<BalanceOf<T>, T::AccountId> for cml::Pallet<T> {
	/// Calculate issuance balance with given total task point of current staking window.
	fn increase_issuance(total_point: ServiceTaskPoint) -> BalanceOf<T> {
		(total_point * DOLLARS).into()
	}

	/// Calculate total staking rewards of the given miner, the staking rewards should split to all staking
	/// users.
	fn total_staking_rewards_of_miner(
		miner_point: ServiceTaskPoint,
		_total_point: ServiceTaskPoint,
		_performance: Performance,
	) -> BalanceOf<T> {
		(miner_point * DOLLARS).into()
	}

	/// Calculate all staking weight about the given miner.
	fn miner_total_staking_weight(
		snapshots: &Vec<StakingSnapshotItem<T::AccountId>>,
	) -> BalanceOf<T> {
		(snapshots.len() as u32 * DOLLARS).into()
	}

	/// Calculate a single staking reward.
	fn single_staking_reward(
		_miner_total_rewards: BalanceOf<T>,
		_total_staking_point: BalanceOf<T>,
		_snapshot_item: &StakingSnapshotItem<T::AccountId>,
	) -> BalanceOf<T> {
		DOLLARS.into()
	}
}

#[cfg(test)]
mod tests {
	use crate::mock::*;
	use crate::tests::new_genesis_seed;
	use crate::{
		AccountRewards, ActiveStakingSnapshot, CmlStore, CmlType, Config, GenesisMinerCreditStore,
		MinerItem, MinerItemStore, MinerStatus, StakingCategory, StakingItem, StakingProperties,
		StakingSnapshotItem, CML,
	};
	use frame_support::traits::Currency;

	#[test]
	fn staking_period_related_works() {
		new_test_ext().execute_with(|| {
			assert!(Cml::is_staking_period_end(0));
			assert!(Cml::is_staking_period_start(1));

			for i in 2..STAKING_PERIOD_LENGTH as u64 {
				assert!(!Cml::is_staking_period_end(i));
				assert!(!Cml::is_staking_period_start(i));
			}

			assert!(Cml::is_staking_period_end(STAKING_PERIOD_LENGTH as u64));
			assert!(Cml::is_staking_period_start(
				STAKING_PERIOD_LENGTH as u64 + 1
			));
		})
	}

	#[test]
	fn collect_staking_info_works() {
		new_test_ext().execute_with(|| {
			let cml_id1 = 1;
			let cml_id2 = 2;
			let cml_id3 = 3;
			let cml_id4 = 4;
			let cml_id5 = 5;

			let mut cml1 = CML::from_genesis_seed(new_genesis_seed(cml_id1));
			cml1.staking_slots_mut().push(StakingItem {
				owner: 1,
				category: StakingCategory::Tea,
				amount: Some(1),
				cml: None,
			});
			cml1.staking_slots_mut().push(StakingItem {
				owner: 3,
				category: StakingCategory::Cml,
				amount: None,
				cml: Some(cml_id3),
			});
			cml1.staking_slots_mut().push(StakingItem {
				owner: 5,
				category: StakingCategory::Cml,
				amount: None,
				cml: Some(cml_id5),
			});
			CmlStore::<Test>::insert(cml_id1, cml1);

			let mut cml2 = CML::from_genesis_seed(new_genesis_seed(cml_id2));
			cml2.staking_slots_mut().push(StakingItem {
				owner: 2,
				category: StakingCategory::Tea,
				amount: Some(1),
				cml: None,
			});
			cml2.staking_slots_mut().push(StakingItem {
				owner: 4,
				category: StakingCategory::Cml,
				amount: None,
				cml: Some(cml_id4),
			});
			CmlStore::<Test>::insert(cml_id2, cml2);

			CmlStore::<Test>::insert(cml_id3, CML::from_genesis_seed(new_genesis_seed(cml_id3)));

			let mut seed4 = new_genesis_seed(cml_id4);
			seed4.cml_type = CmlType::B; // let cml4 be CmlType B
			CmlStore::<Test>::insert(cml_id4, CML::from_genesis_seed(seed4));

			CmlStore::<Test>::insert(cml_id5, CML::from_genesis_seed(new_genesis_seed(cml_id5)));

			MinerItemStore::<Test>::insert(
				[1; 32],
				MinerItem {
					cml_id: cml_id1,
					id: [1; 32],
					ip: vec![],
					status: MinerStatus::Active,
				},
			);
			MinerItemStore::<Test>::insert(
				[2; 32],
				MinerItem {
					cml_id: cml_id2,
					id: [2; 32],
					ip: vec![],
					status: MinerStatus::Active,
				},
			);

			Cml::collect_staking_info();

			assert_eq!(ActiveStakingSnapshot::<Test>::iter().count(), 2);
			let snapshot1 = ActiveStakingSnapshot::<Test>::get(cml_id1);
			assert_eq!(snapshot1.len(), 3);
			assert_eq!(snapshot1[0].owner, 1);
			assert_eq!(snapshot1[0].weight, 1);
			assert_eq!(snapshot1[0].staking_at, 0);
			assert_eq!(snapshot1[1].owner, 3);
			assert_eq!(snapshot1[1].weight, 3);
			assert_eq!(snapshot1[1].staking_at, 1);
			assert_eq!(snapshot1[2].owner, 5);
			assert_eq!(snapshot1[2].weight, 3);
			assert_eq!(snapshot1[2].staking_at, 4);

			let snapshot2 = ActiveStakingSnapshot::<Test>::get(cml_id2);
			assert_eq!(snapshot2.len(), 2);
			assert_eq!(snapshot2[0].owner, 2);
			assert_eq!(snapshot2[0].weight, 1);
			assert_eq!(snapshot2[0].staking_at, 0);
			assert_eq!(snapshot2[1].owner, 4);
			assert_eq!(snapshot2[1].weight, 2);
			assert_eq!(snapshot2[1].staking_at, 1);
		})
	}

	#[test]
	fn calculate_staking_works() {
		new_test_ext().execute_with(|| {
			let cml_id1 = 1;
			ActiveStakingSnapshot::<Test>::insert(
				cml_id1,
				vec![
					StakingSnapshotItem {
						owner: 1,
						weight: 1,
						staking_at: 0,
					},
					StakingSnapshotItem {
						owner: 2,
						weight: 2,
						staking_at: 1,
					},
				],
			);

			let cml_id2 = 2;
			ActiveStakingSnapshot::<Test>::insert(
				cml_id2,
				vec![
					StakingSnapshotItem {
						owner: 3,
						weight: 1,
						staking_at: 0,
					},
					StakingSnapshotItem {
						owner: 4,
						weight: 3,
						staking_at: 1,
					},
					StakingSnapshotItem {
						owner: 5,
						weight: 1,
						staking_at: 4,
					},
				],
			);

			Cml::calculate_staking();

			assert_eq!(AccountRewards::<Test>::iter().count(), 5);
			for user_id in 1..=2 {
				assert_eq!(AccountRewards::<Test>::get(user_id), DOLLARS);
			}
			for user_id in 3..=5 {
				assert_eq!(AccountRewards::<Test>::get(user_id), DOLLARS);
			}
		})
	}

	#[test]
	fn calculate_staking_works_when_there_are_genesis_miner_credits() {
		new_test_ext().execute_with(|| {
			let user1 = 1;
			let user2 = 2;
			let user3 = 3;
			let origin_free_amount = 10000;
			<Test as Config>::Currency::make_free_balance_be(&user1, origin_free_amount);
			<Test as Config>::Currency::make_free_balance_be(&user2, origin_free_amount);
			<Test as Config>::Currency::make_free_balance_be(&user3, origin_free_amount);

			let cml_id1 = 1;
			ActiveStakingSnapshot::<Test>::insert(
				cml_id1,
				vec![
					StakingSnapshotItem {
						owner: user1,
						weight: 1,
						staking_at: 0,
					},
					StakingSnapshotItem {
						owner: user1,
						weight: 2,
						staking_at: 1,
					},
				],
			);

			let cml_id2 = 2;
			ActiveStakingSnapshot::<Test>::insert(
				cml_id2,
				vec![
					StakingSnapshotItem {
						owner: user2,
						weight: 1,
						staking_at: 0,
					},
					StakingSnapshotItem {
						owner: user3,
						weight: 3,
						staking_at: 1,
					},
				],
			);

			let credit_amount = DOLLARS * 2;
			GenesisMinerCreditStore::<Test>::insert(user1, cml_id1, credit_amount);
			GenesisMinerCreditStore::<Test>::insert(user2, cml_id2, credit_amount);

			Cml::calculate_staking();

			assert!(!GenesisMinerCreditStore::<Test>::contains_key(
				&user1, cml_id1
			));
			assert!(!AccountRewards::<Test>::contains_key(&user1));
			assert_eq!(
				<Test as Config>::Currency::reserved_balance(&user1),
				DOLLARS * 2
			);

			assert!(GenesisMinerCreditStore::<Test>::contains_key(
				&user2, cml_id2
			));
			assert_eq!(
				GenesisMinerCreditStore::<Test>::get(user2, cml_id2),
				DOLLARS
			);
			assert!(!AccountRewards::<Test>::contains_key(&user2));
			assert_eq!(
				<Test as Config>::Currency::reserved_balance(&user2),
				DOLLARS
			);

			assert!(AccountRewards::<Test>::contains_key(&user3));
			assert_eq!(AccountRewards::<Test>::get(&user3), DOLLARS);
			assert_eq!(<Test as Config>::Currency::reserved_balance(&user3), 0);
		})
	}

	#[test]
	fn calculate_staking_works_if_snapshots_is_empty() {
		new_test_ext().execute_with(|| {
			assert_eq!(ActiveStakingSnapshot::<Test>::iter().count(), 0);
			Cml::calculate_staking();
			assert_eq!(AccountRewards::<Test>::iter().count(), 0);
		})
	}

	#[test]
	fn try_return_left_staking_reward_works() {
		new_test_ext().execute_with(|| {
			let user1 = 1;
			let initial_amount = 1000;
			<Test as Config>::Currency::make_free_balance_be(&user1, initial_amount);

			let initial_reward1 = 50;
			let cml_id = 11;
			assert!(!GenesisMinerCreditStore::<Test>::contains_key(
				&user1, cml_id
			));
			let reward = Cml::try_return_left_staking_reward(&user1, initial_reward1);
			assert_eq!(reward, initial_reward1); // return all rewards if not have credit
			assert_eq!(
				<Test as Config>::Currency::free_balance(&user1),
				initial_amount
			);
			assert_eq!(<Test as Config>::Currency::reserved_balance(&user1), 0);

			let credit_amount2 = 1000;
			let initial_reward2 = 100; // initial reward small than credit amount
			GenesisMinerCreditStore::<Test>::insert(user1, cml_id, credit_amount2);
			let reward = Cml::try_return_left_staking_reward(&user1, initial_reward2);
			assert_eq!(reward, 0); // should have no reward
			assert_eq!(
				<Test as Config>::Currency::free_balance(&user1),
				initial_amount
			);
			assert_eq!(
				<Test as Config>::Currency::reserved_balance(&user1),
				initial_reward2
			);
			assert_eq!(
				GenesisMinerCreditStore::<Test>::get(user1, cml_id),
				credit_amount2 - initial_reward2
			);

			let initial_reward3 = 1000; // initial reward can pay all the credit
			let reward = Cml::try_return_left_staking_reward(&user1, initial_reward3);
			assert_eq!(reward, initial_reward3 - (credit_amount2 - initial_reward2));
			assert_eq!(
				<Test as Config>::Currency::free_balance(&user1),
				initial_amount
			);
			assert_eq!(
				<Test as Config>::Currency::reserved_balance(&user1),
				credit_amount2
			);
			assert!(!GenesisMinerCreditStore::<Test>::contains_key(
				user1, cml_id
			))
		})
	}

	#[test]
	fn try_return_left_staking_reward_works_with_multipile_credits() {
		new_test_ext().execute_with(|| {
			let user1 = 1;

			let cml_id1 = 11;
			GenesisMinerCreditStore::<Test>::insert(&user1, cml_id1, STAKING_PRICE);

			let cml_id2 = 22;
			GenesisMinerCreditStore::<Test>::insert(&user1, cml_id2, STAKING_PRICE);

			let reward =
				Cml::try_return_left_staking_reward(&user1, STAKING_PRICE + STAKING_PRICE / 2);
			assert_eq!(reward, 0);
			assert_eq!(
				GenesisMinerCreditStore::<Test>::iter_prefix(user1).count(),
				1
			);
			assert_eq!(
				GenesisMinerCreditStore::<Test>::iter_prefix(user1)
					.next()
					.map(|(_, v)| v)
					.unwrap(),
				STAKING_PRICE / 2
			);
			assert_eq!(
				<Test as Config>::Currency::reserved_balance(&user1),
				STAKING_PRICE + STAKING_PRICE / 2
			);

			let reward = Cml::try_return_left_staking_reward(&user1, STAKING_PRICE);
			assert_eq!(reward, STAKING_PRICE / 2);
			assert_eq!(
				GenesisMinerCreditStore::<Test>::iter_prefix(user1).count(),
				0
			);
			assert_eq!(
				<Test as Config>::Currency::reserved_balance(&user1),
				STAKING_PRICE * 2
			);
		})
	}
}
