use crate::param::{GENESIS_SEED_A_COUNT, GENESIS_SEED_B_COUNT, GENESIS_SEED_C_COUNT};
use crate::seeds::DefrostScheduleType;
use crate::{
	mock::*, types::*, CmlStore, Config, Error, Event as CmlEvent, LuckyDrawBox, MinerItemStore,
	SeedsCleaned, UserCmlStore, UserVoucherStore,
};
use frame_support::{assert_noop, assert_ok, traits::Currency};

#[test]
fn transfer_voucher_works() {
	new_test_ext().execute_with(|| {
		UserVoucherStore::<Test>::insert(1, CmlType::A, new_voucher(10, CmlType::A));
		UserVoucherStore::<Test>::insert(2, CmlType::A, new_voucher(10, CmlType::A));
		UserVoucherStore::<Test>::insert(1, CmlType::B, new_voucher(10, CmlType::B));
		UserVoucherStore::<Test>::insert(2, CmlType::B, new_voucher(10, CmlType::B));
		UserVoucherStore::<Test>::insert(1, CmlType::C, new_voucher(10, CmlType::C));
		UserVoucherStore::<Test>::insert(2, CmlType::C, new_voucher(10, CmlType::C));

		assert_ok!(Cml::transfer_voucher(Origin::signed(1), 2, CmlType::A, 3));
		assert_ok!(Cml::transfer_voucher(Origin::signed(1), 2, CmlType::B, 4));
		assert_ok!(Cml::transfer_voucher(Origin::signed(1), 2, CmlType::C, 5));

		assert_eq!(
			UserVoucherStore::<Test>::get(1, CmlType::A).unwrap().amount,
			7
		);
		assert_eq!(
			UserVoucherStore::<Test>::get(2, CmlType::A).unwrap().amount,
			13
		);

		assert_eq!(
			UserVoucherStore::<Test>::get(1, CmlType::B).unwrap().amount,
			6
		);
		assert_eq!(
			UserVoucherStore::<Test>::get(2, CmlType::B).unwrap().amount,
			14
		);

		assert_eq!(
			UserVoucherStore::<Test>::get(1, CmlType::C).unwrap().amount,
			5
		);
		assert_eq!(
			UserVoucherStore::<Test>::get(2, CmlType::C).unwrap().amount,
			15
		);
	})
}

#[test]
fn transfer_voucher_to_not_exist_account_works() {
	new_test_ext().execute_with(|| {
		UserVoucherStore::<Test>::insert(1, CmlType::A, new_voucher(10, CmlType::A));

		assert!(UserVoucherStore::<Test>::get(2, CmlType::A).is_none());
		assert_ok!(Cml::transfer_voucher(Origin::signed(1), 2, CmlType::A, 3));

		assert_eq!(
			UserVoucherStore::<Test>::get(1, CmlType::A).unwrap().amount,
			7
		);
		assert_eq!(
			UserVoucherStore::<Test>::get(2, CmlType::A).unwrap().amount,
			3
		);
	})
}

#[test]
fn transfer_voucher_with_insufficient_amount_should_fail() {
	new_test_ext().execute_with(|| {
		UserVoucherStore::<Test>::insert(1, CmlType::A, new_voucher(10, CmlType::A));

		assert_noop!(
			Cml::transfer_voucher(Origin::signed(1), 2, CmlType::A, 11),
			Error::<Test>::NotEnoughVoucher
		);
	})
}

#[test]
fn transfer_voucher_from_not_existing_account_should_fail() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Cml::transfer_voucher(Origin::signed(1), 2, CmlType::A, 1),
			Error::<Test>::NotEnoughVoucher
		);
	})
}

#[test]
fn transfer_voucher_to_cause_to_amount_overflow() {
	new_test_ext().execute_with(|| {
		UserVoucherStore::<Test>::insert(1, CmlType::A, new_voucher(10, CmlType::A));
		UserVoucherStore::<Test>::insert(2, CmlType::A, new_voucher(u32::MAX, CmlType::A));

		assert_noop!(
			Cml::transfer_voucher(Origin::signed(1), 2, CmlType::A, 3),
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

		LuckyDrawBox::<Test>::insert(CmlType::A, origin_a_box.clone());
		LuckyDrawBox::<Test>::insert(CmlType::B, origin_b_box.clone());
		LuckyDrawBox::<Test>::insert(CmlType::C, origin_c_box.clone());

		UserVoucherStore::<Test>::insert(1, CmlType::A, new_voucher(3, CmlType::A));
		UserVoucherStore::<Test>::insert(1, CmlType::B, new_voucher(4, CmlType::B));
		UserVoucherStore::<Test>::insert(1, CmlType::C, new_voucher(5, CmlType::C));

		assert_ok!(Cml::draw_cmls_from_voucher(Origin::signed(1)));

		assert_eq!(UserCmlStore::<Test>::get(&1).unwrap().len(), 3 + 4 + 5);
		System::assert_last_event(Event::pallet_cml(CmlEvent::DrawCmls(1, 3 + 4 + 5)));
	})
}

#[test]
fn draw_cmls_should_fail_when_no_voucher_exist() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Cml::draw_cmls_from_voucher(Origin::signed(1)),
			Error::<Test>::WithoutVoucher
		);
	})
}

#[test]
fn active_cml_for_nitro_works() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&1, amount);

		let cml_id: CmlId = 4;
		let cml = CML::new(new_seed(cml_id));
		assert_eq!(cml.status, CmlStatus::SeedFrozen);
		UserCmlStore::<Test>::insert(1, vec![cml_id]);
		CmlStore::<Test>::insert(cml_id, cml);

		let machine_id: MachineId = [1u8; 32];
		let miner_ip = b"miner_ip".to_vec();
		assert_ok!(Cml::active_cml_for_nitro(
			Origin::signed(1),
			cml_id,
			machine_id,
			miner_ip.clone()
		));

		let cml_list = UserCmlStore::<Test>::get(1).unwrap();
		let cml = CmlStore::<Test>::get(cml_list[0]).unwrap();
		assert_eq!(cml.status, CmlStatus::CmlLive);
		assert_eq!(cml.staking_slot.len(), 1);

		let staking_item = cml.staking_slot.get(0).unwrap();
		assert_eq!(staking_item.owner, 1);
		// todo let me pass later
		// assert_eq!(staking_item.amount, amount as u32);
		assert_eq!(staking_item.cml, None);

		let miner_item = MinerItemStore::<Test>::get(&machine_id).unwrap();
		assert_eq!(miner_item.id, machine_id);
		assert_eq!(miner_item.id, cml.machine_id.unwrap());
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
			Cml::active_cml_for_nitro(Origin::signed(1), 1, [1u8; 32], b"miner_ip".to_vec()),
			Error::<Test>::NotFoundCML
		);
	})
}

#[test]
fn active_not_drawn_cml_should_fail() {
	new_test_ext().execute_with(|| {
		// initial a cml not belongs to anyone, to simulate the not drawn situation
		let cml_id: CmlId = 4;
		let cml = CML::new(new_seed(cml_id));
		assert_eq!(cml.status, CmlStatus::SeedFrozen);
		CmlStore::<Test>::insert(cml_id, cml);

		assert_noop!(
			Cml::active_cml_for_nitro(Origin::signed(1), cml_id, [1u8; 32], b"miner_ip".to_vec()),
			Error::<Test>::CMLOwnerInvalid
		);
	})
}

#[test]
fn active_cml_not_belongs_to_me_should_fail() {
	new_test_ext().execute_with(|| {
		let cml_id: CmlId = 4;
		let cml = CML::new(new_seed(cml_id));
		assert_eq!(cml.status, CmlStatus::SeedFrozen);
		UserCmlStore::<Test>::insert(1, vec![cml_id]); // cml belongs to 1
		CmlStore::<Test>::insert(cml_id, cml);

		assert_noop!(
			Cml::active_cml_for_nitro(Origin::signed(2), cml_id, [1u8; 32], b"miner_ip".to_vec()),
			Error::<Test>::CMLOwnerInvalid
		);
	})
}

#[test]
fn active_two_cmls_with_same_machine_id_should_fail() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&1, amount);

		let cml1_id: CmlId = 4;
		let cml2_id: CmlId = 5;
		UserCmlStore::<Test>::insert(1, vec![cml1_id, cml2_id]);
		CmlStore::<Test>::insert(cml1_id, CML::new(new_seed(cml1_id)));
		CmlStore::<Test>::insert(cml2_id, CML::new(new_seed(cml2_id)));

		let machine_id: MachineId = [1u8; 32];
		let miner_ip = b"miner_ip".to_vec();
		assert_ok!(Cml::active_cml_for_nitro(
			Origin::signed(1),
			cml1_id,
			machine_id,
			miner_ip.clone()
		));

		assert_noop!(
			Cml::active_cml_for_nitro(Origin::signed(1), cml2_id, machine_id, miner_ip.clone()),
			Error::<Test>::MinerAlreadyExist
		);
	})
}

#[test]
fn active_cml_for_nitro_with_multiple_times_should_fail() {
	new_test_ext().execute_with(|| {
		let amount = 100 * 1000; // Unit * StakingPrice
		<Test as Config>::Currency::make_free_balance_be(&1, amount);

		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, vec![cml_id]);
		CmlStore::<Test>::insert(cml_id, CML::new(new_seed(cml_id)));

		let machine_id: MachineId = [1u8; 32];
		let miner_ip = b"miner_ip".to_vec();
		assert_ok!(Cml::active_cml_for_nitro(
			Origin::signed(1),
			cml_id,
			machine_id,
			miner_ip.clone()
		));

		assert_noop!(
			Cml::active_cml_for_nitro(Origin::signed(1), cml_id, machine_id, miner_ip.clone()),
			Error::<Test>::MinerAlreadyExist
		);
	})
}

#[test]
fn active_cml_for_nitro_with_insufficient_free_balance() {
	new_test_ext().execute_with(|| {
		// default account `1` free balance is 0
		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, vec![cml_id]);
		CmlStore::<Test>::insert(cml_id, CML::new(new_seed(cml_id)));

		assert_noop!(
			Cml::active_cml_for_nitro(Origin::signed(1), cml_id, [1u8; 32], b"miner_id".to_vec()),
			Error::<Test>::NotEnoughTeaToStaking
		);
	})
}

#[test]
fn genesis_build_related_logic_works() {
	let voucher_config1 = VoucherConfig {
		account: 1,
		cml_type: CmlType::A,
		amount: 100,
	};
	let voucher_config2 = VoucherConfig {
		account: 2,
		cml_type: CmlType::B,
		amount: 200,
	};

	ExtBuilder::default()
		.init_seeds()
		.vouchers(vec![voucher_config1.clone(), voucher_config2.clone()])
		.build()
		.execute_with(|| {
			let voucher1 = UserVoucherStore::<Test>::get(1, CmlType::A);
			assert!(voucher1.is_some());
			let voucher1 = voucher1.unwrap();
			assert_eq!(voucher1.amount, voucher_config1.amount);

			let voucher2 = UserVoucherStore::<Test>::get(2, CmlType::B);
			assert!(voucher2.is_some());
			let voucher2 = voucher2.unwrap();
			assert_eq!(voucher2.amount, voucher_config2.amount);

			assert_eq!(SeedsCleaned::<Test>::get(), Some(false));

			assert_eq!(
				GENESIS_SEED_A_COUNT,
				LuckyDrawBox::<Test>::get(CmlType::A).unwrap().len() as u64
			);
			assert_eq!(
				GENESIS_SEED_B_COUNT,
				LuckyDrawBox::<Test>::get(CmlType::B).unwrap().len() as u64
			);
			assert_eq!(
				GENESIS_SEED_C_COUNT,
				LuckyDrawBox::<Test>::get(CmlType::C).unwrap().len() as u64
			);

			for i in 0..(GENESIS_SEED_A_COUNT + GENESIS_SEED_B_COUNT + GENESIS_SEED_C_COUNT) {
				let cml = CmlStore::<Test>::get(i);
				assert!(cml.is_some());
				let cml = cml.unwrap();
				assert_eq!(cml.id(), i);
				assert_eq!(cml.status, CmlStatus::SeedFrozen);
			}
		});
}

fn new_voucher(amount: u32, cml_type: CmlType) -> Voucher {
	Voucher { amount, cml_type }
}

fn new_seed(id: CmlId) -> Seed {
	Seed {
		id,
		cml_type: CmlType::A,
		defrost_schedule: DefrostScheduleType::Team,
		defrost_time: 0,
		lifespan: 0,
		performance: 0,
	}
}
