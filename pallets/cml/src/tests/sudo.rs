use crate::tests::new_genesis_seed;
use crate::{mock::*, types::*, CmlStore, Error, LuckyDrawBox};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError};

#[test]
fn clean_outdated_seeds_works() {
	new_test_ext().execute_with(|| {
		frame_system::Pallet::<Test>::set_block_number(SEEDS_TIMEOUT_HEIGHT as u64 + 1);

		LuckyDrawBox::<Test>::insert(CmlType::A, DefrostScheduleType::Investor, vec![11]);
		LuckyDrawBox::<Test>::insert(CmlType::B, DefrostScheduleType::Team, vec![21]);
		LuckyDrawBox::<Test>::insert(CmlType::C, DefrostScheduleType::Investor, vec![31]);
		CmlStore::<Test>::insert(11, CML::from_genesis_seed(new_genesis_seed(11)));
		CmlStore::<Test>::insert(12, CML::from_genesis_seed(new_genesis_seed(12)));
		CmlStore::<Test>::insert(21, CML::from_genesis_seed(new_genesis_seed(21)));
		CmlStore::<Test>::insert(22, CML::from_genesis_seed(new_genesis_seed(22)));
		CmlStore::<Test>::insert(31, CML::from_genesis_seed(new_genesis_seed(31)));
		CmlStore::<Test>::insert(32, CML::from_genesis_seed(new_genesis_seed(32)));

		assert_ok!(Cml::clean_outdated_seeds(Origin::root()));

		assert!(LuckyDrawBox::<Test>::get(CmlType::A, DefrostScheduleType::Investor).is_empty());
		assert!(LuckyDrawBox::<Test>::get(CmlType::B, DefrostScheduleType::Team).is_empty());
		assert!(LuckyDrawBox::<Test>::get(CmlType::C, DefrostScheduleType::Investor).is_empty());
		assert!(!CmlStore::<Test>::contains_key(11));
		assert!(!CmlStore::<Test>::contains_key(21));
		assert!(!CmlStore::<Test>::contains_key(31));

		assert!(CmlStore::<Test>::contains_key(12));
		assert!(CmlStore::<Test>::contains_key(22));
		assert!(CmlStore::<Test>::contains_key(32));
	})
}

#[test]
fn no_root_user_clean_outdated_seeds_should_fail() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Cml::clean_outdated_seeds(Origin::signed(1)),
			DispatchError::BadOrigin
		);
	})
}

#[test]
fn clean_should_fail_when_not_outdated() {
	new_test_ext().execute_with(|| {
		frame_system::Pallet::<Test>::set_block_number(SEEDS_TIMEOUT_HEIGHT as u64);

		assert_noop!(
			Cml::clean_outdated_seeds(Origin::root()),
			Error::<Test>::SeedsNotOutdatedYet
		);
	})
}

#[test]
fn clean_should_fail_when_there_is_no_need_to_clean() {
	new_test_ext().execute_with(|| {
		frame_system::Pallet::<Test>::set_block_number(SEEDS_TIMEOUT_HEIGHT as u64 + 1);

		assert!(Cml::lucky_draw_box_all_empty(vec![
			DefrostScheduleType::Investor,
			DefrostScheduleType::Team
		]));
		assert_noop!(
			Cml::clean_outdated_seeds(Origin::root()),
			Error::<Test>::NoNeedToCleanOutdatedSeeds
		);
	})
}
