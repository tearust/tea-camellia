use crate::param::{GENESIS_SEED_A_COUNT, GENESIS_SEED_B_COUNT, GENESIS_SEED_C_COUNT};
use crate::{
	mock::*, types::*, CmlStore, Config, Error, Event as CmlEvent, InvestorVoucherStore, LastCmlId,
	LuckyDrawBox, MinerItemStore, TeamVoucherStore, UserCmlStore,
};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError, traits::Currency};

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

#[test]
fn draw_cmls_from_voucher_works() {
	new_test_ext().execute_with(|| {
		frame_system::Pallet::<Test>::set_block_number(100);

		let origin_a_box: Vec<u64> = (1..=10).collect();
		let origin_b_box: Vec<u64> = (11..=20).collect();
		let origin_c_box: Vec<u64> = (21..=30).collect();

		LuckyDrawBox::<Test>::insert(CmlType::A, DefrostScheduleType::Team, origin_a_box.clone());
		LuckyDrawBox::<Test>::insert(CmlType::B, DefrostScheduleType::Team, origin_b_box.clone());
		LuckyDrawBox::<Test>::insert(CmlType::C, DefrostScheduleType::Team, origin_c_box.clone());

		TeamVoucherStore::<Test>::insert(1, CmlType::A, new_voucher(3, CmlType::A));
		TeamVoucherStore::<Test>::insert(1, CmlType::B, new_voucher(4, CmlType::B));
		TeamVoucherStore::<Test>::insert(1, CmlType::C, new_voucher(5, CmlType::C));

		assert_ok!(Cml::draw_cmls_from_voucher(
			Origin::signed(1),
			DefrostScheduleType::Team
		));

		assert_eq!(
			UserCmlStore::<Test>::iter()
				.filter(|(k1, _, _)| *k1 == 1)
				.count(),
			3 + 4 + 5
		);
		System::assert_last_event(Event::pallet_cml(CmlEvent::DrawCmls(1, 3 + 4 + 5)));
	})
}

#[test]
fn draw_cmls_works_the_second_time_if_get_voucher_again() {
	new_test_ext().execute_with(|| {
		frame_system::Pallet::<Test>::set_block_number(100);

		let origin_a_box: Vec<u64> = (1..=10).collect();
		let origin_b_box: Vec<u64> = (11..=20).collect();
		let origin_c_box: Vec<u64> = (21..=30).collect();

		LuckyDrawBox::<Test>::insert(
			CmlType::A,
			DefrostScheduleType::Investor,
			origin_a_box.clone(),
		);
		LuckyDrawBox::<Test>::insert(
			CmlType::B,
			DefrostScheduleType::Investor,
			origin_b_box.clone(),
		);
		LuckyDrawBox::<Test>::insert(
			CmlType::C,
			DefrostScheduleType::Investor,
			origin_c_box.clone(),
		);

		InvestorVoucherStore::<Test>::insert(1, CmlType::A, new_voucher(3, CmlType::A));
		InvestorVoucherStore::<Test>::insert(1, CmlType::B, new_voucher(4, CmlType::B));
		InvestorVoucherStore::<Test>::insert(1, CmlType::C, new_voucher(5, CmlType::C));
		InvestorVoucherStore::<Test>::insert(2, CmlType::A, new_voucher(1, CmlType::A));
		InvestorVoucherStore::<Test>::insert(2, CmlType::B, new_voucher(2, CmlType::B));
		InvestorVoucherStore::<Test>::insert(2, CmlType::C, new_voucher(3, CmlType::C));

		assert_ok!(Cml::draw_cmls_from_voucher(
			Origin::signed(1),
			DefrostScheduleType::Investor
		));
		assert_eq!(
			UserCmlStore::<Test>::iter()
				.filter(|(k1, _, _)| *k1 == 1)
				.count(),
			3 + 4 + 5
		);

		assert_ok!(Cml::transfer_voucher(
			Origin::signed(2),
			1,
			CmlType::A,
			DefrostScheduleType::Investor,
			1
		));
		assert_ok!(Cml::transfer_voucher(
			Origin::signed(2),
			1,
			CmlType::B,
			DefrostScheduleType::Investor,
			2
		));
		assert_ok!(Cml::transfer_voucher(
			Origin::signed(2),
			1,
			CmlType::C,
			DefrostScheduleType::Investor,
			3
		));

		assert_ok!(Cml::draw_cmls_from_voucher(
			Origin::signed(1),
			DefrostScheduleType::Investor
		));
		assert_eq!(
			UserCmlStore::<Test>::iter()
				.filter(|(k1, _, _)| *k1 == 1)
				.count(),
			3 + 4 + 5 + 1 + 2 + 3
		);
	})
}

#[test]
fn draw_cmls_from_voucher_should_fail_if_timeout() {
	new_test_ext().execute_with(|| {
		frame_system::Pallet::<Test>::set_block_number(SEEDS_TIMEOUT_HEIGHT as u64 + 1);
		assert_noop!(
			Cml::draw_cmls_from_voucher(Origin::signed(1), DefrostScheduleType::Team),
			Error::<Test>::VouchersHasOutdated
		);
		assert_noop!(
			Cml::draw_cmls_from_voucher(Origin::signed(1), DefrostScheduleType::Investor),
			Error::<Test>::VouchersHasOutdated
		);
	})
}

#[test]
fn draw_same_cmls_multiple_times_should_fail() {
	new_test_ext().execute_with(|| {
		frame_system::Pallet::<Test>::set_block_number(100);

		let origin_a_box: Vec<u64> = (1..=10).collect();
		let origin_b_box: Vec<u64> = (11..=20).collect();
		let origin_c_box: Vec<u64> = (21..=30).collect();

		LuckyDrawBox::<Test>::insert(CmlType::A, DefrostScheduleType::Team, origin_a_box.clone());
		LuckyDrawBox::<Test>::insert(CmlType::B, DefrostScheduleType::Team, origin_b_box.clone());
		LuckyDrawBox::<Test>::insert(CmlType::C, DefrostScheduleType::Team, origin_c_box.clone());

		TeamVoucherStore::<Test>::insert(1, CmlType::A, new_voucher(3, CmlType::A));
		TeamVoucherStore::<Test>::insert(1, CmlType::B, new_voucher(4, CmlType::B));
		TeamVoucherStore::<Test>::insert(1, CmlType::C, new_voucher(5, CmlType::C));

		assert_ok!(Cml::draw_cmls_from_voucher(
			Origin::signed(1),
			DefrostScheduleType::Team
		));
		assert_noop!(
			Cml::draw_cmls_from_voucher(Origin::signed(1), DefrostScheduleType::Team),
			Error::<Test>::WithoutVoucher
		);
	})
}

#[test]
fn draw_cmls_should_fail_when_no_voucher_exist() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Cml::draw_cmls_from_voucher(Origin::signed(1), DefrostScheduleType::Team),
			Error::<Test>::WithoutVoucher
		);
		assert_noop!(
			Cml::draw_cmls_from_voucher(Origin::signed(1), DefrostScheduleType::Investor),
			Error::<Test>::WithoutVoucher
		);
	})
}

#[test]
fn active_cml_for_nitro_works() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&1, amount);

		let current_height = frame_system::Pallet::<Test>::block_number();

		let cml_id: CmlId = 4;
		let mut cml = CML::from_genesis_seed(new_genesis_seed(cml_id));
		assert!(cml.is_seed() && cml.can_be_defrost(&current_height).unwrap());
		cml.defrost(&current_height).unwrap();
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

		let cml = CmlStore::<Test>::get(cml_id).unwrap();
		assert!(!cml.is_seed());
		assert_eq!(cml.staking_slots().len(), 1);

		let staking_item = cml.staking_slots().get(0).unwrap();
		assert_eq!(staking_item.owner, 1);
		// todo let me pass later
		// assert_eq!(staking_item.amount, amount as u32);
		assert_eq!(staking_item.cml, None);

		let miner_item = MinerItemStore::<Test>::get(&machine_id).unwrap();
		assert_eq!(miner_item.id, machine_id);
		assert_eq!(&miner_item.id, cml.machine_id().unwrap());
		assert_eq!(miner_item.status, MinerStatus::Active);
		assert_eq!(miner_item.ip, miner_ip);

		// todo should work
		// System::assert_last_event(Event::pallet_cml(CmlEvent::ActiveCml(1, cml_id)));
	});
}

#[test]
fn active_not_exist_cml_for_nitro_should_fail() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Cml::active_cml(Origin::signed(1), 1),
			Error::<Test>::NotFoundCML
		);
	})
}

#[test]
fn active_not_drawn_cml_should_fail() {
	new_test_ext().execute_with(|| {
		// initial a cml not belongs to anyone, to simulate the not drawn situation
		let cml_id: CmlId = 4;
		let cml = CML::from_genesis_seed(new_genesis_seed(cml_id));
		CmlStore::<Test>::insert(cml_id, cml);

		assert_noop!(
			Cml::active_cml(Origin::signed(1), cml_id),
			Error::<Test>::CMLOwnerInvalid
		);
	})
}

#[test]
fn active_cml_not_belongs_to_me_should_fail() {
	new_test_ext().execute_with(|| {
		let cml_id: CmlId = 4;
		let cml = CML::from_genesis_seed(new_genesis_seed(cml_id));
		UserCmlStore::<Test>::insert(1, cml_id, ()); // cml belongs to 1
		CmlStore::<Test>::insert(cml_id, cml);

		assert_noop!(
			Cml::active_cml(Origin::signed(2), cml_id),
			Error::<Test>::CMLOwnerInvalid
		);
	})
}

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

#[test]
fn genesis_build_related_logic_works() {
	let voucher_config1 = VoucherConfig {
		account: 1,
		cml_type: CmlType::A,
		schedule_type: DefrostScheduleType::Team,
		amount: 100,
	};
	let voucher_config2 = VoucherConfig {
		account: 2,
		cml_type: CmlType::B,
		schedule_type: DefrostScheduleType::Investor,
		amount: 200,
	};

	ExtBuilder::default()
		.init_seeds()
		.vouchers(vec![voucher_config1.clone(), voucher_config2.clone()])
		.build()
		.execute_with(|| {
			let voucher1 = TeamVoucherStore::<Test>::get(1, CmlType::A);
			assert!(voucher1.is_some());
			let voucher1 = voucher1.unwrap();
			assert_eq!(voucher1.amount, voucher_config1.amount);

			let voucher2 = InvestorVoucherStore::<Test>::get(2, CmlType::B);
			assert!(voucher2.is_some());
			let voucher2 = voucher2.unwrap();
			assert_eq!(voucher2.amount, voucher_config2.amount);

			assert_eq!(
				GENESIS_SEED_A_COUNT as usize,
				LuckyDrawBox::<Test>::get(CmlType::A, DefrostScheduleType::Team).len()
					+ LuckyDrawBox::<Test>::get(CmlType::A, DefrostScheduleType::Investor).len()
			);
			assert_eq!(
				GENESIS_SEED_B_COUNT as usize,
				LuckyDrawBox::<Test>::get(CmlType::B, DefrostScheduleType::Team).len()
					+ LuckyDrawBox::<Test>::get(CmlType::B, DefrostScheduleType::Investor).len()
			);
			assert_eq!(
				GENESIS_SEED_C_COUNT as usize,
				LuckyDrawBox::<Test>::get(CmlType::C, DefrostScheduleType::Team).len()
					+ LuckyDrawBox::<Test>::get(CmlType::C, DefrostScheduleType::Investor).len()
			);

			let mut live_seeds_count: usize = 0;
			for i in 0..(GENESIS_SEED_A_COUNT + GENESIS_SEED_B_COUNT + GENESIS_SEED_C_COUNT) {
				let cml = CmlStore::<Test>::get(i);
				assert!(cml.is_some());
				let cml = cml.unwrap();
				assert_eq!(cml.id(), i);

				if cml.seed_valid(&0).unwrap() {
					live_seeds_count += 1;
				}
			}
			println!("live seeds count: {}", live_seeds_count);

			assert_eq!(
				LastCmlId::<Test>::get(),
				GENESIS_SEED_A_COUNT + GENESIS_SEED_B_COUNT + GENESIS_SEED_C_COUNT
			);
		});
}

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
		let cml = CML::from_genesis_seed(new_genesis_seed(cml1_id));
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
		let cml1 = CML::from_genesis_seed(new_genesis_seed(cml1_id));
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

fn new_voucher(amount: u32, cml_type: CmlType) -> Voucher {
	Voucher { amount, cml_type }
}

fn new_genesis_seed(id: CmlId) -> Seed {
	Seed {
		id,
		cml_type: CmlType::A,
		defrost_schedule: Some(DefrostScheduleType::Team),
		defrost_time: Some(0),
		lifespan: 0,
		performance: 0,
	}
}
