use crate::mock::*;
use crate::*;
use frame_support::{assert_noop, assert_ok};

#[test]
fn buy_tea_to_usd_works() {
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

		let buy_usd_amount = 100;
		assert_ok!(GenesisExchange::tea_to_usd(
			Origin::signed(user),
			Some(buy_usd_amount),
			None
		));

		assert_eq!(
			<Test as Config>::Currency::free_balance(&user),
			tea_amount - buy_usd_amount
		);
		assert_eq!(USDStore::<Test>::get(user), buy_usd_amount);

		assert_eq!(
			<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
			OPERATION_TEA_AMOUNT + buy_usd_amount
		);
		assert_eq!(
			USDStore::<Test>::get(OperationAccount::<Test>::get()),
			OPERATION_USD_AMOUNT - buy_usd_amount
		);
	})
}

#[test]
fn sell_tea_to_usd_works() {
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

		let sell_tea_amount = 100;
		assert_ok!(GenesisExchange::tea_to_usd(
			Origin::signed(user),
			None,
			Some(sell_tea_amount),
		));

		assert_eq!(
			<Test as Config>::Currency::free_balance(&user),
			tea_amount - sell_tea_amount
		);
		assert_eq!(USDStore::<Test>::get(user), sell_tea_amount);

		assert_eq!(
			<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
			OPERATION_TEA_AMOUNT + sell_tea_amount
		);
		assert_eq!(
			USDStore::<Test>::get(OperationAccount::<Test>::get()),
			OPERATION_USD_AMOUNT - sell_tea_amount
		);
	})
}

#[test]
fn buy_tea_to_usd_works_if_withdraw_all_remains_usd() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let usd_amount = 1_000_900_000_000_000;
		let user_tea_amount = 1_026_587_793_051_634;
		<Test as Config>::Currency::make_free_balance_be(&user, user_tea_amount);

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
fn sell_tea_to_usd_works_if_withdraw_all_remains_usd() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let user_tea_amount = 1_000_900_000_000_000;
		<Test as Config>::Currency::make_free_balance_be(&user, user_tea_amount);

		assert_ok!(GenesisExchange::tea_to_usd(
			Origin::signed(user),
			None,
			Some(user_tea_amount),
		));
		assert_eq!(<Test as Config>::Currency::free_balance(&user), 0);

		// deposit to exchange again
		<Test as Config>::Currency::make_free_balance_be(&user, user_tea_amount);
		assert_ok!(GenesisExchange::tea_to_usd(
			Origin::signed(user),
			None,
			Some(user_tea_amount),
		));
		assert_eq!(<Test as Config>::Currency::free_balance(&user), 0);
	})
}

#[test]
fn tea_to_usd_should_fail_if_both_amount_params_is_some() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			GenesisExchange::tea_to_usd(Origin::signed(1), Some(1), Some(1)),
			Error::<Test>::BuyAndSellAmountShouldNotBothExist
		);
	})
}

#[test]
fn tea_to_usd_should_fail_if_both_amount_params_is_empty() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			GenesisExchange::tea_to_usd(Origin::signed(1), None, None),
			Error::<Test>::BuyOrSellAmountShouldExist
		);
	})
}

#[test]
fn buy_tea_to_usd_should_fail_if_withdraw_amount_larger_than_exchange_really_has() {
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
fn sell_tea_to_usd_works_if_withdraw_amount_larger_than_exchange_amount() {
	new_test_ext().execute_with(|| {
		let user = 1;
		assert_eq!(
			USDStore::<Test>::get(OperationAccount::<Test>::get()),
			OPERATION_USD_AMOUNT
		);
		<Test as Config>::Currency::make_free_balance_be(&user, OPERATION_USD_AMOUNT + 1);
		assert_ok!(GenesisExchange::tea_to_usd(
			Origin::signed(user),
			None,
			Some(OPERATION_USD_AMOUNT + 1)
		));

		assert_eq!(<Test as Config>::Currency::free_balance(&user), 0);
	})
}

#[test]
fn buy_test_to_usd_should_fail_if_withdraw_amount_is_zero() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			GenesisExchange::tea_to_usd(Origin::signed(1), Some(0), None),
			Error::<Test>::AmountShouldNotBeZero
		);
	})
}

#[test]
fn sell_test_to_usd_should_fail_if_withdraw_amount_is_zero() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			GenesisExchange::tea_to_usd(Origin::signed(1), None, Some(0)),
			Error::<Test>::AmountShouldNotBeZero
		);
	})
}

#[test]
fn buy_tea_to_usd_should_fail_if_user_do_not_have_enough_tea() {
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
fn sell_tea_to_usd_should_fail_if_user_do_not_have_enough_tea() {
	new_test_ext().execute_with(|| {
		let user = 1;
		assert_eq!(<Test as Config>::Currency::free_balance(&user), 0);

		assert_noop!(
			GenesisExchange::tea_to_usd(Origin::signed(1), None, Some(100)),
			Error::<Test>::UserInsufficientTEA
		);
	})
}

#[test]
fn buy_usd_to_tea_works() {
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
fn sell_usd_to_tea_works() {
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
			None,
			Some(withdraw_amount),
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
fn usd_to_tea_should_fail_if_both_amount_params_is_some() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			GenesisExchange::usd_to_tea(Origin::signed(1), Some(1), Some(1)),
			Error::<Test>::BuyAndSellAmountShouldNotBothExist
		);
	})
}

#[test]
fn usd_to_tea_should_fail_if_both_amount_params_is_empty() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			GenesisExchange::usd_to_tea(Origin::signed(1), None, None),
			Error::<Test>::BuyOrSellAmountShouldExist
		);
	})
}

#[test]
fn buy_usd_to_tea_works_if_withdraw_all_remains_usd() {
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
fn sell_usd_to_tea_works_if_withdraw_all_remains_usd() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let tea_amount = 1_000_900_000_000_000;
		USDStore::<Test>::insert(user, tea_amount);

		assert_ok!(GenesisExchange::usd_to_tea(
			Origin::signed(user),
			None,
			Some(tea_amount),
		));
		assert_eq!(USDStore::<Test>::get(user), 0);

		// deposit to exchange again
		USDStore::<Test>::insert(user, tea_amount);

		assert_ok!(GenesisExchange::usd_to_tea(
			Origin::signed(user),
			None,
			Some(tea_amount),
		));
		assert_eq!(USDStore::<Test>::get(user), 0);
	})
}

#[test]
fn buy_usd_to_tea_should_fail_if_withdraw_amount_larger_than_exchange_really_has() {
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
fn sell_usd_to_tea_work_if_withdraw_amount_larger_than_exchange_amount() {
	new_test_ext().execute_with(|| {
		let user = 1;
		assert_eq!(
			<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
			OPERATION_TEA_AMOUNT
		);
		USDStore::<Test>::insert(user, OPERATION_USD_AMOUNT + 1);
		assert_ok!(GenesisExchange::usd_to_tea(
			Origin::signed(user),
			None,
			Some(OPERATION_USD_AMOUNT + 1)
		));

		assert_eq!(USDStore::<Test>::get(user), 0);
	})
}

#[test]
fn buy_usd_to_tea_should_fail_if_withdraw_amount_is_zero() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			GenesisExchange::usd_to_tea(Origin::signed(1), Some(0), None),
			Error::<Test>::AmountShouldNotBeZero
		);
	})
}

#[test]
fn sell_usd_to_tea_should_fail_if_withdraw_amount_is_zero() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			GenesisExchange::usd_to_tea(Origin::signed(1), None, Some(0)),
			Error::<Test>::AmountShouldNotBeZero
		);
	})
}

#[test]
fn buy_usd_to_tea_should_fail_if_user_do_not_have_enough_tea() {
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
fn sell_usd_to_tea_should_fail_if_user_do_not_have_enough_tea() {
	new_test_ext().execute_with(|| {
		let user = 1;
		assert_eq!(USDStore::<Test>::get(user), 0);

		assert_noop!(
			GenesisExchange::usd_to_tea(Origin::signed(1), None, Some(100)),
			Error::<Test>::UserInsufficientUSD
		);
	})
}

#[test]
fn borrow_usd_works() {
	new_test_ext().execute_with(|| {
		let user = 1;
		assert_eq!(USDDebt::<Test>::get(user), 0);
		assert_eq!(USDStore::<Test>::get(user), 0);

		let debt = 1000;
		assert_ok!(GenesisExchange::borrow_usd(Origin::signed(user), debt));
		assert_eq!(USDDebt::<Test>::get(user), debt);
		assert_eq!(USDStore::<Test>::get(user), debt);
	})
}

#[test]
fn borrow_usd_works_if_usd_store_is_not_zero() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let amount = 10000;
		USDStore::<Test>::insert(user, amount);
		assert_eq!(USDDebt::<Test>::get(user), 0);

		let debt = 1000;
		assert_ok!(GenesisExchange::borrow_usd(Origin::signed(user), debt));
		assert_eq!(USDDebt::<Test>::get(user), debt);
		assert_eq!(USDStore::<Test>::get(user), debt + amount);
	})
}

#[test]
fn borrow_usd_should_fail_if_borrow_amount_is_zero() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			GenesisExchange::borrow_usd(Origin::signed(1), 0),
			Error::<Test>::BorrowAmountShouldNotBeZero
		);
	})
}

#[test]
fn borrow_usd_should_fail_if_borrowed_debt_is_overflow() {
	new_test_ext().execute_with(|| {
		let user = 1;

		let debt = 1000;
		assert_ok!(GenesisExchange::borrow_usd(Origin::signed(user), debt));

		assert_noop!(
			GenesisExchange::borrow_usd(Origin::signed(user), u128::MAX),
			Error::<Test>::BorrowDebtHasOverflow
		);
		assert_eq!(USDDebt::<Test>::get(user), debt);
	})
}

#[test]
fn borrow_usd_should_fail_if_borrowed_amount_is_overflow() {
	new_test_ext().execute_with(|| {
		let user = 1;

		USDStore::<Test>::insert(user, 100);

		assert_noop!(
			GenesisExchange::borrow_usd(Origin::signed(user), u128::MAX),
			Error::<Test>::BorrowAmountHasOverflow
		);
		assert_eq!(USDDebt::<Test>::get(user), 0);
		assert_eq!(USDStore::<Test>::get(user), 100);
	})
}

#[test]
fn borrow_usd_works_if_borrowed_amount_less_than_borrow_allowance() {
	new_test_ext().execute_with(|| {
		let user = 1;

		assert_ok!(GenesisExchange::borrow_usd(
			Origin::signed(user),
			BORROW_ALLOWANCE
		));
		assert_eq!(USDDebt::<Test>::get(user), BORROW_ALLOWANCE);
		assert_eq!(USDStore::<Test>::get(user), BORROW_ALLOWANCE);
	})
}

#[test]
fn borrow_usd_should_if_initial_amount_larger_than_borrow_allowance() {
	new_test_ext().execute_with(|| {
		let user = 1;

		assert_noop!(
			GenesisExchange::borrow_usd(Origin::signed(user), BORROW_ALLOWANCE + 1),
			Error::<Test>::BorrowedDebtAmountHasOverThanMaxAllowed
		);
	})
}

#[test]
fn borrow_usd_should_if_borrowed_max_allowance_amount_usd_and_continue_borrow() {
	new_test_ext().execute_with(|| {
		let user = 1;

		assert_ok!(GenesisExchange::borrow_usd(
			Origin::signed(user),
			BORROW_ALLOWANCE
		));
		assert_eq!(USDDebt::<Test>::get(user), BORROW_ALLOWANCE);
		assert_eq!(USDStore::<Test>::get(user), BORROW_ALLOWANCE);

		assert_noop!(
			GenesisExchange::borrow_usd(Origin::signed(user), 1),
			Error::<Test>::BorrowedDebtAmountHasOverThanMaxAllowed
		);
	})
}

#[test]
fn if_asset_larger_than_max_borrow_allowance_user_borrowed_amount_should_lower_than_ratio_cap() {
	new_test_ext().execute_with(|| {
		let user = 1;
		assert_ok!(GenesisExchange::borrow_usd(
			Origin::signed(user),
			BORROW_ALLOWANCE
		));
		assert_eq!(USDDebt::<Test>::get(user), BORROW_ALLOWANCE);
		assert_eq!(USDStore::<Test>::get(user), BORROW_ALLOWANCE);
		USDStore::<Test>::mutate(user, |amount| {
			*amount = amount.saturating_add(BORROW_ALLOWANCE.into())
		});
		assert_eq!(USDDebt::<Test>::get(user), BORROW_ALLOWANCE);
		assert_eq!(USDStore::<Test>::get(user), BORROW_ALLOWANCE*2);
		assert_ok!(GenesisExchange::borrow_usd(Origin::signed(user), BORROW_ALLOWANCE));
		assert_eq!(USDDebt::<Test>::get(user), BORROW_ALLOWANCE*2);
		assert_eq!(USDStore::<Test>::get(user), BORROW_ALLOWANCE*3);
		assert_noop!(
			GenesisExchange::borrow_usd(Origin::signed(user), 1),
			Error::<Test>::BorrowedDebtAmountHasOverThanMaxAllowed
		);
	})
}

#[test]
fn repay_usd_debts_works() {
	new_test_ext().execute_with(|| {
		let user = 1;

		let debt = 1000;
		assert_ok!(GenesisExchange::borrow_usd(Origin::signed(user), debt));
		assert_eq!(USDDebt::<Test>::get(user), debt);

		let usd_amount = 10000;
		USDStore::<Test>::insert(user, usd_amount);
		assert_ok!(GenesisExchange::repay_usd_debts(
			Origin::signed(user),
			Some(100)
		));
		assert_eq!(USDDebt::<Test>::get(user), debt - 100);
		assert_eq!(USDStore::<Test>::get(user), usd_amount - 100);
	})
}

#[test]
fn repay_usd_debts_works_if_pay_out_all_debts() {
	new_test_ext().execute_with(|| {
		let user = 1;

		let debt = 1000;
		assert_ok!(GenesisExchange::borrow_usd(Origin::signed(user), debt));
		assert_eq!(USDDebt::<Test>::get(user), debt);

		let usd_amount = 10000;
		USDStore::<Test>::insert(user, usd_amount);
		assert_ok!(GenesisExchange::repay_usd_debts(Origin::signed(user), None));
		assert_eq!(USDDebt::<Test>::get(user), 0);
		assert!(!USDDebt::<Test>::contains_key(user));
		assert_eq!(USDStore::<Test>::get(user), usd_amount - debt);
	})
}

#[test]
fn repay_usd_debts_should_fail_if_user_debts_is_zero() {
	new_test_ext().execute_with(|| {
		let user = 1;

		let usd_amount = 10000;
		USDStore::<Test>::insert(user, usd_amount);

		USDDebt::<Test>::insert(user, 1000);
		assert_noop!(
			GenesisExchange::repay_usd_debts(Origin::signed(user), Some(0)),
			Error::<Test>::RepayUSDAmountShouldNotBeZero
		);
	})
}

#[test]
fn repay_usd_debts_should_fail_if_no_need_to_pay_usd_debts() {
	new_test_ext().execute_with(|| {
		let user = 1;

		assert_eq!(USDDebt::<Test>::get(user), 0);
		assert_noop!(
			GenesisExchange::repay_usd_debts(Origin::signed(user), None),
			Error::<Test>::NoNeedToRepayUSDDebts
		);
	})
}

#[test]
fn repay_usd_debts_should_fail_if_repay_amount_more_than_debts() {
	new_test_ext().execute_with(|| {
		let user = 1;

		let usd_amount = 10000;
		USDStore::<Test>::insert(user, usd_amount);

		USDDebt::<Test>::insert(user, 1000);
		assert_noop!(
			GenesisExchange::repay_usd_debts(Origin::signed(user), Some(2000)),
			Error::<Test>::RepayUSDAmountMoreThanDebtAmount
		);
	})
}

#[test]
fn repay_usd_debts_should_fail_if_usd_amount_less_than_repay_amount() {
	new_test_ext().execute_with(|| {
		let user = 1;

		let usd_amount = 100;
		USDStore::<Test>::insert(user, usd_amount);

		USDDebt::<Test>::insert(user, 1000);
		assert_noop!(
			GenesisExchange::repay_usd_debts(Origin::signed(user), Some(200)),
			Error::<Test>::InsufficientUSDToRepayDebts
		);
	})
}
