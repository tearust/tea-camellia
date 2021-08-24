use crate::tests::coupon::new_coupon;
use crate::{
	mock::*, types::*, EnableTransferCoupon, Error, Event as CmlEvent, InvestorCouponStore,
	LuckyDrawBox, TeamCouponStore, UserCmlStore,
};
use frame_support::{assert_noop, assert_ok};

#[test]
fn draw_cmls_from_coupon_works() {
	new_test_ext().execute_with(|| {
		frame_system::Pallet::<Test>::set_block_number(100);

		let origin_a_box: Vec<u64> = (1..=10).collect();
		let origin_b_box: Vec<u64> = (11..=20).collect();
		let origin_c_box: Vec<u64> = (21..=30).collect();

		LuckyDrawBox::<Test>::insert(CmlType::A, DefrostScheduleType::Team, origin_a_box.clone());
		LuckyDrawBox::<Test>::insert(CmlType::B, DefrostScheduleType::Team, origin_b_box.clone());
		LuckyDrawBox::<Test>::insert(CmlType::C, DefrostScheduleType::Team, origin_c_box.clone());

		TeamCouponStore::<Test>::insert(1, CmlType::A, new_coupon(3, CmlType::A));
		TeamCouponStore::<Test>::insert(1, CmlType::B, new_coupon(4, CmlType::B));
		TeamCouponStore::<Test>::insert(1, CmlType::C, new_coupon(5, CmlType::C));

		assert_ok!(Cml::draw_cmls_from_coupon(
			Origin::signed(1),
			DefrostScheduleType::Team
		));

		assert_eq!(
			UserCmlStore::<Test>::iter()
				.filter(|(k1, _, _)| *k1 == 1)
				.count(),
			3 + 4 + 5
		);
		System::assert_last_event(Event::pallet_cml(CmlEvent::DrawCmls(1, 3 + 4 + 5)));
	})
}

#[test]
fn draw_cmls_works_the_second_time_if_get_coupon_again() {
	new_test_ext().execute_with(|| {
		frame_system::Pallet::<Test>::set_block_number(100);
		EnableTransferCoupon::<Test>::set(true);

		let origin_a_box: Vec<u64> = (1..=10).collect();
		let origin_b_box: Vec<u64> = (11..=20).collect();
		let origin_c_box: Vec<u64> = (21..=30).collect();

		LuckyDrawBox::<Test>::insert(
			CmlType::A,
			DefrostScheduleType::Investor,
			origin_a_box.clone(),
		);
		LuckyDrawBox::<Test>::insert(
			CmlType::B,
			DefrostScheduleType::Investor,
			origin_b_box.clone(),
		);
		LuckyDrawBox::<Test>::insert(
			CmlType::C,
			DefrostScheduleType::Investor,
			origin_c_box.clone(),
		);

		InvestorCouponStore::<Test>::insert(1, CmlType::A, new_coupon(3, CmlType::A));
		InvestorCouponStore::<Test>::insert(1, CmlType::B, new_coupon(4, CmlType::B));
		InvestorCouponStore::<Test>::insert(1, CmlType::C, new_coupon(5, CmlType::C));
		InvestorCouponStore::<Test>::insert(2, CmlType::A, new_coupon(1, CmlType::A));
		InvestorCouponStore::<Test>::insert(2, CmlType::B, new_coupon(2, CmlType::B));
		InvestorCouponStore::<Test>::insert(2, CmlType::C, new_coupon(3, CmlType::C));

		assert_ok!(Cml::draw_cmls_from_coupon(
			Origin::signed(1),
			DefrostScheduleType::Investor
		));
		assert_eq!(
			UserCmlStore::<Test>::iter()
				.filter(|(k1, _, _)| *k1 == 1)
				.count(),
			3 + 4 + 5
		);

		assert_ok!(Cml::transfer_coupon(
			Origin::signed(2),
			1,
			CmlType::A,
			DefrostScheduleType::Investor,
			1
		));
		assert_ok!(Cml::transfer_coupon(
			Origin::signed(2),
			1,
			CmlType::B,
			DefrostScheduleType::Investor,
			2
		));
		assert_ok!(Cml::transfer_coupon(
			Origin::signed(2),
			1,
			CmlType::C,
			DefrostScheduleType::Investor,
			3
		));

		assert_ok!(Cml::draw_cmls_from_coupon(
			Origin::signed(1),
			DefrostScheduleType::Investor
		));
		assert_eq!(
			UserCmlStore::<Test>::iter()
				.filter(|(k1, _, _)| *k1 == 1)
				.count(),
			3 + 4 + 5 + 1 + 2 + 3
		);
	})
}

#[test]
fn draw_cmls_from_coupon_should_fail_if_timeout() {
	new_test_ext().execute_with(|| {
		frame_system::Pallet::<Test>::set_block_number(SEEDS_TIMEOUT_HEIGHT as u64 + 1);
		assert_noop!(
			Cml::draw_cmls_from_coupon(Origin::signed(1), DefrostScheduleType::Team),
			Error::<Test>::CouponsHasOutdated
		);
		assert_noop!(
			Cml::draw_cmls_from_coupon(Origin::signed(1), DefrostScheduleType::Investor),
			Error::<Test>::CouponsHasOutdated
		);
	})
}

#[test]
fn draw_same_cmls_multiple_times_should_fail() {
	new_test_ext().execute_with(|| {
		frame_system::Pallet::<Test>::set_block_number(100);

		let origin_a_box: Vec<u64> = (1..=10).collect();
		let origin_b_box: Vec<u64> = (11..=20).collect();
		let origin_c_box: Vec<u64> = (21..=30).collect();

		LuckyDrawBox::<Test>::insert(CmlType::A, DefrostScheduleType::Team, origin_a_box.clone());
		LuckyDrawBox::<Test>::insert(CmlType::B, DefrostScheduleType::Team, origin_b_box.clone());
		LuckyDrawBox::<Test>::insert(CmlType::C, DefrostScheduleType::Team, origin_c_box.clone());

		TeamCouponStore::<Test>::insert(1, CmlType::A, new_coupon(3, CmlType::A));
		TeamCouponStore::<Test>::insert(1, CmlType::B, new_coupon(4, CmlType::B));
		TeamCouponStore::<Test>::insert(1, CmlType::C, new_coupon(5, CmlType::C));

		assert_ok!(Cml::draw_cmls_from_coupon(
			Origin::signed(1),
			DefrostScheduleType::Team
		));
		assert_noop!(
			Cml::draw_cmls_from_coupon(Origin::signed(1), DefrostScheduleType::Team),
			Error::<Test>::WithoutCoupon
		);
	})
}

#[test]
fn draw_cmls_should_fail_when_no_coupon_exist() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Cml::draw_cmls_from_coupon(Origin::signed(1), DefrostScheduleType::Team),
			Error::<Test>::WithoutCoupon
		);
		assert_noop!(
			Cml::draw_cmls_from_coupon(Origin::signed(1), DefrostScheduleType::Investor),
			Error::<Test>::WithoutCoupon
		);
	})
}
