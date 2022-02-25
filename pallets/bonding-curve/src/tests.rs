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
fn update_tapp_last_activity_works() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);

		let npc = NPCAccount::<Test>::get();
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);
		assert_ok!(create_default_tapp(user));

		let tapp_id = 1;

		let block_number = 100;
		let activity_data = 43;
		frame_system::Pallet::<Test>::set_block_number(block_number);
		assert_ok!(BondingCurve::update_tapp_last_activity(
			Origin::signed(npc),
			tapp_id,
			activity_data
		));

		assert_eq!(
			TAppLastActivity::<Test>::get(tapp_id),
			Some((activity_data, block_number))
		);
	})
}

#[test]
fn normal_user_update_tapp_last_activity_should_fail() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);

		let user = 1;
		assert_noop!(
			BondingCurve::update_tapp_last_activity(Origin::signed(user), 1, 22),
			Error::<Test>::OnlyNPCAccountAllowedToUpdateActivity
		);
	})
}

#[test]
fn update_tapp_last_activity_should_fail_if_tapp_not_exist() {
	new_test_ext().execute_with(|| {
		let npc = NPCAccount::<Test>::get();
		assert_noop!(
			BondingCurve::update_tapp_last_activity(Origin::signed(npc), 1, 22),
			Error::<Test>::TAppIdNotExist
		);
	})
}

#[test]
fn register_tapp_link_works() {
	new_test_ext().execute_with(|| {
		let npc = NPCAccount::<Test>::get();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			"https://teaproject.org".into(),
			"test description".into(),
			None,
		));
	})
}

#[test]
fn normal_user_register_tapp_link_should_fail() {
	new_test_ext().execute_with(|| {
		let user1 = 1;
		assert_noop!(
			BondingCurve::register_tapp_link(
				Origin::signed(user1),
				"https://teaproject.org".into(),
				"test description".into(),
				None,
			),
			Error::<Test>::OnlyNPCAccountAllowedToRegisterLinkUrl
		);
	})
}

#[test]
fn register_tapp_link_should_fail_if_tapp_link_is_too_long() {
	new_test_ext().execute_with(|| {
		let npc = NPCAccount::<Test>::get();
		assert_noop!(
			BondingCurve::register_tapp_link(
				Origin::signed(npc),
				[0; TAPP_LINK_MAX_LENGTH as usize + 1].to_vec(),
				"test description".into(),
				None,
			),
			Error::<Test>::TAppLinkIsTooLong
		);
	})
}

#[test]
fn register_tapp_link_should_fail_if_tapp_link_desc_is_too_long() {
	new_test_ext().execute_with(|| {
		let npc = NPCAccount::<Test>::get();
		assert_noop!(
			BondingCurve::register_tapp_link(
				Origin::signed(npc),
				"https://teaproject.org".into(),
				[0; TAPP_LINK_DESCRIPTION_MAX_LENGTH as usize + 1].to_vec(),
				None,
			),
			Error::<Test>::LinkDescriptionIsTooLong
		);
	})
}

#[test]
fn register_tapp_link_should_fail_if_link_already_exist() {
	new_test_ext().execute_with(|| {
		let npc = NPCAccount::<Test>::get();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			"https://teaproject.org".into(),
			"test description".into(),
			None,
		));

		assert_noop!(
			BondingCurve::register_tapp_link(
				Origin::signed(npc),
				"https://teaproject.org".into(),
				"test description".into(),
				None,
			),
			Error::<Test>::LinkUrlAlreadyExist
		);
	})
}

#[test]
fn unregister_tapp_link_works() {
	new_test_ext().execute_with(|| {
		let npc = NPCAccount::<Test>::get();
		let link: Vec<u8> = "https://teaproject.org".into();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			link.clone(),
			"test description".into(),
			None,
		));

		assert!(TAppApprovedLinks::<Test>::contains_key(&link));
		assert_ok!(BondingCurve::unregister_tapp_link(
			Origin::signed(npc),
			link.clone(),
		));
		assert!(!TAppApprovedLinks::<Test>::contains_key(&link));
	})
}

#[test]
fn unregister_tapp_link_should_fail_if_user_not_npc_account() {
	new_test_ext().execute_with(|| {
		let npc = NPCAccount::<Test>::get();
		let link: Vec<u8> = "https://teaproject.org".into();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			link.clone(),
			"test description".into(),
			None,
		));

		assert_noop!(
			BondingCurve::unregister_tapp_link(Origin::signed(11), link.clone(),),
			Error::<Test>::OnlyNPCAccountAllowedToRegisterLinkUrl
		);
	})
}

#[test]
fn unregister_tapp_link_should_fail_if_link_not_registered() {
	new_test_ext().execute_with(|| {
		let npc = NPCAccount::<Test>::get();
		let link: Vec<u8> = "https://teaproject.org".into();

		assert_noop!(
			BondingCurve::unregister_tapp_link(Origin::signed(npc), link.clone()),
			Error::<Test>::LinkUrlNotExist
		);
	})
}

#[test]
fn create_new_fixed_fee_tapp_works() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		let tapp_name = "test name";
		let ticker = "tea";
		let detail = "test detail";
		let link = "https://teaproject.org";
		let init_fund = 1000000;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		let npc = NPCAccount::<Test>::get();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			"https://teaproject.org".into(),
			"test description".into(),
			None,
		));
		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(user),
			tapp_name.as_bytes().to_vec(),
			ticker.as_bytes().to_vec(),
			init_fund,
			detail.as_bytes().to_vec(),
			link.as_bytes().to_vec(),
			10,
			TAppType::Twitter,
			false,
			Some(10000),
			None,
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
		assert_eq!(tapp_item.buy_curve_k, DEFAULT_BUY_CURVE_THETA);
		assert_eq!(tapp_item.sell_curve_k, DEFAULT_SELL_CURVE_THETA);
		assert_eq!(tapp_item.owner, user);
		assert_eq!(&String::from_utf8(tapp_item.name).unwrap(), tapp_name);
		assert_eq!(&String::from_utf8(tapp_item.ticker).unwrap(), ticker);
		assert_eq!(&String::from_utf8(tapp_item.detail).unwrap(), detail);
		assert_eq!(&String::from_utf8(tapp_item.link).unwrap(), link);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&user),
			99999534 - RESERVED_LINK_RENT_AMOUNT
		);
		assert_eq!(tapp_item.max_allowed_hosts, 10);
		assert_eq!(tapp_item.billing_mode, BillingMode::FixedHostingFee(10000));
		assert_eq!(tapp_item.tapp_type, TAppType::Twitter);
		assert_eq!(tapp_item.status, TAppStatus::Pending);
	})
}

#[test]
fn create_new_fixed_token_tapp_works() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		let tapp_name = "test name";
		let ticker = "tea";
		let detail = "test detail";
		let link = "https://teaproject.org";
		let init_fund = 1000000;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		let npc = NPCAccount::<Test>::get();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			"https://teaproject.org".into(),
			"test description".into(),
			None,
		));
		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(user),
			tapp_name.as_bytes().to_vec(),
			ticker.as_bytes().to_vec(),
			init_fund,
			detail.as_bytes().to_vec(),
			link.as_bytes().to_vec(),
			10,
			TAppType::Reddit,
			true,
			None,
			Some(1000),
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
		assert_eq!(tapp_item.buy_curve_k, DEFAULT_BUY_CURVE_THETA);
		assert_eq!(tapp_item.sell_curve_k, DEFAULT_SELL_CURVE_THETA);
		assert_eq!(tapp_item.owner, user);
		assert_eq!(&String::from_utf8(tapp_item.name).unwrap(), tapp_name);
		assert_eq!(&String::from_utf8(tapp_item.ticker).unwrap(), ticker);
		assert_eq!(&String::from_utf8(tapp_item.detail).unwrap(), detail);
		assert_eq!(&String::from_utf8(tapp_item.link).unwrap(), link);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&user),
			99999534 - RESERVED_LINK_RENT_AMOUNT
		);
		assert_eq!(tapp_item.max_allowed_hosts, 10);
		assert_eq!(tapp_item.billing_mode, BillingMode::FixedHostingToken(1000));
		assert_eq!(tapp_item.tapp_type, TAppType::Reddit);
		assert_eq!(tapp_item.status, TAppStatus::Pending);
	})
}

#[test]
fn create_new_tapp_with_custom_theta_works() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		let tapp_name = "test name";
		let ticker = "tea";
		let detail = "test detail";
		let link = "https://teaproject.org";
		let init_fund = 1000000;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		let buy_curve_theta = 100;
		let sell_curve_theta = 99;
		let npc = NPCAccount::<Test>::get();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			"https://teaproject.org".into(),
			"test description".into(),
			None,
		));
		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(user),
			tapp_name.as_bytes().to_vec(),
			ticker.as_bytes().to_vec(),
			init_fund,
			detail.as_bytes().to_vec(),
			link.as_bytes().to_vec(),
			10,
			TAppType::Reddit,
			true,
			None,
			Some(1000),
			Some(buy_curve_theta),
			Some(sell_curve_theta),
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
		assert_eq!(tapp_item.buy_curve_k, buy_curve_theta);
		assert_eq!(tapp_item.sell_curve_k, sell_curve_theta);
		assert_eq!(tapp_item.owner, user);
		assert_eq!(&String::from_utf8(tapp_item.name).unwrap(), tapp_name);
		assert_eq!(&String::from_utf8(tapp_item.ticker).unwrap(), ticker);
		assert_eq!(&String::from_utf8(tapp_item.detail).unwrap(), detail);
		assert_eq!(&String::from_utf8(tapp_item.link).unwrap(), link);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&user),
			99993400 - RESERVED_LINK_RENT_AMOUNT
		);
		assert_eq!(tapp_item.max_allowed_hosts, 10);
		assert_eq!(tapp_item.billing_mode, BillingMode::FixedHostingToken(1000));
		assert_eq!(tapp_item.tapp_type, TAppType::Reddit);
		assert_eq!(tapp_item.status, TAppStatus::Pending);
	})
}

#[test]
fn create_new_reserved_tapp_using_npc_account_works() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		LastTAppId::<Test>::set(RESERVED_TAPP_ID_COUNT);
		LastReservedTAppId::<Test>::set(0);
		ReservedBalanceAccount::<Test>::set(100);

		let npc = NPCAccount::<Test>::get();
		let tapp_name = "test name";
		let ticker = "tea";
		let detail = "test detail";
		let link = "https://teaproject.org";
		let init_fund = 1000000;
		<Test as Config>::Currency::make_free_balance_be(&npc, 100000000);

		let buy_curve_theta = 100;
		let sell_curve_theta = 99;
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			"https://teaproject.org".into(),
			"test description".into(),
			None,
		));
		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(npc),
			tapp_name.as_bytes().to_vec(),
			ticker.as_bytes().to_vec(),
			init_fund,
			detail.as_bytes().to_vec(),
			link.as_bytes().to_vec(),
			10,
			TAppType::Reddit,
			true,
			None,
			Some(1000),
			Some(buy_curve_theta),
			Some(sell_curve_theta),
		));
		assert_eq!(BondingCurve::next_id(), RESERVED_TAPP_ID_COUNT + 1);

		// this is the first tapp so tapp id is 1
		let tapp_id = 1;
		assert_eq!(LastReservedTAppId::<Test>::get(), tapp_id);
		assert_eq!(AccountTable::<Test>::get(npc, tapp_id), init_fund);
		assert_eq!(TotalSupplyTable::<Test>::get(tapp_id), init_fund);
		assert_eq!(TAppNames::<Test>::get(tapp_name.as_bytes()), tapp_id);
		assert_eq!(TAppTickers::<Test>::get(ticker.as_bytes()), tapp_id);
		let tapp_item = TAppBondingCurve::<Test>::get(tapp_id);
		assert_eq!(tapp_item.id, tapp_id);
		assert_eq!(tapp_item.buy_curve_k, buy_curve_theta);
		assert_eq!(tapp_item.sell_curve_k, sell_curve_theta);
		assert_eq!(tapp_item.owner, npc);
		assert_eq!(&String::from_utf8(tapp_item.name).unwrap(), tapp_name);
		assert_eq!(&String::from_utf8(tapp_item.ticker).unwrap(), ticker);
		assert_eq!(&String::from_utf8(tapp_item.detail).unwrap(), detail);
		assert_eq!(&String::from_utf8(tapp_item.link).unwrap(), link);
		assert_eq!(tapp_item.max_allowed_hosts, 10);
		assert_eq!(tapp_item.billing_mode, BillingMode::FixedHostingToken(1000));
		assert_eq!(tapp_item.tapp_type, TAppType::Reddit);
		assert_eq!(tapp_item.status, TAppStatus::Pending);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&npc),
			99993400 - RESERVED_LINK_RENT_AMOUNT
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_name_already_exist() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_ok!(create_default_tapp(user));

		assert_noop!(
			BondingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name".to_vec(),
				b"tea".to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
				None,
				None,
			),
			Error::<Test>::TAppNameAlreadyExist
		);
	})
}

#[test]
fn create_new_tapp_should_word_if_name_already_exist_with_bbs_type() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_ok!(create_default_tapp(user));

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(user),
			b"test name".to_vec(),
			b"tea2".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			10,
			TAppType::Bbs,
			true,
			None,
			Some(1000),
			None,
			None,
		));
	})
}

#[test]
fn create_new_tapp_should_word_if_link_not_exist_with_bbs_type() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(user),
			b"test name".to_vec(),
			b"tea2".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			10,
			TAppType::Bbs,
			true,
			None,
			Some(1000),
			None,
			None,
		));
		// bbs tapp should not have tapp link lease
		assert_eq!(<Test as Config>::Currency::free_balance(&user), 99999534);
	})
}

#[test]
fn create_new_tapp_should_fail_if_ticker_already_exist() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		let ticker = b"tea";
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_ok!(create_default_tapp(user));

		assert_noop!(
			BondingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name2".to_vec(),
				ticker.to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
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
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
				None,
				None,
			),
			Error::<Test>::NotAllowedNormalUserCreateTApp,
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_max_allowed_host_lower_than_min_allowed_host_count() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		let npc = NPCAccount::<Test>::get();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			"https://teaproject.org".into(),
			"test description".into(),
			None,
		));
		assert_noop!(
			BondingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name".to_vec(),
				b"tea".to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
				MIN_TAPP_HOSTS_AMOUNT - 1,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
				None,
				None,
			),
			Error::<Test>::MaxAllowedHostShouldLargerEqualThanMinAllowedHosts,
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_stake_token_is_none_in_fixed_token_mode() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		let npc = NPCAccount::<Test>::get();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			"https://teaproject.org".into(),
			"test description".into(),
			None,
		));
		assert_noop!(
			BondingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name".to_vec(),
				b"tea".to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
				10,
				TAppType::Twitter,
				true,
				None,
				None,
				None,
				None,
			),
			Error::<Test>::StakeTokenIsNoneInFixedTokenMode,
		);
	})
}

#[test]
fn create_new_tapp_works_if_link_not_in_approve_list() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		let link = b"https://teaproject.org".to_vec();
		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(user),
			b"test name".to_vec(),
			b"tea".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			link.clone(),
			10,
			TAppType::Twitter,
			true,
			None,
			Some(1000),
			None,
			None,
		));

		assert!(!TAppApprovedLinks::<Test>::contains_key(&link));
	})
}

#[test]
fn create_new_tapp_should_fail_if_link_created_by_other_users() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		let user2 = 2;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		let npc = NPCAccount::<Test>::get();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			"https://teaproject.org".into(),
			"test description".into(),
			Some(user2),
		));
		assert_noop!(
			BondingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name".to_vec(),
				b"tea".to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
				None,
				None,
			),
			Error::<Test>::UserReservedLink,
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_link_already_be_used() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		assert_ok!(create_default_tapp(user));

		assert_noop!(
			create_default_tapp(user),
			Error::<Test>::LinkUrlAlreadyExist,
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_stake_token_is_zero_in_fixed_token_mode() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		let npc = NPCAccount::<Test>::get();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			"https://teaproject.org".into(),
			"test description".into(),
			None,
		));
		assert_noop!(
			BondingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name".to_vec(),
				b"tea".to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
				10,
				TAppType::Twitter,
				true,
				None,
				Some(0),
				None,
				None,
			),
			Error::<Test>::StakeTokenShouldNotBeZero,
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_reward_per_1k_performance_is_none_in_fixed_fee_mode() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		let npc = NPCAccount::<Test>::get();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			"https://teaproject.org".into(),
			"test description".into(),
			None,
		));
		assert_noop!(
			BondingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name".to_vec(),
				b"tea".to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
				10,
				TAppType::Twitter,
				false,
				None,
				None,
				None,
				None,
			),
			Error::<Test>::RewardPerPerformanceIsNoneInFixedFeeMode,
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_reward_per_1k_performance_is_zero_in_fixed_fee_mode() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, 100000000);

		let npc = NPCAccount::<Test>::get();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			"https://teaproject.org".into(),
			"test description".into(),
			None,
		));
		assert_noop!(
			BondingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name".to_vec(),
				b"tea".to_vec(),
				1_000_000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
				10,
				TAppType::Twitter,
				false,
				Some(0),
				None,
				None,
				None,
			),
			Error::<Test>::RewardPerPerformanceShouldNotBeZero,
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
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
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
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
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
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
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
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
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

		let npc = NPCAccount::<Test>::get();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			"https://teaproject.org".into(),
			"test description".into(),
			None,
		));
		assert_noop!(
			BondingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name".to_vec(),
				b"tea".to_vec(),
				1000000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
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

		let npc = NPCAccount::<Test>::get();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			"https://teaproject.org".into(),
			"test description".into(),
			None,
		));
		assert_noop!(
			BondingCurve::create_new_tapp(
				Origin::signed(user),
				b"test name".to_vec(),
				b"tea".to_vec(),
				1000,
				b"test detail".to_vec(),
				b"https://teaproject.org".to_vec(),
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
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
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
				None,
				None,
			),
			Error::<Test>::TAppNameIsTooLong
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_buy_curve_theta_is_zero() {
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
				b"https://teaproject.org".to_vec(),
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
				Some(0),
				None,
			),
			Error::<Test>::BuyCurveThetaCanNotBeZero
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_sell_curve_theta_is_zero() {
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
				b"https://teaproject.org".to_vec(),
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
				None,
				Some(0),
			),
			Error::<Test>::SellCurveThetaCanNotBeZero
		);
	})
}

#[test]
fn create_new_tapp_should_fail_if_buy_curve_theta_small_than_sell_curve() {
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
				b"https://teaproject.org".to_vec(),
				10,
				TAppType::Twitter,
				true,
				None,
				Some(1000),
				Some(1),
				Some(2),
			),
			Error::<Test>::BuyCurveThetaShouldLargerEqualThanSellCurveTheta
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

		assert_ok!(create_default_tapp(owner));

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
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user, DOLLARS);

		assert_ok!(create_default_tapp(owner));

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
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user, DOLLARS);

		assert_ok!(create_default_tapp(owner));

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
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user, 0);

		assert_ok!(create_default_tapp(owner));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::buy_token(Origin::signed(user), tapp_id, 1000000),
			Error::<Test>::InsufficientFreeBalance
		);
	})
}

#[test]
fn buy_token_should_fail_if_total_supply_larger_than_max_allowed() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let owner = 1;
		let user = 2;
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user, u128::MAX);

		assert_ok!(create_default_tapp(owner));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::buy_token(Origin::signed(user), tapp_id, TOTAL_SUPPLY_MAX_VALUE),
			Error::<Test>::TotalSupplyOverTheMaxValue
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

		assert_ok!(create_default_tapp(owner));

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
			1000
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

		let npc = NPCAccount::<Test>::get();
		let link = b"https://teaproject.org".to_vec();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			link.clone(),
			"test description".into(),
			Some(owner),
		));
		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			link,
			10,
			TAppType::Twitter,
			true,
			None,
			Some(1000),
			None,
			None,
		));

		let tapp_id = 1;

		let link = b"https://teaproject.org".to_vec();
		assert_eq!(TAppApprovedLinks::<Test>::get(&link).tapp_id, Some(tapp_id));
		assert_eq!(TAppApprovedLinks::<Test>::get(&link).creator, Some(owner));

		assert_ok!(BondingCurve::sell_token(
			Origin::signed(owner),
			tapp_id,
			tapp_amount
		));

		assert_eq!(TAppApprovedLinks::<Test>::get(&link).tapp_id, None);
		assert_eq!(TAppApprovedLinks::<Test>::get(&link).creator, None);
		assert!(!AccountTable::<Test>::contains_key(&owner, tapp_id));
		assert!(!TotalSupplyTable::<Test>::contains_key(tapp_id));
		assert!(!TAppBondingCurve::<Test>::contains_key(tapp_id));
		assert!(!TAppNames::<Test>::contains_key(name));
		assert!(!TAppTickers::<Test>::contains_key(ticker));
		assert_eq!(
			<Test as Config>::Currency::free_balance(&owner),
			999999999534
		);
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

		assert_ok!(create_default_tapp(owner));

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
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user, DOLLARS);

		assert_ok!(create_default_tapp(owner));

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
		<Test as Config>::Currency::make_free_balance_be(&owner, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user, DOLLARS);

		assert_ok!(create_default_tapp(owner));

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

		assert_ok!(create_default_tapp(owner));

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
fn consume_works_large_tea() {
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
		<Test as Config>::Currency::make_free_balance_be(&user4, 100 * DOLLARS);
		assert_ok!(create_default_tapp(user1));

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
		let spend_tea = 10 * DOLLARS;
		assert_ok!(BondingCurve::consume(
			Origin::signed(user4),
			tapp_id,
			spend_tea,
			None
		));
		let left_balance = <Test as Config>::Currency::free_balance(&user4);
		// println!("2 {:?}", &left_balance);
		assert!(approximately_equals::<Test>(
			left_balance,
			100 * DOLLARS - spend_tea,
			10,
		));
	})
}
#[test]
fn consume_should_fail_if_tapp_not_exist() {
	new_test_ext().execute_with(|| {
		let user1 = 1;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);

		assert_noop!(
			BondingCurve::consume(Origin::signed(user1), 1, 10000, None),
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
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user2, DOLLARS);

		assert_ok!(create_default_tapp(user1));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::consume(Origin::signed(user2), tapp_id, 0, None),
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
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user2, 0);

		assert_ok!(create_default_tapp(user1));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::consume(Origin::signed(user2), tapp_id, 1_000_000, None),
			Error::<Test>::InsufficientFreeBalance
		);
	})
}

#[test]
fn consume_should_fail_if_note_is_too_long() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user1 = 1;
		let user2 = 2;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user2, DOLLARS);

		assert_ok!(create_default_tapp(user1));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::consume(
				Origin::signed(user2),
				tapp_id,
				1_000_000,
				Some(vec![1; CONSUME_NOTE_MAX_LENGTH as usize + 1])
			),
			Error::<Test>::ConsumeNoteIsTooLong
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
		assert_ok!(create_default_tapp(user1));

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
		let spend_tea = 1000000;
		assert_ok!(BondingCurve::consume(
			Origin::signed(user4),
			tapp_id,
			spend_tea,
			Some(b"test notes".to_vec())
		));
		let left_balance = <Test as Config>::Currency::free_balance(&user4);
		assert!(approximately_equals::<Test>(
			left_balance,
			DOLLARS - spend_tea,
			10,
		));
		assert_eq!(AccountTable::<Test>::get(user1, tapp_id), 18873325);
		assert_eq!(AccountTable::<Test>::get(user2, tapp_id), 37746650);
		assert_eq!(AccountTable::<Test>::get(user3, tapp_id), 75493301);
		assert_eq!(AccountTable::<Test>::get(user4, tapp_id), 0);
		assert_eq!(TotalSupplyTable::<Test>::get(tapp_id), 132113276)
	})
}

#[test]
fn consume_works_with_miner() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user1 = 1;
		let user2 = 2;
		let user3 = 3;
		let user4 = 4;
		let miner1 = 5;
		let miner2 = 6;
		let tapp_amount1 = 1_000_000;
		let tapp_amount2 = 2_000_000;
		let tapp_amount3 = 4_000_000;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user2, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user3, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user4, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&miner1, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&miner2, DOLLARS);

		let npc = NPCAccount::<Test>::get();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			"https://teaproject.org".into(),
			"test description".into(),
			None,
		));
		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(user1),
			b"test name".to_vec(),
			b"tea".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			10,
			TAppType::Twitter,
			true,
			None,
			Some(1_000_000),
			None,
			None,
		));

		let cml_id1 = 11;
		let cml_id2 = 22;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id1, 100, 10000));
		let cml2 = CML::from_genesis_seed(seed_from_lifespan(cml_id2, 100, 10000));
		UserCmlStore::<Test>::insert(miner1, cml_id1, ());
		UserCmlStore::<Test>::insert(miner2, cml_id2, ());
		CmlStore::<Test>::insert(cml_id1, cml);
		CmlStore::<Test>::insert(cml_id2, cml2);
		assert_ok!(Cml::start_mining(
			Origin::signed(miner1),
			cml_id1,
			[1u8; 32],
			miner1,
			b"miner_ip".to_vec(),
			None,
		));
		assert_ok!(Cml::start_mining(
			Origin::signed(miner2),
			cml_id2,
			[2u8; 32],
			miner2,
			b"miner_ip2".to_vec(),
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
		assert_ok!(BondingCurve::host(Origin::signed(miner1), cml_id1, tapp_id));
		assert_ok!(BondingCurve::host(Origin::signed(miner2), cml_id2, tapp_id));
		// total supply only including staking amount
		assert_eq!(
			TotalSupplyTable::<Test>::get(tapp_id),
			tapp_amount1 + tapp_amount2 + tapp_amount3
		);

		Cml::collect_staking_info();
		const MOCKED_DOLLARS: u128 = 100000; // this is the mocked DOLLARS returned by dummy cml staking implementation

		let spend_tea = 1000000;
		assert_ok!(BondingCurve::consume(
			Origin::signed(user4),
			tapp_id,
			spend_tea,
			Some(b"test notes".to_vec())
		));
		let left_balance = <Test as Config>::Currency::free_balance(&user4);
		assert!(approximately_equals::<Test>(
			left_balance,
			DOLLARS - spend_tea,
			10,
		));
		assert_eq!(
			AccountTable::<Test>::get(user1, tapp_id),
			13901475 + 1_000_000
		);
		assert_eq!(AccountTable::<Test>::get(user2, tapp_id), 29802950);
		assert_eq!(AccountTable::<Test>::get(user3, tapp_id), 59605901);
		assert_eq!(AccountTable::<Test>::get(miner1, tapp_id), MOCKED_DOLLARS);
		assert_eq!(AccountTable::<Test>::get(miner2, tapp_id), MOCKED_DOLLARS);
		assert_eq!(AccountTable::<Test>::get(user4, tapp_id), 0);
		assert_eq!(TotalSupplyTable::<Test>::get(tapp_id), 104510326)
	})
}

#[test]
fn miner_cannot_sell_reserved_token_however_allowed_to_sell_consume_rewards() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user1 = 1;
		let user2 = 1;
		let miner1 = 5;
		let miner2 = 6;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&user2, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&miner1, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&miner2, DOLLARS);
		assert_ok!(create_default_tapp(user1));

		let cml_id1 = 11;
		let cml_id2 = 22;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id1, 100, 10000));
		let cml2 = CML::from_genesis_seed(seed_from_lifespan(cml_id2, 100, 10000));
		UserCmlStore::<Test>::insert(miner1, cml_id1, ());
		UserCmlStore::<Test>::insert(miner2, cml_id2, ());
		CmlStore::<Test>::insert(cml_id1, cml);
		CmlStore::<Test>::insert(cml_id2, cml2);
		assert_ok!(Cml::start_mining(
			Origin::signed(miner1),
			cml_id1,
			[1u8; 32],
			miner1,
			b"miner_ip".to_vec(),
			None,
		));
		assert_ok!(Cml::start_mining(
			Origin::signed(miner2),
			cml_id2,
			[2u8; 32],
			miner2,
			b"miner_ip2".to_vec(),
			None,
		));

		let tapp_id = 1;
		assert_ok!(BondingCurve::host(Origin::signed(miner1), cml_id1, tapp_id));
		assert_ok!(BondingCurve::host(Origin::signed(miner2), cml_id2, tapp_id));

		Cml::collect_staking_info();
		const MOCKED_DOLLARS: u128 = 100000; // this is the mocked DOLLARS returned by dummy cml staking implementation

		let spend_tea = 1000000;
		assert_ok!(BondingCurve::consume(
			Origin::signed(user2),
			tapp_id,
			spend_tea,
			Some(b"test notes".to_vec())
		));
		assert_eq!(AccountTable::<Test>::get(miner1, tapp_id), MOCKED_DOLLARS);
		assert_eq!(
			TAppReservedBalance::<Test>::get(tapp_id, miner1)[0],
			(1000, cml_id1)
		);
		assert_eq!(AccountTable::<Test>::get(miner2, tapp_id), MOCKED_DOLLARS);
		assert_eq!(
			TAppReservedBalance::<Test>::get(tapp_id, miner2)[0],
			(1000, cml_id2)
		);

		assert_ok!(BondingCurve::sell_token(
			Origin::signed(miner1),
			tapp_id,
			MOCKED_DOLLARS
		));
		assert_eq!(AccountTable::<Test>::get(miner1, tapp_id), 0);
		assert_eq!(
			TAppReservedBalance::<Test>::get(tapp_id, miner1)[0],
			(1000, cml_id1)
		);
		// can not sell reserved token
		assert_noop!(
			BondingCurve::sell_token(Origin::signed(miner1), tapp_id, 1000),
			Error::<Test>::InsufficientTAppToken
		);

		// can not sell tapp token mixed with reserved token
		assert_noop!(
			BondingCurve::sell_token(Origin::signed(miner2), tapp_id, MOCKED_DOLLARS + 1000),
			Error::<Test>::InsufficientTAppToken
		);
	})
}

#[test]
fn consume_auto_works() {
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
		<Test as Config>::Currency::make_free_balance_be(&user4, 100 * DOLLARS);
		assert_ok!(create_default_tapp(user1));

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
		let spend_tea = 10 * DOLLARS;
		assert_ok!(BondingCurve::consume_auto(
			Origin::signed(user4),
			tapp_id,
			spend_tea,
			None,
			b"test tsid".to_vec()
		));
		let left_balance = <Test as Config>::Currency::free_balance(&user4);
		// println!("2 {:?}", &left_balance);
		assert!(approximately_equals::<Test>(
			left_balance,
			100 * DOLLARS - spend_tea,
			10,
		));
	})
}

#[test]
fn consume_auto_should_fail_if_tsid_is_same() {
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
		<Test as Config>::Currency::make_free_balance_be(&user4, 100 * DOLLARS);
		assert_ok!(create_default_tapp(user1));

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
		let spend_tea = 10 * DOLLARS;
		assert_ok!(BondingCurve::consume_auto(
			Origin::signed(user4),
			tapp_id,
			spend_tea,
			None,
			b"test tsid".to_vec()
		));

		assert_noop!(
			BondingCurve::consume_auto(
				Origin::signed(user4),
				tapp_id,
				spend_tea,
				None,
				b"test tsid".to_vec()
			),
			Error::<Test>::ConsumeTsidAlreadyExist
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
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);

		assert_ok!(create_default_tapp(user1));

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
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);

		assert_ok!(create_default_tapp(user1));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::expense(Origin::signed(user1), tapp_id),
			Error::<Test>::OperationAmountCanNotBeZero
		);
	})
}

#[test]
fn expense_works_if_expense_amount_more_than_reserved_balance() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let user1 = 1;
		let miner = 2;
		<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);
		<Test as Config>::Currency::make_free_balance_be(&miner, DOLLARS);

		assert_ok!(create_default_tapp(user1));

		let tapp_id = 1;

		let cml_id = 11;
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 10000, 10000));
		cml.set_owner(&miner);
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			[1u8; 32],
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		frame_system::Pallet::<Test>::set_block_number(101);
		assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id));

		let expense_amount = 1_000_000;
		TAppBondingCurve::<Test>::mutate(tapp_id, |tapp| tapp.current_cost = expense_amount);

		<Test as Config>::Currency::make_free_balance_be(&user1, 0);
		assert_ok!(BondingCurve::expense(Origin::signed(user1), tapp_id));

		assert!(!TAppBondingCurve::<Test>::contains_key(tapp_id));
		assert!(!TAppCurrentHosts::<Test>::contains_key(tapp_id, cml_id));
		assert_eq!(CmlHostingTApps::<Test>::get(cml_id).len(), 0);
	})
}

#[test]
fn host_works_with_fixed_fee() {
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
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		let npc = NPCAccount::<Test>::get();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			"https://teaproject.org".into(),
			"test description".into(),
			None,
		));
		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(tapp_owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			10,
			TAppType::Twitter,
			false,
			Some(1000),
			None,
			None,
			None,
		));

		let tapp_id = 1;
		assert_eq!(
			TAppBondingCurve::<Test>::get(tapp_id).status,
			TAppStatus::Pending
		);
		assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id));

		assert_eq!(
			TAppBondingCurve::<Test>::get(tapp_id).status,
			TAppStatus::Active(0)
		);
		assert!(TAppCurrentHosts::<Test>::contains_key(tapp_id, cml_id));
		assert_eq!(CmlHostingTApps::<Test>::get(cml_id).len(), 1);
		assert_eq!(CmlHostingTApps::<Test>::get(cml_id)[0], tapp_id);
		assert!(!TAppReservedBalance::<Test>::contains_key(tapp_id, miner));
		assert_eq!(
			Utils::free_balance(&miner),
			10000 - STAKING_PRICE - HOST_PLEDGE_AMOUNT
		);
		assert_eq!(
			Utils::reserved_balance(&miner),
			STAKING_PRICE + HOST_PLEDGE_AMOUNT
		);
		assert_eq!(
			TAppHostPledge::<Test>::get(tapp_id, cml_id),
			HOST_PLEDGE_AMOUNT
		);
	})
}

#[test]
fn host_works_fixed_token() {
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
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		assert_ok!(create_default_tapp(tapp_owner));

		let tapp_id = 1;
		assert_eq!(
			TAppBondingCurve::<Test>::get(tapp_id).status,
			TAppStatus::Pending
		);
		assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id));

		assert_eq!(
			TAppBondingCurve::<Test>::get(tapp_id).status,
			TAppStatus::Active(0)
		);
		assert!(TAppCurrentHosts::<Test>::contains_key(tapp_id, cml_id));
		assert_eq!(CmlHostingTApps::<Test>::get(cml_id).len(), 1);
		assert_eq!(CmlHostingTApps::<Test>::get(cml_id)[0], tapp_id);
		assert_eq!(
			TAppReservedBalance::<Test>::get(tapp_id, miner)[0],
			(1000, cml_id)
		);
		assert_eq!(
			Utils::free_balance(&miner),
			10000 - STAKING_PRICE - HOST_PLEDGE_AMOUNT
		);
		assert_eq!(
			Utils::reserved_balance(&miner),
			STAKING_PRICE + HOST_PLEDGE_AMOUNT
		);
		assert_eq!(
			TAppHostPledge::<Test>::get(tapp_id, cml_id),
			HOST_PLEDGE_AMOUNT
		);
	})
}

#[test]
fn fixed_token_host_works_with_miner_hosts_multi_times() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let miner = 2;
		let tapp_owner = 1;
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);
		<Test as Config>::Currency::make_free_balance_be(&miner, 20000);

		let cml_id = 11;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100, 10000));
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		let cml_id2 = 12;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id2, 100, 10000));
		UserCmlStore::<Test>::insert(miner, cml_id2, ());
		CmlStore::<Test>::insert(cml_id2, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			[1u8; 32],
			miner,
			b"miner_ip".to_vec(),
			None,
		));
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id2,
			[2u8; 32],
			miner,
			b"miner_ip2".to_vec(),
			None,
		));

		assert_ok!(create_default_tapp(tapp_owner));

		let tapp_id = 1;
		assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id));
		assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id2, tapp_id));

		assert_eq!(TAppReservedBalance::<Test>::get(tapp_id, miner).len(), 2);
		assert_eq!(
			TAppReservedBalance::<Test>::get(tapp_id, miner)[0],
			(1000, cml_id)
		);
		assert_eq!(
			TAppReservedBalance::<Test>::get(tapp_id, miner)[1],
			(1000, cml_id2)
		);
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
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		assert_ok!(create_default_tapp(tapp_owner));

		let tapp_id = 1;
		assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id));

		assert_noop!(
			BondingCurve::host(Origin::signed(miner), cml_id, tapp_id),
			Error::<Test>::CmlIsAlreadyHosting
		);
	})
}

#[test]
fn host_should_fail_if_cml_c_type() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let miner = 2;
		let tapp_owner = 1;
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);

		let cml_id = 11;
		let mut seed = seed_from_lifespan(cml_id, 100, 10000);
		seed.cml_type = CmlType::C;
		let cml = CML::from_genesis_seed(seed);
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			[1u8; 32],
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		assert_ok!(create_default_tapp(tapp_owner));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::host(Origin::signed(miner), cml_id, tapp_id),
			Error::<Test>::NotAllowedTypeCHostingTApp
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
			miner,
			b"miner_ip".to_vec(),
			None,
		));
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id2,
			[2u8; 32],
			miner,
			b"miner_ip2".to_vec(),
			None,
		));

		let npc = NPCAccount::<Test>::get();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			"https://teaproject.org".into(),
			"test description".into(),
			None,
		));
		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(tapp_owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			1,
			TAppType::Twitter,
			true,
			None,
			Some(1000),
			None,
			None,
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
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		let npc = NPCAccount::<Test>::get();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			"https://teaproject.org".into(),
			"test description".into(),
			None,
		));
		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(tapp_owner),
			b"test name".to_vec(),
			b"tea".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			b"https://teaproject.org".to_vec(),
			1,
			TAppType::Twitter,
			true,
			None,
			Some(1000),
			None,
			None,
		));

		let npc = NPCAccount::<Test>::get();
		let link2 = b"https://tearust.org".to_vec();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			link2.clone(),
			"test description".into(),
			None,
		));
		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(tapp_owner),
			b"test name2".to_vec(),
			b"tea2".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			link2,
			1,
			TAppType::Twitter,
			true,
			None,
			Some(1000),
			None,
			None,
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
fn host_should_fail_if_cml_is_suspended() {
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

		let machine_id = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			machine_id,
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		Cml::suspend_mining(machine_id);

		assert_ok!(create_default_tapp(tapp_owner));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::host(Origin::signed(miner), cml_id, tapp_id),
			Error::<Test>::MiningCmlStatusShouldBeActive
		);
	})
}

#[test]
fn host_should_fail_if_not_enough_money() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let miner = 2;
		let tapp_owner = 1;
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);
		<Test as Config>::Currency::make_free_balance_be(&miner, STAKING_PRICE + 1);

		let cml_id = 11;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100, 4000));
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			[1u8; 32],
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		assert_ok!(create_default_tapp(tapp_owner));

		let tapp_id = 1;
		assert_noop!(
			BondingCurve::host(Origin::signed(miner), cml_id, tapp_id),
			Error::<Test>::InsufficientFreeBalance
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

		assert_ok!(create_default_tapp(tapp_owner));

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
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 10000, 10000));
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			[1u8; 32],
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		assert_ok!(create_default_tapp(tapp_owner));

		let tapp_id = 1;
		assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id));

		assert_eq!(
			TAppBondingCurve::<Test>::get(tapp_id).status,
			TAppStatus::Active(0)
		);
		assert!(TAppCurrentHosts::<Test>::contains_key(tapp_id, cml_id));
		assert_eq!(CmlHostingTApps::<Test>::get(cml_id).len(), 1);
		assert_eq!(CmlHostingTApps::<Test>::get(cml_id)[0], tapp_id);
		assert_eq!(
			Utils::free_balance(&miner),
			10000 - STAKING_PRICE - HOST_PLEDGE_AMOUNT
		);
		assert_eq!(
			Utils::reserved_balance(&miner),
			STAKING_PRICE + HOST_PLEDGE_AMOUNT
		);
		assert_eq!(
			TAppHostPledge::<Test>::get(tapp_id, cml_id),
			HOST_PLEDGE_AMOUNT
		);
		assert_eq!(TAppReservedBalance::<Test>::get(tapp_id, miner).len(), 1);

		frame_system::Pallet::<Test>::set_block_number(1001);
		assert_ok!(BondingCurve::unhost(Origin::signed(miner), cml_id, tapp_id));

		assert_eq!(
			TAppBondingCurve::<Test>::get(tapp_id).status,
			TAppStatus::Pending
		);
		assert!(!TAppCurrentHosts::<Test>::contains_key(tapp_id, cml_id));
		assert_eq!(CmlHostingTApps::<Test>::get(cml_id).len(), 0);
		assert_eq!(Utils::free_balance(&miner), 10000 - STAKING_PRICE);
		assert_eq!(Utils::reserved_balance(&miner), STAKING_PRICE);
		assert!(!TAppHostPledge::<Test>::contains_key(tapp_id, cml_id));
		assert!(!TAppReservedBalance::<Test>::contains_key(tapp_id, miner));
	})
}

#[test]
fn unhost_should_fail_if_not_after_locking_block_height() {
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
			miner,
			b"miner_ip".to_vec(),
			None,
		));
		assert_ok!(create_default_tapp(tapp_owner));

		let tapp_id = 1;
		assert_ok!(BondingCurve::host(Origin::signed(miner), cml_id, tapp_id));
		assert_noop!(
			BondingCurve::unhost(Origin::signed(miner), cml_id, tapp_id),
			Error::<Test>::HostLockingBlockHeightNotReached
		);
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
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		assert_ok!(create_default_tapp(tapp_owner));

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
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		assert_ok!(create_default_tapp(tapp_owner));

		let npc = NPCAccount::<Test>::get();
		let link = b"https://tearust.org".to_vec();
		assert_ok!(BondingCurve::register_tapp_link(
			Origin::signed(npc),
			link.clone(),
			"test description".into(),
			None,
		));
		assert_ok!(BondingCurve::create_new_tapp(
			Origin::signed(tapp_owner),
			b"test name2".to_vec(),
			b"tea2".to_vec(),
			1_000_000,
			b"test detail".to_vec(),
			link,
			10,
			TAppType::Twitter,
			true,
			None,
			Some(1000),
			None,
			None,
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

#[test]
fn update_tapp_resource_works() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let tapp_owner = 1;
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);

		assert_ok!(create_default_tapp(tapp_owner));

		let tapp_id = 1;
		assert!(!TAppResourceMap::<Test>::contains_key(tapp_id));

		let cid = b"test cid".to_vec();
		assert_ok!(BondingCurve::update_tapp_resource(
			Origin::signed(tapp_owner),
			tapp_id,
			cid.clone()
		));
		assert!(TAppResourceMap::<Test>::contains_key(tapp_id));
		assert_eq!(TAppResourceMap::<Test>::get(tapp_id), cid);
	})
}

#[test]
fn update_tapp_resource_should_fail_if_cid_is_too_long() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			BondingCurve::update_tapp_resource(
				Origin::signed(1),
				1,
				vec![0; CID_MAX_LENGTH as usize + 1]
			),
			Error::<Test>::CidIsToLong
		);
	})
}

#[test]
fn update_tapp_resource_should_fail_if_tapp_id_not_exist() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			BondingCurve::update_tapp_resource(Origin::signed(1), 1, b"test cid".to_vec(),),
			Error::<Test>::TAppIdNotExist
		);
	})
}

#[test]
fn update_tapp_resource_should_fail_if_user_is_not_tapp_owner() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let tapp_owner = 1;
		let user = 2;
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, 100000000);

		assert_ok!(create_default_tapp(tapp_owner));

		let tapp_id = 1;

		let cid = b"test cid".to_vec();
		assert_noop!(
			BondingCurve::update_tapp_resource(Origin::signed(user), tapp_id, cid.clone()),
			Error::<Test>::OnlyTAppOwnerAllowedToExpense
		);
	})
}

#[test]
fn topup_works() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let operation_account = 1;
		let user = 2;
		let tapp_owner = 3;
		let initial_amount = 100000000;
		<Test as Config>::Currency::make_free_balance_be(&operation_account, initial_amount);
		<Test as Config>::Currency::make_free_balance_be(&user, initial_amount);
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, initial_amount);

		assert_ok!(create_default_tapp(tapp_owner));
		let tapp_id = 1;

		let transfer_amount = 10000;
		assert_ok!(BondingCurve::topup(
			Origin::signed(user),
			tapp_id,
			operation_account,
			transfer_amount
		));
		assert_eq!(
			<Test as Config>::Currency::free_balance(&operation_account),
			initial_amount + transfer_amount
		);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&user),
			initial_amount - transfer_amount
		);
	})
}

#[test]
fn topup_should_fail_if_tapp_not_exist() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			BondingCurve::topup(Origin::signed(2), 1, 3, 1000),
			Error::<Test>::TAppIdNotExist
		);
	})
}

#[test]
fn topup_should_fail_if_user_balance_is_not_enough() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let operation_account = 1;
		let user = 2;
		let tapp_owner = 3;
		let initial_amount = 100000000;
		<Test as Config>::Currency::make_free_balance_be(&operation_account, initial_amount);
		<Test as Config>::Currency::make_free_balance_be(&user, initial_amount);
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, initial_amount);

		assert_ok!(create_default_tapp(tapp_owner));
		let tapp_id = 1;

		let transfer_amount = 200000000;
		assert_noop!(
			BondingCurve::topup(
				Origin::signed(user),
				tapp_id,
				operation_account,
				transfer_amount
			),
			Error::<Test>::InsufficientFreeBalance
		);
	})
}

#[test]
fn withdraw_works() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let operation_account = 1;
		let user = 2;
		let tapp_owner = 3;
		let initial_amount = 100000000;
		<Test as Config>::Currency::make_free_balance_be(&operation_account, 0);
		<Test as Config>::Currency::make_free_balance_be(&user, initial_amount);
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, initial_amount);

		assert_ok!(create_default_tapp(tapp_owner));
		let tapp_id = 1;

		let transfer_amount = 10000;
		assert_ok!(BondingCurve::topup(
			Origin::signed(user),
			tapp_id,
			operation_account,
			transfer_amount
		));
		assert_eq!(
			<Test as Config>::Currency::free_balance(&operation_account),
			transfer_amount
		);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&user),
			initial_amount - transfer_amount
		);

		assert_ok!(BondingCurve::withdraw(
			Origin::signed(operation_account),
			user,
			transfer_amount,
			tapp_id,
			b"test tsid".to_vec()
		));
		assert_eq!(
			<Test as Config>::Currency::free_balance(&operation_account),
			0
		);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&user),
			initial_amount
		);
	})
}

#[test]
fn withdraw_should_fail_if_tsid_used_same_times() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let operation_account = 1;
		let user = 2;
		let tapp_owner = 3;
		let initial_amount = 100000000;
		<Test as Config>::Currency::make_free_balance_be(&operation_account, 0);
		<Test as Config>::Currency::make_free_balance_be(&user, initial_amount);
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, initial_amount);

		assert_ok!(create_default_tapp(tapp_owner));
		let tapp_id = 1;

		let transfer_amount = 10000;
		assert_ok!(BondingCurve::topup(
			Origin::signed(user),
			tapp_id,
			operation_account,
			transfer_amount
		));

		let tsid = b"test tsid".to_vec();
		assert_ok!(BondingCurve::withdraw(
			Origin::signed(operation_account),
			user,
			1,
			tapp_id,
			tsid.clone(),
		));

		assert_noop!(
			BondingCurve::withdraw(Origin::signed(operation_account), user, 1, tapp_id, tsid),
			Error::<Test>::WithdrawTsidAlreadyExist
		);
	})
}

#[test]
fn withdraw_should_fail_if_tapp_not_exist() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let operation_account = 1;
		let user = 2;
		let initial_amount = 100000000;
		<Test as Config>::Currency::make_free_balance_be(&operation_account, 0);
		<Test as Config>::Currency::make_free_balance_be(&user, initial_amount);

		let tapp_id = 1;

		let tsid = b"test tsid".to_vec();
		assert_noop!(
			BondingCurve::withdraw(
				Origin::signed(operation_account),
				user,
				1,
				tapp_id,
				tsid.clone(),
			),
			Error::<Test>::TAppIdNotExist
		);
	})
}

#[test]
fn withdraw_should_fail_if_insufficient_free_balance() {
	new_test_ext().execute_with(|| {
		EnableUserCreateTApp::<Test>::set(true);
		let operation_account = 1;
		let user = 2;
		let tapp_owner = 3;
		let initial_amount = 100000000;
		<Test as Config>::Currency::make_free_balance_be(&operation_account, 0);
		<Test as Config>::Currency::make_free_balance_be(&user, initial_amount);
		<Test as Config>::Currency::make_free_balance_be(&tapp_owner, initial_amount);

		assert_ok!(create_default_tapp(tapp_owner));
		let tapp_id = 1;

		let transfer_amount = 10000;
		assert_ok!(BondingCurve::topup(
			Origin::signed(user),
			tapp_id,
			operation_account,
			transfer_amount
		));

		assert_noop!(
			BondingCurve::withdraw(
				Origin::signed(operation_account),
				user,
				transfer_amount + 1,
				tapp_id,
				b"test tsid".to_vec()
			),
			Error::<Test>::InsufficientFreeBalance
		);
	})
}

#[test]
fn push_notifications_works() {
	new_test_ext().execute_with(|| {
		let current_height = 10;
		frame_system::Pallet::<Test>::set_block_number(current_height);

		let user1 = 1;
		let user2 = 2;
		let notification_account = 3;
		NotificationAccount::<Test>::set(Some(notification_account));

		let tapp_id = 1;
		let expired_height1 = 50;
		let expired_height2 = 80;
		assert_ok!(BondingCurve::push_notifications(
			Origin::signed(notification_account),
			vec![user1, user2],
			vec![expired_height1, expired_height2],
			tapp_id,
			b"test tsid".to_vec(),
		));

		assert_eq!(UserNotifications::<Test>::get(user1).len(), 1);
		assert_eq!(
			UserNotifications::<Test>::get(user1)[0].expired_height,
			expired_height1
		);
		assert_eq!(
			UserNotifications::<Test>::get(user1)[0].start_height,
			current_height
		);
		assert_eq!(UserNotifications::<Test>::get(user1)[0].tapp_id, tapp_id);
		assert!(!has_paid::<Test>(&UserNotifications::<Test>::get(user1)[0]));

		assert_eq!(UserNotifications::<Test>::get(user2).len(), 1);
		assert_eq!(
			UserNotifications::<Test>::get(user2)[0].expired_height,
			expired_height2
		);
		assert_eq!(
			UserNotifications::<Test>::get(user2)[0].start_height,
			current_height
		);
		assert_eq!(UserNotifications::<Test>::get(user2)[0].tapp_id, tapp_id);
		assert!(!has_paid::<Test>(&UserNotifications::<Test>::get(user2)[0]));
	})
}

#[test]
fn push_notifications_should_fail_if_tsid_exists() {
	new_test_ext().execute_with(|| {
		let user1 = 1;
		let user2 = 2;
		let notification_account = 3;
		NotificationAccount::<Test>::set(Some(notification_account));

		let tapp_id = 1;
		let expired_height1 = 50;
		let expired_height2 = 80;
		assert_ok!(BondingCurve::push_notifications(
			Origin::signed(notification_account),
			vec![user1, user2],
			vec![expired_height1, expired_height2],
			tapp_id,
			b"test tsid".to_vec(),
		));

		assert_noop!(
			BondingCurve::push_notifications(
				Origin::signed(notification_account),
				vec![user1, user2],
				vec![expired_height1, expired_height2],
				tapp_id,
				b"test tsid".to_vec(),
			),
			Error::<Test>::NotificationTsidAlreadyExist
		);
	})
}

#[test]
fn push_notifications_should_fail_if_notification_account_is_empty() {
	new_test_ext().execute_with(|| {
		let user1 = 1;

		assert_noop!(
			BondingCurve::push_notifications(Origin::signed(user1), vec![], vec![], 1, vec![]),
			Error::<Test>::NotificationAccountNotInit
		);
	})
}

#[test]
fn push_notifications_should_fail_if_committer_not_notification_account() {
	new_test_ext().execute_with(|| {
		let user1 = 1;
		let notification_account = 3;
		NotificationAccount::<Test>::set(Some(notification_account));

		assert_noop!(
			BondingCurve::push_notifications(Origin::signed(user1), vec![], vec![], 1, vec![]),
			Error::<Test>::NotAllowedPushNotification
		);
	})
}

#[test]
fn push_notifications_should_fail_if_notification_and_account_list_not_matched() {
	new_test_ext().execute_with(|| {
		let user1 = 1;
		let user2 = 2;
		let notification_account = 3;
		NotificationAccount::<Test>::set(Some(notification_account));

		assert_noop!(
			BondingCurve::push_notifications(
				Origin::signed(notification_account),
				vec![user1, user2],
				vec![11],
				1,
				vec![]
			),
			Error::<Test>::NotificationAndAccountListNotMatched
		);
	})
}

#[test]
fn push_notifications_should_fail_if_account_list_is_empty() {
	new_test_ext().execute_with(|| {
		let notification_account = 3;
		NotificationAccount::<Test>::set(Some(notification_account));

		assert_noop!(
			BondingCurve::push_notifications(
				Origin::signed(notification_account),
				vec![],
				vec![],
				1,
				vec![]
			),
			Error::<Test>::NotificationListIsEmpty
		);
	})
}

#[test]
fn clear_notifications_works() {
	new_test_ext().execute_with(|| {
		let current_height = 10;
		frame_system::Pallet::<Test>::set_block_number(current_height);

		let user1 = 1;
		let user2 = 2;
		let notification_account = 3;
		NotificationAccount::<Test>::set(Some(notification_account));

		let tapp_id = 1;
		let expired_height1 = 50;
		let expired_height2 = 80;
		assert_ok!(BondingCurve::push_notifications(
			Origin::signed(notification_account),
			vec![user1, user2],
			vec![expired_height1, expired_height2],
			tapp_id,
			b"test tsid".to_vec(),
		));
		assert_ok!(BondingCurve::push_notifications(
			Origin::signed(notification_account),
			vec![user1, user2],
			vec![expired_height1, expired_height2],
			2,
			b"test tsid2".to_vec(),
		));
		assert_eq!(UserNotifications::<Test>::get(user1).len(), 2);
		assert_eq!(UserNotifications::<Test>::get(user2).len(), 2);

		let current_height2 = 60;
		frame_system::Pallet::<Test>::set_block_number(current_height2);

		assert_ok!(BondingCurve::clear_notifications(
			Origin::signed(notification_account),
			current_height2,
			b"test tsid3".to_vec(),
		));

		assert_eq!(UserNotifications::<Test>::get(user1).len(), 0);
		assert_eq!(UserNotifications::<Test>::get(user2).len(), 2);
		assert!(has_paid::<Test>(&UserNotifications::<Test>::get(user2)[0]));
		assert!(has_paid::<Test>(&UserNotifications::<Test>::get(user2)[1]));
	})
}

#[test]
fn clear_notifications_should_fail_if_notification_account_is_empty() {
	new_test_ext().execute_with(|| {
		let current_height = 10;
		frame_system::Pallet::<Test>::set_block_number(current_height);

		assert_noop!(
			BondingCurve::clear_notifications(Origin::signed(1), 1, b"test tsid".to_vec(),),
			Error::<Test>::NotificationAccountNotInit
		);
	})
}

#[test]
fn clear_notifications_should_fail_if_not_notification_account() {
	new_test_ext().execute_with(|| {
		let current_height = 10;
		frame_system::Pallet::<Test>::set_block_number(current_height);

		let user1 = 1;
		let user2 = 2;
		let notification_account = 3;
		NotificationAccount::<Test>::set(Some(notification_account));

		let tapp_id = 1;
		let expired_height1 = 50;
		let expired_height2 = 80;
		assert_ok!(BondingCurve::push_notifications(
			Origin::signed(notification_account),
			vec![user1, user2],
			vec![expired_height1, expired_height2],
			tapp_id,
			b"test tsid".to_vec(),
		));

		let tapp1_pay_account = 5;
		assert_noop!(
			BondingCurve::clear_notifications(
				Origin::signed(tapp1_pay_account),
				tapp_id,
				b"test tsid".to_vec(),
			),
			Error::<Test>::NotAllowedClearNotification
		);
	})
}

#[test]
fn clear_notifications_should_fail_if_tsid_exists() {
	new_test_ext().execute_with(|| {
		let current_height = 10;
		frame_system::Pallet::<Test>::set_block_number(current_height);

		let user1 = 1;
		let user2 = 2;
		let notification_account = 3;
		NotificationAccount::<Test>::set(Some(notification_account));

		let tapp_id = 1;
		let expired_height1 = 50;
		let expired_height2 = 80;
		assert_ok!(BondingCurve::push_notifications(
			Origin::signed(notification_account),
			vec![user1, user2],
			vec![expired_height1, expired_height2],
			tapp_id,
			b"test tsid".to_vec(),
		));

		assert_ok!(BondingCurve::clear_notifications(
			Origin::signed(notification_account),
			30,
			b"test tsid".to_vec(),
		));
		assert_noop!(
			BondingCurve::clear_notifications(
				Origin::signed(notification_account),
				30,
				b"test tsid".to_vec(),
			),
			Error::<Test>::NotificationTsidAlreadyExist
		);
	})
}

#[test]
fn clear_notifications_should_fail_if_stop_height_is_invalid() {
	new_test_ext().execute_with(|| {
		let current_height = 10;
		frame_system::Pallet::<Test>::set_block_number(current_height);

		let user1 = 1;
		let user2 = 2;
		let notification_account = 3;
		NotificationAccount::<Test>::set(Some(notification_account));

		let tapp_id = 1;
		let expired_height1 = 50;
		let expired_height2 = 80;
		assert_ok!(BondingCurve::push_notifications(
			Origin::signed(notification_account),
			vec![user1, user2],
			vec![expired_height1, expired_height2],
			tapp_id,
			b"test tsid".to_vec(),
		));

		NotificationsLastPayHeight::<Test>::set(40);
		assert_noop!(
			BondingCurve::clear_notifications(
				Origin::signed(notification_account),
				30,
				b"test tsid".to_vec(),
			),
			Error::<Test>::InvalidClearNotificationHeight
		);
	})
}

#[test]
fn npc_mint_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(
			<Test as Config>::Currency::free_balance(&NPCAccount::<Test>::get()),
			0
		);

		let amount = 1000;
		assert_ok!(BondingCurve::npc_mint(
			Origin::signed(NPCAccount::<Test>::get()),
			amount
		));
		assert_eq!(
			<Test as Config>::Currency::free_balance(&NPCAccount::<Test>::get()),
			amount
		);

		let amount2 = 2000;
		assert_ok!(BondingCurve::npc_mint(
			Origin::signed(NPCAccount::<Test>::get()),
			amount2
		));
		assert_eq!(
			<Test as Config>::Currency::free_balance(&NPCAccount::<Test>::get()),
			amount + amount2
		);
	})
}

#[test]
fn npc_mint_should_fail_if_not_npc() {
	new_test_ext().execute_with(|| {
		let amount = 1000;
		assert_ok!(BondingCurve::npc_mint(
			Origin::signed(NPCAccount::<Test>::get()),
			amount
		));
	})
}

#[test]
fn batch_transfer_works() {
	new_test_ext().execute_with(|| {
		let total_amount = 1000;
		assert_ok!(BondingCurve::npc_mint(
			Origin::signed(NPCAccount::<Test>::get()),
			total_amount
		));

		let user1 = 1;
		let user2 = 2;
		let user3 = 3;
		assert_ok!(BondingCurve::batch_transfer(
			Origin::signed(NPCAccount::<Test>::get()),
			vec![user1, user2, user3],
			10,
		));

		assert_eq!(
			<Test as Config>::Currency::free_balance(&NPCAccount::<Test>::get()),
			total_amount - 10 * 3
		);
		assert_eq!(<Test as Config>::Currency::free_balance(&user1), 10);
		assert_eq!(<Test as Config>::Currency::free_balance(&user2), 10);
		assert_eq!(<Test as Config>::Currency::free_balance(&user3), 10);

		assert_ok!(BondingCurve::batch_transfer(
			Origin::signed(NPCAccount::<Test>::get()),
			vec![user1, user2],
			100,
		));

		assert_eq!(
			<Test as Config>::Currency::free_balance(&NPCAccount::<Test>::get()),
			total_amount - 10 * 3 - 100 * 2
		);
		assert_eq!(<Test as Config>::Currency::free_balance(&user1), 10 + 100);
		assert_eq!(<Test as Config>::Currency::free_balance(&user2), 10 + 100);
		assert_eq!(<Test as Config>::Currency::free_balance(&user3), 10);
	})
}

#[test]
fn batch_transfer_should_fail_if_not_npc_account() {
	new_test_ext().execute_with(|| {
		assert_ok!(BondingCurve::npc_mint(
			Origin::signed(NPCAccount::<Test>::get()),
			10
		));

		let user1 = 1;
		let user2 = 2;
		let user3 = 3;
		assert_noop!(
			BondingCurve::batch_transfer(
				Origin::signed(NPCAccount::<Test>::get()),
				vec![user1, user2, user3],
				10,
			),
			Error::<Test>::BatchTransferInsufficientBalance
		);
	})
}

#[test]
fn batch_transfer_should_not_enough_free_balance() {
	new_test_ext().execute_with(|| {
		let user1 = 1;
		let user2 = 2;
		let user3 = 3;
		assert_noop!(
			BondingCurve::batch_transfer(Origin::signed(111), vec![user1, user2, user3], 10,),
			Error::<Test>::OnlyNpcCanBatchTransfer
		);
	})
}

pub fn has_paid<T>(item: &NotificationItem<T::BlockNumber>) -> bool
where
	T: Config,
{
	item.start_height < NotificationsLastPayHeight::<T>::get()
}

pub fn create_default_tapp(tapp_owner: u64) -> DispatchResult {
	let npc = NPCAccount::<Test>::get();
	let link = b"https://teaproject.org".to_vec();
	BondingCurve::register_tapp_link(
		Origin::signed(npc),
		link.clone(),
		"test description".into(),
		None,
	)?;

	BondingCurve::create_new_tapp(
		Origin::signed(tapp_owner),
		b"test name".to_vec(),
		b"tea".to_vec(),
		1_000_000,
		b"test detail".to_vec(),
		link,
		10,
		TAppType::Twitter,
		true,
		None,
		Some(1000),
		None,
		None,
	)
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
