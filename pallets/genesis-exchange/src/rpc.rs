use super::*;

const TASK_POINT_BASE: u32 = 1000;
// precision is 1-(18)0 about 0.1 CENT
const K_COEFFICIENT_TOLERANCE_PRECISION: u128 = 1000000000000000000;
// precision is 1-(23)0 about 0.33 DOLLAR
const RATE_PRODUCTION_TOLERANCE_PRECISION: u128 = 1000000000000000000000000;

impl<T: genesis_exchange::Config> genesis_exchange::Pallet<T> {
	/// Returns
	/// 1. current 1TEA equals how many USD amount
	/// 2. current 1USD equals how many TEA amount
	/// 3. exchange remains USD
	/// 4. exchange remains TEA
	/// 5. product of  exchange remains USD and exchange remains TEA
	pub fn current_exchange_rate() -> (
		BalanceOf<T>,
		BalanceOf<T>,
		BalanceOf<T>,
		BalanceOf<T>,
		BalanceOf<T>,
	) {
		let tea_dollar = Self::one_tea_dollar();
		let usd_dollar = Self::one_tea_dollar();

		let exchange_remains_usd = USDStore::<T>::get(OperationAccount::<T>::get());
		let exchange_remains_tea =
			T::CurrencyOperations::free_balance(&OperationAccount::<T>::get());
		let tea_rate =
			Self::delta_withdraw_amount(&tea_dollar, &exchange_remains_tea, &exchange_remains_usd);
		let reverse_rate =
			Self::delta_withdraw_amount(&usd_dollar, &exchange_remains_usd, &exchange_remains_tea);

		if Self::subtract_abs(
			AMMCurveKCoefficient::<T>::get(),
			exchange_remains_usd * exchange_remains_tea,
		) > u128_to_balance::<T>(K_COEFFICIENT_TOLERANCE_PRECISION)
		{
			#[cfg_attr(not(feature = "std"), no_std)]
			{
				log::warn!(
					"exchange production error: expect is {:?}, actual is: {:?}",
					AMMCurveKCoefficient::<T>::get(),
					exchange_remains_usd * exchange_remains_tea,
				);
			}
			#[cfg(feature = "std")]
			{
				println!(
					"exchange production error: expect is {:?}, actual is: {:?}",
					AMMCurveKCoefficient::<T>::get(),
					exchange_remains_usd * exchange_remains_tea,
				);
			}
		}
		if Self::subtract_abs(tea_rate * reverse_rate, tea_dollar * usd_dollar)
			> u128_to_balance::<T>(RATE_PRODUCTION_TOLERANCE_PRECISION)
		{
			#[cfg_attr(not(feature = "std"), no_std)]
			{
				log::warn!(
					"exchange rate error: tea_rate is {:?}, reverse_rate is: {:?}, expect production is: {:?}, actual is :{:?}",
					tea_rate,
					reverse_rate,
					tea_dollar * usd_dollar,
					tea_rate * reverse_rate
				);
			}
			#[cfg(feature = "std")]
			{
				println!(
					"exchange rate error: tea_rate is {:?}, reverse_rate is: {:?}, expect production is: {:?}, actual is :{:?}",
					tea_rate,
					reverse_rate,
					tea_dollar * usd_dollar,
					tea_rate * reverse_rate
				);
			}
		}

		(
			tea_rate,
			reverse_rate,
			exchange_remains_usd,
			exchange_remains_tea,
			exchange_remains_usd * exchange_remains_tea,
		)
	}

	fn subtract_abs(a: BalanceOf<T>, b: BalanceOf<T>) -> BalanceOf<T> {
		if a >= b {
			a - b
		} else {
			b - a
		}
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

	/// each of list items contains the following field:
	/// 1. Account
	/// 2. Projected  7 day mining income (USD)
	/// 3. TEA Account balance (in USD)
	/// 4. USD account balance
	/// 5. genesis loan
	/// 6. USD debt
	/// 7. Total account value
	pub fn user_asset_list() -> Vec<(
		T::AccountId,
		BalanceOf<T>,
		BalanceOf<T>,
		BalanceOf<T>,
		BalanceOf<T>,
		BalanceOf<T>,
		BalanceOf<T>,
	)> {
		// let mut asset_usd_map = BTreeMap::new();
		let cml_assets = Self::collect_cml_assets();
		let tea_assets = Self::collect_tea_assets();
		let usd_assets = Self::collect_usd_assets();
		let genesis_loan_credits = Self::collect_genesis_loan_credit();
		let usd_debts = Self::collect_usd_debts();

		let mut total_assets = Vec::new();
		for (user, _) in CompetitionUsers::<T>::iter() {
			let cml = Self::amount_from_map(&user, &cml_assets);
			let tea = Self::amount_from_map(&user, &tea_assets);
			let usd = Self::amount_from_map(&user, &usd_assets);
			let loan_credit = Self::amount_from_map(&user, &genesis_loan_credits);
			let usd_debt = Self::amount_from_map(&user, &usd_debts);
			let mut total: BalanceOf<T> = Zero::zero();
			total = total
				.saturating_add(cml)
				.saturating_add(tea)
				.saturating_add(usd)
				.saturating_sub(loan_credit)
				.saturating_sub(usd_debt);

			total_assets.push((user.clone(), cml, tea, usd, loan_credit, usd_debt, total));
		}

		total_assets.sort_by(|(_, _, _, _, _, _, a), (_, _, _, _, _, _, b)| a.cmp(b));
		total_assets.reverse();
		total_assets
	}

	pub fn one_tea_dollar() -> BalanceOf<T> {
		u128_to_balance::<T>(10_000_000_000 * 100)
	}

	fn collect_genesis_loan_credit() -> BTreeMap<T::AccountId, BalanceOf<T>> {
		let mut asset_usd_map = BTreeMap::new();
		let (current_exchange_rate, _, _, _, _) = Self::current_exchange_rate();
		let one_tea_dollar = Self::one_tea_dollar();

		CompetitionUsers::<T>::iter().for_each(|(user, _)| {
			let mut credit_total: BalanceOf<T> = Zero::zero();
			for (cml_id, _) in T::GenesisBankOperation::user_collaterals(&user) {
				credit_total = credit_total.saturating_add(
					T::GenesisBankOperation::calculate_loan_amount(cml_id, false),
				);
			}
			asset_usd_map.insert(user, credit_total * current_exchange_rate / one_tea_dollar);
		});
		asset_usd_map
	}

	fn collect_usd_debts() -> BTreeMap<T::AccountId, BalanceOf<T>> {
		let mut asset_usd_map = BTreeMap::new();

		CompetitionUsers::<T>::iter().for_each(|(user, _)| {
			asset_usd_map.insert(user.clone(), USDDebt::<T>::get(&user));
		});
		asset_usd_map
	}

	fn collect_usd_assets() -> BTreeMap<T::AccountId, BalanceOf<T>> {
		let mut asset_usd_map = BTreeMap::new();
		USDStore::<T>::iter()
			.filter(|(user, _)| CompetitionUsers::<T>::contains_key(user))
			.for_each(|(user, amount)| Self::new_or_add_assets(&user, amount, &mut asset_usd_map));
		asset_usd_map
	}

	fn collect_tea_assets() -> BTreeMap<T::AccountId, BalanceOf<T>> {
		let mut asset_usd_map = BTreeMap::new();
		let (current_exchange_rate, _, _, _, _) = Self::current_exchange_rate();
		let one_tea_dollar = Self::one_tea_dollar();

		CompetitionUsers::<T>::iter().for_each(|(user, _)| {
			let tea_free_amount = T::CurrencyOperations::free_balance(&user);
			let tea_reserved_amount = T::CurrencyOperations::reserved_balance(&user);
			Self::new_or_add_assets(
				&user,
				(tea_free_amount.saturating_add(tea_reserved_amount)) * current_exchange_rate
					/ one_tea_dollar,
				&mut asset_usd_map,
			)
		});
		asset_usd_map
	}

	fn collect_cml_assets() -> BTreeMap<T::AccountId, BalanceOf<T>> {
		let mut asset_usd_map = BTreeMap::new();
		// calculate reward statement of current block, we assume each mining cml will get the
		// mining change equally, and each mining task point are same.
		let mining_cmls_count = T::CmlOperation::current_mining_cmls().iter().count() as u32;
		let cml_reward_statements = T::CmlOperation::estimate_reward_statements(
			|| mining_cmls_count,
			|_cml_id| {
				if mining_cmls_count == 0 {
					TASK_POINT_BASE
				} else {
					TASK_POINT_BASE / mining_cmls_count
				}
			},
		);
		let (current_exchange_rate, _, _, _, _) = Self::current_exchange_rate();
		let one_tea_dollar = Self::one_tea_dollar();
		for (user, _, single_block_reward) in cml_reward_statements {
			if !CompetitionUsers::<T>::contains_key(&user) {
				continue;
			}

			let reward_in_tea = Self::estimate_cml_asset_value(single_block_reward);
			let reward_in_usd = reward_in_tea * current_exchange_rate / one_tea_dollar;

			Self::new_or_add_assets(&user, reward_in_usd, &mut asset_usd_map);
		}
		asset_usd_map
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

	fn amount_from_map(
		user: &T::AccountId,
		map: &BTreeMap<T::AccountId, BalanceOf<T>>,
	) -> BalanceOf<T> {
		if map.contains_key(user) {
			map[user]
		} else {
			Zero::zero()
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
	use frame_support::assert_ok;
	use pallet_cml::{ActiveStakingSnapshot, CmlType, StakingSnapshotItem};
	use pallet_genesis_bank::{
		from_cml_id, AssetType, AssetUniqueId, CollateralStore, Loan, UserCollateralStore,
	};

	#[test]
	fn current_exchange_rate_works() {
		new_test_ext().execute_with(|| {
			let (current_exchange_rate, _, _, _, _) = GenesisExchange::current_exchange_rate();
			assert_eq!(current_exchange_rate, 999975000625);

			// test to check precision
			/*
			let user = 1;
			<Test as Config>::Currency::make_free_balance_be(
				&user,
				1000000000000000000000000000000000,
			);

			let one_tea_dollar = GenesisExchange::one_tea_dollar();
			for i in 0..39999 {
				assert_ok!(GenesisExchange::tea_to_usd(
					Origin::signed(user),
					Some(one_tea_dollar),
					None,
				));
				let (_, _, exchange_remains_usd, exchange_remains_tea, _) =
					GenesisExchange::current_exchange_rate();
				if i == 39998 {
					println!("---end---");
					println!("exchange_remains_usd: {}", exchange_remains_usd);
					println!("exchange_remains_tea: {}", exchange_remains_tea);
				}
			}
			*/
		})
	}

	#[test]
	fn reverse_exchange_rate_works() {
		new_test_ext().execute_with(|| {
			let (_, reverse_rate, _, _, _) = GenesisExchange::current_exchange_rate();
			assert_eq!(reverse_rate, 999975000625);

			// test to check precision
			/*
			let user = 1;
			USDStore::<Test>::insert(&user, 1000000000000000000000000000000000);

			let one_usd_dollar = GenesisExchange::one_tea_dollar();
			for i in 0..39999 {
				assert_ok!(GenesisExchange::usd_to_tea(
					Origin::signed(user),
					Some(one_usd_dollar),
					None,
				));
				let (_, _, exchange_remains_usd, exchange_remains_tea, _) =
					GenesisExchange::current_exchange_rate();
				if i == 39998 {
					println!("---end---");
					println!("exchange_remains_usd: {}", exchange_remains_usd);
					println!("exchange_remains_tea: {}", exchange_remains_tea);
				}
			}
			*/
		})
	}

	#[test]
	fn collect_genesis_loan_credit_works() {
		new_test_ext().execute_with(|| {
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
					term_update_at: 0,
					owner: COMPETITION_USERS2,
					loan_type: CmlType::B,
					principal: 100_000,
					interest: 0,
				},
			);
			CollateralStore::<Test>::insert(
				&asset2,
				Loan {
					start_at: 0,
					term_update_at: 0,
					owner: COMPETITION_USERS2,
					loan_type: CmlType::B,
					principal: 100_000,
					interest: 0,
				},
			);
			CollateralStore::<Test>::insert(
				&asset3,
				Loan {
					start_at: 0,
					term_update_at: 0,
					owner: COMPETITION_USERS2,
					loan_type: CmlType::B,
					principal: 100_000,
					interest: 0,
				},
			);
			CollateralStore::<Test>::insert(
				&asset4,
				Loan {
					start_at: 0,
					term_update_at: 0,
					owner: COMPETITION_USERS3,
					loan_type: CmlType::B,
					principal: 100_000,
					interest: 0,
				},
			);

			UserCollateralStore::<Test>::insert(COMPETITION_USERS2, asset1, ());
			UserCollateralStore::<Test>::insert(COMPETITION_USERS2, asset2, ());
			UserCollateralStore::<Test>::insert(COMPETITION_USERS2, asset3, ());
			UserCollateralStore::<Test>::insert(COMPETITION_USERS3, asset4, ());

			let asset_usd_map = GenesisExchange::collect_genesis_loan_credit();

			assert_eq!(asset_usd_map[&COMPETITION_USERS1], 0);
			assert_eq!(asset_usd_map[&COMPETITION_USERS2], 300_292);
			assert_eq!(asset_usd_map[&COMPETITION_USERS3], 100_097);
		})
	}

	#[test]
	fn collect_usd_assets_works() {
		new_test_ext().execute_with(|| {
			let asset_usd_map = GenesisExchange::collect_usd_assets();

			assert_eq!(asset_usd_map.len(), 3);
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
			let asset_usd_map = GenesisExchange::collect_tea_assets();
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

			let asset_usd_map = GenesisExchange::collect_tea_assets();
			assert_eq!(asset_usd_map.len(), 3);
			assert_eq!(asset_usd_map[&COMPETITION_USERS1], 99);
			assert_eq!(asset_usd_map[&COMPETITION_USERS2], 199);
			assert_eq!(asset_usd_map[&COMPETITION_USERS3], 299);
		})
	}

	#[test]
	fn collect_cml_assets_works() {
		new_test_ext().execute_with(|| {
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

			let asset_usd_map = GenesisExchange::collect_cml_assets();

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

			// prepare genesis loan
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
					term_update_at: 0,
					owner: COMPETITION_USERS2,
					loan_type: CmlType::B,
					principal: 100_000,
					interest: 0,
				},
			);
			CollateralStore::<Test>::insert(
				&asset2,
				Loan {
					start_at: 0,
					term_update_at: 0,
					owner: COMPETITION_USERS2,
					loan_type: CmlType::B,
					principal: 100_000,
					interest: 0,
				},
			);
			CollateralStore::<Test>::insert(
				&asset3,
				Loan {
					start_at: 0,
					term_update_at: 0,
					owner: COMPETITION_USERS2,
					loan_type: CmlType::B,
					principal: 100_000,
					interest: 0,
				},
			);
			CollateralStore::<Test>::insert(
				&asset4,
				Loan {
					start_at: 0,
					term_update_at: 0,
					owner: COMPETITION_USERS3,
					loan_type: CmlType::B,
					principal: 100_000,
					interest: 0,
				},
			);
			UserCollateralStore::<Test>::insert(COMPETITION_USERS2, asset1, ());
			UserCollateralStore::<Test>::insert(COMPETITION_USERS2, asset2, ());
			UserCollateralStore::<Test>::insert(COMPETITION_USERS2, asset3, ());
			UserCollateralStore::<Test>::insert(COMPETITION_USERS3, asset4, ());

			let asset_list = GenesisExchange::user_asset_list();
			assert_eq!(asset_list.len(), 3);
			// asset list is reverse order with total USD amount
			assert_eq!(
				asset_list[0],
				(
					COMPETITION_USERS1,
					14399640008,
					99,
					COMPETITION_USER_USD_AMOUNT,
					0,
					0,
					COMPETITION_USER_USD_AMOUNT + 99 + 14399640008
				)
			);
			assert_eq!(
				asset_list[1],
				(
					COMPETITION_USERS3,
					7199820004,
					299,
					COMPETITION_USER_USD_AMOUNT,
					100097,
					0,
					COMPETITION_USER_USD_AMOUNT + 299 + 7199820004 - 100097
				)
			);
			assert_eq!(
				asset_list[2],
				(
					COMPETITION_USERS2,
					7199820004,
					199,
					COMPETITION_USER_USD_AMOUNT,
					300_292,
					0,
					COMPETITION_USER_USD_AMOUNT + 199 + 7199820004 - 300_292
				)
			);
		})
	}

	#[test]
	fn buy_tea_to_usd_works_after_large_amount_exchange() {
		new_test_ext().execute_with(|| {
			let user = 1;

			let (current_exchange_rate, reverse_rate, _, _, _) =
				GenesisExchange::current_exchange_rate();
			assert_eq!(current_exchange_rate, 999975000625);
			assert_eq!(reverse_rate, 999975000625);

			let buy_usd_amount = 30_000 * 10_000_000_000 * 100;
			let user_tea_amount = 120_000 * 10_000_000_000 * 100;
			<Test as Config>::Currency::make_free_balance_be(&user, user_tea_amount);

			assert_ok!(GenesisExchange::tea_to_usd(
				Origin::signed(user),
				Some(buy_usd_amount),
				None
			));
			assert_eq!(<Test as Config>::Currency::free_balance(&user), 0);
			assert_eq!(USDStore::<Test>::get(user), buy_usd_amount);

			assert_eq!(
				<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
				OPERATION_TEA_AMOUNT + user_tea_amount
			);
			assert_eq!(
				USDStore::<Test>::get(&OperationAccount::<Test>::get()),
				OPERATION_USD_AMOUNT - buy_usd_amount
			);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get())
					* USDStore::<Test>::get(&OperationAccount::<Test>::get()),
				AMMCurveKCoefficient::<Test>::get(),
			);

			let (current_exchange_rate, reverse_rate, _, _, _) =
				GenesisExchange::current_exchange_rate();
			assert_eq!(current_exchange_rate, 62499609378);
			assert_eq!(reverse_rate, 15998400159985);
		})
	}

	#[test]
	fn sell_tea_to_usd_works_after_large_amount_exchange() {
		new_test_ext().execute_with(|| {
			let user = 1;

			let (current_exchange_rate, reverse_rate, _, _, _) =
				GenesisExchange::current_exchange_rate();
			assert_eq!(current_exchange_rate, 999975000625);
			assert_eq!(reverse_rate, 999975000625);

			let withdraw_usd_amount = 30_000 * 10_000_000_000 * 100;
			let user_tea_amount = 120_000 * 10_000_000_000 * 100;
			<Test as Config>::Currency::make_free_balance_be(&user, user_tea_amount);

			assert_ok!(GenesisExchange::tea_to_usd(
				Origin::signed(user),
				None,
				Some(user_tea_amount),
			));
			assert_eq!(<Test as Config>::Currency::free_balance(&user), 0);
			assert_eq!(USDStore::<Test>::get(user), withdraw_usd_amount);

			assert_eq!(
				<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
				OPERATION_TEA_AMOUNT + user_tea_amount
			);
			assert_eq!(
				USDStore::<Test>::get(&OperationAccount::<Test>::get()),
				OPERATION_USD_AMOUNT - withdraw_usd_amount
			);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get())
					* USDStore::<Test>::get(&OperationAccount::<Test>::get()),
				AMMCurveKCoefficient::<Test>::get(),
			);

			let (current_exchange_rate, reverse_rate, _, _, _) =
				GenesisExchange::current_exchange_rate();
			assert_eq!(current_exchange_rate, 62499609378);
			assert_eq!(reverse_rate, 15998400159985);
		})
	}

	#[test]
	fn buy_usd_to_tea_works_after_large_amount_exchange() {
		new_test_ext().execute_with(|| {
			let user = 1;

			let (current_exchange_rate, reverse_rate, _, _, _) =
				GenesisExchange::current_exchange_rate();
			assert_eq!(current_exchange_rate, 999975000625);
			assert_eq!(reverse_rate, 999975000625);

			let buy_tea_amount = 30_000 * 10_000_000_000 * 100;
			let deposit_amount = 120_000 * 10_000_000_000 * 100;
			USDStore::<Test>::insert(user, deposit_amount);

			assert_ok!(GenesisExchange::usd_to_tea(
				Origin::signed(user),
				Some(buy_tea_amount),
				None
			));
			assert_eq!(
				<Test as Config>::Currency::free_balance(&user),
				buy_tea_amount
			);
			assert_eq!(USDStore::<Test>::get(user), 0);

			assert_eq!(
				<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
				OPERATION_TEA_AMOUNT - buy_tea_amount
			);
			assert_eq!(
				USDStore::<Test>::get(&OperationAccount::<Test>::get()),
				OPERATION_USD_AMOUNT + deposit_amount
			);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get())
					* USDStore::<Test>::get(&OperationAccount::<Test>::get()),
				AMMCurveKCoefficient::<Test>::get(),
			);

			let (current_exchange_rate, reverse_rate, _, _, _) =
				GenesisExchange::current_exchange_rate();
			assert_eq!(current_exchange_rate, 15998400159985);
			assert_eq!(reverse_rate, 62499609378);
		})
	}

	#[test]
	fn sell_usd_to_tea_works_after_large_amount_exchange() {
		new_test_ext().execute_with(|| {
			let user = 1;

			let (current_exchange_rate, reverse_rate, _, _, _) =
				GenesisExchange::current_exchange_rate();
			assert_eq!(current_exchange_rate, 999975000625);
			assert_eq!(reverse_rate, 999975000625);

			let withdraw_delta = 30_000 * 10_000_000_000 * 100;
			let user_usd_amount = 120_000 * 10_000_000_000 * 100;
			USDStore::<Test>::insert(user, user_usd_amount);

			assert_ok!(GenesisExchange::usd_to_tea(
				Origin::signed(user),
				None,
				Some(user_usd_amount),
			));
			assert_eq!(
				<Test as Config>::Currency::free_balance(&user),
				withdraw_delta
			);
			assert_eq!(USDStore::<Test>::get(user), 0);

			assert_eq!(
				<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
				OPERATION_TEA_AMOUNT - withdraw_delta
			);
			assert_eq!(
				USDStore::<Test>::get(&OperationAccount::<Test>::get()),
				OPERATION_USD_AMOUNT + user_usd_amount
			);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get())
					* USDStore::<Test>::get(&OperationAccount::<Test>::get()),
				AMMCurveKCoefficient::<Test>::get(),
			);

			let (current_exchange_rate, reverse_rate, _, _, _) =
				GenesisExchange::current_exchange_rate();
			assert_eq!(current_exchange_rate, 15998400159985);
			assert_eq!(reverse_rate, 62499609378);
		})
	}
}
