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

	pub fn collect_staking_info() {
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

		for (owner, _cml_id, reward) in reward_statements.iter() {
			if AccountRewards::<T>::contains_key(owner) {
				AccountRewards::<T>::mutate(owner, |balance| {
					*balance = balance.saturating_add(*reward);
				});
			} else {
				AccountRewards::<T>::insert(owner, reward);
			}
		}

		Self::deposit_event(Event::RewardStatements(reward_statements));
	}

	pub(crate) fn service_task_point_total() -> ServiceTaskPoint {
		let mut total: ServiceTaskPoint = 0;
		for (_, point) in MiningCmlTaskPoints::<T>::iter() {
			total = total.saturating_add(point);
		}
		total * TaskPointBase::<T>::get()
	}

	pub(crate) fn miner_task_point(cml_id: CmlId) -> ServiceTaskPoint {
		MiningCmlTaskPoints::<T>::get(cml_id) * TaskPointBase::<T>::get()
	}

	pub(crate) fn check_miner_first_staking(who: &T::AccountId) -> DispatchResult {
		Self::check_balance_staking(who)?;
		Ok(())
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
		AccountRewards, ActiveStakingSnapshot, CmlStore, CmlType, MinerItem, MinerItemStore,
		MinerStatus, StakingCategory, StakingItem, StakingProperties, StakingSnapshotItem, CML,
	};

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
	fn calculate_staking_works_if_snapshots_is_empty() {
		new_test_ext().execute_with(|| {
			assert_eq!(ActiveStakingSnapshot::<Test>::iter().count(), 0);
			Cml::calculate_staking();
			assert_eq!(AccountRewards::<Test>::iter().count(), 0);
		})
	}
}
