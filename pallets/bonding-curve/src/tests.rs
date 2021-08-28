use crate::functions::approximately_equals;
use crate::mock::*;
use crate::*;
use frame_support::{assert_noop, assert_ok};
use pallet_cml::{CmlStore, CmlType, DefrostScheduleType, Seed, UserCmlStore, CML};

const CENTS: node_primitives::Balance = 10_000_000_000;
const DOLLARS: node_primitives::Balance = 100 * CENTS;

#[test]
fn set_tapp_creation_settings_works() {
	new_test_ext().execute_with(|| {
		let npc = 3;
		assert_eq!(NPCAccount::<Test>::get(), 0);
		assert!(!EnableUserCreateTApp::<Test>::get());

		assert_ok!(BondingCurve::tapp_creation_settings(
			Origin::root(),
			Some(true),
			Some(npc)
		));
		assert_eq!(NPCAccount::<Test>::get(), npc);
		assert!(EnableUserCreateTApp::<Test>::get());
	})
}

#[test]
fn set_tapp_creation_settings_should_fail_if_not_root_user() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			BondingCurve::tapp_creation_settings(Origin::signed(1), Some(true), None),
			DispatchError::BadOrigin
		);
	})
}

#[test]
fn create_new_tapp_works() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		let tapp_name = "test name";
		let ticker = "tea";
		let detail = "test detail";
		let link = "https://teaproject.org";
		let init_fund = 1000000;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(user),
			tapp_name.as_bytes().to_vec(),
			ticker.as_bytes().to_vec(),
			init_fund,
			detail.as_bytes().to_vec(),
			link.as_bytes().to_vec(),
			None,
			None,
		));

		// this is the first tapp so tapp id is 1
		let tapp_id = 1;
		assert_eq!(LastTAppId::<Test>::get(), tapp_id);
		assert_eq!(AccountTable::<Test>::get(user, tapp_id), init_fund);
		assert_eq!(TotalSupplyTable::<Test>::get(tapp_id), init_fund);
		assert_eq!(TAppNames::<Test>::get(tapp_name.as_bytes()), tapp_id);
		assert_eq!(TAppTickers::<Test>::get(ticker.as_bytes()), tapp_id);
		let tapp_item = TAppBondingCurve::<Test>::get(tapp_id);
		assert_eq!(tapp_item.id, tapp_id);
		assert_eq!(tapp_item.buy_curve, CurveType::UnsignedSquareRoot_10);
		assert_eq!(tapp_item.sell_curve, CurveType::UnsignedSquareRoot_7);
		assert_eq!(tapp_item.owner, user);
		assert_eq!(&String::from_utf8(tapp_item.name).unwrap(), tapp_name);
		assert_eq!(&String::from_utf8(tapp_item.ticker).unwrap(), ticker);
		assert_eq!(&String::from_utf8(tapp_item.detail).unwrap(), detail);
		assert_eq!(&String::from_utf8(tapp_item.link).unwrap(), link);
		assert_eq!(<Test as Config>::Currency::free_balance(&user), 99999534);
	})
}

#[test]
fn create_new_tapp_should_fail_if_name_already_exist() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		let tapp_name = "test name";
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(user),
			tapp_name.as_bytes().to_vec(),
			b"tea".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			None,
			None,
		));

		assert_noop!(
			BondingCurve::create_new_tapp(
				Origin::signed(user),
				tapp_name.as_bytes().to_vec(),
				b"tea".to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
				None,
				None,
			),
			Error::<Test>::TAppNameAlreadyExist
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_ticker_already_exist() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		let ticker = b"tea";
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(user),
			b"test name".to_vec(),
			ticker.to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			None,
			None,
		));

		assert_noop!(
			BondingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name2".to_vec(),
				ticker.to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
				None,
				None,
			),
			Error::<Test>::TAppTickerAlreadyExist
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_not_allowed_user_create_tapp() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(false);
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_noop!(
			BondingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name".to_vec(),
				b"tea".to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
				None,
				None,
			),
			Error::<Test>::NotAllowedNormalUserCreateTApp,
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_name_is_too_long() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_noop!(
			BondingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name".to_vec(),
				[1; TAPP_TICKER_MAX_LENGTH as usize + 1].to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
				None,
				None,
			),
			Error::<Test>::TAppTickerIsTooLong,
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_name_is_too_short() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_noop!(
			BondingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name".to_vec(),
				[1; TAPP_TICKER_MIN_LENGTH as usize - 1].to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
				None,
				None,
			),
			Error::<Test>::TAppTickerIsTooShort,
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_detail_is_too_long() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_noop!(
			BondingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name".to_vec(),
				b"tea".to_vec(),
				1_000_000,
				[1; TAPP_DETAIL_MAX_LENGTH as usize + 1].to_vec(),
				b"https://teaproject.org".to_vec(),
				None,
				None,
			),
			Error::<Test>::TAppDetailIsTooLong,
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_link_is_too_long() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_noop!(
			BondingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name".to_vec(),
				b"tea".to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				[1; TAPP_LINK_MAX_LENGTH as usize + 1].to_vec(),
				None,
				None,
			),
			Error::<Test>::TAppLinkIsTooLong,
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_free_balance_is_not_enough() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 0);

		assert_noop!(
			BondingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name".to_vec(),
				b"tea".to_vec(),
				1000000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
				None,
				None,
			),
			Error::<Test>::InsufficientFreeBalance,
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_tapp_amount_is_too_low() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 0);

		assert_noop!(
			BondingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name".to_vec(),
				b"tea".to_vec(),
				1000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
				None,
				None,
			),
			Error::<Test>::BuyTeaAmountCanNotBeZero,
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_ticker_is_too_long() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_noop!(
			BondingCurve::create_new_tapp(
				Origin::signed(user),
				[1; TAPP_NAME_MAX_LENGTH as usize + 1].to_vec(),
				b"tea".to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
				None,
				None,
			),
			Error::<Test>::TAppNameIsTooLong
		);
	})
}

#[test]
fn buy_token_works() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let owner = 1;
		let user = 2;
		let tapp_amount = 1_000_000;
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user, DOLLARS);

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			None,
			None,
		));

		let tapp_id = 1;
		assert_ok!(BondingCurve::buy_token(
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
			BondingCurve::buy_token(Origin::signed(user), tapp_id, 1_000_000,),
			Error::<Test>::TAppIdNotExist
		);
	})
}

#[test]
fn buy_token_should_fail_if_tapp_amount_is_zero() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let owner = 1;
		let user = 2;
		let tapp_amount = 1_000_000;
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user, DOLLARS);

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			None,
			None,
		));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::buy_token(Origin::signed(user), tapp_id, 0),
			Error::<Test>::OperationAmountCanNotBeZero
		);
	})
}

#[test]
fn buy_token_should_fail_if_tapp_amount_is_too_low() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let owner = 1;
		let user = 2;
		let tapp_amount = 1_000_000;
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user, DOLLARS);

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			None,
			None,
		));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::buy_token(Origin::signed(user), tapp_id, 100),
			Error::<Test>::BuyTeaAmountCanNotBeZero
		);
	})
}

#[test]
fn buy_token_should_fail_if_free_balance_is_not_enough() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let owner = 1;
		let user = 2;
		let tapp_amount = 1_000_000;
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user, 0);

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			None,
			None,
		));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::buy_token(Origin::signed(user), tapp_id, 1000000),
			Error::<Test>::InsufficientFreeBalance
		);
	})
}

#[test]
fn sell_token_works() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let owner = 1;
		let user = 2;
		let tapp_amount = 1000000;
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user, DOLLARS);

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			None,
			None,
		));

		let tapp_id = 1;
		assert_ok!(BondingCurve::buy_token(
			Origin::signed(user),
			tapp_id,
			tapp_amount
		));
		assert_ok!(BondingCurve::sell_token(
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
		EnableUserCreateTApp::<Test>::set(true);
		let owner = 1;
		let tapp_amount = 1_000_000;
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);

		let name = b"test name".to_vec();
		let ticker = b"tea".to_vec();
		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(owner),
			name.clone(),
			ticker.clone(),
			tapp_amount,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			None,
			None,
		));

		let tapp_id = 1;
		assert_ok!(BondingCurve::sell_token(
			Origin::signed(owner),
			tapp_id,
			tapp_amount
		));

		assert!(!AccountTable::<Test>::contains_key(&owner, tapp_id));
		assert!(!TotalSupplyTable::<Test>::contains_key(tapp_id));
		assert!(!TAppBondingCurve::<Test>::contains_key(tapp_id));
		assert!(!TAppNames::<Test>::contains_key(name));
		assert!(!TAppTickers::<Test>::contains_key(ticker));
		assert_eq!(<Test as Config>::Currency::free_balance(&owner), DOLLARS);
	})
}

#[test]
fn sell_token_should_fail_if_tapp_not_exist() {
	new_test_ext().execute_with(|| {
		let user = 2;
		let tapp_amount = 1_000_000;
		<Test as Config>::Currency::make_free_balance_be(&user, DOLLARS);

		assert_noop!(
			BondingCurve::sell_token(Origin::signed(user), 1, tapp_amount),
			Error::<Test>::TAppIdNotExist
		);
	})
}

#[test]
fn sell_token_should_fail_if_tapp_amount_is_not_enough() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let owner = 1;
		let user = 2;
		let tapp_amount = 1_000_000;
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user, DOLLARS);

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			None,
			None,
		));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::sell_token(Origin::signed(user), tapp_id, tapp_amount + 1),
			Error::<Test>::InsufficientTAppToken
		);
	})
}

#[test]
fn sell_token_should_fail_if_tapp_amount_is_zero() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let owner = 1;
		let user = 2;
		let tapp_amount = 1_000_000;
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user, DOLLARS);

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			None,
			None,
		));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::sell_token(Origin::signed(user), tapp_id, 0),
			Error::<Test>::OperationAmountCanNotBeZero
		);
	})
}

#[test]
fn sell_token_should_fail_if_tapp_amount_is_too_low() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let owner = 1;
		let user = 2;
		let tapp_amount = 1_000_000;
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user, DOLLARS);

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			None,
			None,
		));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::sell_token(Origin::signed(owner), tapp_id, 100),
			Error::<Test>::SellTeaAmountCanNotBeZero
		);
	})
}

#[test]
fn sell_token_should_fail_if_tapp_total_supply_is_not_enough() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let owner = 1;
		let tapp_amount = 1_000_000;
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			None,
			None,
		));

		let tapp_id = 1;
		// should never happen, set here just to cover the test case.
		TotalSupplyTable::<Test>::mutate(tapp_id, |amount| *amount = tapp_amount - 1);
		assert_noop!(
			BondingCurve::sell_token(Origin::signed(owner), tapp_id, tapp_amount),
			Error::<Test>::InsufficientTotalSupply
		);
	})
}

#[test]
fn consume_works() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user1 = 1;
		let user2 = 2;
		let user3 = 3;
		let user4 = 4;
		let tapp_amount1 = 1_000_000;
		let tapp_amount2 = 2_000_000;
		let tapp_amount3 = 4_000_000;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user2, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user3, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user4, DOLLARS);

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(user1),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount1,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			None,
			None,
		));

		let tapp_id = 1;
		assert_ok!(BondingCurve::buy_token(
			Origin::signed(user2),
			tapp_id,
			tapp_amount2
		));
		assert_ok!(BondingCurve::buy_token(
			Origin::signed(user3),
			tapp_id,
			tapp_amount3
		));
		assert_eq!(
			TotalSupplyTable::<Test>::get(tapp_id),
			tapp_amount1 + tapp_amount2 + tapp_amount3
		);

		assert_ok!(BondingCurve::consume(Origin::signed(user4), tapp_id, 10000));
		assert!(approximately_equals::<Test>(
			<Test as Config>::Currency::free_balance(&user4),
			DOLLARS - 10000,
			1,
		));
		assert_eq!(AccountTable::<Test>::get(user1, tapp_id), 1484987);
		assert_eq!(AccountTable::<Test>::get(user2, tapp_id), 2969974);
		assert_eq!(AccountTable::<Test>::get(user3, tapp_id), 5939948);
		assert_eq!(AccountTable::<Test>::get(user4, tapp_id), 0);
		assert_eq!(TotalSupplyTable::<Test>::get(tapp_id), 10394910)
	})
}

#[test]
fn consume_should_fail_if_tapp_not_exist() {
	new_test_ext().execute_with(|| {
		let user1 = 1;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);

		assert_noop!(
			BondingCurve::consume(Origin::signed(user1), 1, 10000),
			Error::<Test>::TAppIdNotExist
		);
	})
}

#[test]
fn consume_should_fail_if_consume_amount_is_zero() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user1 = 1;
		let user2 = 2;
		let tapp_amount1 = 1_000_000;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user2, DOLLARS);

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(user1),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount1,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			None,
			None,
		));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::consume(Origin::signed(user2), tapp_id, 0),
			Error::<Test>::OperationAmountCanNotBeZero
		);
	})
}

#[test]
fn consume_should_fail_if_free_balance_is_not_enough() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user1 = 1;
		let user2 = 2;
		let tapp_amount1 = 1_000_000;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user2, 0);

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(user1),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount1,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			None,
			None,
		));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::consume(Origin::signed(user2), tapp_id, 1_000_000),
			Error::<Test>::InsufficientFreeBalance
		);
	})
}

#[test]
fn expense_works() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user1 = 1;
		let user2 = 2;
		let user3 = 3;
		let miner = 5;
		let tapp_amount1 = 1000000;
		let tapp_amount2 = 2000000;
		let tapp_amount3 = 4000000;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user2, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user3, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&miner, DOLLARS);

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(user1),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount1,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			Some(1000),
			Some(10),
		));

		let tapp_id = 1;
		assert_ok!(BondingCurve::buy_token(
			Origin::signed(user2),
			tapp_id,
			tapp_amount2
		));
		assert_ok!(BondingCurve::buy_token(
			Origin::signed(user3),
			tapp_id,
			tapp_amount3
		));
		assert_eq!(
			TotalSupplyTable::<Test>::get(tapp_id),
			tapp_amount1 + tapp_amount2 + tapp_amount3
		);

		let cml_id = 11;
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100, 10000));
		cml.set_owner(&miner);
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));
		assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id));

		let expense_amount = 46;
		TAppBondingCurve::<Test>::mutate(tapp_id, |tapp| tapp.current_cost = expense_amount);
		assert_ok!(BondingCurve::expense(Origin::signed(user1), tapp_id));

		assert_eq!(AccountTable::<Test>::get(user1, tapp_id), 996013);
		assert_eq!(AccountTable::<Test>::get(user2, tapp_id), 1992025);
		assert_eq!(AccountTable::<Test>::get(user3, tapp_id), 3984050);
		assert_eq!(AccountTable::<Test>::get(miner, tapp_id), 0);
		assert_eq!(TotalSupplyTable::<Test>::get(tapp_id), 6972086);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&miner),
			DOLLARS + expense_amount - STAKING_PRICE
		);
	})
}

#[test]
fn expense_should_fail_if_tapp_not_exist() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user1 = 1;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);

		assert_noop!(
			BondingCurve::expense(Origin::signed(user1), 1),
			Error::<Test>::TAppIdNotExist
		);
	})
}

#[test]
fn expense_should_fail_if_sender_is_not_tapp_owner() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user1 = 1;
		let user2 = 2;
		let tapp_amount1 = 1_000_000;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(user1),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount1,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			None,
			None,
		));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::expense(Origin::signed(user2), tapp_id),
			Error::<Test>::OnlyTAppOwnerAllowedToExpense
		);
	})
}

#[test]
fn expense_should_fail_if_expense_amount_is_zero() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user1 = 1;
		let tapp_amount1 = 1_000_000;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(user1),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount1,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			None,
			None,
		));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::expense(Origin::signed(user1), tapp_id),
			Error::<Test>::OperationAmountCanNotBeZero
		);
	})
}

#[test]
fn expense_should_fail_if_expense_amount_more_than_reserved_balance() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user1 = 1;
		let miner = 2;
		let tapp_amount1 = 1_000_000;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&miner, DOLLARS);

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(user1),
			b"test name".to_vec(),
			b"tea".to_vec(),
			tapp_amount1,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			Some(1000),
			Some(10),
		));

		let tapp_id = 1;

		let cml_id = 11;
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100, 10000));
		cml.set_owner(&miner);
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));
		assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id));

		let expense_amount = 1000000;
		TAppBondingCurve::<Test>::mutate(tapp_id, |tapp| tapp.current_cost = expense_amount);

		<Test as Config>::Currency::make_free_balance_be(&user1, 0);
		assert_noop!(
			BondingCurve::expense(Origin::signed(user1), tapp_id),
			Error::<Test>::TAppInsufficientFreeBalance
		);
	})
}

#[test]
fn host_works() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let miner = 2;
		let tapp_owner = 1;
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);

		let cml_id = 11;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100, 10000));
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(tapp_owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			Some(1000),
			Some(10),
		));

		let tapp_id = 1;
		assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id));

		assert!(TAppCurrentHosts::<Test>::contains_key(tapp_id, cml_id));
		assert_eq!(CmlHostingTApps::<Test>::get(cml_id).len(), 1);
		assert_eq!(CmlHostingTApps::<Test>::get(cml_id)[0], tapp_id);
	})
}

#[test]
fn host_should_fail_if_cml_not_belongs_to_user() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let miner = 2;
		let tapp_owner = 1;
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);

		let cml_id = 11;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100, 10000));
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::host(Origin::signed(33), cml_id, tapp_id),
			pallet_cml::Error::<Test>::CMLOwnerInvalid
		);

		assert_noop!(
			BondingCurve::host(Origin::signed(33), 44, tapp_id),
			pallet_cml::Error::<Test>::NotFoundCML
		);
	})
}

#[test]
fn host_should_fail_if_tapp_not_exist() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let miner = 2;
		let tapp_owner = 1;
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);

		let cml_id = 11;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100, 10000));
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::host(Origin::signed(miner), cml_id, tapp_id),
			Error::<Test>::TAppIdNotExist
		);
	})
}

#[test]
fn host_should_fail_if_not_supported_for_hosting() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let miner = 2;
		let tapp_owner = 1;
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);

		let cml_id = 11;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100, 10000));
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(tapp_owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			None,
			None,
		));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::host(Origin::signed(miner), cml_id, tapp_id),
			Error::<Test>::TAppNotSupportToHost
		);
	})
}

#[test]
fn host_should_fail_if_cml_is_already_hosting() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let miner = 2;
		let tapp_owner = 1;
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);

		let cml_id = 11;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100, 10000));
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(tapp_owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			Some(1000),
			Some(1),
		));

		let tapp_id = 1;
		assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id));

		assert_noop!(
			BondingCurve::host(Origin::signed(miner), cml_id, tapp_id),
			Error::<Test>::CmlIsAlreadyHosting
		);
	})
}

#[test]
fn host_should_fail_if_tapp_hosts_if_full() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let miner = 2;
		let tapp_owner = 1;
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);

		let cml_id = 11;
		let cml_id2 = 22;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100, 10000));
		let cml2 = CML::from_genesis_seed(seed_from_lifespan(cml_id2, 100, 10000));
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		UserCmlStore::<Test>::insert(miner, cml_id2, ());
		CmlStore::<Test>::insert(cml_id, cml);
		CmlStore::<Test>::insert(cml_id2, cml2);

		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id2,
			[2u8; 32],
			b"miner_ip".to_vec()
		));

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(tapp_owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			Some(1000),
			Some(1),
		));

		let tapp_id = 1;
		assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id));

		assert_noop!(
			BondingCurve::host(Origin::signed(miner), cml_id2, tapp_id),
			Error::<Test>::TAppHostsIsFull
		);
	})
}

#[test]
fn host_should_fail_if_cml_is_full_load() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let miner = 2;
		let tapp_owner = 1;
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);

		let cml_id = 11;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100, 4000));
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(tapp_owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			Some(1000),
			Some(1),
		));
		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(tapp_owner),
			b"test name2".to_vec(),
			b"tea2".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			Some(1000),
			Some(1),
		));

		let tapp_id = 1;
		let tapp_id2 = 2;
		assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id));
		assert_noop!(
			BondingCurve::host(Origin::signed(miner), cml_id, tapp_id2),
			Error::<Test>::CmlMachineIsFullLoad
		);
	})
}

#[test]
fn host_should_fail_if_cml_is_not_mining() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let miner = 2;
		let tapp_owner = 1;
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);

		let cml_id = 11;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100, 10000));
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(tapp_owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			Some(1000),
			Some(1),
		));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::host(Origin::signed(miner), cml_id, tapp_id),
			Error::<Test>::OnlyMiningCmlCanHost
		);
	})
}

#[test]
fn unhost_works() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let miner = 2;
		let tapp_owner = 1;
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);

		let cml_id = 11;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100, 10000));
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(tapp_owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			Some(1000),
			Some(10),
		));

		let tapp_id = 1;
		assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id));

		assert!(TAppCurrentHosts::<Test>::contains_key(tapp_id, cml_id));
		assert_eq!(CmlHostingTApps::<Test>::get(cml_id).len(), 1);
		assert_eq!(CmlHostingTApps::<Test>::get(cml_id)[0], tapp_id);

		assert_ok!(BondingCurve::unhost(Origin::signed(miner), cml_id, tapp_id));

		assert!(!TAppCurrentHosts::<Test>::contains_key(tapp_id, cml_id));
		assert_eq!(CmlHostingTApps::<Test>::get(cml_id).len(), 0);
	})
}

#[test]
fn unhost_should_fail_if_cml_not_belongs_to_user() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let miner = 2;
		let tapp_owner = 1;
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);

		let cml_id = 11;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100, 10000));
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(tapp_owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			Some(1000),
			Some(10),
		));

		let tapp_id = 1;
		assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id));

		assert_noop!(
			BondingCurve::unhost(Origin::signed(4), cml_id, tapp_id),
			pallet_cml::Error::<Test>::CMLOwnerInvalid
		);
	})
}

#[test]
fn unhost_should_fail_if_tapp_id_not_exist() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let miner = 2;
		let tapp_owner = 1;
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);

		let cml_id = 11;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100, 10000));
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::unhost(Origin::signed(miner), cml_id, tapp_id),
			Error::<Test>::TAppIdNotExist
		);
	})
}

#[test]
fn unhost_should_fail_if_cml_not_host_the_tapp() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let miner = 2;
		let tapp_owner = 1;
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);

		let cml_id = 11;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100, 10000));
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(tapp_owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			Some(1000),
			Some(10),
		));
		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(tapp_owner),
			b"test name2".to_vec(),
			b"tea2".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			Some(1000),
			Some(10),
		));

		let tapp_id = 1;
		let tapp_id2 = 2;
		assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id));

		assert_noop!(
			BondingCurve::unhost(Origin::signed(miner), cml_id, tapp_id2),
			Error::<Test>::CmlNotHostTheTApp
		);
	})
}

pub fn seed_from_lifespan(id: CmlId, lifespan: u32, performance: u32) -> Seed {
	Seed {
		id,
		cml_type: CmlType::A,
		defrost_schedule: Some(DefrostScheduleType::Team),
		defrost_time: Some(0),
		lifespan,
		performance,
	}
}
