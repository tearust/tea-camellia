use crate::param::{GENESIS_SEED_A_COUNT, GENESIS_SEED_B_COUNT, GENESIS_SEED_C_COUNT};
use crate::tests::{new_genesis_frozen_seed, new_genesis_seed, seed_from_lifespan};
use crate::{
	mock::*, types::*, CmlOperation, CmlStore, Config, Error, MinerItemStore, MiningCmlTaskPoints,
	UserCmlStore,
};
use frame_support::{assert_noop, assert_ok, traits::Currency};
use pallet_utils::CurrencyOperations;

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
		assert!(cml.check_defrost(&0).is_ok());
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		let miner_ip = b"miner_ip".to_vec();
		assert_ok!(Cml::start_mining(
			Origin::signed(user_id),
			cml_id,
			machine_id,
			user_id,
			miner_ip.clone(),
			None,
		));

		let cml = CmlStore::<Test>::get(cml_id);
		assert!(cml.is_mining());
		assert_eq!(cml.staking_slots().len(), 1);
		assert_eq!(cml.staking_slots()[0].amount, Some(STAKING_PRICE));
		assert_eq!(cml.staking_slots()[0].owner, user_id);

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
fn start_mining_should_fail_with_insufficient_balance() {
	new_test_ext().execute_with(|| {
		let user_id = 1;
		let amount = STAKING_PRICE - 1;
		<Test as Config>::Currency::make_free_balance_be(&user_id, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(user_id, cml_id, ());
		let cml = CML::from_genesis_seed(new_genesis_seed(cml_id));
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		assert_noop!(
			Cml::start_mining(
				Origin::signed(user_id),
				cml_id,
				machine_id,
				user_id,
				b"miner_ip".to_vec(),
				None,
			),
			Error::<Test>::InsufficientFreeBalance
		);
		assert!(!CmlStore::<Test>::get(cml_id).is_mining());
		assert_eq!(
			<Test as Config>::Currency::free_balance(&user_id),
			STAKING_PRICE - 1
		);
		assert_eq!(<Test as Config>::Currency::reserved_balance(&user_id), 0);
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
			Cml::start_mining(
				Origin::signed(2),
				cml_id,
				[1u8; 32],
				2,
				b"miner_ip".to_vec(),
				None,
			),
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
			1,
			miner_ip.clone(),
			None,
		));

		assert_noop!(
			Cml::start_mining(
				Origin::signed(1),
				cml2_id,
				machine_id,
				1,
				miner_ip.clone(),
				None,
			),
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
			1,
			miner_ip_1.clone(),
			None,
		));

		assert_noop!(
			Cml::start_mining(
				Origin::signed(1),
				cml1_id,
				machine_id_2,
				1,
				miner_ip_2.clone(),
				None,
			),
			Error::<Test>::CmlIsMiningAlready
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
			1,
			miner_ip.clone(),
			None,
		));

		assert_noop!(
			Cml::start_mining(
				Origin::signed(1),
				cml_id,
				machine_id,
				1,
				miner_ip.clone(),
				None,
			),
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
			Cml::start_mining(
				Origin::signed(1),
				cml_id,
				[1u8; 32],
				1,
				b"miner_id".to_vec(),
				None,
			),
			Error::<Test>::InsufficientFreeBalance
		);
	})
}

#[test]
fn start_mining_should_fail_if_miner_ip_already_registered() {
	new_test_ext().execute_with(|| {
		let owner1 = 11;
		let owner2 = 22;
		let owner_origin_balance = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&owner1, owner_origin_balance);
		<Test as Config>::Currency::make_free_balance_be(&owner2, owner_origin_balance);

		let cml_id1 = 1;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id1, 100));
		UserCmlStore::<Test>::insert(owner1, cml_id1, ());
		CmlStore::<Test>::insert(cml_id1, cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(owner1),
			cml_id1,
			[1u8; 32],
			owner1,
			b"miner_ip".to_vec(),
			None,
		));

		let cml_id2 = 2;
		let cml2 = CML::from_genesis_seed(seed_from_lifespan(cml_id2, 100));
		UserCmlStore::<Test>::insert(owner2, cml_id2, ());
		CmlStore::<Test>::insert(cml_id2, cml2);
		assert_noop!(
			Cml::start_mining(
				Origin::signed(owner2),
				cml_id2,
				[2u8; 32],
				owner2,
				b"miner_ip".to_vec(),
				None,
			),
			Error::<Test>::MinerIpAlreadyExist
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
		assert!(cml.check_start_mining(&100).is_err());
		CmlStore::<Test>::insert(cml_id, cml);

		frame_system::Pallet::<Test>::set_block_number(100);
		// for all kinds of mining invalid situation please see unit tests in `types/cml.rs`
		assert_noop!(
			Cml::start_mining(
				Origin::signed(1),
				cml_id,
				[1u8; 32],
				1,
				b"miner_id".to_vec(),
				None,
			),
			Error::<Test>::CmlShouldDead
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
		assert!(cml.is_seed() && cml.check_defrost(&current_height).is_ok());
		cml.defrost(&current_height);
		UserCmlStore::<Test>::insert(1, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		let miner_ip = b"".to_vec(); //not valid
		assert_ok!(Cml::active_cml(Origin::signed(1), cml_id));
		assert_noop!(
			Cml::start_mining(
				Origin::signed(1),
				cml_id,
				machine_id,
				1,
				miner_ip.clone(),
				None,
			),
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
		assert!(cml.is_seed() && cml.check_defrost(&current_height).is_ok());
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
			1,
			miner_ip.clone(),
			None,
		));

		let cml_id: CmlId = 5;
		let mut cml = CML::from_genesis_seed(new_genesis_seed(cml_id));
		assert!(cml.is_seed() && cml.check_defrost(&current_height).is_ok());
		cml.defrost(&current_height);
		UserCmlStore::<Test>::insert(2, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		let miner_ip = b"miner_ip".to_vec();
		assert_ok!(Cml::active_cml(Origin::signed(2), cml_id));
		assert_noop!(
			Cml::start_mining(
				Origin::signed(2),
				cml_id,
				machine_id,
				2,
				miner_ip.clone(),
				None,
			),
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
		assert!(cml.is_seed() && cml.check_defrost(&current_height).is_err());
		UserCmlStore::<Test>::insert(1, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		let miner_ip = b"miner_ip".to_vec();
		assert_noop!(
			Cml::start_mining(
				Origin::signed(1),
				cml_id,
				machine_id,
				1,
				miner_ip.clone(),
				None,
			),
			Error::<Test>::CmlStillInFrozenLockedPeriod
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
		<Test as Config>::Currency::make_free_balance_be(&1, STAKING_PRICE);
		assert_ok!(Cml::start_mining(
			Origin::signed(1),
			cml_id,
			machine_id,
			1,
			b"miner_ip".to_vec(),
			None,
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
fn stop_mining_works_with_balance_staking() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let user1 = 2;
		let user2 = 3;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);
		<Test as Config>::Currency::make_free_balance_be(&user1, amount);
		<Test as Config>::Currency::make_free_balance_be(&user2, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			machine_id,
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		assert_ok!(Cml::start_staking(
			Origin::signed(user1),
			cml_id,
			None,
			None
		));
		assert_ok!(Cml::start_staking(
			Origin::signed(user2),
			cml_id,
			None,
			None
		));

		assert_eq!(
			<Test as Config>::Currency::free_balance(&miner),
			amount - STAKING_PRICE
		);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&user1),
			amount - STAKING_PRICE
		);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&user2),
			amount - STAKING_PRICE
		);

		assert_ok!(Cml::stop_mining(Origin::signed(1), cml_id, machine_id));

		assert_eq!(
			<Test as Config>::Currency::free_balance(&miner),
			amount - STOP_MINING_PUNISHMENT * 2
		);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&user1),
			amount + STOP_MINING_PUNISHMENT
		);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&user2),
			amount + STOP_MINING_PUNISHMENT
		);
	})
}

#[test]
fn stop_mining_works_with_cml_staking() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let user1 = 2;
		let user2 = 3;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);
		<Test as Config>::Currency::make_free_balance_be(&user1, amount);
		<Test as Config>::Currency::make_free_balance_be(&user2, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		let cml_id1: CmlId = 5;
		UserCmlStore::<Test>::insert(user1, cml_id1, ());
		let mut cml1 = CML::from_genesis_seed(seed_from_lifespan(cml_id1, 100));
		cml1.set_owner(&user1);
		CmlStore::<Test>::insert(cml_id1, cml1);

		let cml_id2: CmlId = 6;
		UserCmlStore::<Test>::insert(user2, cml_id2, ());
		let mut seed2 = seed_from_lifespan(cml_id2, 100);
		seed2.cml_type = CmlType::B;
		let mut cml2 = CML::from_genesis_seed(seed2);
		cml2.set_owner(&user2);
		CmlStore::<Test>::insert(cml_id2, cml2);

		let machine_id: MachineId = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			machine_id,
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		assert_ok!(Cml::start_staking(
			Origin::signed(user1),
			cml_id,
			Some(cml_id1),
			None
		));
		assert_ok!(Cml::start_staking(
			Origin::signed(user2),
			cml_id,
			Some(cml_id2),
			None
		));

		assert!(CmlStore::<Test>::get(cml_id1).is_staking());
		assert!(CmlStore::<Test>::get(cml_id2).is_staking());

		assert_eq!(
			<Test as Config>::Currency::free_balance(&miner),
			amount - STAKING_PRICE
		);
		assert_eq!(<Test as Config>::Currency::free_balance(&user1), amount);
		assert_eq!(<Test as Config>::Currency::free_balance(&user2), amount);

		assert_ok!(Cml::stop_mining(Origin::signed(1), cml_id, machine_id));

		assert!(!CmlStore::<Test>::get(cml_id1).is_staking());
		assert!(!CmlStore::<Test>::get(cml_id2).is_staking());

		assert_eq!(
			<Test as Config>::Currency::free_balance(&miner),
			amount - STOP_MINING_PUNISHMENT * 6
		);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&user1),
			amount + STOP_MINING_PUNISHMENT * 4
		);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&user2),
			amount + STOP_MINING_PUNISHMENT * 2
		);
	})
}

#[test]
fn stop_mining_works_if_only_miner_stakes() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			machine_id,
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		<Test as Config>::Currency::make_free_balance_be(&miner, STAKING_PRICE * 2);
		assert_ok!(Cml::start_staking(
			Origin::signed(miner),
			cml_id,
			None,
			None
		));
		assert_ok!(Cml::start_staking(
			Origin::signed(miner),
			cml_id,
			None,
			None
		));
		assert_eq!(<Test as Config>::Currency::free_balance(&miner), 0);
		assert_eq!(Utils::reserved_balance(&miner), STAKING_PRICE * 3);

		assert_ok!(Cml::stop_mining(Origin::signed(miner), cml_id, machine_id));
		assert_eq!(
			<Test as Config>::Currency::free_balance(&miner),
			STAKING_PRICE * 3
		);
		assert_eq!(Utils::reserved_balance(&miner), 0);
	})
}

#[test]
fn stop_mining_should_fail_if_miner_free_balance_is_not_enoungh_pay_for_stakers() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let user1 = 2;
		let user2 = 3;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, STAKING_PRICE);
		<Test as Config>::Currency::make_free_balance_be(&user1, amount);
		<Test as Config>::Currency::make_free_balance_be(&user2, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			machine_id,
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		assert_ok!(Cml::start_staking(
			Origin::signed(user1),
			cml_id,
			None,
			None
		));
		assert_ok!(Cml::start_staking(
			Origin::signed(user2),
			cml_id,
			None,
			None
		));
		assert_eq!(<Test as Config>::Currency::free_balance(&miner), 0);

		assert_noop!(
			Cml::stop_mining(Origin::signed(1), cml_id, machine_id),
			Error::<Test>::InsufficientFreeBalanceToPayForPunishment
		);

		assert_eq!(<Test as Config>::Currency::free_balance(&miner), 0);
		assert_eq!(Utils::free_balance(&user1), amount - STAKING_PRICE);
		assert_eq!(Utils::free_balance(&user2), amount - STAKING_PRICE);
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
		<Test as Config>::Currency::make_free_balance_be(&user1, STAKING_PRICE);
		assert_ok!(Cml::start_mining(
			Origin::signed(user1),
			cml_id,
			machine_id,
			user1,
			b"miner_ip".to_vec(),
			None,
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
fn stop_mining_should_fail_if_is_hosting() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = HOSTING_CML_ID;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			machine_id,
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		assert_noop!(
			Cml::stop_mining(Origin::signed(miner), cml_id, machine_id),
			Error::<Test>::CannotStopMiningWhenHostingTApp
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
				orbitdb_id: None,
				controller_account: Default::default(),
				status: MinerStatus::Active,
				suspend_height: None,
				schedule_down_height: None,
			},
		);

		assert!(!MiningCmlTaskPoints::<Test>::contains_key(&cml_id));

		assert_ok!(Cml::dummy_ra_task(Origin::signed(1), machine_id, 1));
		assert_eq!(MiningCmlTaskPoints::<Test>::get(&cml_id), 1);

		assert_ok!(Cml::dummy_ra_task(Origin::signed(1), machine_id, 1));
		assert_eq!(MiningCmlTaskPoints::<Test>::get(&cml_id), 2);

		assert_ok!(Cml::dummy_ra_task(Origin::signed(1), machine_id, 3));
		assert_eq!(MiningCmlTaskPoints::<Test>::get(&cml_id), 5);

		// a machine can only have u32::MAX points
		MiningCmlTaskPoints::<Test>::mutate(&cml_id, |point| *point = u32::MAX);
		assert_ok!(Cml::dummy_ra_task(Origin::signed(1), machine_id, 1));
		assert_eq!(MiningCmlTaskPoints::<Test>::get(&cml_id), u32::MAX);
	})
}

#[test]
fn suspend_mining_works() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			machine_id,
			miner,
			b"miner_ip".to_vec(),
			None,
		));
		let miner_item = MinerItemStore::<Test>::get(machine_id);
		assert_eq!(miner_item.status, MinerStatus::Active);
		assert_eq!(miner_item.suspend_height, None);

		frame_system::Pallet::<Test>::set_block_number(100);
		Cml::suspend_mining(machine_id);
		let miner_item = MinerItemStore::<Test>::get(machine_id);
		assert_eq!(miner_item.status, MinerStatus::Offline);
		assert_eq!(miner_item.suspend_height, Some(100));
	})
}

#[test]
fn resume_mining_works() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			machine_id,
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		frame_system::Pallet::<Test>::set_block_number(100);
		Cml::suspend_mining(machine_id);
		let miner_item = MinerItemStore::<Test>::get(machine_id);
		assert_eq!(miner_item.status, MinerStatus::Offline);
		assert_eq!(miner_item.suspend_height, Some(100));

		assert_ok!(Cml::resume_mining(Origin::signed(miner), cml_id));
		let miner_item = MinerItemStore::<Test>::get(machine_id);
		assert_eq!(miner_item.status, MinerStatus::Active);
		assert_eq!(miner_item.suspend_height, None);
	})
}

#[test]
fn resume_mining_should_fail_if_cml_not_exist() {
	new_test_ext().execute_with(|| {
		let cml_id: CmlId = 4;

		assert_noop!(
			Cml::resume_mining(Origin::signed(1), cml_id),
			Error::<Test>::NotFoundCML
		);
	})
}

#[test]
fn resume_mining_should_fail_if_user_is_not_cml_owner() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let user = 2;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		assert_noop!(
			Cml::resume_mining(Origin::signed(user), cml_id),
			Error::<Test>::CMLOwnerInvalid
		);
	})
}

#[test]
fn resume_mining_should_fail_if_cml_is_not_mining() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		assert_noop!(
			Cml::resume_mining(Origin::signed(miner), cml_id),
			Error::<Test>::NotFoundMiner
		);
	})
}

#[test]
fn resume_mining_should_fail_if_cml_is_active_already() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			machine_id,
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		assert_noop!(
			Cml::resume_mining(Origin::signed(miner), cml_id),
			Error::<Test>::NoNeedToResume
		);
	})
}

#[test]
fn resume_mining_should_fail_free_balance_is_not_enough() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = INSUFFICIENT_CML_ID;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			machine_id,
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		Cml::suspend_mining(machine_id);

		assert_noop!(
			Cml::resume_mining(Origin::signed(miner), cml_id),
			Error::<Test>::InsufficientFreeBalanceToAppendPledge
		);
	})
}

#[test]
fn schedule_down_works() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 1000));
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			machine_id,
			miner,
			b"miner_ip".to_vec(),
			None,
		));
		let miner_item = MinerItemStore::<Test>::get(machine_id);
		assert_eq!(miner_item.status, MinerStatus::Active);
		assert_eq!(miner_item.schedule_down_height, None);

		frame_system::Pallet::<Test>::set_block_number(100);

		assert_ok!(Cml::schedule_down(Origin::signed(miner), cml_id));
		let miner_item = MinerItemStore::<Test>::get(machine_id);
		assert_eq!(miner_item.status, MinerStatus::ScheduleDown);
		assert_eq!(miner_item.schedule_down_height, Some(100));
	})
}

#[test]
fn schedule_down_should_fail_if_cml_not_exist() {
	new_test_ext().execute_with(|| {
		let cml_id: CmlId = 4;

		assert_noop!(
			Cml::schedule_down(Origin::signed(1), cml_id),
			Error::<Test>::NotFoundCML
		);
	})
}

#[test]
fn schedule_down_should_fail_if_user_is_not_cml_owner() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let user = 2;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		assert_noop!(
			Cml::schedule_down(Origin::signed(user), cml_id),
			Error::<Test>::CMLOwnerInvalid
		);
	})
}

#[test]
fn schedule_down_should_fail_if_cml_is_not_mining() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		assert_noop!(
			Cml::schedule_down(Origin::signed(miner), cml_id),
			Error::<Test>::NotFoundMiner
		);
	})
}

#[test]
fn schedule_down_should_fail_if_cml_is_not_active() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			machine_id,
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		Cml::suspend_mining(machine_id);
		assert_noop!(
			Cml::schedule_down(Origin::signed(miner), cml_id),
			Error::<Test>::CanNotScheduleDownWhenInactive
		);
	})
}

#[test]
fn schedule_up_works() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 1000));
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			machine_id,
			miner,
			b"miner_ip".to_vec(),
			None,
		));
		let miner_item = MinerItemStore::<Test>::get(machine_id);
		assert_eq!(miner_item.status, MinerStatus::Active);
		assert_eq!(miner_item.schedule_down_height, None);

		frame_system::Pallet::<Test>::set_block_number(100);

		assert_ok!(Cml::schedule_down(Origin::signed(miner), cml_id));
		let miner_item = MinerItemStore::<Test>::get(machine_id);
		assert_eq!(miner_item.status, MinerStatus::ScheduleDown);
		assert_eq!(miner_item.schedule_down_height, Some(100));

		assert_ok!(Cml::schedule_up(Origin::signed(miner), cml_id));
		let miner_item = MinerItemStore::<Test>::get(machine_id);
		assert_eq!(miner_item.status, MinerStatus::Active);
		assert_eq!(miner_item.schedule_down_height, None);
	})
}

#[test]
fn schedule_up_should_fail_if_cml_not_exist() {
	new_test_ext().execute_with(|| {
		let cml_id: CmlId = 4;

		assert_noop!(
			Cml::schedule_up(Origin::signed(1), cml_id),
			Error::<Test>::NotFoundCML
		);
	})
}

#[test]
fn schedule_up_should_fail_if_user_is_not_cml_owner() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let user = 2;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		assert_noop!(
			Cml::schedule_up(Origin::signed(user), cml_id),
			Error::<Test>::CMLOwnerInvalid
		);
	})
}

#[test]
fn schedule_up_should_fail_if_cml_is_not_mining() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		assert_noop!(
			Cml::schedule_up(Origin::signed(miner), cml_id),
			Error::<Test>::NotFoundMiner
		);
	})
}

#[test]
fn schedule_up_should_fail_if_cml_is_offline() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			machine_id,
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		Cml::suspend_mining(machine_id);
		assert_noop!(
			Cml::schedule_up(Origin::signed(miner), cml_id),
			Error::<Test>::NoNeedToScheduleUp
		);
	})
}

#[test]
fn schedule_up_should_fail_if_cml_is_active() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			machine_id,
			miner,
			b"miner_ip".to_vec(),
			None,
		));
		let miner_item = MinerItemStore::<Test>::get(machine_id);
		assert_eq!(miner_item.status, MinerStatus::Active);

		assert_noop!(
			Cml::schedule_up(Origin::signed(miner), cml_id),
			Error::<Test>::NoNeedToScheduleUp
		);
	})
}

#[test]
fn migrate_works_if_cml_is_offline() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			machine_id,
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		Cml::suspend_mining(machine_id);
		let miner_item = MinerItemStore::<Test>::get(machine_id);
		assert_eq!(miner_item.status, MinerStatus::Offline);

		let new_machine_id = [2u8; 32];
		let new_miner_ip = b"miner_ip2".to_vec();
		assert_ok!(Cml::migrate(
			Origin::signed(miner),
			cml_id,
			new_machine_id,
			new_miner_ip.clone()
		));

		assert!(!MinerItemStore::<Test>::contains_key(machine_id));
		let miner_item = MinerItemStore::<Test>::get(new_machine_id);
		assert_eq!(miner_item.status, MinerStatus::Offline);
		assert_eq!(miner_item.id, new_machine_id);
		assert_eq!(miner_item.ip, new_miner_ip);
	})
}

#[test]
fn migrate_works_if_cml_is_schedule_down() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			machine_id,
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		assert_ok!(Cml::schedule_down(Origin::signed(miner), cml_id));
		let miner_item = MinerItemStore::<Test>::get(machine_id);
		assert_eq!(miner_item.status, MinerStatus::ScheduleDown);

		let new_machine_id = [2u8; 32];
		let new_miner_ip = b"miner_ip2".to_vec();
		assert_ok!(Cml::migrate(
			Origin::signed(miner),
			cml_id,
			new_machine_id,
			new_miner_ip.clone()
		));

		assert!(!MinerItemStore::<Test>::contains_key(machine_id));
		let miner_item = MinerItemStore::<Test>::get(new_machine_id);
		assert_eq!(miner_item.status, MinerStatus::ScheduleDown);
		assert_eq!(miner_item.id, new_machine_id);
		assert_eq!(miner_item.ip, new_miner_ip);
	})
}

#[test]
fn migrate_should_fail_if_cml_not_exist() {
	new_test_ext().execute_with(|| {
		let cml_id: CmlId = 4;

		assert_noop!(
			Cml::migrate(Origin::signed(1), cml_id, [1u8; 32], vec![]),
			Error::<Test>::NotFoundCML
		);
	})
}

#[test]
fn migrate_should_fail_if_user_is_not_cml_owner() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let user = 2;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		assert_noop!(
			Cml::migrate(Origin::signed(user), cml_id, [1u8; 32], vec![]),
			Error::<Test>::CMLOwnerInvalid
		);
	})
}

#[test]
fn migrate_should_fail_if_cml_is_not_mining() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		assert_noop!(
			Cml::migrate(Origin::signed(miner), cml_id, [1u8; 32], vec![]),
			Error::<Test>::NotFoundMiner
		);
	})
}

#[test]
fn migrate_should_fail_if_cml_is_active() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			machine_id,
			miner,
			b"miner_ip".to_vec(),
			None,
		));
		let miner_item = MinerItemStore::<Test>::get(machine_id);
		assert_eq!(miner_item.status, MinerStatus::Active);

		assert_noop!(
			Cml::migrate(Origin::signed(miner), cml_id, [1u8; 32], vec![]),
			Error::<Test>::CannotMigrateWhenActive
		);
	})
}

#[test]
fn migrate_should_fail_if_machine_id_already_exist() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let miner2 = 2;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);
		<Test as Config>::Currency::make_free_balance_be(&miner2, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		let cml_id2: CmlId = 5;
		UserCmlStore::<Test>::insert(miner2, cml_id2, ());
		let cml2 = CML::from_genesis_seed(seed_from_lifespan(cml_id2, 100));
		CmlStore::<Test>::insert(cml_id2, cml2);

		let machine_id: MachineId = [1u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			machine_id,
			miner,
			b"miner_ip".to_vec(),
			None,
		));

		let machine_id2: MachineId = [2u8; 32];
		assert_ok!(Cml::start_mining(
			Origin::signed(miner2),
			cml_id2,
			machine_id2,
			miner2,
			b"miner_ip2".to_vec(),
			None,
		));

		assert_noop!(
			Cml::migrate(Origin::signed(miner), cml_id, machine_id2, vec![]),
			Error::<Test>::MinerAlreadyExist
		);
	})
}

#[test]
fn migrate_should_fail_if_ip_address_already_exist() {
	new_test_ext().execute_with(|| {
		let miner = 1;
		let miner2 = 2;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&miner, amount);
		<Test as Config>::Currency::make_free_balance_be(&miner2, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(miner, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		let cml_id2: CmlId = 5;
		UserCmlStore::<Test>::insert(miner2, cml_id2, ());
		let cml2 = CML::from_genesis_seed(seed_from_lifespan(cml_id2, 100));
		CmlStore::<Test>::insert(cml_id2, cml2);

		let machine_id: MachineId = [1u8; 32];
		let ip_address = b"miner_ip".to_vec();
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			cml_id,
			machine_id,
			miner,
			ip_address,
			None,
		));

		let machine_id2: MachineId = [2u8; 32];
		let ip_address2 = b"miner_ip2".to_vec();
		assert_ok!(Cml::start_mining(
			Origin::signed(miner2),
			cml_id2,
			machine_id2,
			miner2,
			ip_address2.clone(),
			None,
		));

		assert_noop!(
			Cml::migrate(Origin::signed(miner), cml_id, machine_id, ip_address2),
			Error::<Test>::MinerIpAlreadyExist
		);
	})
}
