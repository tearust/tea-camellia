use crate::param::{GENESIS_SEED_A_COUNT, GENESIS_SEED_B_COUNT, GENESIS_SEED_C_COUNT};
use crate::tests::{new_genesis_frozen_seed, new_genesis_seed, seed_from_lifespan};
use crate::{
	mock::*, types::*, CmlStore, Config, Error, GenesisMinerCreditStore, MinerItemStore,
	MiningCmlTaskPoints, UserCmlStore,
};
use frame_support::{assert_noop, assert_ok, traits::Currency};

#[test]
fn start_mining_with_frozen_seed_works() {
	new_test_ext().execute_with(|| {
		let user_id = 1;
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&user_id, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(user_id, cml_id, ());
		let cml = CML::from_genesis_seed(new_genesis_seed(cml_id));
		assert!(cml.is_seed());
		assert!(cml.can_be_defrost(&0));
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		let miner_ip = b"miner_ip".to_vec();
		assert_ok!(Cml::start_mining(
			Origin::signed(user_id),
			cml_id,
			machine_id,
			miner_ip.clone()
		));

		let cml = CmlStore::<Test>::get(cml_id);
		assert!(cml.is_mining());
		assert_eq!(cml.staking_slots().len(), 1);
		assert_eq!(cml.staking_slots()[0].amount, Some(STAKING_PRICE));
		assert_eq!(cml.staking_slots()[0].owner, user_id);

		assert!(!GenesisMinerCreditStore::<Test>::contains_key(&user_id));
		assert_eq!(
			<Test as Config>::Currency::free_balance(&user_id),
			amount - STAKING_PRICE
		);
		assert_eq!(
			<Test as Config>::Currency::reserved_balance(&user_id),
			STAKING_PRICE
		);
	})
}

#[test]
fn start_mining_works_with_insufficient_balance() {
	new_test_ext().execute_with(|| {
		let user_id = 1;
		let amount = STAKING_PRICE - 1;
		<Test as Config>::Currency::make_free_balance_be(&user_id, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(user_id, cml_id, ());
		let cml = CML::from_genesis_seed(new_genesis_seed(cml_id));
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(user_id),
			cml_id,
			machine_id,
			b"miner_ip".to_vec()
		));

		assert_eq!(GenesisMinerCreditStore::<Test>::get(&user_id), 1);
		assert_eq!(<Test as Config>::Currency::free_balance(&user_id), 0);
		assert_eq!(
			<Test as Config>::Currency::reserved_balance(&user_id),
			STAKING_PRICE - 1
		);
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
		let mut cml1 = CML::from_genesis_seed(seed_from_lifespan(cml1_id, 100));
		cml1.defrost(&0);
		cml1.convert_to_tree(&0);

		let cml2_id: CmlId = 5;
		let mut cml2 = CML::from_genesis_seed(seed_from_lifespan(cml2_id, 100));
		cml2.defrost(&0);
		cml2.convert_to_tree(&0);

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
		let mut cml1 = CML::from_genesis_seed(seed_from_lifespan(cml1_id, 100));
		cml1.defrost(&0);
		cml1.convert_to_tree(&0);

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
			Error::<Test>::InvalidMiner
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
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		cml.defrost(&0);
		cml.convert_to_tree(&0);
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
		let cml_id: CmlId = GENESIS_SEED_A_COUNT + GENESIS_SEED_B_COUNT + GENESIS_SEED_C_COUNT;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		CmlStore::<Test>::insert(
			cml_id,
			CML::from_dao_seed(
				Seed {
					id: cml_id,
					cml_type: CmlType::B,
					defrost_schedule: None,
					defrost_time: None,
					lifespan: 100,
					performance: 10,
				},
				0,
			),
		);

		assert_noop!(
			Cml::start_mining(Origin::signed(1), cml_id, [1u8; 32], b"miner_id".to_vec()),
			Error::<Test>::InsufficientFreeBalance
		);
	})
}

#[test]
fn start_mining_should_fail_if_cml_is_not_valid() {
	new_test_ext().execute_with(|| {
		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		cml.defrost(&0);
		cml.convert_to_tree(&0);
		assert!(!cml.can_start_mining(&100));
		CmlStore::<Test>::insert(cml_id, cml);

		frame_system::Pallet::<Test>::set_block_number(100);
		// for all kinds of mining invalid situation please see unit tests in `types/cml.rs`
		assert_noop!(
			Cml::start_mining(Origin::signed(1), cml_id, [1u8; 32], b"miner_id".to_vec()),
			Error::<Test>::InvalidMiner
		);
	})
}

#[test]
fn miner_ip_is_empty_or_invalid_should_fail() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&1, amount);

		let current_height = frame_system::Pallet::<Test>::block_number();

		let cml_id: CmlId = 4;
		let mut cml = CML::from_genesis_seed(new_genesis_seed(cml_id));
		assert!(cml.is_seed() && cml.can_be_defrost(&current_height));
		cml.defrost(&current_height);
		UserCmlStore::<Test>::insert(1, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		let miner_ip = b"".to_vec(); //not valid
		assert_ok!(Cml::active_cml(Origin::signed(1), cml_id));
		assert_noop!(
			Cml::start_mining(Origin::signed(1), cml_id, machine_id, miner_ip.clone()),
			Error::<Test>::InvalidMinerIp,
		);
	})
}

#[test]
fn active_cml_to_already_started_mining_machine_should_fail() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&1, amount);

		let current_height = frame_system::Pallet::<Test>::block_number();

		let cml_id: CmlId = 4;
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		assert!(cml.is_seed() && cml.can_be_defrost(&current_height));
		cml.defrost(&current_height);
		UserCmlStore::<Test>::insert(1, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		let miner_ip = b"miner_ip".to_vec();
		assert_ok!(Cml::active_cml(Origin::signed(1), cml_id));
		assert_ok!(Cml::start_mining(
			Origin::signed(1),
			cml_id,
			machine_id,
			miner_ip.clone()
		));

		let cml_id: CmlId = 5;
		let mut cml = CML::from_genesis_seed(new_genesis_seed(cml_id));
		assert!(cml.is_seed() && cml.can_be_defrost(&current_height));
		cml.defrost(&current_height);
		UserCmlStore::<Test>::insert(2, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		let miner_ip = b"miner_ip".to_vec();
		assert_ok!(Cml::active_cml(Origin::signed(2), cml_id));
		assert_noop!(
			Cml::start_mining(Origin::signed(2), cml_id, machine_id, miner_ip.clone()),
			Error::<Test>::MinerAlreadyExist
		);
	})
}

#[test]
fn start_mining_with_frozen_cml_should_fail() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&1, amount);

		let current_height = frame_system::Pallet::<Test>::block_number();

		let cml_id: CmlId = 4;
		let cml = CML::from_genesis_seed(new_genesis_frozen_seed(cml_id));
		assert!(cml.is_seed() && !cml.can_be_defrost(&current_height));
		UserCmlStore::<Test>::insert(1, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		let miner_ip = b"miner_ip".to_vec();
		assert_noop!(
			Cml::start_mining(Origin::signed(1), cml_id, machine_id, miner_ip.clone()),
			Error::<Test>::InvalidMiner
		);
	});
}

#[test]
fn stop_mining_works() {
	new_test_ext().execute_with(|| {
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
		let cml = CmlStore::<Test>::get(cml_id);
		assert!(cml.is_mining());
		assert!(cml.machine_id().is_some());
		assert_eq!(cml.staking_slots().len(), 1);

		assert_ok!(Cml::stop_mining(Origin::signed(1), cml_id, machine_id,));

		assert!(!MinerItemStore::<Test>::contains_key(machine_id));
		let cml = CmlStore::<Test>::get(cml_id);
		assert!(!cml.is_mining());
		assert!(cml.machine_id().is_none());
		assert_eq!(cml.staking_slots().len(), 0);
	})
}

#[test]
fn stop_mining_should_fail_if_cml_not_belongs_to_user() {
	new_test_ext().execute_with(|| {
		let user1 = 1;
		let user2 = 2;

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(user1, cml_id, ());
		let cml = CML::from_genesis_seed(new_genesis_seed(cml_id));
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(user1),
			cml_id,
			machine_id,
			b"miner_ip".to_vec()
		));

		assert_noop!(
			Cml::stop_mining(Origin::signed(user2), cml_id, machine_id),
			Error::<Test>::CMLOwnerInvalid
		);
	})
}

#[test]
fn stop_mining_should_fail_if_cml_not_exist() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Cml::stop_mining(Origin::signed(1), 11, [1u8; 32]),
			Error::<Test>::NotFoundCML
		);
	})
}

#[test]
fn stop_mining_should_fail_if_cml_not_mining() {
	new_test_ext().execute_with(|| {
		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let cml = CML::from_genesis_seed(new_genesis_seed(cml_id));
		CmlStore::<Test>::insert(cml_id, cml);

		assert_noop!(
			Cml::stop_mining(Origin::signed(1), cml_id, [1u8; 32]),
			Error::<Test>::InvalidMiner
		);
	})
}

#[test]
fn stop_mining_should_fail_if_machine_id_not_exist_in_miner_item_store() {
	new_test_ext().execute_with(|| {
		let cml_id: CmlId = 4;
		let machine_id = [1u8; 32];
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let mut cml = CML::from_genesis_seed(new_genesis_seed(cml_id));
		cml.start_mining(machine_id, StakingItem::default(), &0);
		CmlStore::<Test>::insert(cml_id, cml);

		assert_noop!(
			Cml::stop_mining(Origin::signed(1), cml_id, machine_id),
			Error::<Test>::NotFoundMiner
		);
	})
}

#[test]
fn dummy_ra_task_works() {
	new_test_ext().execute_with(|| {
		let cml_id: CmlId = 4;
		let machine_id = [1u8; 32];
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let mut cml = CML::from_genesis_seed(new_genesis_seed(cml_id));
		cml.start_mining(machine_id, StakingItem::default(), &0);
		CmlStore::<Test>::insert(cml_id, cml);

		MinerItemStore::<Test>::insert(
			machine_id,
			MinerItem {
				cml_id,
				id: machine_id,
				ip: vec![],
				status: MinerStatus::Active,
			},
		);

		assert!(!MiningCmlTaskPoints::<Test>::contains_key(&cml_id));

		assert_ok!(Cml::dummy_ra_task(Origin::signed(1), machine_id));
		assert_eq!(MiningCmlTaskPoints::<Test>::get(&cml_id), 1);

		assert_ok!(Cml::dummy_ra_task(Origin::signed(1), machine_id));
		assert_eq!(MiningCmlTaskPoints::<Test>::get(&cml_id), 2);

		// a machine can only have u32::MAX points
		MiningCmlTaskPoints::<Test>::mutate(&cml_id, |point| *point = u32::MAX);
		assert_ok!(Cml::dummy_ra_task(Origin::signed(1), machine_id));
		assert_eq!(MiningCmlTaskPoints::<Test>::get(&cml_id), u32::MAX);
	})
}
