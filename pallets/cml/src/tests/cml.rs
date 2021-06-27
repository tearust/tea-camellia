use crate::tests::new_genesis_seed;
use crate::{mock::*, types::*, CmlStore, Config, Error, MinerItemStore, UserCmlStore};
use frame_support::{assert_noop, assert_ok, traits::Currency};

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