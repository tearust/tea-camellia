use crate::mock::*;
use crate::*;
use frame_support::{assert_noop, assert_ok};

#[test]
fn transfer_usd_works() {
	new_test_ext().execute_with(|| {
		let user1 = 1;
		let user2 = 2;

		let user1_balance = 10000;
		USDStore::<Test>::insert(user1, user1_balance);
		assert_eq!(USDStore::<Test>::get(user2), 0);

		let amount1 = 100;
		assert_ok!(GenesisExchange::transfer_usd(
			Origin::signed(user1),
			user2,
			amount1,
		));

		assert_eq!(USDStore::<Test>::get(user1), user1_balance - amount1);
		assert_eq!(USDStore::<Test>::get(user2), amount1);

		let amount2 = 200;
		assert_ok!(GenesisExchange::transfer_usd(
			Origin::signed(user1),
			user2,
			amount2,
		));

		assert_eq!(
			USDStore::<Test>::get(user1),
			user1_balance - amount1 - amount2
		);
		assert_eq!(USDStore::<Test>::get(user2), amount1 + amount2);
	})
}

#[test]
fn transfer_usd_should_fail_if_user_usd_amount_is_not_enough() {
	new_test_ext().execute_with(|| {
		let user1 = 1;
		let user2 = 2;

		let user1_balance = 10000;
		USDStore::<Test>::insert(user1, user1_balance);

		assert_noop!(
			GenesisExchange::transfer_usd(Origin::signed(user1), user2, user1_balance + 1,),
			Error::<Test>::InvalidTransferUSDAmount
		);
	})
}

#[test]
fn transfer_usd_should_fail_if_receiver_usd_amount_is_overflow() {
	new_test_ext().execute_with(|| {
		let user1 = 1;
		let user2 = 2;

		let user1_balance = 10000;
		USDStore::<Test>::insert(user1, user1_balance);
		USDStore::<Test>::insert(user2, u128::MAX - 1);

		assert_noop!(
			GenesisExchange::transfer_usd(Origin::signed(user1), user2, user1_balance,),
			Error::<Test>::InvalidTransferUSDAmount
		);
	})
}

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
fn register_for_competition_works() {
	new_test_ext().execute_with(|| {
		let user2 = 2;
		<Test as Config>::Currency::make_free_balance_be(&user2, 1000);

		let erc20 = b"test erc20".to_vec();
		let email = b"test email".to_vec();
		assert_ok!(GenesisExchange::register_for_competition(
			Origin::signed(user2),
			user2,
			erc20.clone(),
			email.clone(),
		));

		assert!(CompetitionUsers::<Test>::contains_key(user2));
		assert_eq!(CompetitionUsers::<Test>::get(user2), (erc20, email))
	})
}

#[test]
fn register_for_competition_should_fail_if_already_registered() {
	new_test_ext().execute_with(|| {
		let user = 2;
		<Test as Config>::Currency::make_free_balance_be(&user, 1000);

		assert_ok!(GenesisExchange::register_for_competition(
			Origin::signed(user),
			user,
			b"test erc20".to_vec(),
			b"test email".to_vec(),
		));

		assert_noop!(
			GenesisExchange::register_for_competition(
				Origin::signed(user),
				user,
				b"test erc20".to_vec(),
				b"test email".to_vec(),
			),
			Error::<Test>::CompetitionUserAlreadyRegistered
		);
	})
}

#[test]
fn register_for_competition_should_fail_if_user_free_balance_is_not_enough() {
	new_test_ext().execute_with(|| {
		let user = 2;
		assert_noop!(
			GenesisExchange::register_for_competition(
				Origin::signed(user),
				user,
				b"test erc20".to_vec(),
				b"test email".to_vec(),
			),
			Error::<Test>::CompetitionUserInsufficientFreeBalance
		);

		<Test as Config>::Currency::make_free_balance_be(
			&user,
			REGISTER_FOR_COMPETITION_ALLOWANCE - 1,
		);
		assert_noop!(
			GenesisExchange::register_for_competition(
				Origin::signed(user),
				user,
				b"test erc20".to_vec(),
				b"test email".to_vec(),
			),
			Error::<Test>::CompetitionUserInsufficientFreeBalance
		);
	})
}

#[test]
fn remove_competition_user_works() {
	new_test_ext().execute_with(|| {
		let user2 = 2;
		<Test as Config>::Currency::make_free_balance_be(&user2, 1000);

		let erc20 = b"test erc20".to_vec();
		let email = b"test email".to_vec();
		assert_ok!(GenesisExchange::register_for_competition(
			Origin::signed(user2),
			user2,
			erc20.clone(),
			email.clone(),
		));

		assert!(CompetitionUsers::<Test>::contains_key(user2));
		assert_eq!(CompetitionUsers::<Test>::get(user2), (erc20, email));

		assert_ok!(GenesisExchange::remove_competition_user(
			Origin::root(),
			user2
		));
		assert!(!CompetitionUsers::<Test>::contains_key(user2));
	})
}

#[test]
fn remove_competition_user_should_fail_if_not_root_user() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			GenesisExchange::remove_competition_user(Origin::signed(2), 1),
			DispatchError::BadOrigin
		);
	})
}

#[test]
fn remove_competition_user_should_fail_if_user_not_exist() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			GenesisExchange::remove_competition_user(Origin::root(), 1),
			Error::<Test>::CompetitionUserNotExist
		);
	})
}
