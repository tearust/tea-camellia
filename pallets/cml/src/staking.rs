use super::*;

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
			if let Some(cml) = CmlStore::<T>::get(miner_item.cml_id) {
				let mut snapshot_items = Vec::new();
				let mut current_index = 0;
				for slot in cml.staking_slots() {
					let weight = match slot.cml {
						Some(cml_id) => {
							if let Some(cml) = CmlStore::<T>::get(cml_id) {
								cml.staking_weight()
							} else {
								1
							}
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
			}
		});
	}

	pub(crate) fn clear_staking_info() {
		ActiveStakingSnapshot::<T>::remove_all();
	}

	pub(crate) fn calculate_staking() {
		let total_task_point = Self::service_task_point_total();

		let snapshots: Vec<(CmlId, Vec<StakingSnapshotItem<T::AccountId>>)> =
			ActiveStakingSnapshot::<T>::iter().collect();
		for (cml_id, snapshot_items) in snapshots {
			let miner_task_point = Self::get_miner_task_point(cml_id);
			let miner_staking_point = T::StakingEconomics::miner_staking_point(&snapshot_items);

			let miner_total_reward = T::StakingEconomics::total_staking_rewards_of_miner(
				miner_task_point,
				total_task_point,
			);

			for item in snapshot_items.iter() {
				let mut reward = T::StakingEconomics::single_staking_reward(
					miner_total_reward,
					miner_staking_point,
					item,
				);

				let owner = item.owner.clone();
				if GenesisMinerCreditStore::<T>::contains_key(&owner) {
					let credit = GenesisMinerCreditStore::<T>::get(&owner);
					if credit > reward {
						GenesisMinerCreditStore::<T>::insert(&owner, credit.saturating_sub(reward));
						reward = BalanceOf::<T>::zero();
					} else {
						GenesisMinerCreditStore::<T>::remove(&owner);
						reward = reward.saturating_sub(credit);
					}
				}
				if reward.is_zero() {
					continue;
				}

				AccountRewards::<T>::mutate(&item.owner, |balance| match balance {
					Some(balance) => {
						*balance = balance.saturating_add(reward);
					}
					None => {
						*balance = Some(reward.into());
					}
				})
			}
		}
	}

	pub(crate) fn service_task_point_total() -> ServiceTaskPoint {
		// todo calculate service task total point later
		1
	}

	pub(crate) fn get_miner_task_point(_cml_id: CmlId) -> ServiceTaskPoint {
		// todo implement me later
		1
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
	) -> Result<StakingItem<T::AccountId, BalanceOf<T>>, DispatchError> {
		let free_balance = T::CurrencyOperations::free_balance(who);
		let (actual_staking, credit_amount) = if free_balance < T::StakingPrice::get() {
			let credit_amount = T::StakingPrice::get()
				.checked_sub(&free_balance)
				.ok_or(Error::<T>::InvalidCreditAmount)?;
			(free_balance, Some(credit_amount))
		} else {
			(T::StakingPrice::get(), None)
		};

		let result = Self::create_balance_staking(who, actual_staking)?;
		if let Some(credit_amount) = credit_amount {
			GenesisMinerCreditStore::<T>::insert(who, credit_amount);
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

impl<T: cml::Config> StakingEconomics<BalanceOf<T>> for cml::Pallet<T> {
	type AccountId = T::AccountId;

	fn increase_issuance(_total_point: u64) -> BalanceOf<T> {
		// todo implement me later
		BalanceOf::<T>::zero()
	}

	fn total_staking_rewards_of_miner(_miner_point: u64, _total_point: u64) -> BalanceOf<T> {
		// todo implement me later
		BalanceOf::<T>::zero()
	}

	fn miner_staking_point(
		_snapshots: &Vec<StakingSnapshotItem<Self::AccountId>>,
	) -> MinerStakingPoint {
		// todo implement me later
		1
	}

	fn single_staking_reward(
		_miner_total_rewards: BalanceOf<T>,
		_total_staking_point: MinerStakingPoint,
		_snapshot_item: &StakingSnapshotItem<Self::AccountId>,
	) -> BalanceOf<T> {
		// todo implement me later
		const CENTS: node_primitives::Balance = 10_000_000_000;
		const DOLLARS: node_primitives::Balance = 100 * CENTS;
		(1 * DOLLARS).try_into().unwrap_or(BalanceOf::<T>::zero())
	}
}

#[cfg(test)]
mod tests {
	use crate::mock::*;
	use crate::tests::new_genesis_seed;
	use crate::{
		AccountRewards, ActiveStakingSnapshot, CmlStore, CmlType, GenesisMinerCreditStore,
		MinerItem, MinerItemStore, MinerStatus, StakingCategory, StakingItem, StakingProperties,
		StakingSnapshotItem, CML,
	};

	const CENTS: node_primitives::Balance = 10_000_000_000;
	const DOLLARS: node_primitives::Balance = 100 * CENTS;

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
			for user_id in 1..=5 {
				assert_eq!(AccountRewards::<Test>::get(user_id).unwrap(), 1 * DOLLARS);
			}
		})
	}

	#[test]
	fn calculate_staking_works_when_there_are_genesis_miner_credits() {
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
						owner: 1,
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
						owner: 2,
						weight: 1,
						staking_at: 0,
					},
					StakingSnapshotItem {
						owner: 3,
						weight: 3,
						staking_at: 1,
					},
				],
			);

			let credit_amount = DOLLARS * 2;
			GenesisMinerCreditStore::<Test>::insert(1, credit_amount);
			GenesisMinerCreditStore::<Test>::insert(2, credit_amount);

			Cml::calculate_staking();

			assert!(!GenesisMinerCreditStore::<Test>::contains_key(1));
			assert_eq!(AccountRewards::<Test>::get(&1), None);

			assert!(GenesisMinerCreditStore::<Test>::contains_key(2));
			assert_eq!(GenesisMinerCreditStore::<Test>::get(2), DOLLARS);
			assert!(AccountRewards::<Test>::get(&2).is_none());

			assert!(AccountRewards::<Test>::get(&3).is_some());
			assert_eq!(AccountRewards::<Test>::get(&3).unwrap(), DOLLARS);
		})
	}
}
