use super::*;

impl<T: genesis_exchange::Config> genesis_exchange::Pallet<T> {
	/// current 1TEA equals how many USD amount.
	pub fn current_exchange_rate() -> BalanceOf<T> {
		let dollar = Self::one_tea_dollar();

		let exchange_remains_usd = USDStore::<T>::get(OperationAccount::<T>::get());
		let exchange_remains_tea =
			T::CurrencyOperations::free_balance(&OperationAccount::<T>::get());
		Self::delta_deposit_amount(&dollar, &exchange_remains_tea, &exchange_remains_usd)
	}

	pub fn estimate_amount(withdraw_amount: BalanceOf<T>, buy_tea: bool) -> BalanceOf<T> {
		let exchange_remains_usd = USDStore::<T>::get(OperationAccount::<T>::get());
		let exchange_remains_tea =
			T::CurrencyOperations::free_balance(&OperationAccount::<T>::get());

		match buy_tea {
			true => Self::delta_deposit_amount(
				&withdraw_amount,
				&exchange_remains_tea,
				&exchange_remains_usd,
			),
			false => Self::delta_deposit_amount(
				&withdraw_amount,
				&exchange_remains_usd,
				&exchange_remains_tea,
			),
		}
	}

	pub fn user_asset_list() -> Vec<(T::AccountId, BalanceOf<T>)> {
		let mut asset_usd_map = BTreeMap::new();
		Self::collect_cml_assets(&mut asset_usd_map);
		Self::collect_tea_assets(&mut asset_usd_map);
		Self::collect_usd_assets(&mut asset_usd_map);

		let mut total_assets: Vec<(T::AccountId, BalanceOf<T>)> = asset_usd_map
			.iter()
			.filter(|(user, _)| CompetitionUsers::<T>::contains_key(user))
			.map(|(user, reward)| (user.clone(), *reward))
			.collect();
		total_assets.sort_by(|(_, a), (_, b)| a.cmp(b));
		total_assets.reverse();
		total_assets
	}

	pub fn one_tea_dollar() -> BalanceOf<T> {
		u128_to_balance::<T>(10_000_000_000 * 100)
	}

	fn collect_usd_assets(asset_usd_map: &mut BTreeMap<T::AccountId, BalanceOf<T>>) {
		USDStore::<T>::iter()
			.for_each(|(user, amount)| Self::new_or_add_assets(&user, amount, asset_usd_map))
	}

	fn collect_tea_assets(asset_usd_map: &mut BTreeMap<T::AccountId, BalanceOf<T>>) {
		let current_exchange_rate = Self::current_exchange_rate();
		let one_tea_dollar = Self::one_tea_dollar();

		CompetitionUsers::<T>::iter().for_each(|(user, _)| {
			let tea_amount = T::CurrencyOperations::free_balance(&user);
			Self::new_or_add_assets(
				&user,
				tea_amount * current_exchange_rate / one_tea_dollar,
				asset_usd_map,
			)
		});
	}

	fn collect_cml_assets(asset_usd_map: &mut BTreeMap<T::AccountId, BalanceOf<T>>) {
		// calculate reward statement of current block, we assume each mining cml will get the
		// mining change equally, and each mining task point are same.
		let cml_reward_statements = T::CmlOperation::estimate_reward_statements(
			|| T::CmlOperation::current_mining_cmls().iter().count() as u32,
			|_cml_id| 1u32,
		);
		let current_exchange_rate = Self::current_exchange_rate();
		let one_tea_dollar = Self::one_tea_dollar();
		for (user, _, single_block_reward) in cml_reward_statements {
			let reward_in_tea = Self::estimate_cml_asset_value(single_block_reward);
			let reward_in_usd = reward_in_tea * current_exchange_rate / one_tea_dollar;

			Self::new_or_add_assets(&user, reward_in_usd, asset_usd_map);
		}
	}

	fn new_or_add_assets(
		user: &T::AccountId,
		amount: BalanceOf<T>,
		asset_usd_map: &mut BTreeMap<T::AccountId, BalanceOf<T>>,
	) {
		if let Some(old) = asset_usd_map.remove(user) {
			asset_usd_map.insert(user.clone(), old + amount);
		} else {
			asset_usd_map.insert(user.clone(), amount);
		}
	}

	fn estimate_cml_asset_value(single_block_reward: BalanceOf<T>) -> BalanceOf<T> {
		Self::reward_of_one_day(single_block_reward) * T::PER::get()
	}

	fn reward_of_one_day(single_block_reward: BalanceOf<T>) -> BalanceOf<T> {
		// average block timespan is 6 seconds
		single_block_reward * (10u32 * 60u32 * 24u32).into()
	}
}

fn u128_to_balance<T: Config>(amount: u128) -> BalanceOf<T> {
	amount.try_into().map_err(|_| "").unwrap()
}

#[cfg(test)]
mod tests {
	use crate::mock::*;
	use crate::*;
	use pallet_cml::{ActiveStakingSnapshot, StakingSnapshotItem};

	#[test]
	fn collect_usd_assets_works() {
		new_test_ext().execute_with(|| {
			let mut asset_usd_map = BTreeMap::new();
			GenesisExchange::collect_usd_assets(&mut asset_usd_map);

			assert_eq!(asset_usd_map.len(), 4);

			assert_eq!(
				USDStore::<Test>::get(OPERATION_ACCOUNT),
				OPERATION_USD_AMOUNT
			);
			assert_eq!(
				USDStore::<Test>::get(COMPETITION_USERS1),
				COMPETITION_USER_USD_AMOUNT
			);
			assert_eq!(
				USDStore::<Test>::get(COMPETITION_USERS2),
				COMPETITION_USER_USD_AMOUNT
			);
			assert_eq!(
				USDStore::<Test>::get(COMPETITION_USERS3),
				COMPETITION_USER_USD_AMOUNT
			);
		});
	}

	#[test]
	fn collect_tea_assets_works() {
		new_test_ext().execute_with(|| {
			let mut asset_usd_map = BTreeMap::new();

			GenesisExchange::collect_tea_assets(&mut asset_usd_map);
			assert_eq!(asset_usd_map.len(), 3);
			assert_eq!(asset_usd_map[&COMPETITION_USERS1], 0);
			assert_eq!(asset_usd_map[&COMPETITION_USERS2], 0);
			assert_eq!(asset_usd_map[&COMPETITION_USERS3], 0);

			let amount1 = 100;
			let amount2 = 200;
			let amount3 = 200;
			<Test as Config>::Currency::make_free_balance_be(&COMPETITION_USERS1, amount1);
			<Test as Config>::Currency::make_free_balance_be(&COMPETITION_USERS2, amount2);
			<Test as Config>::Currency::make_free_balance_be(&COMPETITION_USERS3, amount3);

			GenesisExchange::collect_tea_assets(&mut asset_usd_map);
			assert_eq!(asset_usd_map.len(), 3);
			assert_eq!(asset_usd_map[&COMPETITION_USERS1], amount1);
			assert_eq!(asset_usd_map[&COMPETITION_USERS2], amount2);
			assert_eq!(asset_usd_map[&COMPETITION_USERS3], amount3);
		})
	}

	#[test]
	fn collect_cml_assets_works() {
		new_test_ext().execute_with(|| {
			let mut asset_usd_map = BTreeMap::new();

			let cml_id1 = 1;
			ActiveStakingSnapshot::<Test>::insert(
				cml_id1,
				vec![
					StakingSnapshotItem {
						owner: COMPETITION_USERS1,
						weight: 1,
						staking_at: 0,
					},
					StakingSnapshotItem {
						owner: COMPETITION_USERS1,
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
						owner: COMPETITION_USERS2,
						weight: 1,
						staking_at: 0,
					},
					StakingSnapshotItem {
						owner: COMPETITION_USERS3,
						weight: 3,
						staking_at: 1,
					},
				],
			);

			GenesisExchange::collect_cml_assets(&mut asset_usd_map);

			assert_eq!(asset_usd_map.len(), 3);
			assert_eq!(asset_usd_map[&COMPETITION_USERS1], 14400014400);
			assert_eq!(asset_usd_map[&COMPETITION_USERS2], 7200007200);
			assert_eq!(asset_usd_map[&COMPETITION_USERS3], 7200007200);
		});
	}

	#[test]
	fn user_asset_list_works() {
		new_test_ext().execute_with(|| {
			// prepare tea balance
			let amount1 = 100;
			let amount2 = 200;
			let amount3 = 200;
			<Test as Config>::Currency::make_free_balance_be(&COMPETITION_USERS1, amount1);
			<Test as Config>::Currency::make_free_balance_be(&COMPETITION_USERS2, amount2);
			<Test as Config>::Currency::make_free_balance_be(&COMPETITION_USERS3, amount3);

			// prepare cml
			ActiveStakingSnapshot::<Test>::insert(
				1,
				vec![
					StakingSnapshotItem {
						owner: COMPETITION_USERS1,
						weight: 1,
						staking_at: 0,
					},
					StakingSnapshotItem {
						owner: COMPETITION_USERS1,
						weight: 2,
						staking_at: 1,
					},
				],
			);
			ActiveStakingSnapshot::<Test>::insert(
				2,
				vec![
					StakingSnapshotItem {
						owner: COMPETITION_USERS2,
						weight: 1,
						staking_at: 0,
					},
					StakingSnapshotItem {
						owner: COMPETITION_USERS3,
						weight: 3,
						staking_at: 1,
					},
				],
			);

			let asset_list = GenesisExchange::user_asset_list();
			assert_eq!(asset_list.len(), 3);
			// asset list is reverse order with total USD amount
			assert_eq!(
				asset_list[0],
				(
					COMPETITION_USERS1,
					COMPETITION_USER_USD_AMOUNT + amount1 + 14400014400
				)
			);
			assert_eq!(
				asset_list[1],
				(
					COMPETITION_USERS3,
					COMPETITION_USER_USD_AMOUNT + amount3 + 7200007200
				)
			);
			assert_eq!(
				asset_list[2],
				(
					COMPETITION_USERS2,
					COMPETITION_USER_USD_AMOUNT + amount2 + 7200007200
				)
			);
		})
	}
}
