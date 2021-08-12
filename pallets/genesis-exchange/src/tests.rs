use crate::mock::*;
use crate::*;
use frame_support::{assert_noop, assert_ok};

#[test]
fn tea_to_usd_works() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let tea_amount = 1000;
		<Test as Config>::Currency::make_free_balance_be(&user, tea_amount);

		assert_eq!(<Test as Config>::Currency::free_balance(&user), tea_amount);
		assert_eq!(USDStore::<Test>::get(user), 0);

		assert_eq!(
			<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
			OPERATION_TEA_AMOUNT
		);
		assert_eq!(
			USDStore::<Test>::get(OperationAccount::<Test>::get()),
			OPERATION_USD_AMOUNT
		);

		let withdraw_amount = 100;
		assert_ok!(GenesisExchange::tea_to_usd(
			Origin::signed(user),
			Some(withdraw_amount),
			None
		));

		assert_eq!(
			<Test as Config>::Currency::free_balance(&user),
			tea_amount - withdraw_amount
		);
		assert_eq!(USDStore::<Test>::get(user), withdraw_amount);

		assert_eq!(
			<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
			OPERATION_TEA_AMOUNT + withdraw_amount
		);
		assert_eq!(
			USDStore::<Test>::get(OperationAccount::<Test>::get()),
			OPERATION_USD_AMOUNT - withdraw_amount
		);
	})
}

#[test]
fn tea_to_usd_works_if_withdraw_all_remains_usd() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let usd_amount = 1_000_900_000_000_000;
		let deposit_amount = 1_026_587_793_051_634;
		<Test as Config>::Currency::make_free_balance_be(&user, deposit_amount);

		assert_ok!(GenesisExchange::tea_to_usd(
			Origin::signed(user),
			Some(usd_amount),
			None
		));
		assert_eq!(<Test as Config>::Currency::free_balance(&user), 0);

		// deposit to exchange again
		let deposit_amount = 1_080_669_919_155_786;
		<Test as Config>::Currency::make_free_balance_be(&user, deposit_amount);
		assert_ok!(GenesisExchange::tea_to_usd(
			Origin::signed(user),
			Some(usd_amount),
			None
		));
		assert_eq!(<Test as Config>::Currency::free_balance(&user), 0);
	})
}

#[test]
fn tea_to_usd_should_fail_if_withdraw_amount_larger_than_exchange_really_has() {
	new_test_ext().execute_with(|| {
		let user = 1;
		assert_eq!(
			USDStore::<Test>::get(OperationAccount::<Test>::get()),
			OPERATION_USD_AMOUNT
		);
		<Test as Config>::Currency::make_free_balance_be(&user, OPERATION_USD_AMOUNT + 1);
		assert_noop!(
			GenesisExchange::tea_to_usd(Origin::signed(user), Some(OPERATION_USD_AMOUNT + 1), None),
			Error::<Test>::ExchangeInsufficientUSD
		);
		assert_noop!(
			GenesisExchange::tea_to_usd(Origin::signed(user), Some(OPERATION_USD_AMOUNT), None),
			Error::<Test>::ExchangeInsufficientUSD
		);
	})
}

#[test]
fn test_to_usd_should_fail_if_withdraw_amount_is_zero() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			GenesisExchange::tea_to_usd(Origin::signed(1), Some(0), None),
			Error::<Test>::AmountShouldNotBeZero
		);
	})
}

#[test]
fn test_to_usd_should_fail_if_user_do_not_have_enough_tea() {
	new_test_ext().execute_with(|| {
		let user = 1;
		assert_eq!(<Test as Config>::Currency::free_balance(&user), 0);

		assert_noop!(
			GenesisExchange::tea_to_usd(Origin::signed(1), Some(100), None),
			Error::<Test>::UserInsufficientTEA
		);
	})
}

#[test]
fn usd_to_tea_works() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let usd_amount = 1000;
		USDStore::<Test>::insert(user, usd_amount);

		assert_eq!(<Test as Config>::Currency::free_balance(&user), 0);
		assert_eq!(USDStore::<Test>::get(user), usd_amount);

		assert_eq!(
			<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
			OPERATION_TEA_AMOUNT
		);
		assert_eq!(
			USDStore::<Test>::get(OperationAccount::<Test>::get()),
			OPERATION_USD_AMOUNT
		);

		let withdraw_amount = 100;
		assert_ok!(GenesisExchange::usd_to_tea(
			Origin::signed(user),
			Some(withdraw_amount),
			None
		));

		assert_eq!(
			<Test as Config>::Currency::free_balance(&user),
			withdraw_amount
		);
		assert_eq!(USDStore::<Test>::get(user), usd_amount - withdraw_amount);

		assert_eq!(
			<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
			OPERATION_TEA_AMOUNT - withdraw_amount
		);
		assert_eq!(
			USDStore::<Test>::get(OperationAccount::<Test>::get()),
			OPERATION_USD_AMOUNT + withdraw_amount
		);
	})
}

#[test]
fn usd_to_tea_works_if_withdraw_all_remains_usd() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let tea_amount = 1_000_900_000_000_000;
		let deposit_amount = 1_026_587_793_051_634;
		USDStore::<Test>::insert(user, deposit_amount);

		assert_ok!(GenesisExchange::usd_to_tea(
			Origin::signed(user),
			Some(tea_amount),
			None
		));
		assert_eq!(USDStore::<Test>::get(user), 0);

		// deposit to exchange again
		let deposit_amount = 1_080_669_919_155_786;
		USDStore::<Test>::insert(user, deposit_amount);

		assert_ok!(GenesisExchange::usd_to_tea(
			Origin::signed(user),
			Some(tea_amount),
			None
		));
		assert_eq!(USDStore::<Test>::get(user), 0);
	})
}

#[test]
fn usd_to_tea_should_fail_if_withdraw_amount_larger_than_exchange_really_has() {
	new_test_ext().execute_with(|| {
		let user = 1;
		assert_eq!(
			<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
			OPERATION_TEA_AMOUNT
		);
		USDStore::<Test>::insert(user, OPERATION_USD_AMOUNT + 1);
		assert_noop!(
			GenesisExchange::usd_to_tea(Origin::signed(user), Some(OPERATION_USD_AMOUNT + 1), None),
			Error::<Test>::ExchangeInsufficientTEA
		);
		assert_noop!(
			GenesisExchange::usd_to_tea(Origin::signed(user), Some(OPERATION_USD_AMOUNT), None),
			Error::<Test>::ExchangeInsufficientTEA
		);
	})
}

#[test]
fn usd_to_tea_should_fail_if_withdraw_amount_is_zero() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			GenesisExchange::usd_to_tea(Origin::signed(1), Some(0), None),
			Error::<Test>::AmountShouldNotBeZero
		);
	})
}

#[test]
fn usd_to_tea_should_fail_if_user_do_not_have_enough_tea() {
	new_test_ext().execute_with(|| {
		let user = 1;
		assert_eq!(USDStore::<Test>::get(user), 0);

		assert_noop!(
			GenesisExchange::usd_to_tea(Origin::signed(1), Some(100), None),
			Error::<Test>::UserInsufficientUSD
		);
	})
}

#[test]
fn tea_to_usd_works_after_large_amount_exchange() {
	new_test_ext().execute_with(|| {
		let user = 1;

		assert_eq!(GenesisExchange::current_exchange_rate(), 1000025000625);
		assert_eq!(GenesisExchange::reverse_exchange_rate(), 1000025000625);

		let withdraw_delta = 30_000 * 10_000_000_000 * 100;
		let deposit_amount = 120_000 * 10_000_000_000 * 100;
		<Test as Config>::Currency::make_free_balance_be(&user, deposit_amount);

		assert_ok!(GenesisExchange::tea_to_usd(
			Origin::signed(user),
			Some(withdraw_delta),
			None
		));
		assert_eq!(<Test as Config>::Currency::free_balance(&user), 0);
		assert_eq!(USDStore::<Test>::get(user), withdraw_delta);

		assert_eq!(
			<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
			OPERATION_TEA_AMOUNT + deposit_amount
		);
		assert_eq!(
			USDStore::<Test>::get(&OperationAccount::<Test>::get()),
			OPERATION_USD_AMOUNT - withdraw_delta
		);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get())
				* USDStore::<Test>::get(&OperationAccount::<Test>::get()),
			AMMCurveKCoefficient::<Test>::get(),
		);

		assert_eq!(GenesisExchange::current_exchange_rate(), 16001600160016);
		assert_eq!(GenesisExchange::reverse_exchange_rate(), 62500390627);
	})
}

#[test]
fn usd_to_tea_works_after_large_amount_exchange() {
	new_test_ext().execute_with(|| {
		let user = 1;

		assert_eq!(GenesisExchange::current_exchange_rate(), 1000025000625);
		assert_eq!(GenesisExchange::reverse_exchange_rate(), 1000025000625);

		let withdraw_delta = 30_000 * 10_000_000_000 * 100;
		let deposit_amount = 120_000 * 10_000_000_000 * 100;
		USDStore::<Test>::insert(user, deposit_amount);

		assert_ok!(GenesisExchange::usd_to_tea(
			Origin::signed(user),
			Some(withdraw_delta),
			None
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
			OPERATION_USD_AMOUNT + deposit_amount
		);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get())
				* USDStore::<Test>::get(&OperationAccount::<Test>::get()),
			AMMCurveKCoefficient::<Test>::get(),
		);

		assert_eq!(GenesisExchange::current_exchange_rate(), 62500390627);
		assert_eq!(GenesisExchange::reverse_exchange_rate(), 16001600160016);
	})
}
