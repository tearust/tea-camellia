use crate::{mock::*, types::*, BuiltinNodes, Error, Nodes, RuntimeActivities};
use frame_support::{assert_noop, assert_ok};
use hex_literal::hex;

#[test]
fn add_new_node_works() {
    new_test_ext().execute_with(|| {
        let public: [u8; 32] =
            hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
        assert_ok!(Tea::add_new_node(Origin::signed(1), public));
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
        let _ = Tea::add_new_node(Origin::signed(1), public);

        assert_noop!(
            Tea::add_new_node(Origin::signed(1), public),
            Error::<Test>::NodeAlreadyExist
        );
    })
}

#[test]
fn builtin_node_update_node_profile_works() {
    new_test_ext().execute_with(|| {
        frame_system::Pallet::<Test>::set_block_number(100);

        let (node, tea_id, ephemeral_id, peer_id) = new_node();
        Nodes::<Test>::insert(&tea_id, node);
        BuiltinNodes::<Test>::insert(&tea_id, ());

        assert_ok!(Tea::update_node_profile(
            Origin::signed(1),
            tea_id.clone(),
            ephemeral_id.clone(),
            Vec::new(),
            Vec::new(),
            peer_id,
        ));
        assert!(Tea::is_builtin_node(&tea_id));

        let new_node = Nodes::<Test>::get(&tea_id).unwrap();
        assert_eq!(ephemeral_id, new_node.ephemeral_id);
        assert_eq!(NodeStatus::Active, new_node.status);
    })
}

#[test]
fn normal_node_update_node_profile_works() {
    new_test_ext().execute_with(|| {
        frame_system::Pallet::<Test>::set_block_number(100);

        let (node, tea_id, ephemeral_id, peer_id) = new_node();
        Nodes::<Test>::insert(&tea_id, node);

        assert_ok!(Tea::update_node_profile(
            Origin::signed(1),
            tea_id.clone(),
            ephemeral_id.clone(),
            Vec::new(),
            Vec::new(),
            peer_id,
        ));
        assert!(!Tea::is_builtin_node(&tea_id));

        let new_node = Nodes::<Test>::get(&tea_id).unwrap();
        assert_eq!(ephemeral_id, new_node.ephemeral_id);
        assert_eq!(NodeStatus::Pending, new_node.status);
    })
}

#[test]
fn update_node_profile_before_register_node() {
    new_test_ext().execute_with(|| {
        let (_, tea_id, ephemeral_id, peer_id) = new_node::<u64>();

        assert_noop!(
            Tea::update_node_profile(
                Origin::signed(1),
                tea_id.clone(),
                ephemeral_id.clone(),
                Vec::new(),
                Vec::new(),
                peer_id,
            ),
            Error::<Test>::NodeNotExist
        );
    })
}

#[test]
fn update_node_profile_with_empty_peer_id() {
    new_test_ext().execute_with(|| {
        let (node, tea_id, ephemeral_id, _) = new_node();
        Nodes::<Test>::insert(&tea_id, node);

        assert_noop!(
            Tea::update_node_profile(
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

#[test]
fn remote_attestation_works() {
    new_test_ext().execute_with(|| {
        let mut ra_nodes: Vec<(TeaPubKey, bool)> = Vec::new();

        let validator_1 = hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
        Nodes::<Test>::insert(&validator_1, Node::default());
        ra_nodes.push((validator_1.clone(), false));

        let validator_2 = hex!("c7e016fad0796bb68594e49a6ef1942cf7e73497e69edb32d19ba2fab3696596");
        Nodes::<Test>::insert(&validator_2, Node::default());
        ra_nodes.push((validator_2.clone(), false));

        let validator_3 = hex!("c9380fde1ba795fc656ab08ab4ef4482cf554790fd3abcd4642418ae8fb5fd52");
        Nodes::<Test>::insert(&validator_3, Node::default());
        ra_nodes.push((validator_3.clone(), false));

        let validator_4 = hex!("2754d7e9c73ced5b302e12464594110850980027f8f83c469e8145eef59220b6");
        Nodes::<Test>::insert(&validator_4, Node::default());
        ra_nodes.push((validator_4.clone(), false));

        let (mut node, tea_id, _, _) = new_node();
        node.ra_nodes = ra_nodes;
        Nodes::<Test>::insert(&tea_id, node);

        assert_ok!(Tea::remote_attestation(
            Origin::signed(1),
            validator_1,
            tea_id.clone(),
            true,
            Vec::new()
        ));
        assert_eq!(
            Nodes::<Test>::get(&tea_id).unwrap().status,
            NodeStatus::Pending
        );

        assert_ok!(Tea::remote_attestation(
            Origin::signed(1),
            validator_2,
            tea_id.clone(),
            true,
            Vec::new()
        ));
        assert_eq!(
            Nodes::<Test>::get(&tea_id).unwrap().status,
            NodeStatus::Pending
        );

        assert_ok!(Tea::remote_attestation(
            Origin::signed(1),
            validator_3,
            tea_id.clone(),
            true,
            Vec::new()
        ));
        assert_eq!(
            Nodes::<Test>::get(&tea_id).unwrap().status,
            NodeStatus::Active
        );

        // the 4th validator commit should see a `NodeAlreadyActive` error, this is ok because
        // the apply node already work well.
        assert_noop!(
            Tea::remote_attestation(
                Origin::signed(1),
                validator_4,
                tea_id.clone(),
                true,
                Vec::new()
            ),
            Error::<Test>::NodeAlreadyActive
        );
        assert_eq!(
            Nodes::<Test>::get(&tea_id).unwrap().status,
            NodeStatus::Active
        );
    })
}

#[test]
fn ra_node_not_exist() {
    // validator node not exist
    new_test_ext().execute_with(|| {
        let (node, tea_id, _, _) = new_node();
        Nodes::<Test>::insert(&tea_id, node);

        let validator_tea_id =
            hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");

        assert_noop!(
            Tea::remote_attestation(
                Origin::signed(1),
                validator_tea_id,
                tea_id,
                true,
                Vec::new()
            ),
            Error::<Test>::NodeNotExist
        );
    });

    // apply node not exist
    new_test_ext().execute_with(|| {
        let (_, tea_id, _, _) = new_node::<u32>();

        let validator_tea_id =
            hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
        Nodes::<Test>::insert(&validator_tea_id, Node::default());

        assert_noop!(
            Tea::remote_attestation(
                Origin::signed(1),
                validator_tea_id,
                tea_id,
                true,
                Vec::new()
            ),
            Error::<Test>::ApplyNodeNotExist
        );
    });
}

#[test]
fn node_already_active() {
    new_test_ext().execute_with(|| {
        let (mut node, tea_id, _, _) = new_node();
        node.status = NodeStatus::Active;
        Nodes::<Test>::insert(&tea_id, node);

        let validator_tea_id =
            hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
        Nodes::<Test>::insert(&validator_tea_id, Node::default());

        assert_noop!(
            Tea::remote_attestation(
                Origin::signed(1),
                validator_tea_id,
                tea_id,
                true,
                Vec::new()
            ),
            Error::<Test>::NodeAlreadyActive
        );
    })
}

#[test]
fn validator_not_in_ra_nodes() {
    new_test_ext().execute_with(|| {
        let (mut node, tea_id, _, _) = new_node();
        node.ra_nodes = Vec::new(); // validator tea id not in ra node list
        Nodes::<Test>::insert(&tea_id, node);

        let validator_tea_id =
            hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
        Nodes::<Test>::insert(&validator_tea_id, Node::default());

        assert_noop!(
            Tea::remote_attestation(
                Origin::signed(1),
                validator_tea_id,
                tea_id,
                true,
                Vec::new()
            ),
            Error::<Test>::NotInRaNodes
        );
    })
}

#[test]
fn update_node_status_works() {
    let index_to_pub_key = |i: u8| [i; 32];

    // test normal ra procedure
    new_test_ext().execute_with(|| {
        let node_index = 4u8;
        let mut node = Node::default();
        for i in 0..=3 {
            node.ra_nodes.push((index_to_pub_key(i), false));
        }
        Nodes::<Test>::insert(index_to_pub_key(node_index), node.clone());
        assert_eq!(node.status, NodeStatus::Pending);

        for i in 0..=1 {
            assert_eq!(
                Tea::update_node_status(&index_to_pub_key(node_index), i, true),
                NodeStatus::Pending
            );
            assert_eq!(
                Nodes::<Test>::get(&index_to_pub_key(node_index))
                    .unwrap()
                    .status,
                NodeStatus::Pending
            );
        }

        for i in 2..=3 {
            assert_eq!(
                Tea::update_node_status(&index_to_pub_key(node_index), i, true),
                NodeStatus::Active
            );
            assert_eq!(
                Nodes::<Test>::get(&index_to_pub_key(node_index))
                    .unwrap()
                    .status,
                NodeStatus::Active
            );
        }
    });

    // test node status become invalid
    new_test_ext().execute_with(|| {
        let node_index = 4u8;
        let mut node = Node::default();
        for i in 1..=4 {
            node.ra_nodes.push((index_to_pub_key(i), false));
        }
        Nodes::<Test>::insert(index_to_pub_key(node_index), node);

        assert_eq!(
            Tea::update_node_status(&index_to_pub_key(node_index), 0, false),
            NodeStatus::Invalid
        );
        assert_eq!(
            Nodes::<Test>::get(&index_to_pub_key(node_index))
                .unwrap()
                .status,
            NodeStatus::Invalid
        );

        // node status should be invalid even if the rest of nodes (total count >= 3/4) agreed
        for i in 1..=3 {
            assert_eq!(
                Tea::update_node_status(&index_to_pub_key(node_index), i, false),
                NodeStatus::Invalid
            );
            assert_eq!(
                Tea::update_node_status(&index_to_pub_key(node_index), i, false),
                NodeStatus::Invalid
            );
        }
    });
}

#[test]
fn update_runtime_activity_works() {
    use ed25519_dalek::ed25519::signature::Signature;
    use ed25519_dalek::{Keypair, Signer};
    use rand::rngs::OsRng;

    new_test_ext().execute_with(|| {
        let (node, tea_id, _, _) = new_node();
        Nodes::<Test>::insert(&tea_id, node);

        let mut csprng = OsRng {};
        let kp = Keypair::generate(&mut csprng);
        let signature = kp.sign(&tea_id);

        assert_ok!(Tea::update_runtime_activity(
            Origin::signed(1),
            tea_id,
            None,
            kp.public.as_bytes().clone(),
            signature.as_bytes().to_vec(),
        ));
    })
}

#[test]
fn update_runtime_activity_when_node_registered() {
    new_test_ext().execute_with(|| {
        let (_, tea_id, ephemeral_id, _) = new_node::<u32>();

        assert_noop!(
            Tea::update_runtime_activity(
                Origin::signed(1),
                tea_id,
                None,
                ephemeral_id,
                vec![0u8; 64],
            ),
            Error::<Test>::NodeNotExist
        );
    })
}

#[test]
fn verify_ed25519_signature_works() {
    use ed25519_dalek::ed25519::signature::Signature;
    use ed25519_dalek::{Keypair, Signer};
    use rand::rngs::OsRng;

    new_test_ext().execute_with(|| {
        let tea_id = [3u8; 32];
        let mut csprng = OsRng {};
        let kp = Keypair::generate(&mut csprng);
        let signature = kp.sign(&tea_id);

        assert!(kp.verify(&tea_id, &signature).is_ok());
        assert_ok!(Tea::verify_ed25519_signature(
            &kp.public.as_bytes(),
            &tea_id,
            &signature.as_bytes().to_vec(),
        ));

        assert_noop!(
            Tea::verify_ed25519_signature(
                &kp.public.as_bytes(),
                &tea_id,
                &vec![0u8; 33], // wrong signature length
            ),
            Error::<Test>::InvalidSignatureLength
        );

        let wrong_message = [2u8; 32];
        assert!(kp.verify(&wrong_message, &signature).is_err());
        assert_noop!(
            Tea::verify_ed25519_signature(
                &kp.public.as_bytes(),
                &wrong_message,
                &signature.as_bytes().to_vec(),
            ),
            Error::<Test>::InvalidSignature
        );
    })
}

#[test]
fn update_runtime_status_works() {
    // without activity record
    new_test_ext().execute_with(|| {
        let initial_height = 10;
        let threshold_height = RUNTIME_ACTIVITY_THRESHOLD as u64;

        let tea_id1: TeaPubKey = [1; 32];
        let mut node1 = Node::default();
        node1.update_time = initial_height;
        node1.status = NodeStatus::Active;
        Nodes::<Test>::insert(&tea_id1, node1);

        let tea_id2: TeaPubKey = [2; 32];
        let mut node2 = Node::default();
        node2.update_time = initial_height + 1;
        node2.status = NodeStatus::Active;
        Nodes::<Test>::insert(&tea_id2, node2);

        Tea::update_runtime_status(initial_height + 2);
        assert_eq!(
            Nodes::<Test>::get(&tea_id1).unwrap().status,
            NodeStatus::Active
        );
        assert_eq!(
            Nodes::<Test>::get(&tea_id2).unwrap().status,
            NodeStatus::Active
        );

        Tea::update_runtime_status(initial_height + threshold_height);
        assert_eq!(
            Nodes::<Test>::get(&tea_id1).unwrap().status,
            NodeStatus::Active
        );
        assert_eq!(
            Nodes::<Test>::get(&tea_id2).unwrap().status,
            NodeStatus::Active
        );

        Tea::update_runtime_status(initial_height + threshold_height + 1);
        assert_eq!(
            Nodes::<Test>::get(&tea_id1).unwrap().status,
            NodeStatus::Inactive
        );
        assert_eq!(
            Nodes::<Test>::get(&tea_id2).unwrap().status,
            NodeStatus::Active
        );

        Tea::update_runtime_status(initial_height + threshold_height + 2);
        assert_eq!(
            Nodes::<Test>::get(&tea_id1).unwrap().status,
            NodeStatus::Inactive
        );
        assert_eq!(
            Nodes::<Test>::get(&tea_id2).unwrap().status,
            NodeStatus::Inactive
        );
    });

    // has activity record, and `update_height` of the recode is one block after `update_time` of
    // the node
    new_test_ext().execute_with(|| {
        let initial_height = 10;
        let threshold_height = RUNTIME_ACTIVITY_THRESHOLD as u64;

        let tea_id1: TeaPubKey = [1; 32];
        let mut node1 = Node::default();
        node1.update_time = initial_height;
        node1.status = NodeStatus::Active;
        Nodes::<Test>::insert(&tea_id1, node1);
        RuntimeActivities::<Test>::insert(
            &tea_id1,
            RuntimeActivity {
                tea_id: tea_id1.clone(),
                cid: None,
                ephemeral_id: [0; 32],
                update_height: initial_height + 1,
            },
        );

        let tea_id2: TeaPubKey = [2; 32];
        let mut node2 = Node::default();
        node2.update_time = initial_height + 1;
        node2.status = NodeStatus::Active;
        Nodes::<Test>::insert(&tea_id2, node2);
        RuntimeActivities::<Test>::insert(
            &tea_id2,
            RuntimeActivity {
                tea_id: tea_id2.clone(),
                cid: None,
                ephemeral_id: [0; 32],
                update_height: initial_height + 2,
            },
        );

        Tea::update_runtime_status(initial_height + 2);
        assert_eq!(
            Nodes::<Test>::get(&tea_id1).unwrap().status,
            NodeStatus::Active
        );
        assert_eq!(
            Nodes::<Test>::get(&tea_id2).unwrap().status,
            NodeStatus::Active
        );

        Tea::update_runtime_status(initial_height + threshold_height);
        assert_eq!(
            Nodes::<Test>::get(&tea_id1).unwrap().status,
            NodeStatus::Active
        );
        assert_eq!(
            Nodes::<Test>::get(&tea_id2).unwrap().status,
            NodeStatus::Active
        );

        Tea::update_runtime_status(initial_height + threshold_height + 1);
        assert_eq!(
            Nodes::<Test>::get(&tea_id1).unwrap().status,
            NodeStatus::Active
        );
        assert_eq!(
            Nodes::<Test>::get(&tea_id2).unwrap().status,
            NodeStatus::Active
        );

        Tea::update_runtime_status(initial_height + threshold_height + 2);
        assert_eq!(
            Nodes::<Test>::get(&tea_id1).unwrap().status,
            NodeStatus::Inactive
        );
        assert_eq!(
            Nodes::<Test>::get(&tea_id2).unwrap().status,
            NodeStatus::Active
        );

        Tea::update_runtime_status(initial_height + threshold_height + 3);
        assert_eq!(
            Nodes::<Test>::get(&tea_id1).unwrap().status,
            NodeStatus::Inactive
        );
        assert_eq!(
            Nodes::<Test>::get(&tea_id2).unwrap().status,
            NodeStatus::Inactive
        );
    });
}

fn new_node<T>() -> (Node<T>, TeaPubKey, TeaPubKey, PeerId)
where
    T: Default,
{
    let tea_id = hex!("df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b");
    let ephemeral_id = hex!("ba9147ba50faca694452db7c458e33a9a0322acbaac24bf35db7bb5165dff3ac");
    let peer_id = "12D3KooWLCU9sscGSP7GySktL2awwNouPwrqvZECLaDafpwLKKvt";

    let mut node = Node::default();
    node.tea_id = tea_id.clone();
    (node, tea_id, ephemeral_id, peer_id.as_bytes().to_vec())
}
