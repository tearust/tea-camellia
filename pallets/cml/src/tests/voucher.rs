use crate::{mock::*, types::*, Error, InvestorVoucherStore, TeamVoucherStore};
use frame_support::{assert_noop, assert_ok};

#[test]
fn transfer_voucher_works() {
	new_test_ext().execute_with(|| {
		TeamVoucherStore::<Test>::insert(1, CmlType::A, new_voucher(10, CmlType::A));
		TeamVoucherStore::<Test>::insert(2, CmlType::A, new_voucher(10, CmlType::A));
		TeamVoucherStore::<Test>::insert(1, CmlType::B, new_voucher(10, CmlType::B));
		TeamVoucherStore::<Test>::insert(2, CmlType::B, new_voucher(10, CmlType::B));
		TeamVoucherStore::<Test>::insert(1, CmlType::C, new_voucher(10, CmlType::C));
		TeamVoucherStore::<Test>::insert(2, CmlType::C, new_voucher(10, CmlType::C));

		assert_ok!(Cml::transfer_voucher(
			Origin::signed(1),
			2,
			CmlType::A,
			DefrostScheduleType::Team,
			3
		));
		assert_ok!(Cml::transfer_voucher(
			Origin::signed(1),
			2,
			CmlType::B,
			DefrostScheduleType::Team,
			4
		));
		assert_ok!(Cml::transfer_voucher(
			Origin::signed(1),
			2,
			CmlType::C,
			DefrostScheduleType::Team,
			5
		));

		assert_eq!(
			TeamVoucherStore::<Test>::get(1, CmlType::A).unwrap().amount,
			7
		);
		assert_eq!(
			TeamVoucherStore::<Test>::get(2, CmlType::A).unwrap().amount,
			13
		);

		assert_eq!(
			TeamVoucherStore::<Test>::get(1, CmlType::B).unwrap().amount,
			6
		);
		assert_eq!(
			TeamVoucherStore::<Test>::get(2, CmlType::B).unwrap().amount,
			14
		);

		assert_eq!(
			TeamVoucherStore::<Test>::get(1, CmlType::C).unwrap().amount,
			5
		);
		assert_eq!(
			TeamVoucherStore::<Test>::get(2, CmlType::C).unwrap().amount,
			15
		);
	})
}

#[test]
fn transfer_voucher_to_not_exist_account_works() {
	new_test_ext().execute_with(|| {
		InvestorVoucherStore::<Test>::insert(1, CmlType::A, new_voucher(10, CmlType::A));

		assert!(InvestorVoucherStore::<Test>::get(2, CmlType::A).is_none());
		assert_ok!(Cml::transfer_voucher(
			Origin::signed(1),
			2,
			CmlType::A,
			DefrostScheduleType::Investor,
			3
		));

		assert_eq!(
			InvestorVoucherStore::<Test>::get(1, CmlType::A)
				.unwrap()
				.amount,
			7
		);
		assert_eq!(
			InvestorVoucherStore::<Test>::get(2, CmlType::A)
				.unwrap()
				.amount,
			3
		);
	})
}

#[test]
fn transfer_voucher_when_timeout_should_fail() {
	new_test_ext().execute_with(|| {
		frame_system::Pallet::<Test>::set_block_number(SEEDS_TIMEOUT_HEIGHT as u64 + 1);

		InvestorVoucherStore::<Test>::insert(1, CmlType::A, new_voucher(10, CmlType::A));
		assert_noop!(
			Cml::transfer_voucher(
				Origin::signed(1),
				2,
				CmlType::A,
				DefrostScheduleType::Investor,
				3
			),
			Error::<Test>::VouchersHasOutdated
		);
	})
}

#[test]
fn transfer_voucher_with_insufficient_amount_should_fail() {
	new_test_ext().execute_with(|| {
		TeamVoucherStore::<Test>::insert(1, CmlType::A, new_voucher(10, CmlType::A));

		assert_noop!(
			Cml::transfer_voucher(
				Origin::signed(1),
				2,
				CmlType::A,
				DefrostScheduleType::Team,
				11
			),
			Error::<Test>::NotEnoughVoucher
		);
	})
}

#[test]
fn transfer_voucher_from_not_existing_account_should_fail() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Cml::transfer_voucher(
				Origin::signed(1),
				2,
				CmlType::A,
				DefrostScheduleType::Team,
				1
			),
			Error::<Test>::NotEnoughVoucher
		);
	})
}

#[test]
fn transfer_voucher_to_cause_to_amount_overflow() {
	new_test_ext().execute_with(|| {
		TeamVoucherStore::<Test>::insert(1, CmlType::A, new_voucher(10, CmlType::A));
		TeamVoucherStore::<Test>::insert(2, CmlType::A, new_voucher(u32::MAX, CmlType::A));

		assert_noop!(
			Cml::transfer_voucher(
				Origin::signed(1),
				2,
				CmlType::A,
				DefrostScheduleType::Team,
				3
			),
			Error::<Test>::InvalidVoucherAmount
		);
	})
}

pub fn new_voucher(amount: u32, cml_type: CmlType) -> Voucher {
	Voucher { amount, cml_type }
}
