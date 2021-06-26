use super::*;

impl<T: cml::Config> cml::Pallet<T> {
	pub(crate) fn is_staking_period_start(height: T::BlockNumber) -> bool {
		height % T::StakingPeriodLength::get() == 1u32.into()
	}

	pub(crate) fn is_staking_period_end(height: T::BlockNumber) -> bool {
		height % T::StakingPeriodLength::get() == 0u32.into()
	}

	pub(crate) fn check_balance_staking(_who: &T::AccountId) -> DispatchResult {
		// todo implement me later
		// ensure!(
		// 	T::CurrencyOperations::free_balance(&sender) > T::StakingPrice::get(),
		// 	Error::<T>::InsufficientFreeBalance,
		// );
		Ok(())
	}

	pub(crate) fn collect_staking_info() {
		CmlStore::<T>::iter()
			.filter(|(_, cml)| cml.is_mining())
			.for_each(|(_, cml)| {
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
			});
	}

	pub(crate) fn clear_staking_info() {
		ActiveStakingSnapshot::<T>::remove_all();
	}

	pub(crate) fn calculate_staking() {
		let total_task_point = Self::service_task_point_total();

		ActiveStakingSnapshot::<T>::iter().for_each(|(cml_id, snapshot_items)| {
			let miner_task_point = Self::get_miner_task_point(cml_id);
			let miner_staking_point = T::StakingEconomics::miner_staking_point(&snapshot_items);

			let miner_total_reward = T::StakingEconomics::total_staking_rewards_of_miner(
				miner_task_point,
				total_task_point,
			);

			snapshot_items.iter().for_each(|item| {
				let reward = T::StakingEconomics::single_staking_reward(
					miner_total_reward,
					miner_staking_point,
					item,
				);
				AccountRewards::<T>::mutate(&item.owner, |balance| match balance {
					Some(balance) => {
						*balance = balance.saturating_add(reward);
					}
					None => {
						*balance = Some(reward);
					}
				})
			})
		});
	}

	pub(crate) fn service_task_point_total() -> ServiceTaskPoint {
		// todo calculate service task total point later
		1
	}

	pub(crate) fn get_miner_task_point(_cml_id: CmlId) -> ServiceTaskPoint {
		// todo implement me later
		1
	}

	pub(crate) fn create_balance_staking(
		who: &T::AccountId,
	) -> Result<StakingItem<T::AccountId, BalanceOf<T>>, DispatchError> {
		let staking_price: BalanceOf<T> = T::StakingPrice::get();

		// todo implement me later
		// T::CurrencyOperations::reserve(who, staking_price)?;
		Ok(StakingItem {
			owner: who.clone(),
			category: StakingCategory::Tea,
			amount: Some(staking_price),
			cml: None,
		})
	}

	pub(crate) fn check_seed_staking(
		cml_id: CmlId,
		current_height: &T::BlockNumber,
	) -> DispatchResult {
		let cml = CmlStore::<T>::get(cml_id);
		ensure!(cml.is_some(), Error::<T>::NotFoundCML);
		let cml = cml.unwrap();
		ensure!(
			cml.seed_valid(current_height)
				.map_err(|e| Error::<T>::from(e))?
				|| cml
					.tree_valid(current_height)
					.map_err(|e| Error::<T>::from(e))?,
			Error::<T>::ShouldStakingLiveTree
		);
		Ok(())
	}

	#[allow(dead_code)]
	pub(crate) fn create_seed_staking(
		who: &T::AccountId,
		cml_id: CmlId,
		current_height: &T::BlockNumber,
	) -> Result<StakingItem<T::AccountId, BalanceOf<T>>, DispatchError> {
		CmlStore::<T>::mutate(cml_id, |cml| match cml {
			Some(cml) => {
				if cml.is_seed() {
					Self::seed_to_tree(cml, current_height)?;
				}
				Ok(())
			}
			None => Err(Error::<T>::NotFoundCML),
		})?;

		Ok(StakingItem {
			owner: who.clone(),
			category: StakingCategory::Cml,
			amount: None,
			cml: Some(cml_id),
		})
	}
}

impl<T: cml::Config> StakingEconomics for cml::Pallet<T> {
	type AccountId = T::AccountId;

	fn increase_issuance(_total_point: u64) -> node_primitives::Balance {
		// todo implement me later
		1
	}

	fn total_staking_rewards_of_miner(
		_miner_point: u64,
		_total_point: u64,
	) -> node_primitives::Balance {
		// todo implement me later
		1
	}

	fn miner_staking_point(
		_snapshots: &Vec<StakingSnapshotItem<Self::AccountId>>,
	) -> MinerStakingPoint {
		// todo implement me later
		1
	}

	fn single_staking_reward(
		_miner_total_rewards: node_primitives::Balance,
		_total_staking_point: MinerStakingPoint,
		_snapshot_item: &StakingSnapshotItem<Self::AccountId>,
	) -> node_primitives::Balance {
		// todo implement me later
		const CENTS: node_primitives::Balance = 10_000_000_000;
		const DOLLARS: node_primitives::Balance = 100 * CENTS;
		1 * DOLLARS
	}
}

#[cfg(test)]
mod tests {
	use crate::mock::*;

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
}
