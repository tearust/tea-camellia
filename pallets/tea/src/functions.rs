use super::*;

impl<T: tea::Config> tea::Pallet<T> {
    pub(crate) fn pop_existing_node(tea_id: &TeaPubKey) -> Node<T::BlockNumber> {
        let old_node = Nodes::<T>::get(tea_id).unwrap();
        BootNodes::<T>::remove(&old_node.tea_id);
        EphemeralIds::<T>::remove(&old_node.ephemeral_id);
        PeerIds::<T>::remove(&old_node.peer_id);
        old_node
    }

    pub(crate) fn is_builtin_node(tea_id: &TeaPubKey) -> bool {
        BuiltinNodes::<T>::get(tea_id).is_some()
    }

    pub(crate) fn get_initial_node_status(tea_id: &TeaPubKey) -> NodeStatus {
        match Self::is_builtin_node(tea_id) {
            true => NodeStatus::Active,
            false => NodeStatus::Pending,
        }
    }

    pub(crate) fn select_ra_nodes(tea_id: &TeaPubKey, _seed: U256) -> Vec<(TeaPubKey, bool)> {
        if Self::is_builtin_node(tea_id) {
            return Vec::new();
        }

        let mut ra_nodes = Vec::new();
        // todo: select 4 active nodes(calculate with `seed`) as ra nodes.
        for (tea_id, _) in BuiltinNodes::<T>::iter() {
            ra_nodes.push((tea_id, false));
        }
        ra_nodes
    }

    pub(crate) fn get_index_in_ra_nodes(
        tea_id: &TeaPubKey,
        target_tea_id: &TeaPubKey,
    ) -> Option<usize> {
        let target_node = Nodes::<T>::get(target_tea_id).unwrap();
        for i in 0..target_node.ra_nodes.len() {
            let (ra_tea_id, _) = target_node.ra_nodes[i];
            if ra_tea_id.eq(tea_id) {
                return Some(i);
            }
        }
        None
    }

    pub(crate) fn update_node_status(
        tea_id: &TeaPubKey,
        index: usize,
        is_pass: bool,
    ) -> NodeStatus {
        let mut target_node = Nodes::<T>::get(tea_id).unwrap();
        target_node.ra_nodes[index] = (tea_id.clone(), is_pass);
        let status = if is_pass {
            let approved_count = target_node
                .ra_nodes
                .iter()
                .filter(|(_, is_pass)| *is_pass)
                .count() as u32;
            // need 3/4 vote at least for now.
            if approved_count >= T::MinRaPassedThreshold::get() {
                NodeStatus::Active
            } else {
                NodeStatus::Pending
            }
        } else {
            NodeStatus::Invalid
        };
        target_node.status = status.clone();
        Nodes::<T>::insert(tea_id, &target_node);

        status
    }

    pub(crate) fn verify_ed25519_signature(
        pubkey: &TeaPubKey,
        content: &[u8],
        signature: &Signature,
    ) -> DispatchResult {
        let ed25519_pubkey = ed25519::Public(pubkey.clone());
        ensure!(signature.len() == 64, Error::<T>::InvalidSignatureLength);
        let ed25519_sig = ed25519::Signature::from_slice(&signature[..]);
        ensure!(
            ed25519_sig.verify(content, &ed25519_pubkey),
            Error::<T>::InvalidSignature
        );
        Ok(())
    }

    pub(crate) fn update_runtime_status(block_number: T::BlockNumber) {
        for (tea_id, mut node) in Nodes::<T>::iter() {
            if node.status == NodeStatus::Active {
                if block_number - node.update_time <= T::RuntimeActivityThreshold::get().into() {
                    continue;
                }
                match RuntimeActivities::<T>::get(&tea_id) {
                    Some(runtime_activity) => {
                        if block_number - runtime_activity.update_height
                            > T::RuntimeActivityThreshold::get().into()
                        {
                            node.status = NodeStatus::Inactive;
                            Nodes::<T>::insert(&tea_id, node);
                        }
                    }
                    None => {
                        node.status = NodeStatus::Inactive;
                        Nodes::<T>::insert(&tea_id, node);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{mock::*, types::*, Error, Nodes, RuntimeActivities};
    use frame_support::{assert_noop, assert_ok};

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
}
