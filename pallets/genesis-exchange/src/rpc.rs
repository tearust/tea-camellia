use super::*;
use sp_runtime::traits::Saturating;

impl<T: genesis_exchange::Config> genesis_exchange::Pallet<T> {
	/// current 1TEA equals how many USD amount.
	pub fn current_exchange_rate() -> BalanceOf<T> {
		let tea_dollar = Self::one_tea_dollar();

		let exchange_remains_usd = USDStore::<T>::get(OperationAccount::<T>::get());
		let exchange_remains_tea =
			T::CurrencyOperations::free_balance(&OperationAccount::<T>::get());
		Self::delta_withdraw_amount(&tea_dollar, &exchange_remains_tea, &exchange_remains_usd)
	}

	/// current 1USD equals how many TEA amount.
	pub fn reverse_exchange_rate() -> BalanceOf<T> {
		let usd_dollar = Self::one_tea_dollar();

		let exchange_remains_usd = USDStore::<T>::get(OperationAccount::<T>::get());
		let exchange_remains_tea =
			T::CurrencyOperations::free_balance(&OperationAccount::<T>::get());
		Self::delta_withdraw_amount(&usd_dollar, &exchange_remains_usd, &exchange_remains_tea)
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
		Self::exclude_genesis_miner_credit(&mut asset_usd_map);
		Self::exclude_genesis_loan_credit(&mut asset_usd_map);

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

	fn exclude_genesis_loan_credit(asset_usd_map: &mut BTreeMap<T::AccountId, BalanceOf<T>>) {
		let current_height = frame_system::Pallet::<T>::block_number();
		let current_exchange_rate = Self::current_exchange_rate();
		let one_tea_dollar = Self::one_tea_dollar();

		asset_usd_map.iter_mut().for_each(|(user, amount)| {
			let mut credit_total: BalanceOf<T> = Zero::zero();
			for (cml_id, expired_height) in T::GenesisBankOperation::user_collaterals(user) {
				credit_total =
					credit_total.saturating_add(T::GenesisBankOperation::calculate_loan_amount(
						cml_id,
						max(current_height, expired_height),
					));
			}
			*amount = amount.saturating_sub(credit_total * current_exchange_rate / one_tea_dollar)
		})
	}

	fn exclude_genesis_miner_credit(asset_usd_map: &mut BTreeMap<T::AccountId, BalanceOf<T>>) {
		let current_exchange_rate = Self::current_exchange_rate();
		let one_tea_dollar = Self::one_tea_dollar();

		asset_usd_map.iter_mut().for_each(|(user, amount)| {
			let mut credit_total: BalanceOf<T> = Zero::zero();
			for (_, credit_amount) in T::CmlOperation::user_credits(user) {
				credit_total = credit_total.saturating_add(credit_amount);
			}
			*amount = amount.saturating_sub(credit_total * current_exchange_rate / one_tea_dollar)
		})
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
	use pallet_cml::{ActiveStakingSnapshot, GenesisMinerCreditStore, StakingSnapshotItem};
	use pallet_genesis_bank::{
		from_cml_id, AssetType, AssetUniqueId, CollateralStore, Loan, UserCollateralStore,
	};

	#[test]
	fn exclude_genesis_loan_credit_works() {
		new_test_ext().execute_with(|| {
			let current_exchange_rate = GenesisExchange::current_exchange_rate();
			let one_tea_dollar = GenesisExchange::one_tea_dollar();

			let asset1 = AssetUniqueId {
				asset_type: AssetType::CML,
				inner_id: from_cml_id(1),
			};
			let asset2 = AssetUniqueId {
				asset_type: AssetType::CML,
				inner_id: from_cml_id(2),
			};
			let asset3 = AssetUniqueId {
				asset_type: AssetType::CML,
				inner_id: from_cml_id(3),
			};
			let asset4 = AssetUniqueId {
				asset_type: AssetType::CML,
				inner_id: from_cml_id(4),
			};

			CollateralStore::<Test>::insert(
				&asset1,
				Loan {
					start_at: 0,
					owner: COMPETITION_USERS2,
				},
			);
			CollateralStore::<Test>::insert(
				&asset2,
				Loan {
					start_at: 0,
					owner: COMPETITION_USERS2,
				},
			);
			CollateralStore::<Test>::insert(
				&asset3,
				Loan {
					start_at: 0,
					owner: COMPETITION_USERS2,
				},
			);
			CollateralStore::<Test>::insert(
				&asset4,
				Loan {
					start_at: 0,
					owner: COMPETITION_USERS3,
				},
			);

			UserCollateralStore::<Test>::insert(COMPETITION_USERS2, asset1, ());
			UserCollateralStore::<Test>::insert(COMPETITION_USERS2, asset2, ());
			UserCollateralStore::<Test>::insert(COMPETITION_USERS2, asset3, ());
			UserCollateralStore::<Test>::insert(COMPETITION_USERS3, asset4, ());

			let mut asset_usd_map = BTreeMap::new();
			asset_usd_map.insert(COMPETITION_USERS1, GENESIS_CML_LOAN_AMOUNT * 10);
			asset_usd_map.insert(COMPETITION_USERS2, GENESIS_CML_LOAN_AMOUNT * 10);
			asset_usd_map.insert(COMPETITION_USERS3, GENESIS_CML_LOAN_AMOUNT * 10);
			GenesisExchange::exclude_genesis_loan_credit(&mut asset_usd_map);

			assert_eq!(
				asset_usd_map[&COMPETITION_USERS1],
				GENESIS_CML_LOAN_AMOUNT * 10
			);
			assert!(
				asset_usd_map[&COMPETITION_USERS2]
					< GENESIS_CML_LOAN_AMOUNT * 7 * current_exchange_rate / one_tea_dollar
			);
			assert!(
				asset_usd_map[&COMPETITION_USERS3]
					< GENESIS_CML_LOAN_AMOUNT * 9 * current_exchange_rate / one_tea_dollar
			);
		})
	}

	#[test]
	fn exclude_genesis_miner_credit_works() {
		new_test_ext().execute_with(|| {
			let current_exchange_rate = GenesisExchange::current_exchange_rate();
			let one_tea_dollar = GenesisExchange::one_tea_dollar();

			GenesisMinerCreditStore::<Test>::insert(COMPETITION_USERS1, 1, STAKING_PRICE);
			GenesisMinerCreditStore::<Test>::insert(COMPETITION_USERS2, 2, STAKING_PRICE);
			GenesisMinerCreditStore::<Test>::insert(COMPETITION_USERS2, 3, STAKING_PRICE);
			GenesisMinerCreditStore::<Test>::insert(COMPETITION_USERS3, 4, STAKING_PRICE);

			let mut asset_usd_map = BTreeMap::new();
			asset_usd_map.insert(COMPETITION_USERS1, STAKING_PRICE * 10);
			asset_usd_map.insert(COMPETITION_USERS2, STAKING_PRICE * 10);
			asset_usd_map.insert(COMPETITION_USERS3, STAKING_PRICE * 10);
			GenesisExchange::exclude_genesis_miner_credit(&mut asset_usd_map);

			assert_eq!(
				asset_usd_map[&COMPETITION_USERS1],
				STAKING_PRICE * 10 - STAKING_PRICE * current_exchange_rate / one_tea_dollar
			);
			assert_eq!(
				asset_usd_map[&COMPETITION_USERS2],
				STAKING_PRICE * 10 - STAKING_PRICE * 2 * current_exchange_rate / one_tea_dollar
			);
			assert_eq!(
				asset_usd_map[&COMPETITION_USERS3],
				STAKING_PRICE * 10 - STAKING_PRICE * current_exchange_rate / one_tea_dollar
			);
		})
	}

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
			let amount3 = 300;
			<Test as Config>::Currency::make_free_balance_be(&COMPETITION_USERS1, amount1);
			<Test as Config>::Currency::make_free_balance_be(&COMPETITION_USERS2, amount2);
			<Test as Config>::Currency::make_free_balance_be(&COMPETITION_USERS3, amount3);

			GenesisExchange::collect_tea_assets(&mut asset_usd_map);
			assert_eq!(asset_usd_map.len(), 3);
			assert_eq!(asset_usd_map[&COMPETITION_USERS1], 99);
			assert_eq!(asset_usd_map[&COMPETITION_USERS2], 199);
			assert_eq!(asset_usd_map[&COMPETITION_USERS3], 299);
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
			assert_eq!(asset_usd_map[&COMPETITION_USERS1], 14399640008);
			assert_eq!(asset_usd_map[&COMPETITION_USERS2], 7199820004);
			assert_eq!(asset_usd_map[&COMPETITION_USERS3], 7199820004);
		});
	}

	#[test]
	fn user_asset_list_works() {
		new_test_ext().execute_with(|| {
			// prepare tea balance
			let amount1 = 100;
			let amount2 = 200;
			let amount3 = 300;
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
					COMPETITION_USER_USD_AMOUNT + 99 + 14399640008
				)
			);
			assert_eq!(
				asset_list[1],
				(
					COMPETITION_USERS3,
					COMPETITION_USER_USD_AMOUNT + 299 + 7199820004
				)
			);
			assert_eq!(
				asset_list[2],
				(
					COMPETITION_USERS2,
					COMPETITION_USER_USD_AMOUNT + 199 + 7199820004
				)
			);
		})
	}
}
