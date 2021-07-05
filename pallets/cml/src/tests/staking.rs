use crate::tests::{new_genesis_seed, seed_from_lifespan};
use crate::{mock::*, types::*, AccountRewards, CmlStore, Config, Error, UserCmlStore};
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
			Some(cml2_id)
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
			Cml::start_staking(Origin::signed(2), cml_id, None),
			Error::<Test>::InsufficientFreeBalance
		);
	})
}

#[test]
fn start_staking_should_fail_if_miner_is_not_exist() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Cml::start_staking(Origin::signed(2), 1, None),
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

		assert!(!cml.can_be_stake(&0, &None, &Some(cml_id)));
		CmlStore::<Test>::insert(cml_id, cml);

		// for all kinds of invalid situation please see unit tests in `types/cml.rs`
		assert_noop!(
			Cml::start_staking(Origin::signed(2), cml_id, None),
			Error::<Test>::InvalidStakee
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
			Cml::start_staking(Origin::signed(2), miner_id, Some(2)), // cml id 2 is not exist
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
			Cml::start_staking(Origin::signed(user1), miner_id, Some(staker_id)),
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
			Cml::start_staking(Origin::signed(2), miner_id, Some(staker_id)),
			Error::<Test>::InvalidStaker
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
			<Test as Config>::Currency::make_free_balance_be(&i, amount);
			assert_ok!(Cml::start_staking(Origin::signed(i), cml_id, None));
		}

		<Test as Config>::Currency::make_free_balance_be(&STAKING_SLOTS_MAX_LENGTH, amount);
		assert_noop!(
			Cml::start_staking(Origin::signed(STAKING_SLOTS_MAX_LENGTH), cml_id, None),
			Error::<Test>::StakingSlotsOverTheMaxLength
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
			Some(cml2_id)
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
			Some(cml2_id)
		));
		assert_eq!(
			CmlStore::<Test>::get(cml2_id).staking_index(),
			Some((cml1_id, 1))
		);

		<Test as Config>::Currency::make_free_balance_be(&user3, amount);
		assert_ok!(Cml::start_staking(Origin::signed(user3), cml1_id, None,));

		let cml4_id: CmlId = 6;
		UserCmlStore::<Test>::insert(user4, cml4_id, ());
		let mut cml4 = CML::from_genesis_seed(new_genesis_seed(cml4_id));
		cml4.set_owner(&user4);
		CmlStore::<Test>::insert(cml4_id, cml4);
		assert_ok!(Cml::start_staking(
			Origin::signed(user4),
			cml1_id,
			Some(cml4_id)
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
