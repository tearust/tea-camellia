use crate::{mock::*, types::*, EnableTransferCoupon, Error, InvestorCouponStore, TeamCouponStore};
use frame_support::{assert_noop, assert_ok};

#[test]
fn transfer_coupon_works() {
	new_test_ext().execute_with(|| {
		EnableTransferCoupon::<Test>::set(true);

		TeamCouponStore::<Test>::insert(1, CmlType::A, new_coupon(10, CmlType::A));
		TeamCouponStore::<Test>::insert(2, CmlType::A, new_coupon(10, CmlType::A));
		TeamCouponStore::<Test>::insert(1, CmlType::B, new_coupon(10, CmlType::B));
		TeamCouponStore::<Test>::insert(2, CmlType::B, new_coupon(10, CmlType::B));
		TeamCouponStore::<Test>::insert(1, CmlType::C, new_coupon(10, CmlType::C));
		TeamCouponStore::<Test>::insert(2, CmlType::C, new_coupon(10, CmlType::C));

		assert_ok!(Cml::transfer_coupon(
			Origin::signed(1),
			2,
			CmlType::A,
			DefrostScheduleType::Team,
			3
		));
		assert_ok!(Cml::transfer_coupon(
			Origin::signed(1),
			2,
			CmlType::B,
			DefrostScheduleType::Team,
			4
		));
		assert_ok!(Cml::transfer_coupon(
			Origin::signed(1),
			2,
			CmlType::C,
			DefrostScheduleType::Team,
			5
		));

		assert_eq!(
			TeamCouponStore::<Test>::get(1, CmlType::A).unwrap().amount,
			7
		);
		assert_eq!(
			TeamCouponStore::<Test>::get(2, CmlType::A).unwrap().amount,
			13
		);

		assert_eq!(
			TeamCouponStore::<Test>::get(1, CmlType::B).unwrap().amount,
			6
		);
		assert_eq!(
			TeamCouponStore::<Test>::get(2, CmlType::B).unwrap().amount,
			14
		);

		assert_eq!(
			TeamCouponStore::<Test>::get(1, CmlType::C).unwrap().amount,
			5
		);
		assert_eq!(
			TeamCouponStore::<Test>::get(2, CmlType::C).unwrap().amount,
			15
		);
	})
}

#[test]
fn transfer_coupon_to_not_exist_account_works() {
	new_test_ext().execute_with(|| {
		EnableTransferCoupon::<Test>::set(true);
		InvestorCouponStore::<Test>::insert(1, CmlType::A, new_coupon(10, CmlType::A));

		assert!(InvestorCouponStore::<Test>::get(2, CmlType::A).is_none());
		assert_ok!(Cml::transfer_coupon(
			Origin::signed(1),
			2,
			CmlType::A,
			DefrostScheduleType::Investor,
			3
		));

		assert_eq!(
			InvestorCouponStore::<Test>::get(1, CmlType::A)
				.unwrap()
				.amount,
			7
		);
		assert_eq!(
			InvestorCouponStore::<Test>::get(2, CmlType::A)
				.unwrap()
				.amount,
			3
		);
	})
}

#[test]
fn transfer_coupon_should_fail_if_enable_transfer_coupon_is_false() {
	new_test_ext().execute_with(|| {
		EnableTransferCoupon::<Test>::set(false);
		InvestorCouponStore::<Test>::insert(1, CmlType::A, new_coupon(10, CmlType::A));
		assert_noop!(
			Cml::transfer_coupon(
				Origin::signed(1),
				2,
				CmlType::A,
				DefrostScheduleType::Investor,
				3
			),
			Error::<Test>::ForbiddenTransferCoupon
		);
	})
}

#[test]
fn transfer_coupon_when_timeout_should_fail() {
	new_test_ext().execute_with(|| {
		EnableTransferCoupon::<Test>::set(true);
		frame_system::Pallet::<Test>::set_block_number(SEEDS_TIMEOUT_HEIGHT as u64 + 1);

		InvestorCouponStore::<Test>::insert(1, CmlType::A, new_coupon(10, CmlType::A));
		assert_noop!(
			Cml::transfer_coupon(
				Origin::signed(1),
				2,
				CmlType::A,
				DefrostScheduleType::Investor,
				3
			),
			Error::<Test>::CouponsHasOutdated
		);
	})
}

#[test]
fn transfer_coupon_with_insufficient_amount_should_fail() {
	new_test_ext().execute_with(|| {
		EnableTransferCoupon::<Test>::set(true);
		TeamCouponStore::<Test>::insert(1, CmlType::A, new_coupon(10, CmlType::A));

		assert_noop!(
			Cml::transfer_coupon(
				Origin::signed(1),
				2,
				CmlType::A,
				DefrostScheduleType::Team,
				11
			),
			Error::<Test>::NotEnoughCoupon
		);
	})
}

#[test]
fn transfer_coupon_from_not_existing_account_should_fail() {
	new_test_ext().execute_with(|| {
		EnableTransferCoupon::<Test>::set(true);
		assert_noop!(
			Cml::transfer_coupon(
				Origin::signed(1),
				2,
				CmlType::A,
				DefrostScheduleType::Team,
				1
			),
			Error::<Test>::NotEnoughCoupon
		);
	})
}

#[test]
fn transfer_coupon_to_cause_to_amount_overflow() {
	new_test_ext().execute_with(|| {
		EnableTransferCoupon::<Test>::set(true);
		TeamCouponStore::<Test>::insert(1, CmlType::A, new_coupon(10, CmlType::A));
		TeamCouponStore::<Test>::insert(2, CmlType::A, new_coupon(u32::MAX, CmlType::A));

		assert_noop!(
			Cml::transfer_coupon(
				Origin::signed(1),
				2,
				CmlType::A,
				DefrostScheduleType::Team,
				3
			),
			Error::<Test>::InvalidCouponAmount
		);
	})
}

pub fn new_coupon(amount: u32, cml_type: CmlType) -> Coupon {
	Coupon { amount, cml_type }
}
