use crate::tests::new_genesis_seed;
use crate::{mock::*, types::*, AccountRewards, CmlStore, Config, UserCmlStore};
use frame_support::{assert_ok, traits::Currency};
use pallet_utils::CurrencyOperations;

#[test]
fn start_staking_with_balance_works() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&1, amount);
		<Test as Config>::Currency::make_free_balance_be(&2, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let cml = CML::from_genesis_seed(new_genesis_seed(cml_id));
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(1),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));

		assert_ok!(Cml::start_staking(Origin::signed(2), cml_id, None));
	})
}

#[test]
fn start_staking_with_cml_works() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&1, amount);

		let cml1_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml1_id, ());
		let mut cml = CML::from_genesis_seed(new_genesis_seed(cml1_id));
		cml.defrost(&0);
		cml.convert_to_tree(&0);
		CmlStore::<Test>::insert(cml1_id, cml);

		let cml2_id: CmlId = 5;
		UserCmlStore::<Test>::insert(2, cml2_id, ());
		let cml = CML::from_genesis_seed(new_genesis_seed(cml2_id));
		CmlStore::<Test>::insert(cml2_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(1),
			cml1_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));

		assert_ok!(Cml::start_staking(
			Origin::signed(2),
			cml1_id,
			Some(cml2_id)
		));
	})
}

#[test]
fn stop_staking_with_balance_works() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&1, amount);
		<Test as Config>::Currency::make_free_balance_be(&2, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let cml = CML::from_genesis_seed(new_genesis_seed(cml_id));
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(1),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));

		assert_eq!(<Test as Config>::Currency::total_balance(&2), amount);
		assert_eq!(<Test as Config>::Currency::free_balance(&2), amount);
		assert_ok!(Cml::start_staking(Origin::signed(2), cml_id, None));

		assert_eq!(<Test as Config>::Currency::total_balance(&2), amount);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&2),
			amount - STAKING_PRICE
		);

		assert_ok!(Cml::stop_staking(Origin::signed(2), cml_id, 1));
		assert_eq!(<Test as Config>::Currency::total_balance(&2), amount);
		assert_eq!(<Test as Config>::Currency::free_balance(&2), amount);
	})
}

#[test]
fn stop_staking_with_cml_works() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&1, amount);

		let cml1_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml1_id, ());
		let mut cml1 = CML::from_genesis_seed(new_genesis_seed(cml1_id));
		cml1.defrost(&0);
		cml1.convert_to_tree(&0);
		CmlStore::<Test>::insert(cml1_id, cml1);

		let cml2_id: CmlId = 5;
		UserCmlStore::<Test>::insert(2, cml2_id, ());
		let mut cml2 = CML::from_genesis_seed(new_genesis_seed(cml2_id));
		cml2.set_owner(&2);
		CmlStore::<Test>::insert(cml2_id, cml2);

		assert_ok!(Cml::start_mining(
			Origin::signed(1),
			cml1_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));

		assert!(!CmlStore::<Test>::get(cml2_id).unwrap().is_staking());
		assert_ok!(Cml::start_staking(
			Origin::signed(2),
			cml1_id,
			Some(cml2_id)
		));
		assert!(CmlStore::<Test>::get(cml2_id).unwrap().is_staking());

		assert_ok!(Cml::stop_staking(Origin::signed(2), cml1_id, 1));
		assert!(!CmlStore::<Test>::get(cml2_id).unwrap().is_staking());
	})
}

#[test]
fn withdraw_staking_reward_works() {
	new_test_ext().execute_with(|| {
		let amount = 100;
		AccountRewards::<Test>::insert(1, amount);

		assert_eq!(Utils::free_balance(&1), 0);
		assert!(AccountRewards::<Test>::contains_key(&1));
		assert_eq!(AccountRewards::<Test>::get(&1).unwrap(), amount);

		assert_ok!(Cml::withdraw_staking_reward(Origin::signed(1)));

		assert_eq!(Utils::free_balance(&1), 100);
		assert!(AccountRewards::<Test>::contains_key(&1));
		assert_eq!(AccountRewards::<Test>::get(&1).unwrap(), 0);
	})
}
