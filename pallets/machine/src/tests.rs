use crate::{
	mock::*, IssuerOwners, Issuers, MachineBindings, Machines, StartupMachineBindings,
	StartupTappBindings,
};
use frame_support::assert_ok;

#[test]
fn register_issuer_works() {
	new_test_ext().execute_with(|| {
		let issuer_owner = 3;

		assert_ok!(Machine::register_issuer(
			Origin::root(),
			issuer_owner,
			b"test ip".to_vec()
		));

		let issuer_id = 1;
		assert_eq!(Issuers::<Test>::get(issuer_id).owner, issuer_owner);
		assert_eq!(IssuerOwners::<Test>::get(issuer_owner), issuer_id);
	})
}

#[test]
fn register_machine_works() {
	new_test_ext().execute_with(|| {
		let issuer_owner = 3;

		assert_ok!(Machine::register_issuer(
			Origin::root(),
			issuer_owner,
			b"test ip".to_vec()
		));
		let issuer_id = 1;

		let user = 6;
		let tea_id = [1; 32];
		assert_ok!(Machine::register_machine(
			Origin::signed(issuer_owner),
			tea_id,
			user,
			issuer_id
		));

		assert!(Machines::<Test>::contains_key(tea_id));
		let machine = Machines::<Test>::get(tea_id);
		assert_eq!(machine.tea_id, tea_id);
		assert_eq!(machine.owner, user);
		assert_eq!(machine.issuer_id, issuer_id);
	})
}

#[test]
fn transfer_machine_works() {
	new_test_ext().execute_with(|| {
		let issuer_owner = 3;

		assert_ok!(Machine::register_issuer(
			Origin::root(),
			issuer_owner,
			b"test ip".to_vec()
		));
		let issuer_id = 1;

		let user = 6;
		let tea_id = [1; 32];
		assert_ok!(Machine::register_machine(
			Origin::signed(issuer_owner),
			tea_id,
			user,
			issuer_id
		));
		assert_eq!(Machines::<Test>::get(tea_id).owner, user);

		let user2 = 8;
		assert_ok!(Machine::transfer_machine(
			Origin::signed(user),
			tea_id,
			user2
		));
		assert_eq!(Machines::<Test>::get(tea_id).owner, user2);
	})
}

#[test]
fn register_for_layer2_works() {
	new_test_ext().execute_with(|| {
		let issuer_owner = 3;

		assert_ok!(Machine::register_issuer(
			Origin::root(),
			issuer_owner,
			b"test ip".to_vec()
		));
		let issuer_id = 1;

		let user = 6;
		let tea_id = [1; 32];
		assert_ok!(Machine::register_machine(
			Origin::signed(issuer_owner),
			tea_id,
			user,
			issuer_id
		));
		assert_eq!(Machines::<Test>::get(tea_id).owner, user);

		let cml_id = 111;
		assert_ok!(Machine::register_for_layer2(
			Origin::signed(user),
			tea_id,
			cml_id
		));
		assert_eq!(MachineBindings::<Test>::get(tea_id), cml_id);
	})
}

#[test]
fn reset_mining_startup_works() {
	new_test_ext().execute_with(|| {
		let tea_id1 = [1; 32];
		let tea_id2 = [2; 32];
		let cml_id1 = 111;
		let cml_id2 = 222;
		let conn_id1 = b"conn_id1".to_vec();
		let conn_id2 = b"conn_id2".to_vec();
		let ip1 = b"ip1".to_vec();
		let ip2 = b"ip2".to_vec();
		assert_ok!(Machine::reset_mining_startup(
			Origin::root(),
			vec![tea_id1, tea_id2],
			vec![cml_id1, cml_id2],
			vec![conn_id1.clone(), conn_id2.clone()],
			vec![ip1.clone(), ip2.clone()]
		));

		assert_eq!(MachineBindings::<Test>::get(tea_id1), cml_id1);
		assert_eq!(MachineBindings::<Test>::get(tea_id2), cml_id2);
		assert_eq!(
			StartupMachineBindings::<Test>::get(),
			vec![
				(tea_id1, cml_id1, conn_id1, ip1),
				(tea_id2, cml_id2, conn_id2, ip2)
			]
		);
	})
}

#[test]
fn reset_tapp_startup_works() {
	new_test_ext().execute_with(|| {
		let tea_id1 = [1; 32];
		let tea_id2 = [2; 32];
		let cml_id1 = 111;
		let cml_id2 = 222;
		let ip1 = b"test ip1".to_vec();
		let ip2 = b"test ip2".to_vec();
		assert_ok!(Machine::reset_tapp_startup(
			Origin::root(),
			vec![tea_id1, tea_id2],
			vec![cml_id1, cml_id2],
			vec![ip1.clone(), ip2.clone()]
		));

		assert_eq!(MachineBindings::<Test>::get(tea_id1), cml_id1);
		assert_eq!(MachineBindings::<Test>::get(tea_id2), cml_id2);
		assert_eq!(
			StartupTappBindings::<Test>::get(),
			vec![(tea_id1, cml_id1, ip1), (tea_id2, cml_id2, ip2)]
		);
	})
}
