use crate::tests::new_genesis_seed;
use crate::{mock::*, types::*, CmlStore, Config, Error, MinerItemStore, UserCmlStore};
use frame_support::{assert_noop, assert_ok, traits::Currency};

#[test]
fn start_mining_with_frozen_seed_works() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&1, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let cml = CML::from_genesis_seed(new_genesis_seed(cml_id));
		assert!(cml.is_seed());
		assert!(cml.can_be_defrost(&0).unwrap());
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		let miner_ip = b"miner_ip".to_vec();
		assert_ok!(Cml::start_mining(
			Origin::signed(1),
			cml_id,
			machine_id,
			miner_ip.clone()
		));
	})
}

#[test]
fn start_mining_not_belongs_to_me_should_fail() {
	new_test_ext().execute_with(|| {
		let cml_id: CmlId = 4;
		let cml = CML::from_genesis_seed(new_genesis_seed(cml_id));
		UserCmlStore::<Test>::insert(1, cml_id, ()); // cml belongs to 1
		CmlStore::<Test>::insert(cml_id, cml);

		assert_noop!(
			Cml::start_mining(Origin::signed(2), cml_id, [1u8; 32], b"miner_ip".to_vec()),
			Error::<Test>::CMLOwnerInvalid
		);
	})
}

#[test]
fn start_mining_with_same_machine_id_should_fail() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&1, amount);

		let cml1_id: CmlId = 4;
		let mut cml1 = CML::from_genesis_seed(new_genesis_seed(cml1_id));
		cml1.defrost(&0).unwrap();
		cml1.convert_to_tree(&0).unwrap();

		let cml2_id: CmlId = 5;
		let mut cml2 = CML::from_genesis_seed(new_genesis_seed(cml2_id));
		cml2.defrost(&0).unwrap();
		cml2.convert_to_tree(&0).unwrap();

		UserCmlStore::<Test>::insert(1, cml1_id, ());
		UserCmlStore::<Test>::insert(1, cml2_id, ());
		CmlStore::<Test>::insert(cml1_id, cml1);
		CmlStore::<Test>::insert(cml2_id, cml2);

		let machine_id: MachineId = [1u8; 32];
		let miner_ip = b"miner_ip".to_vec();
		assert_ok!(Cml::start_mining(
			Origin::signed(1),
			cml1_id,
			machine_id,
			miner_ip.clone()
		));

		assert_noop!(
			Cml::start_mining(Origin::signed(1), cml2_id, machine_id, miner_ip.clone()),
			Error::<Test>::MinerAlreadyExist
		);
	})
}

#[test]
fn start_mining_with_same_cmd_planted_into_two_machine_id_should_fail() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&1, amount);

		let cml1_id: CmlId = 4;
		let mut cml1 = CML::from_genesis_seed(new_genesis_seed(cml1_id));
		cml1.defrost(&0).unwrap();
		cml1.convert_to_tree(&0).unwrap();

		UserCmlStore::<Test>::insert(1, cml1_id, ());
		CmlStore::<Test>::insert(cml1_id, cml1);

		let machine_id_1: MachineId = [1u8; 32];
		let miner_ip_1 = b"miner_ip_1".to_vec();
		let machine_id_2: MachineId = [2u8; 32];
		let miner_ip_2 = b"miner_ip_2".to_vec();
		assert_ok!(Cml::start_mining(
			Origin::signed(1),
			cml1_id,
			machine_id_1,
			miner_ip_1.clone()
		));

		assert_noop!(
			Cml::start_mining(Origin::signed(1), cml1_id, machine_id_2, miner_ip_2.clone()),
			Error::<Test>::MinerAlreadyExist
		);
	})
}

#[test]
fn start_mining_with_multiple_times_should_fail() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&1, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let mut cml = CML::from_genesis_seed(new_genesis_seed(cml_id));
		cml.defrost(&0).unwrap();
		cml.convert_to_tree(&0).unwrap();
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		let miner_ip = b"miner_ip".to_vec();
		assert_ok!(Cml::start_mining(
			Origin::signed(1),
			cml_id,
			machine_id,
			miner_ip.clone()
		));

		assert_noop!(
			Cml::start_mining(Origin::signed(1), cml_id, machine_id, miner_ip.clone()),
			Error::<Test>::MinerAlreadyExist
		);
	})
}

#[test]
fn start_mining_with_insufficient_free_balance_should_fail() {
	new_test_ext().execute_with(|| {
		// default account `1` free balance is 0
		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		CmlStore::<Test>::insert(cml_id, CML::from_genesis_seed(new_genesis_seed(cml_id)));

		// todo implement me later
		// assert_noop!(
		// 	Cml::start_mining(Origin::signed(1), cml_id, [1u8; 32], b"miner_id".to_vec()),
		// 	Error::<Test>::InsufficientFreeBalance
		// );
	})
}

#[test]
fn stop_mining_works() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&1, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let cml = CML::from_genesis_seed(new_genesis_seed(cml_id));
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(1),
			cml_id,
			machine_id,
			b"miner_ip".to_vec()
		));

		assert!(MinerItemStore::<Test>::contains_key(machine_id));
		let cml = CmlStore::<Test>::get(cml_id).unwrap();
		assert!(cml.is_mining());
		assert!(cml.machine_id().is_some());
		assert_eq!(cml.staking_slots().len(), 1);

		assert_ok!(Cml::stop_mining(Origin::signed(1), cml_id, machine_id,));

		assert!(!MinerItemStore::<Test>::contains_key(machine_id));
		let cml = CmlStore::<Test>::get(cml_id).unwrap();
		assert!(!cml.is_mining());
		assert!(cml.machine_id().is_none());
		assert_eq!(cml.staking_slots().len(), 0);
	})
}