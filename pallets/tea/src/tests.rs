use crate::{mock::*, types::*, BuiltinNodes, Error, Nodes};
use frame_support::{assert_noop, assert_ok};
use hex_literal::hex;

#[test]
fn add_new_node_works() {
    new_test_ext().execute_with(|| {
        let public: [u8; 32] =
            hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
        assert_ok!(TeaModule::add_new_node(Origin::signed(1), public));
        let target_node = Nodes::<Test>::get(&public).unwrap();
        assert_eq!(
            target_node.create_time,
            frame_system::Pallet::<Test>::block_number()
        );
    })
}

#[test]
fn add_new_node_already_exist() {
    new_test_ext().execute_with(|| {
        let public: [u8; 32] =
            hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
        let _ = TeaModule::add_new_node(Origin::signed(1), public);

        assert_noop!(
            TeaModule::add_new_node(Origin::signed(1), public),
            Error::<Test>::NodeAlreadyExist
        );
    })
}

#[test]
fn builtin_node_update_node_profile_works() {
    new_test_ext().execute_with(|| {
        frame_system::Pallet::<Test>::set_block_number(100);

        let tea_id = hex!("df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b");
        let ephemeral_id = hex!("ba9147ba50faca694452db7c458e33a9a0322acbaac24bf35db7bb5165dff3ac");
        let peer_id = "12D3KooWLCU9sscGSP7GySktL2awwNouPwrqvZECLaDafpwLKKvt";

        let mut node = Node::default();
        node.tea_id = tea_id.clone();
        Nodes::<Test>::insert(&tea_id, node);
        BuiltinNodes::<Test>::insert(&tea_id, &tea_id);

        assert_ok!(TeaModule::update_node_profile(
            Origin::signed(1),
            tea_id.clone(),
            ephemeral_id.clone(),
            Vec::new(),
            Vec::new(),
            peer_id.as_bytes().to_vec(),
        ));
        assert!(TeaModule::is_builtin_node(&tea_id));

        let new_node = Nodes::<Test>::get(&tea_id).unwrap();
        assert_eq!(ephemeral_id, new_node.ephemeral_id);
        assert_eq!(NodeStatus::Active, new_node.status);
    })
}

#[test]
fn normal_node_update_node_profile_works() {
    new_test_ext().execute_with(|| {
        frame_system::Pallet::<Test>::set_block_number(100);

        let tea_id = hex!("df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b");
        let ephemeral_id = hex!("ba9147ba50faca694452db7c458e33a9a0322acbaac24bf35db7bb5165dff3ac");
        let peer_id = "12D3KooWLCU9sscGSP7GySktL2awwNouPwrqvZECLaDafpwLKKvt";

        let mut node = Node::default();
        node.tea_id = tea_id.clone();
        Nodes::<Test>::insert(&tea_id, node);

        assert_ok!(TeaModule::update_node_profile(
            Origin::signed(1),
            tea_id.clone(),
            ephemeral_id.clone(),
            Vec::new(),
            Vec::new(),
            peer_id.as_bytes().to_vec(),
        ));
        assert!(!TeaModule::is_builtin_node(&tea_id));

        let new_node = Nodes::<Test>::get(&tea_id).unwrap();
        assert_eq!(ephemeral_id, new_node.ephemeral_id);
        assert_eq!(NodeStatus::Pending, new_node.status);
    })
}

#[test]
fn update_node_profile_before_register_node() {
    new_test_ext().execute_with(|| {
        let tea_id = hex!("df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b");
        let ephemeral_id = hex!("ba9147ba50faca694452db7c458e33a9a0322acbaac24bf35db7bb5165dff3ac");
        let peer_id = "12D3KooWLCU9sscGSP7GySktL2awwNouPwrqvZECLaDafpwLKKvt";

        assert_noop!(
            TeaModule::update_node_profile(
                Origin::signed(1),
                tea_id.clone(),
                ephemeral_id.clone(),
                Vec::new(),
                Vec::new(),
                peer_id.as_bytes().to_vec(),
            ),
            Error::<Test>::NodeNotExist
        );
    })
}

#[test]
fn update_node_profile_with_empty_peer_id() {
    new_test_ext().execute_with(|| {
        let tea_id = hex!("df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b");
        let ephemeral_id = hex!("ba9147ba50faca694452db7c458e33a9a0322acbaac24bf35db7bb5165dff3ac");

        let node = Node::default();
        Nodes::<Test>::insert(&tea_id, node);

        assert_noop!(
            TeaModule::update_node_profile(
                Origin::signed(1),
                tea_id.clone(),
                ephemeral_id.clone(),
                Vec::new(),
                Vec::new(),
                Vec::new(),
            ),
            Error::<Test>::InvalidPeerId
        );
    })
}
