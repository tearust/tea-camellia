use crate::mock::*;
use crate::*;
use frame_support::{assert_noop, assert_ok};
use pallet_cml::{CmlId, CmlType, DefrostScheduleType, Error as CmlError, Seed, CML};

#[test]
fn apply_loan_genesis_bank_works() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let cml_id: CmlId = 4;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		Cml::add_cml(&user, cml);

		let current_height = 100;
		frame_system::Pallet::<Test>::set_block_number(current_height);

		assert_eq!(<Test as Config>::Currency::free_balance(user), 0);
		assert_ok!(GenesisBank::apply_loan_genesis_bank(
			Origin::signed(user),
			from_cml_id(cml_id),
			AssetType::CML
		));

		let unique_id = AssetUniqueId {
			asset_type: AssetType::CML,
			inner_id: from_cml_id(cml_id),
		};
		assert!(CollateralStore::<Test>::contains_key(&unique_id));
		let collateral = CollateralStore::<Test>::get(&unique_id);
		assert_eq!(collateral.start_at, current_height);
		assert_eq!(collateral.owner, user);

		assert!(UserCollateralStore::<Test>::contains_key(&user, &unique_id));
		assert_eq!(
			<Test as Config>::Currency::free_balance(user),
			CML_A_LOAN_AMOUNT
		);
	})
}

#[test]
fn apply_loan_genesis_bank_should_fail_if_cml_in_auction() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let cml = CML::from_genesis_seed(seed_from_lifespan(IN_AUCTION_CML_ID, 100));
		Cml::add_cml(&user, cml);

		let current_height = 100;
		frame_system::Pallet::<Test>::set_block_number(current_height);

		assert_noop!(
			GenesisBank::apply_loan_genesis_bank(
				Origin::signed(user),
				from_cml_id(IN_AUCTION_CML_ID),
				AssetType::CML
			),
			Error::<Test>::CannotPawnWhenCmlIsInAuction
		);
	})
}

#[test]
fn apply_load_failed_if_load_already_exist() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let cml_id: CmlId = 4;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		Cml::add_cml(&user, cml);

		assert_ok!(GenesisBank::apply_loan_genesis_bank(
			Origin::signed(user),
			from_cml_id(cml_id),
			AssetType::CML
		));

		assert_noop!(
			GenesisBank::apply_loan_genesis_bank(
				Origin::signed(user),
				from_cml_id(cml_id),
				AssetType::CML
			),
			Error::<Test>::LoanAlreadyExists
		);
	})
}

#[test]
fn apply_loan_should_fail_after_bank_closed() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let cml_id: CmlId = 4;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		Cml::add_cml(&user, cml);

		let close_height = 100;
		CloseHeight::<Test>::set(Some(close_height));

		frame_system::Pallet::<Test>::set_block_number(close_height);
		assert_noop!(
			GenesisBank::apply_loan_genesis_bank(
				Origin::signed(user),
				from_cml_id(cml_id),
				AssetType::CML
			),
			Error::<Test>::CannotApplyLoanAfterClosed
		);
	})
}

#[test]
fn apply_loan_should_fail_if_asset_id_not_valid() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			GenesisBank::apply_loan_genesis_bank(Origin::signed(1), vec![1], AssetType::CML),
			Error::<Test>::ConvertToCmlIdLengthMismatch
		);
	})
}

#[test]
fn apply_load_should_fail_if_cml_not_exist() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			GenesisBank::apply_loan_genesis_bank(Origin::signed(1), from_cml_id(2), AssetType::CML),
			CmlError::<Test>::NotFoundCML
		);
	})
}

#[test]
fn apply_load_should_fail_if_cml_not_belongs_to_user() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let cml_id: CmlId = 4;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		Cml::add_cml(&user, cml);

		assert_noop!(
			GenesisBank::apply_loan_genesis_bank(
				Origin::signed(2),
				from_cml_id(cml_id),
				AssetType::CML
			),
			CmlError::<Test>::CMLOwnerInvalid
		);
	})
}

#[test]
fn apply_loan_should_fail_if_cml_is_not_frozen_seed() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let cml_id: CmlId = 4;
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		cml.defrost(&0);
		Cml::add_cml(&user, cml);

		assert_noop!(
			GenesisBank::apply_loan_genesis_bank(
				Origin::signed(user),
				from_cml_id(cml_id),
				AssetType::CML
			),
			Error::<Test>::ShouldPawnFrozenSeed
		);
	})
}

#[test]
fn apply_loan_should_fail_if_cml_is_not_genesis_seed() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let cml_id: CmlId = 40000;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		assert!(!cml.is_from_genesis());
		Cml::add_cml(&user, cml);

		assert_noop!(
			GenesisBank::apply_loan_genesis_bank(
				Origin::signed(user),
				from_cml_id(cml_id),
				AssetType::CML
			),
			Error::<Test>::ShouldPawnGenesisSeed
		);
	})
}

#[test]
fn payoff_loan_works() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let cml_id: CmlId = 4;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		Cml::add_cml(&user, cml);

		assert_ok!(GenesisBank::apply_loan_genesis_bank(
			Origin::signed(user),
			from_cml_id(cml_id),
			AssetType::CML
		));
		assert_eq!(
			<Test as Config>::Currency::free_balance(&user),
			CML_A_LOAN_AMOUNT
		);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
			BANK_INITIAL_BALANCE - CML_A_LOAN_AMOUNT
		);

		let user_balance = CML_A_LOAN_AMOUNT * 2;
		<Test as Config>::Currency::make_free_balance_be(&user, user_balance);
		assert_eq!(
			<Test as Config>::Currency::total_issuance(),
			BANK_INITIAL_BALANCE + CML_A_LOAN_AMOUNT
		);

		assert_ok!(GenesisBank::payoff_loan(
			Origin::signed(user),
			from_cml_id(cml_id),
			AssetType::CML,
			false,
		));

		let unique_id = AssetUniqueId {
			asset_type: AssetType::CML,
			inner_id: from_cml_id(cml_id),
		};
		assert!(!CollateralStore::<Test>::contains_key(&unique_id));
		assert!(!UserCollateralStore::<Test>::contains_key(user, &unique_id));

		assert_eq!(
			<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
			BANK_INITIAL_BALANCE
		);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&user),
			CML_A_LOAN_AMOUNT - CML_A_LOAN_AMOUNT * BANK_INITIAL_INTEREST_RATE / 10000
		);
		assert_eq!(
			<Test as Config>::Currency::total_issuance(),
			BANK_INITIAL_BALANCE + CML_A_LOAN_AMOUNT
				- CML_A_LOAN_AMOUNT * BANK_INITIAL_INTEREST_RATE / 10000
		);
	})
}

#[test]
fn payoff_loan_should_fail_if_load_not_exist() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			GenesisBank::payoff_loan(Origin::signed(1), from_cml_id(2), AssetType::CML, false),
			Error::<Test>::LoanNotExists
		);
	})
}

#[test]
fn payoff_loan_should_fail_if_load_not_belongs_to_user() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let cml_id: CmlId = 4;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		Cml::add_cml(&user, cml);

		assert_ok!(GenesisBank::apply_loan_genesis_bank(
			Origin::signed(user),
			from_cml_id(cml_id),
			AssetType::CML
		));

		assert_noop!(
			GenesisBank::payoff_loan(
				Origin::signed(4),
				from_cml_id(cml_id),
				AssetType::CML,
				false
			),
			Error::<Test>::InvalidBorrower
		);
	})
}

#[test]
fn payoff_loan_should_fail_if_asset_id_invalid() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let invalid_asset_id = vec![1];
		let unique_id = AssetUniqueId {
			asset_type: AssetType::CML,
			inner_id: invalid_asset_id.clone(),
		};
		CollateralStore::<Test>::insert(&unique_id, Loan::default());
		UserCollateralStore::<Test>::insert(&user, &unique_id, ());

		assert_noop!(
			GenesisBank::payoff_loan(
				Origin::signed(user),
				invalid_asset_id,
				AssetType::CML,
				false
			),
			Error::<Test>::ConvertToCmlIdLengthMismatch
		);
	})
}

#[test]
fn payoff_loan_should_fail_if_expired_loan_term_duration() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let cml_id: CmlId = 4;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		Cml::add_cml(&user, cml);

		frame_system::Pallet::<Test>::set_block_number(0);
		assert_ok!(GenesisBank::apply_loan_genesis_bank(
			Origin::signed(user),
			from_cml_id(cml_id),
			AssetType::CML
		));

		frame_system::Pallet::<Test>::set_block_number(LOAN_TERM_DURATION as u64 + 1);
		assert_noop!(
			GenesisBank::payoff_loan(
				Origin::signed(user),
				from_cml_id(cml_id),
				AssetType::CML,
				false
			),
			Error::<Test>::LoanInDefault
		);
	})
}

#[test]
fn payoff_loan_should_fail_if_have_not_enough_balance() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let cml_id: CmlId = 4;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		Cml::add_cml(&user, cml);

		assert_ok!(GenesisBank::apply_loan_genesis_bank(
			Origin::signed(user),
			from_cml_id(cml_id),
			AssetType::CML
		));
		assert_eq!(
			<Test as Config>::Currency::free_balance(user),
			CML_A_LOAN_AMOUNT
		);

		assert_noop!(
			GenesisBank::payoff_loan(
				Origin::signed(user),
				from_cml_id(cml_id),
				AssetType::CML,
				false
			),
			Error::<Test>::InsufficientRepayBalance
		);
	})
}

#[test]
fn close_bank_works() {
	new_test_ext().execute_with(|| {
		let user = 1;
		let cml_id: CmlId = 4;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		Cml::add_cml(&user, cml);

		let close_height = 100;
		assert_ok!(GenesisBank::close_bank(Origin::root(), close_height));

		frame_system::Pallet::<Test>::set_block_number(close_height);
		assert_noop!(
			GenesisBank::apply_loan_genesis_bank(
				Origin::signed(user),
				from_cml_id(cml_id),
				AssetType::CML
			),
			Error::<Test>::CannotApplyLoanAfterClosed
		);
	})
}

#[test]
fn close_bank_should_fail_if_called_not_by_root() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			GenesisBank::close_bank(Origin::signed(1), 100),
			DispatchError::BadOrigin
		);
	})
}

#[test]
fn close_bank_should_fail_if_given_height_lower_than_current_height() {
	new_test_ext().execute_with(|| {
		let current_height = 1000;
		frame_system::Pallet::<Test>::set_block_number(current_height);

		assert_noop!(
			GenesisBank::close_bank(Origin::root(), current_height - 1),
			Error::<Test>::InvalidCloseHeight
		);
	})
}

fn seed_from_lifespan(id: CmlId, lifespan: u32) -> Seed {
	Seed {
		id,
		cml_type: CmlType::A,
		defrost_schedule: Some(DefrostScheduleType::Team),
		defrost_time: Some(0),
		lifespan,
		performance: 0,
	}
}
