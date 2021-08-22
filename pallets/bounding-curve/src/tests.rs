use crate::functions::approximately_equals;
use crate::mock::*;
use crate::*;
use frame_support::{assert_noop, assert_ok};

const CENTS: node_primitives::Balance = 10_000_000_000;
const DOLLARS: node_primitives::Balance = 100 * CENTS;

#[test]
fn create_new_tapp_works() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let tapp_name = "test name";
		let ticker = "tea";
		let detail = "test detail";
		let link = "https://teaproject.org";
		let init_fund = 1000;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_ok!(BoundingCurve::create_new_tapp(
			Origin::signed(user),
			tapp_name.as_bytes().to_vec(),
			ticker.as_bytes().to_vec(),
			init_fund,
			detail.as_bytes().to_vec(),
			link.as_bytes().to_vec(),
		));

		// this is the first tapp so tapp id is 1
		let tapp_id = 1;
		assert_eq!(LastTAppId::<Test>::get(), tapp_id);
		assert_eq!(AccountTable::<Test>::get(user, tapp_id), init_fund);
		assert_eq!(TotalSupplyTable::<Test>::get(tapp_id), init_fund);
		assert_eq!(TAppNames::<Test>::get(tapp_name.as_bytes()), tapp_id);
		assert_eq!(TAppTickers::<Test>::get(ticker.as_bytes()), tapp_id);
		let tapp_item = TAppBoundingCurve::<Test>::get(tapp_id);
		assert_eq!(tapp_item.id, tapp_id);
		assert_eq!(tapp_item.buy_curve, CurveType::UnsignedSquareRoot_1000);
		assert_eq!(tapp_item.sell_curve, CurveType::UnsignedSquareRoot_700);
		assert_eq!(tapp_item.owner, user);
		assert_eq!(&String::from_utf8(tapp_item.name).unwrap(), tapp_name);
		assert_eq!(&String::from_utf8(tapp_item.ticker).unwrap(), ticker);
		assert_eq!(&String::from_utf8(tapp_item.detail).unwrap(), detail);
		assert_eq!(&String::from_utf8(tapp_item.link).unwrap(), link);
		assert_eq!(<Test as Config>::Currency::free_balance(&user), 99855334);
	})
}

#[test]
fn create_new_tapp_should_fail_if_name_already_exist() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let tapp_name = "test name";
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_ok!(BoundingCurve::create_new_tapp(
			Origin::signed(user),
			tapp_name.as_bytes().to_vec(),
			b"tea".to_vec(),
			1000,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
		));

		assert_noop!(
			BoundingCurve::create_new_tapp(
				Origin::signed(user),
				tapp_name.as_bytes().to_vec(),
				b"tea".to_vec(),
				1000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
			),
			Error::<Test>::TAppNameAlreadyExist
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_ticker_already_exist() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let ticker = b"tea";
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_ok!(BoundingCurve::create_new_tapp(
			Origin::signed(user),
			b"test name".to_vec(),
			ticker.to_vec(),
			1000,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
		));

		assert_noop!(
			BoundingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name2".to_vec(),
				ticker.to_vec(),
				1000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
			),
			Error::<Test>::TAppTickerAlreadyExist
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_name_is_too_long() {
	new_test_ext().execute_with(|| {
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_noop!(
			BoundingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name".to_vec(),
				[1; TAPP_TICKER_MAX_LENGTH as usize + 1].to_vec(),
				1000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
			),
			Error::<Test>::TAppTickerIsTooLong,
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_name_is_too_short() {
	new_test_ext().execute_with(|| {
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_noop!(
			BoundingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name".to_vec(),
				[1; TAPP_TICKER_MIN_LENGTH as usize - 1].to_vec(),
				1000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
			),
			Error::<Test>::TAppTickerIsTooShort,
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_detail_is_too_long() {
	new_test_ext().execute_with(|| {
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_noop!(
			BoundingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name".to_vec(),
				b"tea".to_vec(),
				1000,
				[1; TAPP_DETAIL_MAX_LENGTH as usize + 1].to_vec(),
				b"https://teaproject.org".to_vec(),
			),
			Error::<Test>::TAppDetailIsTooLong,
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_link_is_too_long() {
	new_test_ext().execute_with(|| {
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_noop!(
			BoundingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name".to_vec(),
				b"tea".to_vec(),
				1000,
				b"test detail".to_vec(),
				[1; TAPP_LINK_MAX_LENGTH as usize + 1].to_vec(),
			),
			Error::<Test>::TAppLinkIsTooLong,
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_free_balance_is_not_enough() {
	new_test_ext().execute_with(|| {
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 0);

		assert_noop!(
			BoundingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name".to_vec(),
				b"tea".to_vec(),
				1000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
			),
			Error::<Test>::InsufficientFreeBalance,
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_ticker_is_too_long() {
	new_test_ext().execute_with(|| {
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_noop!(
			BoundingCurve::create_new_tapp(
				Origin::signed(user),
				[1; TAPP_NAME_MAX_LENGTH as usize + 1].to_vec(),
				b"tea".to_vec(),
				1000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
			),
			Error::<Test>::TAppNameIsTooLong
		);
	})
}

#[test]
fn buy_token_works() {
	new_test_ext().execute_with(|| {
		let owner = 1;
		let user = 2;
		let tapp_amount = 1000;
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user, DOLLARS);

		assert_ok!(BoundingCurve::create_new_tapp(
			Origin::signed(owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
		));

		let tapp_id = 1;
		assert_ok!(BoundingCurve::buy_token(
			Origin::signed(user),
			tapp_id,
			tapp_amount
		));

		assert_eq!(AccountTable::<Test>::get(&owner, tapp_id), tapp_amount);
		assert_eq!(AccountTable::<Test>::get(&user, tapp_id), tapp_amount);
		assert_eq!(TotalSupplyTable::<Test>::get(tapp_id), tapp_amount * 2);
	})
}

#[test]
fn buy_token_should_fail_if_tapp_is_not_exist() {
	new_test_ext().execute_with(|| {
		let user = 2;
		<Test as Config>::Currency::make_free_balance_be(&user, DOLLARS);

		let tapp_id = 1;
		assert_noop!(
			BoundingCurve::buy_token(Origin::signed(user), tapp_id, 1000,),
			Error::<Test>::TAppIdNotExist
		);
	})
}

#[test]
fn buy_token_should_fail_if_tapp_amount_is_zero() {
	new_test_ext().execute_with(|| {
		let owner = 1;
		let user = 2;
		let tapp_amount = 1000;
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user, DOLLARS);

		assert_ok!(BoundingCurve::create_new_tapp(
			Origin::signed(owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
		));

		let tapp_id = 1;
		assert_noop!(
			BoundingCurve::buy_token(Origin::signed(user), tapp_id, 0),
			Error::<Test>::OperationAmountCanNotBeZero
		);
	})
}

#[test]
fn buy_token_should_fail_if_free_balance_is_not_enough() {
	new_test_ext().execute_with(|| {
		let owner = 1;
		let user = 2;
		let tapp_amount = 1000;
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user, 0);

		assert_ok!(BoundingCurve::create_new_tapp(
			Origin::signed(owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
		));

		let tapp_id = 1;
		assert_noop!(
			BoundingCurve::buy_token(Origin::signed(user), tapp_id, 1000),
			Error::<Test>::InsufficientFreeBalance
		);
	})
}

#[test]
fn sell_token_works() {
	new_test_ext().execute_with(|| {
		let owner = 1;
		let user = 2;
		let tapp_amount = 1000;
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user, DOLLARS);

		assert_ok!(BoundingCurve::create_new_tapp(
			Origin::signed(owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
		));

		let tapp_id = 1;
		assert_ok!(BoundingCurve::buy_token(
			Origin::signed(user),
			tapp_id,
			tapp_amount
		));
		assert_ok!(BoundingCurve::sell_token(
			Origin::signed(user),
			tapp_id,
			tapp_amount
		));

		assert_eq!(AccountTable::<Test>::get(&user, tapp_id), 0);
		assert!(<Test as Config>::Currency::free_balance(&user) < DOLLARS);
		assert!(approximately_equals::<Test>(
			<Test as Config>::Currency::free_balance(&user)
				+ <Test as Config>::Currency::free_balance(&owner),
			DOLLARS * 2,
			10000000
		));
	})
}

#[test]
fn sell_token_works_when_total_balance_reduce_to_zero() {
	new_test_ext().execute_with(|| {
		let owner = 1;
		let tapp_amount = 1000;
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);

		let name = b"test name".to_vec();
		let ticker = b"tea".to_vec();
		assert_ok!(BoundingCurve::create_new_tapp(
			Origin::signed(owner),
			name.clone(),
			ticker.clone(),
			tapp_amount,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
		));

		let tapp_id = 1;
		assert_ok!(BoundingCurve::sell_token(
			Origin::signed(owner),
			tapp_id,
			tapp_amount
		));

		assert!(!AccountTable::<Test>::contains_key(&owner, tapp_id));
		assert!(!TotalSupplyTable::<Test>::contains_key(tapp_id));
		assert!(!TAppBoundingCurve::<Test>::contains_key(tapp_id));
		assert!(!TAppNames::<Test>::contains_key(name));
		assert!(!TAppTickers::<Test>::contains_key(ticker));
		assert_eq!(<Test as Config>::Currency::free_balance(&owner), DOLLARS);
	})
}

#[test]
fn sell_token_should_fail_if_tapp_not_exist() {
	new_test_ext().execute_with(|| {
		let user = 2;
		let tapp_amount = 1000;
		<Test as Config>::Currency::make_free_balance_be(&user, DOLLARS);

		assert_noop!(
			BoundingCurve::sell_token(Origin::signed(user), 1, tapp_amount),
			Error::<Test>::TAppIdNotExist
		);
	})
}

#[test]
fn sell_token_should_fail_if_tapp_amount_is_not_enough() {
	new_test_ext().execute_with(|| {
		let owner = 1;
		let user = 2;
		let tapp_amount = 1000;
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user, DOLLARS);

		assert_ok!(BoundingCurve::create_new_tapp(
			Origin::signed(owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
		));

		let tapp_id = 1;
		assert_noop!(
			BoundingCurve::sell_token(Origin::signed(user), tapp_id, tapp_amount + 1),
			Error::<Test>::InsufficientTAppToken
		);
	})
}

#[test]
fn sell_token_should_fail_if_tapp_amount_is_zero() {
	new_test_ext().execute_with(|| {
		let owner = 1;
		let user = 2;
		let tapp_amount = 1000;
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user, DOLLARS);

		assert_ok!(BoundingCurve::create_new_tapp(
			Origin::signed(owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
		));

		let tapp_id = 1;
		assert_noop!(
			BoundingCurve::sell_token(Origin::signed(user), tapp_id, 0),
			Error::<Test>::OperationAmountCanNotBeZero
		);
	})
}

#[test]
fn sell_token_should_fail_if_tapp_total_supply_is_not_enough() {
	new_test_ext().execute_with(|| {
		let owner = 1;
		let tapp_amount = 1000;
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);

		assert_ok!(BoundingCurve::create_new_tapp(
			Origin::signed(owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
		));

		let tapp_id = 1;
		// should never happen, set here just to cover the test case.
		TotalSupplyTable::<Test>::mutate(tapp_id, |amount| *amount = tapp_amount - 1);
		assert_noop!(
			BoundingCurve::sell_token(Origin::signed(owner), tapp_id, tapp_amount),
			Error::<Test>::InsufficientTotalSupply
		);
	})
}

#[test]
fn consume_works() {
	new_test_ext().execute_with(|| {
		let user1 = 1;
		let user2 = 2;
		let user3 = 3;
		let user4 = 4;
		let tapp_amount1 = 1000;
		let tapp_amount2 = 2000;
		let tapp_amount3 = 4000;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user2, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user3, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user4, DOLLARS);

		assert_ok!(BoundingCurve::create_new_tapp(
			Origin::signed(user1),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount1,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
		));

		let tapp_id = 1;
		assert_ok!(BoundingCurve::buy_token(
			Origin::signed(user2),
			tapp_id,
			tapp_amount2
		));
		assert_ok!(BoundingCurve::buy_token(
			Origin::signed(user3),
			tapp_id,
			tapp_amount3
		));
		assert_eq!(
			TotalSupplyTable::<Test>::get(tapp_id),
			tapp_amount1 + tapp_amount2 + tapp_amount3
		);

		assert_ok!(BoundingCurve::consume(
			Origin::signed(user4),
			tapp_id,
			10000
		));
		// todo should pass
		// assert_eq!(
		// 	<Test as Config>::Currency::free_balance(&user4),
		// 	DOLLARS - 10000
		// );
		assert_eq!(AccountTable::<Test>::get(user1, tapp_id), 1552428);
		assert_eq!(AccountTable::<Test>::get(user2, tapp_id), 3104857);
		assert_eq!(AccountTable::<Test>::get(user3, tapp_id), 6209714);
		assert_eq!(AccountTable::<Test>::get(user4, tapp_id), 0);
	})
}

#[test]
fn consume_should_fail_if_tapp_not_exist() {
	new_test_ext().execute_with(|| {
		let user1 = 1;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);

		assert_noop!(
			BoundingCurve::consume(Origin::signed(user1), 1, 10000),
			Error::<Test>::TAppIdNotExist
		);
	})
}

#[test]
fn consume_should_fail_if_consume_amount_is_zero() {
	new_test_ext().execute_with(|| {
		let user1 = 1;
		let user2 = 2;
		let tapp_amount1 = 1000;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user2, DOLLARS);

		assert_ok!(BoundingCurve::create_new_tapp(
			Origin::signed(user1),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount1,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
		));

		let tapp_id = 1;
		assert_noop!(
			BoundingCurve::consume(Origin::signed(user2), tapp_id, 0),
			Error::<Test>::OperationAmountCanNotBeZero
		);
	})
}

#[test]
fn consume_should_fail_if_free_balance_is_not_enough() {
	new_test_ext().execute_with(|| {
		let user1 = 1;
		let user2 = 2;
		let tapp_amount1 = 1000;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user2, 0);

		assert_ok!(BoundingCurve::create_new_tapp(
			Origin::signed(user1),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount1,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
		));

		let tapp_id = 1;
		assert_noop!(
			BoundingCurve::consume(Origin::signed(user2), tapp_id, 1000),
			Error::<Test>::InsufficientFreeBalance
		);
	})
}

#[test]
fn expense_works() {
	new_test_ext().execute_with(|| {
		let user1 = 1;
		let user2 = 2;
		let user3 = 3;
		let user4 = 4;
		let tapp_amount1 = 1000;
		let tapp_amount2 = 2000;
		let tapp_amount3 = 4000;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user2, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user3, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user4, DOLLARS);

		assert_ok!(BoundingCurve::create_new_tapp(
			Origin::signed(user1),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount1,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
		));

		let tapp_id = 1;
		assert_ok!(BoundingCurve::buy_token(
			Origin::signed(user2),
			tapp_id,
			tapp_amount2
		));
		assert_ok!(BoundingCurve::buy_token(
			Origin::signed(user3),
			tapp_id,
			tapp_amount3
		));
		assert_eq!(
			TotalSupplyTable::<Test>::get(tapp_id),
			tapp_amount1 + tapp_amount2 + tapp_amount3
		);

		let expense_amount = 4666;
		assert_ok!(BoundingCurve::expense(
			Origin::signed(user4),
			tapp_id,
			expense_amount
		));

		assert_eq!(AccountTable::<Test>::get(user1, tapp_id), tapp_amount1 - 2);
		assert_eq!(AccountTable::<Test>::get(user2, tapp_id), tapp_amount2 - 4);
		assert_eq!(AccountTable::<Test>::get(user3, tapp_id), tapp_amount3 - 9);
		assert_eq!(AccountTable::<Test>::get(user4, tapp_id), 0);
		assert_eq!(
			TotalSupplyTable::<Test>::get(tapp_id),
			tapp_amount1 + tapp_amount2 + tapp_amount3 - 16
		);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&user4),
			DOLLARS + expense_amount
		);
	})
}

#[test]
fn expense_should_fail_if_tapp_not_exist() {
	new_test_ext().execute_with(|| {
		let user1 = 1;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);

		let expense_amount = 4666;
		assert_noop!(
			BoundingCurve::expense(Origin::signed(user1), 1, expense_amount),
			Error::<Test>::TAppIdNotExist
		);
	})
}

#[test]
fn expense_should_fail_if_expense_amount_is_zero() {
	new_test_ext().execute_with(|| {
		let user1 = 1;
		let user2 = 2;
		let tapp_amount1 = 1000;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);

		assert_ok!(BoundingCurve::create_new_tapp(
			Origin::signed(user1),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount1,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
		));

		let tapp_id = 1;
		assert_noop!(
			BoundingCurve::expense(Origin::signed(user2), tapp_id, 0),
			Error::<Test>::OperationAmountCanNotBeZero
		);
	})
}

#[test]
fn expense_should_fail_if_expense_amount_more_than_reserved_balance() {
	new_test_ext().execute_with(|| {
		let user1 = 1;
		let user2 = 2;
		let tapp_amount1 = 1000;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);

		assert_ok!(BoundingCurve::create_new_tapp(
			Origin::signed(user1),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount1,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
		));

		let tapp_id = 1;
		assert_noop!(
			BoundingCurve::expense(Origin::signed(user2), tapp_id, 1000000),
			Error::<Test>::TAppInsufficientFreeBalance
		);
	})
}
