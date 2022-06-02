use super::*;

// precision is 1-(18)0 about 0.1 DOLLAR
const K_COEFFICIENT_TOLERANCE_PRECISION: u128 = 10000000000000000000000;
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

	pub fn one_tea_dollar() -> BalanceOf<T> {
		u128_to_balance::<T>(10_000_000_000 * 100)
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
