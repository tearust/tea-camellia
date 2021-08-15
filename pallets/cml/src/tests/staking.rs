use crate::tests::{new_genesis_seed, seed_from_lifespan};
use crate::{
	mock::*, types::*, AccountRewards, CmlStore, Config, Error, GenesisMinerCreditStore,
	UserCmlStore,
};
use frame_support::{assert_noop, assert_ok, traits::Currency};
use pallet_utils::CurrencyOperations;

#[test]
fn start_staking_with_balance_works() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&1, amount);
		<Test as Config>::Currency::make_free_balance_be(&2, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(1),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));

		assert_ok!(Cml::start_staking(Origin::signed(2), cml_id, None, None));
	})
}

#[test]
fn start_staking_with_cml_works() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&1, amount);

		let cml1_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml1_id, ());
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml1_id, 100));
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
			Some(cml2_id),
			None
		));
	})
}

#[test]
fn start_staking_with_balance_should_fail_if_free_balance_is_not_enough() {
	new_test_ext().execute_with(|| {
		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		// account 2 free balance is zero
		assert_noop!(
			Cml::start_staking(Origin::signed(2), cml_id, None, None),
			Error::<Test>::InsufficientFreeBalance
		);
	})
}

#[test]
fn start_staking_should_fail_if_miner_is_not_exist() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Cml::start_staking(Origin::signed(2), 1, None, None),
			Error::<Test>::NotFoundCML
		);
	})
}

#[test]
fn start_staking_should_fail_if_miner_is_invalid() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&2, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));

		assert!(cml.check_can_be_stake(&0, &None, &Some(cml_id)).is_err());
		CmlStore::<Test>::insert(cml_id, cml);

		// for all kinds of invalid situation please see unit tests in `types/cml.rs`
		assert_noop!(
			Cml::start_staking(Origin::signed(2), cml_id, None, None),
			Error::<Test>::CmlIsNotMining
		);
	})
}

#[test]
fn start_staking_should_fail_if_staking_cml_not_found() {
	new_test_ext().execute_with(|| {
		let miner_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, miner_id, ());
		let miner = CML::from_genesis_seed(seed_from_lifespan(miner_id, 100));
		CmlStore::<Test>::insert(miner_id, miner);

		assert_noop!(
			Cml::start_staking(Origin::signed(2), miner_id, Some(2), None), // cml id 2 is not exist
			Error::<Test>::NotFoundCML
		);
	})
}

#[test]
fn start_staking_should_fail_if_staking_cml_not_belong_to_staker() {
	new_test_ext().execute_with(|| {
		let user1 = 1;
		let user2 = 2;

		let miner_id: CmlId = 4;
		UserCmlStore::<Test>::insert(user1, miner_id, ());
		let miner = CML::from_genesis_seed(seed_from_lifespan(miner_id, 100));
		CmlStore::<Test>::insert(miner_id, miner);

		let staker_id: CmlId = 5;
		UserCmlStore::<Test>::insert(user2, staker_id, ());
		let staker = CML::from_genesis_seed(seed_from_lifespan(staker_id, 100));
		CmlStore::<Test>::insert(staker_id, staker);

		assert_noop!(
			Cml::start_staking(Origin::signed(user1), miner_id, Some(staker_id), None),
			Error::<Test>::CMLOwnerInvalid
		);
	})
}

#[test]
fn start_staking_should_fail_if_staking_cml_is_invalid() {
	new_test_ext().execute_with(|| {
		let miner_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, miner_id, ());
		let miner = CML::from_genesis_seed(seed_from_lifespan(miner_id, 100));
		CmlStore::<Test>::insert(miner_id, miner);

		let staker_id: CmlId = 5;
		UserCmlStore::<Test>::insert(2, staker_id, ());
		let mut staker = CML::from_genesis_seed(seed_from_lifespan(staker_id, 50));
		staker.defrost(&0);
		staker.convert_to_tree(&0);
		assert!(staker.should_dead(&50));
		CmlStore::<Test>::insert(staker_id, staker);

		frame_system::Pallet::<Test>::set_block_number(50);
		// for all kinds of invalid situation please see unit tests in `types/cml.rs`
		assert_noop!(
			Cml::start_staking(Origin::signed(2), miner_id, Some(staker_id), None),
			Error::<Test>::CmlShouldDead
		);
	})
}

#[test]
fn start_staking_should_fail_if_the_stakee_slots_over_than_the_max_length() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice

		let cml_id: CmlId = 0;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(1),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));

		for i in 0..STAKING_SLOTS_MAX_LENGTH {
			<Test as Config>::Currency::make_free_balance_be(&(i as u64), amount);
			assert_ok!(Cml::start_staking(
				Origin::signed(i as u64),
				cml_id,
				None,
				None
			));
		}

		<Test as Config>::Currency::make_free_balance_be(
			&(STAKING_SLOTS_MAX_LENGTH as u64),
			amount,
		);
		assert_noop!(
			Cml::start_staking(
				Origin::signed(STAKING_SLOTS_MAX_LENGTH as u64),
				cml_id,
				None,
				None
			),
			Error::<Test>::StakingSlotsOverTheMaxLength
		);
	})
}

#[test]
fn start_staking_should_fail_if_the_stakee_slots_over_than_acceptable_index() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice

		let cml_id: CmlId = 0;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(1),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));

		let acceptable_slot_index = 10;
		for i in 0..acceptable_slot_index {
			<Test as Config>::Currency::make_free_balance_be(&i, amount);
			assert_ok!(Cml::start_staking(
				Origin::signed(i),
				cml_id,
				None,
				Some(acceptable_slot_index as u32)
			));
		}

		<Test as Config>::Currency::make_free_balance_be(&acceptable_slot_index, amount);
		assert_noop!(
			Cml::start_staking(
				Origin::signed(acceptable_slot_index),
				cml_id,
				None,
				Some(acceptable_slot_index as u32)
			),
			Error::<Test>::StakingSlotsOverAcceptableIndex
		);
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
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(1),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));

		assert_eq!(<Test as Config>::Currency::total_balance(&2), amount);
		assert_eq!(<Test as Config>::Currency::free_balance(&2), amount);
		assert_ok!(Cml::start_staking(Origin::signed(2), cml_id, None, None));

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
fn stop_staking_with_balance_works_if_reserved_balance_has_been_slashed() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&1, amount);
		<Test as Config>::Currency::make_free_balance_be(&2, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(1),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));

		assert_eq!(<Test as Config>::Currency::total_balance(&2), amount);
		assert_eq!(<Test as Config>::Currency::free_balance(&2), amount);
		assert_ok!(Cml::start_staking(Origin::signed(2), cml_id, None, None));

		assert_eq!(<Test as Config>::Currency::total_balance(&2), amount);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&2),
			amount - STAKING_PRICE
		);

		let slashed_amount = STAKING_PRICE / 2;
		Utils::slash_reserved(&2, slashed_amount);

		assert_ok!(Cml::stop_staking(Origin::signed(2), cml_id, 1));
		assert_eq!(
			<Test as Config>::Currency::total_balance(&2),
			amount - slashed_amount
		);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&2),
			amount - slashed_amount
		);
	})
}

#[test]
fn stop_staking_with_cml_works() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&1, amount);

		let cml1_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml1_id, ());
		let mut cml1 = CML::from_genesis_seed(seed_from_lifespan(cml1_id, 100));
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

		assert!(!CmlStore::<Test>::get(cml2_id).is_staking());
		assert_ok!(Cml::start_staking(
			Origin::signed(2),
			cml1_id,
			Some(cml2_id),
			None
		));
		assert!(CmlStore::<Test>::get(cml2_id).is_staking());

		assert_ok!(Cml::stop_staking(Origin::signed(2), cml1_id, 1));
		assert!(!CmlStore::<Test>::get(cml2_id).is_staking());
	})
}

#[test]
fn stop_staking_works_with_mixed_staking_items() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		let user1 = 1;
		let user2 = 2;
		let user3 = 3;
		let user4 = 4;

		let cml1_id: CmlId = 4;
		UserCmlStore::<Test>::insert(user1, cml1_id, ());
		let cml1 = CML::from_genesis_seed(seed_from_lifespan(cml1_id, 100));
		CmlStore::<Test>::insert(cml1_id, cml1);
		assert_ok!(Cml::start_mining(
			Origin::signed(user1),
			cml1_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));

		let cml2_id: CmlId = 5;
		UserCmlStore::<Test>::insert(user2, cml2_id, ());
		let mut cml2 = CML::from_genesis_seed(new_genesis_seed(cml2_id));
		cml2.set_owner(&user2);
		CmlStore::<Test>::insert(cml2_id, cml2);
		assert_ok!(Cml::start_staking(
			Origin::signed(user2),
			cml1_id,
			Some(cml2_id),
			None
		));
		assert_eq!(
			CmlStore::<Test>::get(cml2_id).staking_index(),
			Some((cml1_id, 1))
		);

		<Test as Config>::Currency::make_free_balance_be(&user3, amount);
		assert_ok!(Cml::start_staking(
			Origin::signed(user3),
			cml1_id,
			None,
			None,
		));

		let cml4_id: CmlId = 6;
		UserCmlStore::<Test>::insert(user4, cml4_id, ());
		let mut cml4 = CML::from_genesis_seed(new_genesis_seed(cml4_id));
		cml4.set_owner(&user4);
		CmlStore::<Test>::insert(cml4_id, cml4);
		assert_ok!(Cml::start_staking(
			Origin::signed(user4),
			cml1_id,
			Some(cml4_id),
			None
		));
		assert_eq!(
			CmlStore::<Test>::get(cml4_id).staking_index(),
			Some((cml1_id, 3))
		);

		let staking_slots = CmlStore::<Test>::get(cml1_id).staking_slots().clone();
		assert_eq!(staking_slots.len(), 4);
		assert_eq!(staking_slots[0].owner, user1);
		assert_eq!(staking_slots[1].owner, user2);
		assert_eq!(staking_slots[2].owner, user3);
		assert_eq!(staking_slots[3].owner, user4);
		assert_eq!(
			CmlStore::<Test>::get(cml2_id).staking_index(),
			Some((cml1_id, 1))
		);
		assert_eq!(
			CmlStore::<Test>::get(cml4_id).staking_index(),
			Some((cml1_id, 3))
		);

		assert_ok!(Cml::stop_staking(Origin::signed(user3), cml1_id, 2)); // stop the balance staking item
		let staking_slots = CmlStore::<Test>::get(cml1_id).staking_slots().clone();
		assert_eq!(staking_slots.len(), 3);
		assert_eq!(staking_slots[0].owner, user1);
		assert_eq!(staking_slots[1].owner, user2);
		assert_eq!(staking_slots[2].owner, user4);
		assert_eq!(
			CmlStore::<Test>::get(cml2_id).staking_index(),
			Some((cml1_id, 1))
		);
		// cml4 staking index changed from 3 to 2
		assert_eq!(
			CmlStore::<Test>::get(cml4_id).staking_index(),
			Some((cml1_id, 2))
		);

		assert_ok!(Cml::stop_staking(Origin::signed(user2), cml1_id, 1));
		let staking_slots = CmlStore::<Test>::get(cml1_id).staking_slots().clone();
		assert_eq!(staking_slots.len(), 2);
		assert_eq!(staking_slots[0].owner, user1);
		assert_eq!(staking_slots[1].owner, user4);
		assert!(!CmlStore::<Test>::get(cml2_id).is_staking());
		// cml4 staking index changed from 2 to 1
		assert_eq!(
			CmlStore::<Test>::get(cml4_id).staking_index(),
			Some((cml1_id, 1))
		);
	})
}

#[test]
fn stop_first_slot_staking_should_fail() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&1, amount);
		<Test as Config>::Currency::make_free_balance_be(&2, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(1),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));
		assert_ok!(Cml::start_staking(Origin::signed(2), cml_id, None, None));

		assert_noop!(
			Cml::stop_staking(Origin::signed(1), cml_id, 0),
			Error::<Test>::CannotUnstakeTheFirstSlot
		);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&1),
			amount - STAKING_PRICE
		);
	})
}

#[test]
fn withdraw_staking_reward_works() {
	new_test_ext().execute_with(|| {
		let amount = 100;
		AccountRewards::<Test>::insert(1, amount);

		assert_eq!(Utils::free_balance(&1), 0);
		assert!(AccountRewards::<Test>::contains_key(&1));
		assert_eq!(AccountRewards::<Test>::get(&1), amount);

		assert_ok!(Cml::withdraw_staking_reward(Origin::signed(1)));

		assert_eq!(Utils::free_balance(&1), 100);
		assert!(!AccountRewards::<Test>::contains_key(&1));
	})
}

#[test]
fn withdraw_staking_reward_should_fail_if_user_not_have_reward() {
	new_test_ext().execute_with(|| {
		assert!(!AccountRewards::<Test>::contains_key(&1));
		assert_noop!(
			Cml::withdraw_staking_reward(Origin::signed(1)),
			Error::<Test>::NotFoundRewardAccount
		);
	})
}

#[test]
fn withdraw_staking_reward_should_work_if_there_is_credit() {
	new_test_ext().execute_with(|| {
		AccountRewards::<Test>::insert(1, 1000);
		GenesisMinerCreditStore::<Test>::insert(1, 1, 200);

		assert_ok!(Cml::withdraw_staking_reward(Origin::signed(1)));

		assert_eq!(Utils::free_balance(&1), 1000);
		assert!(!AccountRewards::<Test>::contains_key(&1));
	})
}

#[test]
fn pay_off_mining_credit_works() {
	new_test_ext().execute_with(|| {
		let user1 = 11;
		let free_balance = 10000;
		<Test as Config>::Currency::make_free_balance_be(&user1, free_balance);

		let credit_balance = 1000;
		let cml_id = 11;
		GenesisMinerCreditStore::<Test>::insert(user1, cml_id, credit_balance);

		assert_eq!(
			<Test as Config>::Currency::free_balance(&user1),
			free_balance
		);
		assert_eq!(<Test as Config>::Currency::reserved_balance(&user1), 0);

		assert_ok!(Cml::pay_off_mining_credit(Origin::signed(user1), cml_id));

		assert_eq!(
			<Test as Config>::Currency::free_balance(&user1),
			free_balance - credit_balance
		);
		assert_eq!(
			<Test as Config>::Currency::reserved_balance(&user1),
			credit_balance
		);
		assert!(!GenesisMinerCreditStore::<Test>::contains_key(
			&user1, cml_id
		));
	})
}

#[test]
fn pay_off_mining_credit_should_fail_if_there_is_no_credit() {
	new_test_ext().execute_with(|| {
		let user1 = 11;
		assert_eq!(
			GenesisMinerCreditStore::<Test>::iter_prefix(&user1).count(),
			0
		);

		assert_noop!(
			Cml::pay_off_mining_credit(Origin::signed(user1), 1),
			Error::<Test>::CmlNoNeedToPayOff
		);
	})
}

#[test]
fn pay_off_mining_credit_should_fail_if_not_enough_balance() {
	new_test_ext().execute_with(|| {
		let user1 = 11;
		let free_balance = 100;
		<Test as Config>::Currency::make_free_balance_be(&user1, free_balance);

		let credit_balance = 1000;
		let cml_id = 22;
		GenesisMinerCreditStore::<Test>::insert(user1, cml_id, credit_balance);

		assert!(free_balance < credit_balance);
		assert_noop!(
			Cml::pay_off_mining_credit(Origin::signed(user1), cml_id),
			Error::<Test>::InsufficientFreeBalance
		);
	})
}
