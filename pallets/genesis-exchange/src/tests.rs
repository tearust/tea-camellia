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
			withdraw_amount
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
		let tea_amount = 100090 * 10_000_000_000;
		<Test as Config>::Currency::make_free_balance_be(&user, tea_amount);

		assert_ok!(GenesisExchange::tea_to_usd(
			Origin::signed(user),
			tea_amount
		));
		assert_eq!(<Test as Config>::Currency::free_balance(&user), 0);

		// deposit to exchange again
		<Test as Config>::Currency::make_free_balance_be(&user, tea_amount);
		assert_ok!(GenesisExchange::tea_to_usd(
			Origin::signed(user),
			tea_amount
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
			GenesisExchange::tea_to_usd(Origin::signed(user), OPERATION_USD_AMOUNT + 1),
			Error::<Test>::ExchangeInsufficientTEA
		);
		assert_noop!(
			GenesisExchange::tea_to_usd(Origin::signed(user), OPERATION_USD_AMOUNT),
			Error::<Test>::ExchangeInsufficientTEA
		);
	})
}

#[test]
fn test_to_usd_should_fail_if_withdraw_amount_is_zero() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			GenesisExchange::tea_to_usd(Origin::signed(1), 0),
			Error::<Test>::WithdrawAmountShouldNotBeZero
		);
	})
}

#[test]
fn test_to_usd_should_fail_if_user_do_not_have_enough_tea() {
	new_test_ext().execute_with(|| {
		let user = 1;
		assert_eq!(<Test as Config>::Currency::free_balance(&user), 0);

		assert_noop!(
			GenesisExchange::tea_to_usd(Origin::signed(1), 100),
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
			withdraw_amount
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
		let usd_amount = 100090 * 10_000_000_000;
		USDStore::<Test>::insert(user, usd_amount);

		assert_ok!(GenesisExchange::usd_to_tea(
			Origin::signed(user),
			usd_amount
		));
		assert_eq!(USDStore::<Test>::get(user), 0);

		// deposit to exchange again
		USDStore::<Test>::insert(user, usd_amount);

		assert_ok!(GenesisExchange::usd_to_tea(
			Origin::signed(user),
			usd_amount
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
			GenesisExchange::usd_to_tea(Origin::signed(user), OPERATION_USD_AMOUNT + 1),
			Error::<Test>::ExchangeInsufficientUSD
		);
		assert_noop!(
			GenesisExchange::usd_to_tea(Origin::signed(user), OPERATION_USD_AMOUNT),
			Error::<Test>::ExchangeInsufficientUSD
		);
	})
}

#[test]
fn usd_to_tea_should_fail_if_withdraw_amount_is_zero() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			GenesisExchange::usd_to_tea(Origin::signed(1), 0),
			Error::<Test>::WithdrawAmountShouldNotBeZero
		);
	})
}

#[test]
fn usd_to_tea_should_fail_if_user_do_not_have_enough_tea() {
	new_test_ext().execute_with(|| {
		let user = 1;
		assert_eq!(USDStore::<Test>::get(user), 0);

		assert_noop!(
			GenesisExchange::usd_to_tea(Origin::signed(1), 100),
			Error::<Test>::UserInsufficientUSD
		);
	})
}
