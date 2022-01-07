use crate::{
	group::update_validator_groups_count, mock::*, types::*, AllowedPcrValues, AllowedVersions,
	BuiltinMiners, BuiltinNodes, Config, Error, NodePcr, Nodes, OfflineEvidences, ReportEvidences,
	TipsEvidences, VersionExpiredNodes, VersionsExpiredHeight,
};
use frame_support::{assert_noop, assert_ok, dispatch::DispatchError, traits::Currency};
use hex_literal::hex;
use pallet_cml::{
	CmlId, CmlStore, CmlType, DefrostScheduleType, MinerItemStore, MinerStatus, Seed, UserCmlStore,
	CML,
};
use sp_core::H256;
use sp_runtime::traits::AtLeast32BitUnsigned;
use tea_interface::TeaOperation;

#[test]
fn register_pcr_works() {
	new_test_ext().execute_with(|| {
		let pcr = b"test pcr".to_vec();
		let desc = b"test desc".to_vec();

		assert_eq!(AllowedPcrValues::<Test>::iter().count(), 0);
		assert_ok!(Tea::register_pcr(Origin::root(), vec![pcr], desc));
		assert_eq!(AllowedPcrValues::<Test>::iter().count(), 1);
	})
}

#[test]
fn register_pcr_should_fail_if_not_root_user() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Tea::register_pcr(
				Origin::signed(1),
				vec![b"test pcr".to_vec()],
				b"test desc".to_vec()
			),
			DispatchError::BadOrigin
		);
	})
}

#[test]
fn register_pcr_should_faild_if_register_twice() {
	new_test_ext().execute_with(|| {
		let pcr = b"test pcr".to_vec();
		let desc = b"test desc".to_vec();

		assert_ok!(Tea::register_pcr(
			Origin::root(),
			vec![pcr.clone()],
			desc.clone()
		));
		assert_noop!(
			Tea::register_pcr(Origin::root(), vec![pcr], desc),
			Error::<Test>::PcrAlreadyExists
		);
	})
}

#[test]
fn unregister_pcr_works() {
	new_test_ext().execute_with(|| {
		let pcr = b"test pcr".to_vec();
		let desc = b"test desc".to_vec();

		assert_ok!(Tea::register_pcr(Origin::root(), vec![pcr], desc));
		assert_eq!(AllowedPcrValues::<Test>::iter().count(), 1);

		let hashes: Vec<H256> = AllowedPcrValues::<Test>::iter()
			.map(|(hash, _)| hash)
			.collect();
		let hash = hashes[0];

		NodePcr::<Test>::insert([1; 32], hash);

		assert_ok!(Tea::unregister_pcr(Origin::root(), hash));
		assert_eq!(AllowedPcrValues::<Test>::iter().count(), 0);
		assert_eq!(NodePcr::<Test>::iter().count(), 0);
	})
}

#[test]
fn unregister_pcr_should_fail_if_not_root_user() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Tea::unregister_pcr(Origin::signed(1), Default::default(),),
			DispatchError::BadOrigin
		);
	})
}

#[test]
fn unregister_pcr_should_fail_if_not_exist() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Tea::unregister_pcr(Origin::root(), Default::default()),
			Error::<Test>::PcrNotExists
		);
	})
}

#[test]
fn register_versions_works() {
	new_test_ext().execute_with(|| {
		let version_key1 = b"version_key1".to_vec();
		let version_value1 = b"version_value1".to_vec();
		let desc = b"test desc".to_vec();

		assert_ok!(Tea::register_versions(
			Origin::root(),
			vec![version_key1.clone()],
			vec![version_value1.clone()],
			desc,
		));
		assert_eq!(AllowedVersions::<Test>::iter().count(), 1);
	})
}

#[test]
fn register_versions_should_fail_if_not_root_user() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Tea::register_versions(
				Origin::signed(1),
				vec![b"test key".to_vec()],
				vec![b"test value".to_vec()],
				b"test desc".to_vec()
			),
			DispatchError::BadOrigin
		);
	})
}

#[test]
fn register_versions_should_faild_if_register_twice() {
	new_test_ext().execute_with(|| {
		let version_key1 = b"version_key1".to_vec();
		let version_value1 = b"version_value1".to_vec();
		let desc = b"test desc".to_vec();

		assert_ok!(Tea::register_versions(
			Origin::root(),
			vec![version_key1.clone()],
			vec![version_value1.clone()],
			desc.clone()
		));
		assert_noop!(
			Tea::register_versions(
				Origin::root(),
				vec![version_key1],
				vec![version_value1],
				desc
			),
			Error::<Test>::VersionsAlreadyExists
		);
	})
}

#[test]
fn unregister_versions_works() {
	new_test_ext().execute_with(|| {
		let version_key1 = b"version_key1".to_vec();
		let version_value1 = b"version_value1".to_vec();
		let desc = b"test desc".to_vec();

		assert_eq!(AllowedVersions::<Test>::iter().count(), 0);
		assert_ok!(Tea::register_versions(
			Origin::root(),
			vec![version_key1.clone()],
			vec![version_value1.clone()],
			desc,
		));
		assert_eq!(AllowedVersions::<Test>::iter().count(), 1);

		let hashes: Vec<H256> = AllowedVersions::<Test>::iter()
			.map(|(hash, _)| hash)
			.collect();
		let hash = hashes[0];

		assert_ok!(Tea::unregister_versions(Origin::root(), hash));
		assert_eq!(AllowedVersions::<Test>::iter().count(), 0);
	})
}

#[test]
fn unregister_versions_should_fail_if_not_root_user() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Tea::unregister_versions(Origin::signed(1), Default::default(),),
			DispatchError::BadOrigin
		);
	})
}

#[test]
fn unregister_versions_should_fail_if_not_exist() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Tea::unregister_versions(Origin::root(), Default::default()),
			Error::<Test>::VersionsNotExist
		);
	})
}

#[test]
fn set_version_expired_height_works() {
	new_test_ext().execute_with(|| {
		let version_key1 = b"version_key1".to_vec();
		let version_value1 = b"version_value1".to_vec();
		let desc = b"test desc".to_vec();

		assert_ok!(Tea::register_versions(
			Origin::root(),
			vec![version_key1.clone()],
			vec![version_value1.clone()],
			desc,
		));

		let hashes: Vec<H256> = AllowedVersions::<Test>::iter()
			.map(|(hash, _)| hash)
			.collect();
		let hash = hashes[0];

		assert!(!VersionsExpiredHeight::<Test>::contains_key(hash));

		let height = 100;
		assert_ok!(Tea::set_version_expired_height(
			Origin::root(),
			hash,
			height
		));
		assert!(VersionsExpiredHeight::<Test>::contains_key(hash));
		assert_eq!(VersionsExpiredHeight::<Test>::get(hash), Some(height));
	})
}

#[test]
fn set_version_expired_height_should_fail_if_user_is_not_root_account() {
	new_test_ext().execute_with(|| {
		let version_key1 = b"version_key1".to_vec();
		let version_value1 = b"version_value1".to_vec();
		let desc = b"test desc".to_vec();

		assert_ok!(Tea::register_versions(
			Origin::root(),
			vec![version_key1.clone()],
			vec![version_value1.clone()],
			desc,
		));

		let hashes: Vec<H256> = AllowedVersions::<Test>::iter()
			.map(|(hash, _)| hash)
			.collect();
		let hash = hashes[0];

		assert_noop!(
			Tea::set_version_expired_height(Origin::signed(1), hash, 100,),
			DispatchError::BadOrigin
		);
	})
}

#[test]
fn set_version_expired_height_should_fail_if_hash_not_exist() {
	new_test_ext().execute_with(|| {
		let version_key1 = b"version_key1".to_vec();
		let version_value1 = b"version_value1".to_vec();
		let desc = b"test desc".to_vec();

		assert_ok!(Tea::register_versions(
			Origin::root(),
			vec![version_key1.clone()],
			vec![version_value1.clone()],
			desc,
		));

		assert_noop!(
			Tea::set_version_expired_height(Origin::root(), Default::default(), 100),
			Error::<Test>::VersionsNotExist
		);
	})
}

#[test]
fn set_version_expired_height_should_fail_if_setting_height_lower_equal_than_current_height() {
	new_test_ext().execute_with(|| {
		let version_key1 = b"version_key1".to_vec();
		let version_value1 = b"version_value1".to_vec();
		let desc = b"test desc".to_vec();

		assert_ok!(Tea::register_versions(
			Origin::root(),
			vec![version_key1.clone()],
			vec![version_value1.clone()],
			desc,
		));

		let current_height = 100;
		frame_system::Pallet::<Test>::set_block_number(current_height);

		assert_noop!(
			Tea::set_version_expired_height(Origin::root(), Default::default(), current_height - 1),
			Error::<Test>::VersionsNotExist
		);
		assert_noop!(
			Tea::set_version_expired_height(Origin::root(), Default::default(), current_height),
			Error::<Test>::VersionsNotExist
		);
	})
}

#[test]
fn add_new_node_works() {
	new_test_ext().execute_with(|| {
		let public: [u8; 32] =
			hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
		Tea::add_new_node(public, &1);
		let target_node = Nodes::<Test>::get(&public);
		assert_eq!(
			target_node.create_time,
			frame_system::Pallet::<Test>::block_number()
		);
	})
}

#[test]
fn builtin_node_update_node_profile_works() {
	new_test_ext().execute_with(|| {
		frame_system::Pallet::<Test>::set_block_number(100);
		let builtin_miner = 1;

		let (node, tea_id, ephemeral_id, peer_id) = new_node();
		Nodes::<Test>::insert(&tea_id, node);
		BuiltinNodes::<Test>::insert(&tea_id, ());
		BuiltinMiners::<Test>::insert(builtin_miner, ());

		let pcr_slots = vec![b"test pcr".to_vec()];
		let pcr_hash = Tea::pcr_slots_hash(&pcr_slots);
		AllowedPcrValues::<Test>::insert(
			&pcr_hash,
			PcrSlots {
				slots: pcr_slots,
				description: vec![],
			},
		);

		assert_eq!(<Test as Config>::Currency::free_balance(&builtin_miner), 0);
		let conn_id = vec![1u8; 32];
		assert_ok!(Tea::update_node_profile(
			Origin::signed(builtin_miner),
			tea_id.clone(),
			ephemeral_id.clone(),
			Vec::new(),
			peer_id,
			conn_id.clone(),
			pcr_hash,
		));
		assert!(Tea::is_builtin_node(&tea_id));

		let new_node = Nodes::<Test>::get(&tea_id);
		assert_eq!(ephemeral_id, new_node.ephemeral_id);
		assert_eq!(NodeStatus::Active, new_node.status);
		assert_eq!(conn_id, new_node.conn_id);

		assert_eq!(<Test as Config>::Currency::free_balance(&builtin_miner), 0);
	})
}

#[test]
fn builtin_node_update_node_profile_works_if_pcr_hash_not_allowed() {
	new_test_ext().execute_with(|| {
		frame_system::Pallet::<Test>::set_block_number(100);
		let builtin_miner = 1;

		let (node, tea_id, ephemeral_id, peer_id) = new_node();
		Nodes::<Test>::insert(&tea_id, node);
		BuiltinNodes::<Test>::insert(&tea_id, ());
		BuiltinMiners::<Test>::insert(builtin_miner, ());

		let pcr_slots = vec![b"test pcr".to_vec()];
		let pcr_hash = Tea::pcr_slots_hash(&pcr_slots);

		assert_ok!(Tea::update_node_profile(
			Origin::signed(builtin_miner),
			tea_id.clone(),
			ephemeral_id.clone(),
			Vec::new(),
			peer_id,
			vec![],
			pcr_hash,
		));
	})
}

#[test]
fn builtin_node_update_node_profile_should_fail_if_not_in_builtin_miners_list() {
	new_test_ext().execute_with(|| {
		frame_system::Pallet::<Test>::set_block_number(100);

		let (node, tea_id, ephemeral_id, peer_id) = new_node();
		Nodes::<Test>::insert(&tea_id, node);
		BuiltinNodes::<Test>::insert(&tea_id, ());

		assert_noop!(
			Tea::update_node_profile(
				Origin::signed(1),
				tea_id.clone(),
				ephemeral_id.clone(),
				Vec::new(),
				peer_id,
				vec![],
				Default::default(),
			),
			Error::<Test>::InvalidBuiltinMiner
		);
	})
}

#[test]
fn normal_node_update_node_profile_works() {
	new_test_ext().execute_with(|| {
		let owner = 2;
		let owner_controller = 22;
		<Test as Config>::Currency::make_free_balance_be(&owner, 10000);
		frame_system::Pallet::<Test>::set_block_number(100);

		let (node, tea_id, ephemeral_id, peer_id) = new_node();
		Nodes::<Test>::insert(&tea_id, node);

		let cml_id = 1;
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		cml.set_owner(&owner);
		UserCmlStore::<Test>::insert(owner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(owner),
			cml_id,
			tea_id,
			owner_controller,
			b"miner_ip".to_vec(),
			None,
		));

		let pcr_slots = vec![b"test pcr".to_vec()];
		let pcr_hash = Tea::pcr_slots_hash(&pcr_slots);
		AllowedPcrValues::<Test>::insert(
			&pcr_hash,
			PcrSlots {
				slots: pcr_slots,
				description: vec![],
			},
		);

		assert_eq!(
			<Test as Config>::Currency::free_balance(&owner_controller),
			1
		);
		let conn_id = vec![3u8; 32];
		assert_ok!(Tea::update_node_profile(
			Origin::signed(owner_controller),
			tea_id.clone(),
			ephemeral_id.clone(),
			Vec::new(),
			peer_id,
			conn_id.clone(),
			pcr_hash,
		));
		assert!(!Tea::is_builtin_node(&tea_id));

		let new_node = Nodes::<Test>::get(&tea_id);
		assert_eq!(ephemeral_id, new_node.ephemeral_id);
		assert_eq!(NodeStatus::Pending, new_node.status);
		assert_eq!(conn_id, new_node.conn_id);
		assert_eq!(
			<Test as Config>::Currency::free_balance(&owner_controller),
			1
		);
	})
}

#[test]
fn normal_node_update_node_profile_should_fail_if_pcr_is_invalid() {
	new_test_ext().execute_with(|| {
		let owner = 2;
		let owner_controller = 22;
		<Test as Config>::Currency::make_free_balance_be(&owner, 10000);
		frame_system::Pallet::<Test>::set_block_number(100);

		let (node, tea_id, ephemeral_id, peer_id) = new_node();
		Nodes::<Test>::insert(&tea_id, node);

		let cml_id = 1;
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		cml.set_owner(&owner);
		UserCmlStore::<Test>::insert(owner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(owner),
			cml_id,
			tea_id,
			owner_controller,
			b"miner_ip".to_vec(),
			None,
		));

		let pcr_slots = vec![b"test pcr".to_vec()];
		let pcr_hash = Tea::pcr_slots_hash(&pcr_slots);

		assert_ok!(Tea::update_node_profile(
			Origin::signed(owner_controller),
			tea_id.clone(),
			ephemeral_id.clone(),
			Vec::new(),
			peer_id,
			vec![],
			pcr_hash,
		));
		// todo uncomment if verify pcr related logic is on
		// assert_noop!(
		// 	Tea::update_node_profile(
		// 		Origin::signed(owner_controller),
		// 		tea_id.clone(),
		// 		ephemeral_id.clone(),
		// 		Vec::new(),
		// 		peer_id,
		// 		vec![],
		// 		pcr_hash,
		// 	),
		// 	Error::<Test>::InvalidPcrHash
		// );
	})
}

#[test]
fn normal_node_update_node_profile_should_fail_if_not_the_owner_of_tea_id() {
	new_test_ext().execute_with(|| {
		frame_system::Pallet::<Test>::set_block_number(100);

		let (node, tea_id, ephemeral_id, peer_id) = new_node();
		Nodes::<Test>::insert(&tea_id, node);

		assert_noop!(
			Tea::update_node_profile(
				Origin::signed(1),
				tea_id.clone(),
				ephemeral_id.clone(),
				Vec::new(),
				peer_id,
				vec![],
				Default::default(),
			),
			Error::<Test>::InvalidTeaIdOwner
		);
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
				peer_id,
				vec![],
				Default::default(),
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
				Default::default(),
			),
			Error::<Test>::InvalidPeerId
		);
	})
}

#[test]
fn remote_attestation_works() {
	new_test_ext().execute_with(|| {
		let (mut node, tea_id, _, _) = new_node();

		let mut ra_nodes: Vec<(TeaPubKey, bool)> = Vec::new();
		let validator_1 = hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
		let validator_2 = hex!("c7e016fad0796bb68594e49a6ef1942cf7e73497e69edb32d19ba2fab3696596");
		let validator_3 = hex!("c9380fde1ba795fc656ab08ab4ef4482cf554790fd3abcd4642418ae8fb5fd52");
		let validator_4 = hex!("2754d7e9c73ced5b302e12464594110850980027f8f83c469e8145eef59220b6");

		let owner1 = 2;
		let owner1_controller = 22;
		<Test as Config>::Currency::make_free_balance_be(&owner1, 10000);
		let cml_id1 = 1;
		let mut cml1 = CML::from_genesis_seed(seed_from_lifespan(cml_id1, 100));
		cml1.set_owner(&owner1);
		UserCmlStore::<Test>::insert(owner1, cml_id1, ());
		CmlStore::<Test>::insert(cml_id1, cml1);
		assert_ok!(Cml::start_mining(
			Origin::signed(owner1),
			cml_id1,
			validator_1,
			owner1_controller,
			b"miner_ip1".to_vec(),
			None,
		));

		let owner2 = 3;
		let owner2_controller = 33;
		<Test as Config>::Currency::make_free_balance_be(&owner2, 20000);
		let cml_id2 = 2;
		let mut cml2 = CML::from_genesis_seed(seed_from_lifespan(cml_id2, 200));
		cml2.set_owner(&owner2);
		UserCmlStore::<Test>::insert(owner2, cml_id2, ());
		CmlStore::<Test>::insert(cml_id2, cml2);
		assert_ok!(Cml::start_mining(
			Origin::signed(owner2),
			cml_id2,
			validator_2,
			owner2_controller,
			b"miner_ip2".to_vec(),
			None,
		));

		let owner3 = 4;
		let owner3_controller = 44;
		<Test as Config>::Currency::make_free_balance_be(&owner3, 30000);
		let cml_id3 = 3;
		let mut cml3 = CML::from_genesis_seed(seed_from_lifespan(cml_id3, 300));
		cml3.set_owner(&owner3);
		UserCmlStore::<Test>::insert(owner3, cml_id3, ());
		CmlStore::<Test>::insert(cml_id3, cml3);
		assert_ok!(Cml::start_mining(
			Origin::signed(owner3),
			cml_id3,
			validator_3,
			owner3_controller,
			b"miner_ip3".to_vec(),
			None,
		));

		let owner4 = 5;
		let owner4_controller = 55;
		<Test as Config>::Currency::make_free_balance_be(&owner4, 40000);
		let cml_id4 = 4;
		let mut cml4 = CML::from_genesis_seed(seed_from_lifespan(cml_id4, 400));
		cml4.set_owner(&owner4);
		UserCmlStore::<Test>::insert(owner4, cml_id4, ());
		CmlStore::<Test>::insert(cml_id4, cml4);
		assert_ok!(Cml::start_mining(
			Origin::signed(owner4),
			cml_id4,
			validator_4,
			owner4_controller,
			b"miner_ip4".to_vec(),
			None,
		));

		let (ephemeral_id1, signature1) = generate_pk_and_signature(&validator_1, &tea_id, true);
		Nodes::<Test>::mutate(&validator_1, |node| {
			node.ephemeral_id = ephemeral_id1;
			node.status = NodeStatus::Active;
		});
		ra_nodes.push((validator_1.clone(), false));

		let (ephemeral_id2, signature2) = generate_pk_and_signature(&validator_2, &tea_id, false);
		Nodes::<Test>::mutate(&validator_2, |node| {
			node.ephemeral_id = ephemeral_id2;
			node.status = NodeStatus::Active;
		});
		ra_nodes.push((validator_2.clone(), false));

		let (ephemeral_id3, signature3) = generate_pk_and_signature(&validator_3, &tea_id, true);
		Nodes::<Test>::mutate(&validator_3, |node| {
			node.ephemeral_id = ephemeral_id3;
			node.status = NodeStatus::Active;
		});
		ra_nodes.push((validator_3.clone(), false));

		let (ephemeral_id4, signature4) = generate_pk_and_signature(&validator_4, &tea_id, true);
		Nodes::<Test>::mutate(&validator_4, |node| {
			node.ephemeral_id = ephemeral_id4;
			node.status = NodeStatus::Active;
		});
		ra_nodes.push((validator_4.clone(), false));

		node.ra_nodes = ra_nodes;
		Nodes::<Test>::insert(&tea_id, node);

		Tea::update_validators();
		update_validator_groups_count::<Test>();

		assert_ok!(Tea::remote_attestation(
			Origin::signed(owner1_controller),
			validator_1,
			tea_id.clone(),
			true,
			signature1
		));
		assert_eq!(Nodes::<Test>::get(&tea_id).status, NodeStatus::Pending);

		assert_ok!(Tea::remote_attestation(
			Origin::signed(owner2_controller),
			validator_2,
			tea_id.clone(),
			false,
			signature2
		));
		assert_eq!(Nodes::<Test>::get(&tea_id).status, NodeStatus::Pending);

		assert_ok!(Tea::remote_attestation(
			Origin::signed(owner3_controller),
			validator_3,
			tea_id.clone(),
			true,
			signature3
		));
		assert_eq!(Nodes::<Test>::get(&tea_id).status, NodeStatus::Active);

		assert_ok!(Tea::remote_attestation(
			Origin::signed(owner4_controller),
			validator_4,
			tea_id.clone(),
			true,
			signature4
		));
		assert_eq!(Nodes::<Test>::get(&tea_id).status, NodeStatus::Active);
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
fn remote_attestation_should_fail_if_ra_commit_has_expired() {
	new_test_ext().execute_with(|| {
		let owner = 2;
		let owner_controller = 22;
		<Test as Config>::Currency::make_free_balance_be(&owner, 10000);

		let last_update_height = 100;
		let (mut node, tea_id, _, _) = new_node();
		node.update_time = last_update_height;
		Nodes::<Test>::insert(&tea_id, node);

		let validator_tea_id =
			hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
		Nodes::<Test>::insert(&validator_tea_id, Node::default());

		let cml_id = 1;
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		cml.set_owner(&owner);
		UserCmlStore::<Test>::insert(owner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(owner),
			cml_id,
			validator_tea_id,
			owner_controller,
			b"miner_ip".to_vec(),
			None,
		));

		frame_system::Pallet::<Test>::set_block_number(
			last_update_height + MAX_ALLOWED_RA_COMMIT_DURATION as u64 + 1,
		);
		assert_noop!(
			Tea::remote_attestation(
				Origin::signed(owner_controller),
				validator_tea_id,
				tea_id,
				true,
				Vec::new()
			),
			Error::<Test>::RaCommitExpired
		);
	})
}

#[test]
fn validator_not_in_ra_nodes() {
	new_test_ext().execute_with(|| {
		let owner = 2;
		let owner_controller = 22;
		<Test as Config>::Currency::make_free_balance_be(&owner, 10000);

		let (mut node, tea_id, _, _) = new_node();
		node.ra_nodes = Vec::new(); // validator tea id not in ra node list
		Nodes::<Test>::insert(&tea_id, node);

		let validator_tea_id =
			hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
		Nodes::<Test>::insert(&validator_tea_id, Node::default());

		let cml_id = 1;
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		cml.set_owner(&owner);
		UserCmlStore::<Test>::insert(owner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(owner),
			cml_id,
			validator_tea_id,
			owner_controller,
			b"miner_ip".to_vec(),
			None,
		));

		assert_noop!(
			Tea::remote_attestation(
				Origin::signed(owner_controller),
				validator_tea_id,
				tea_id,
				true,
				Vec::new()
			),
			Error::<Test>::NotRaValidator
		);
	})
}

#[test]
fn update_runtime_activity_works() {
	use ed25519_dalek::ed25519::signature::Signature;
	use ed25519_dalek::{Keypair, Signer};
	use rand::rngs::OsRng;

	new_test_ext().execute_with(|| {
		let (mut node, tea_id, _, _) = new_node();

		let mut csprng = OsRng {};
		let kp = Keypair::generate(&mut csprng);
		let signature = kp.sign(&tea_id);

		node.ephemeral_id.copy_from_slice(kp.public.as_bytes());
		Nodes::<Test>::insert(&tea_id, node);

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
fn update_runtime_activity_should_fail_if_ephemeral_id_not_match() {
	use ed25519_dalek::ed25519::signature::Signature;
	use ed25519_dalek::{Keypair, Signer};
	use rand::rngs::OsRng;

	new_test_ext().execute_with(|| {
		let (node, tea_id, _, _) = new_node();
		Nodes::<Test>::insert(&tea_id, node);

		let mut csprng = OsRng {};
		let kp = Keypair::generate(&mut csprng);
		let signature = kp.sign(&tea_id);

		assert_noop!(
			Tea::update_runtime_activity(
				Origin::signed(1),
				tea_id,
				None,
				kp.public.as_bytes().clone(),
				signature.as_bytes().to_vec(),
			),
			Error::<Test>::NodeEphemeralIdNotMatch
		);
	})
}

#[test]
fn update_runtime_activity_when_node_not_registered() {
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
fn commit_report_evidence_works() {
	new_test_ext().execute_with(|| {
		let committer = 2;
		let committer_controller = 22;
		let reporter = 3;
		let reporter_controller = 33;
		let phisher = 4;
		let phisher_controler = 44;
		<Test as Config>::Currency::make_free_balance_be(&committer, 10000);
		<Test as Config>::Currency::make_free_balance_be(&reporter, 10000);
		<Test as Config>::Currency::make_free_balance_be(&phisher, 10000);

		let committer_cml_id = 1;
		let committer_tea_id = [1u8; 32];
		let reporter_cml_id = 2;
		let reporter_tea_id = [2u8; 32];
		let phisher_cml_id = 3;
		let phisher_tea_id = [3u8; 32];

		let mut committer_cml =
			CML::from_genesis_seed(seed_from_type(committer_cml_id, 100, CmlType::B));
		committer_cml.set_owner(&committer);
		UserCmlStore::<Test>::insert(committer, committer_cml_id, ());
		CmlStore::<Test>::insert(committer_cml_id, committer_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(committer),
			committer_cml_id,
			committer_tea_id,
			committer_controller,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		let mut reporter_cml =
			CML::from_genesis_seed(seed_from_type(reporter_cml_id, 100, CmlType::C));
		reporter_cml.set_owner(&reporter);
		UserCmlStore::<Test>::insert(reporter, reporter_cml_id, ());
		CmlStore::<Test>::insert(reporter_cml_id, reporter_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(reporter),
			reporter_cml_id,
			reporter_tea_id,
			reporter_controller,
			b"miner_ip2".to_vec(),
			None,
		));

		let mut phisher_cml =
			CML::from_genesis_seed(seed_from_type(phisher_cml_id, 100, CmlType::B));
		phisher_cml.set_owner(&phisher);
		UserCmlStore::<Test>::insert(phisher, phisher_cml_id, ());
		CmlStore::<Test>::insert(phisher_cml_id, phisher_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(phisher),
			phisher_cml_id,
			phisher_tea_id,
			phisher_controler,
			b"miner_ip3".to_vec(),
			Some(b"orbit_id2".to_vec()),
		));

		let current_height = 55;
		frame_system::Pallet::<Test>::set_block_number(current_height);

		assert_ok!(Tea::commit_report_evidence(
			Origin::signed(committer_controller),
			committer_tea_id,
			reporter_tea_id,
			phisher_tea_id,
			vec![]
		));

		assert!(ReportEvidences::<Test>::contains_key(phisher_tea_id));
		let evidence = ReportEvidences::<Test>::get(phisher_tea_id);
		assert_eq!(evidence.height, current_height);
		assert_eq!(evidence.reporter, reporter_tea_id);
		assert_eq!(
			<Test as Config>::Currency::free_balance(committer_controller),
			1 + 195000000
		);
	})
}

#[test]
fn commit_report_evidence_should_fail_if_commit_tea_id_not_exist() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Tea::commit_report_evidence(Origin::signed(1), [1u8; 32], [2u8; 32], [3u8; 32], vec![]),
			Error::<Test>::NodeNotExist
		);
	})
}

#[test]
fn commit_report_evidence_should_fail_if_report_tea_id_not_exist() {
	new_test_ext().execute_with(|| {
		let committer = [1u8; 32];
		Nodes::<Test>::insert(committer, Node::<u64>::default());

		assert_noop!(
			Tea::commit_report_evidence(Origin::signed(1), committer, [2u8; 32], [3u8; 32], vec![]),
			Error::<Test>::ReportNodeNotExist
		);
	})
}

#[test]
fn commit_report_evidence_should_fail_if_phishing_tea_id_not_exist() {
	new_test_ext().execute_with(|| {
		let committer = [1u8; 32];
		let reporter = [2u8; 32];
		Nodes::<Test>::insert(committer, Node::<u64>::default());
		Nodes::<Test>::insert(reporter, Node::<u64>::default());

		assert_noop!(
			Tea::commit_report_evidence(Origin::signed(1), committer, reporter, [3u8; 32], vec![]),
			Error::<Test>::PhishingNodeNotExist
		);
	})
}

#[test]
fn commit_report_evidence_should_fail_if_user_is_not_the_owner_of_commit_tea_id() {
	new_test_ext().execute_with(|| {
		let committer = [1u8; 32];
		let reporter = [2u8; 32];
		let phisher = [3u8; 32];
		Nodes::<Test>::insert(committer, Node::<u64>::default());
		Nodes::<Test>::insert(reporter, Node::<u64>::default());
		Nodes::<Test>::insert(phisher, Node::<u64>::default());

		assert_noop!(
			Tea::commit_report_evidence(Origin::signed(1), committer, reporter, phisher, vec![]),
			Error::<Test>::InvalidTeaIdOwner
		);
	});

	new_test_ext().execute_with(|| {
		let committer = 2;
		let reporter = [2u8; 32];
		let phisher = [3u8; 32];
		Nodes::<Test>::insert(reporter, Node::<u64>::default());
		Nodes::<Test>::insert(phisher, Node::<u64>::default());
		<Test as Config>::Currency::make_free_balance_be(&committer, 10000);

		let committer_cml_id = 1;
		let committer_tea_id = [1u8; 32];

		let mut committer_cml =
			CML::from_genesis_seed(seed_from_type(committer_cml_id, 100, CmlType::B));
		committer_cml.set_owner(&committer);
		UserCmlStore::<Test>::insert(committer, committer_cml_id, ());
		CmlStore::<Test>::insert(committer_cml_id, committer_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(committer),
			committer_cml_id,
			committer_tea_id,
			committer,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		assert_noop!(
			Tea::commit_report_evidence(
				Origin::signed(1),
				committer_tea_id,
				reporter,
				phisher,
				vec![]
			),
			Error::<Test>::InvalidTeaIdOwner
		);
	});
}

#[test]
fn commit_report_evidence_should_fail_if_commit_cml_is_not_b_type() {
	new_test_ext().execute_with(|| {
		let committer = 2;
		let reporter = [2u8; 32];
		let phisher = [3u8; 32];
		Nodes::<Test>::insert(reporter, Node::<u64>::default());
		Nodes::<Test>::insert(phisher, Node::<u64>::default());
		<Test as Config>::Currency::make_free_balance_be(&committer, 10000);

		let committer_cml_id = 1;
		let committer_tea_id = [1u8; 32];

		let mut committer_cml =
			CML::from_genesis_seed(seed_from_type(committer_cml_id, 100, CmlType::A));
		committer_cml.set_owner(&committer);
		UserCmlStore::<Test>::insert(committer, committer_cml_id, ());
		CmlStore::<Test>::insert(committer_cml_id, committer_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(committer),
			committer_cml_id,
			committer_tea_id,
			committer,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		assert_noop!(
			Tea::commit_report_evidence(
				Origin::signed(committer),
				committer_tea_id,
				reporter,
				phisher,
				vec![]
			),
			Error::<Test>::OnlyBTypeCmlCanCommitReport
		);
	})
}

#[test]
fn commit_report_evidence_should_fail_if_report_cml_is_not_c_type() {
	new_test_ext().execute_with(|| {
		let committer = 2;
		let reporter = 3;
		let phisher = 4;
		<Test as Config>::Currency::make_free_balance_be(&committer, 10000);
		<Test as Config>::Currency::make_free_balance_be(&reporter, 10000);
		<Test as Config>::Currency::make_free_balance_be(&phisher, 10000);

		let committer_cml_id = 1;
		let committer_tea_id = [1u8; 32];
		let reporter_cml_id = 2;
		let reporter_tea_id = [2u8; 32];
		let phisher_cml_id = 3;
		let phisher_tea_id = [3u8; 32];

		let mut committer_cml =
			CML::from_genesis_seed(seed_from_type(committer_cml_id, 100, CmlType::B));
		committer_cml.set_owner(&committer);
		UserCmlStore::<Test>::insert(committer, committer_cml_id, ());
		CmlStore::<Test>::insert(committer_cml_id, committer_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(committer),
			committer_cml_id,
			committer_tea_id,
			committer,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		let mut reporter_cml =
			CML::from_genesis_seed(seed_from_type(reporter_cml_id, 100, CmlType::A));
		reporter_cml.set_owner(&reporter);
		UserCmlStore::<Test>::insert(reporter, reporter_cml_id, ());
		CmlStore::<Test>::insert(reporter_cml_id, reporter_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(reporter),
			reporter_cml_id,
			reporter_tea_id,
			reporter,
			b"miner_ip2".to_vec(),
			None,
		));

		let mut phisher_cml =
			CML::from_genesis_seed(seed_from_type(phisher_cml_id, 100, CmlType::B));
		phisher_cml.set_owner(&phisher);
		UserCmlStore::<Test>::insert(phisher, phisher_cml_id, ());
		CmlStore::<Test>::insert(phisher_cml_id, phisher_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(phisher),
			phisher_cml_id,
			phisher_tea_id,
			phisher,
			b"miner_ip3".to_vec(),
			Some(b"orbit_id2".to_vec()),
		));

		assert_noop!(
			Tea::commit_report_evidence(
				Origin::signed(committer),
				committer_tea_id,
				reporter_tea_id,
				phisher_tea_id,
				vec![]
			),
			Error::<Test>::OnlyCTypeCmlCanReport
		);
	})
}

#[test]
fn commit_report_evidence_should_fail_if_phishing_cml_is_c_type() {
	new_test_ext().execute_with(|| {
		let committer = 2;
		let reporter = 3;
		let phisher = 4;
		<Test as Config>::Currency::make_free_balance_be(&committer, 10000);
		<Test as Config>::Currency::make_free_balance_be(&reporter, 10000);
		<Test as Config>::Currency::make_free_balance_be(&phisher, 10000);

		let committer_cml_id = 1;
		let committer_tea_id = [1u8; 32];
		let reporter_cml_id = 2;
		let reporter_tea_id = [2u8; 32];
		let phisher_cml_id = 3;
		let phisher_tea_id = [3u8; 32];

		let mut committer_cml =
			CML::from_genesis_seed(seed_from_type(committer_cml_id, 100, CmlType::B));
		committer_cml.set_owner(&committer);
		UserCmlStore::<Test>::insert(committer, committer_cml_id, ());
		CmlStore::<Test>::insert(committer_cml_id, committer_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(committer),
			committer_cml_id,
			committer_tea_id,
			committer,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		let mut reporter_cml =
			CML::from_genesis_seed(seed_from_type(reporter_cml_id, 100, CmlType::C));
		reporter_cml.set_owner(&reporter);
		UserCmlStore::<Test>::insert(reporter, reporter_cml_id, ());
		CmlStore::<Test>::insert(reporter_cml_id, reporter_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(reporter),
			reporter_cml_id,
			reporter_tea_id,
			reporter,
			b"miner_ip2".to_vec(),
			None,
		));

		let mut phisher_cml =
			CML::from_genesis_seed(seed_from_type(phisher_cml_id, 100, CmlType::C));
		phisher_cml.set_owner(&phisher);
		UserCmlStore::<Test>::insert(phisher, phisher_cml_id, ());
		CmlStore::<Test>::insert(phisher_cml_id, phisher_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(phisher),
			phisher_cml_id,
			phisher_tea_id,
			phisher,
			b"miner_ip3".to_vec(),
			Some(b"orbit_id2".to_vec()),
		));

		assert_noop!(
			Tea::commit_report_evidence(
				Origin::signed(committer),
				committer_tea_id,
				reporter_tea_id,
				phisher_tea_id,
				vec![]
			),
			Error::<Test>::PhishingNodeCannotBeTypeC
		);
	})
}

#[test]
fn commit_report_evidence_should_fail_if_phishing_and_commiting_tea_id_are_same() {
	new_test_ext().execute_with(|| {
		let reporter = 3;
		let phisher = 4;
		<Test as Config>::Currency::make_free_balance_be(&reporter, 10000);
		<Test as Config>::Currency::make_free_balance_be(&phisher, 10000);

		let reporter_cml_id = 2;
		let reporter_tea_id = [2u8; 32];
		let phisher_cml_id = 3;
		let phisher_tea_id = [3u8; 32];

		let mut reporter_cml =
			CML::from_genesis_seed(seed_from_type(reporter_cml_id, 100, CmlType::C));
		reporter_cml.set_owner(&reporter);
		UserCmlStore::<Test>::insert(reporter, reporter_cml_id, ());
		CmlStore::<Test>::insert(reporter_cml_id, reporter_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(reporter),
			reporter_cml_id,
			reporter_tea_id,
			reporter,
			b"miner_ip2".to_vec(),
			None,
		));

		let mut phisher_cml =
			CML::from_genesis_seed(seed_from_type(phisher_cml_id, 100, CmlType::B));
		phisher_cml.set_owner(&phisher);
		UserCmlStore::<Test>::insert(phisher, phisher_cml_id, ());
		CmlStore::<Test>::insert(phisher_cml_id, phisher_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(phisher),
			phisher_cml_id,
			phisher_tea_id,
			phisher,
			b"miner_ip3".to_vec(),
			Some(b"orbit_id2".to_vec()),
		));

		let current_height = 55;
		frame_system::Pallet::<Test>::set_block_number(current_height);

		assert_noop!(
			Tea::commit_report_evidence(
				Origin::signed(phisher),
				phisher_tea_id,
				reporter_tea_id,
				phisher_tea_id,
				vec![]
			),
			Error::<Test>::PhishingNodeCannotCommitReport
		);
	})
}

#[test]
fn commit_report_evidence_should_fail_if_repoted_not_long_ago() {
	new_test_ext().execute_with(|| {
		let committer = 2;
		let reporter = 3;
		let phisher = 4;
		<Test as Config>::Currency::make_free_balance_be(&committer, 10000);
		<Test as Config>::Currency::make_free_balance_be(&reporter, 10000);
		<Test as Config>::Currency::make_free_balance_be(&phisher, 10000);

		let committer_cml_id = 1;
		let committer_tea_id = [1u8; 32];
		let reporter_cml_id = 2;
		let reporter_tea_id = [2u8; 32];
		let phisher_cml_id = 3;
		let phisher_tea_id = [3u8; 32];

		let mut committer_cml =
			CML::from_genesis_seed(seed_from_type(committer_cml_id, 100, CmlType::B));
		committer_cml.set_owner(&committer);
		UserCmlStore::<Test>::insert(committer, committer_cml_id, ());
		CmlStore::<Test>::insert(committer_cml_id, committer_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(committer),
			committer_cml_id,
			committer_tea_id,
			committer,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		let mut reporter_cml =
			CML::from_genesis_seed(seed_from_type(reporter_cml_id, 100, CmlType::C));
		reporter_cml.set_owner(&reporter);
		UserCmlStore::<Test>::insert(reporter, reporter_cml_id, ());
		CmlStore::<Test>::insert(reporter_cml_id, reporter_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(reporter),
			reporter_cml_id,
			reporter_tea_id,
			reporter,
			b"miner_ip2".to_vec(),
			None,
		));

		let mut phisher_cml =
			CML::from_genesis_seed(seed_from_type(phisher_cml_id, 100, CmlType::B));
		phisher_cml.set_owner(&phisher);
		UserCmlStore::<Test>::insert(phisher, phisher_cml_id, ());
		CmlStore::<Test>::insert(phisher_cml_id, phisher_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(phisher),
			phisher_cml_id,
			phisher_tea_id,
			phisher,
			b"miner_ip3".to_vec(),
			Some(b"orbit_id2".to_vec()),
		));

		let current_height = 55;
		frame_system::Pallet::<Test>::set_block_number(current_height);

		assert_ok!(Tea::commit_report_evidence(
			Origin::signed(committer),
			committer_tea_id,
			reporter_tea_id,
			phisher_tea_id,
			vec![]
		));
		assert_eq!(
			ReportEvidences::<Test>::get(phisher_tea_id).height,
			current_height
		);

		frame_system::Pallet::<Test>::set_block_number(
			current_height + PHISHING_ALLOWED_DURATION as u64,
		);
		assert_noop!(
			Tea::commit_report_evidence(
				Origin::signed(committer),
				committer_tea_id,
				reporter_tea_id,
				phisher_tea_id,
				vec![]
			),
			Error::<Test>::RedundantReport
		);

		frame_system::Pallet::<Test>::set_block_number(
			current_height + PHISHING_ALLOWED_DURATION as u64 + 1,
		);
		assert_ok!(Tea::commit_report_evidence(
			Origin::signed(committer),
			committer_tea_id,
			reporter_tea_id,
			phisher_tea_id,
			vec![]
		));
	})
}

#[test]
fn commit_report_evidence_should_fail_if_phishing_cml_is_inactive() {
	new_test_ext().execute_with(|| {
		let committer = 2;
		let reporter = 3;
		let phisher = 4;
		<Test as Config>::Currency::make_free_balance_be(&committer, 10000);
		<Test as Config>::Currency::make_free_balance_be(&reporter, 10000);
		<Test as Config>::Currency::make_free_balance_be(&phisher, 10000);

		let committer_cml_id = 1;
		let committer_tea_id = [1u8; 32];
		let reporter_cml_id = 2;
		let reporter_tea_id = [2u8; 32];
		let phisher_cml_id = 3;
		let phisher_tea_id = [3u8; 32];

		let mut committer_cml =
			CML::from_genesis_seed(seed_from_type(committer_cml_id, 100, CmlType::B));
		committer_cml.set_owner(&committer);
		UserCmlStore::<Test>::insert(committer, committer_cml_id, ());
		CmlStore::<Test>::insert(committer_cml_id, committer_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(committer),
			committer_cml_id,
			committer_tea_id,
			committer,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		let mut reporter_cml =
			CML::from_genesis_seed(seed_from_type(reporter_cml_id, 100, CmlType::C));
		reporter_cml.set_owner(&reporter);
		UserCmlStore::<Test>::insert(reporter, reporter_cml_id, ());
		CmlStore::<Test>::insert(reporter_cml_id, reporter_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(reporter),
			reporter_cml_id,
			reporter_tea_id,
			reporter,
			b"miner_ip2".to_vec(),
			None,
		));

		let mut phisher_cml =
			CML::from_genesis_seed(seed_from_type(phisher_cml_id, 100, CmlType::B));
		phisher_cml.set_owner(&phisher);
		UserCmlStore::<Test>::insert(phisher, phisher_cml_id, ());
		CmlStore::<Test>::insert(phisher_cml_id, phisher_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(phisher),
			phisher_cml_id,
			phisher_tea_id,
			phisher,
			b"miner_ip3".to_vec(),
			Some(b"orbit_id2".to_vec()),
		));

		MinerItemStore::<Test>::mutate(&phisher_tea_id, |item| {
			item.status = MinerStatus::Offline;
		});

		assert_noop!(
			Tea::commit_report_evidence(
				Origin::signed(committer),
				committer_tea_id,
				reporter_tea_id,
				phisher_tea_id,
				vec![]
			),
			Error::<Test>::PhishingNodeNotActive
		);
	})
}

#[test]
fn commit_offline_evidence_works() {
	new_test_ext().execute_with(|| {
		let miner = 2;
		let miner_controller = 22;
		let reporter = 3;
		let reporter_controller = 33;
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);
		<Test as Config>::Currency::make_free_balance_be(&reporter, 10000);

		let miner_cml_id = 1;
		let miner_tea_id = [1u8; 32];
		let reporter_cml_id = 2;
		let reporter_tea_id = [2u8; 32];

		let mut miner_cml = CML::from_genesis_seed(seed_from_type(miner_cml_id, 100, CmlType::B));
		miner_cml.set_owner(&miner);
		UserCmlStore::<Test>::insert(miner, miner_cml_id, ());
		CmlStore::<Test>::insert(miner_cml_id, miner_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			miner_cml_id,
			miner_tea_id,
			miner_controller,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		let mut reporter_cml =
			CML::from_genesis_seed(seed_from_type(reporter_cml_id, 100, CmlType::B));
		reporter_cml.set_owner(&reporter);
		UserCmlStore::<Test>::insert(reporter, reporter_cml_id, ());
		CmlStore::<Test>::insert(reporter_cml_id, reporter_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(reporter),
			reporter_cml_id,
			reporter_tea_id,
			reporter_controller,
			b"miner_ip2".to_vec(),
			Some(b"orbit_id2".to_vec()),
		));

		let current_height = 55;
		frame_system::Pallet::<Test>::set_block_number(current_height);

		assert_ok!(Tea::commit_offline_evidence(
			Origin::signed(reporter_controller),
			reporter_tea_id,
			miner_tea_id,
			vec![]
		));
		assert!(OfflineEvidences::<Test>::contains_key(miner_tea_id));
		let evidences = OfflineEvidences::<Test>::get(miner_tea_id);
		assert_eq!(evidences.len(), 1);
		assert_eq!(evidences[0].height, current_height);
		assert_eq!(evidences[0].tea_id, reporter_tea_id);
		assert_eq!(
			<Test as Config>::Currency::free_balance(reporter_controller),
			1 + 195000000
		);
	})
}

#[test]
fn commit_offline_evidence_works_if_commit_multi_times_and_suspend_the_node() {
	new_test_ext().execute_with(|| {
		let miner = 2;
		let reporter = 3;
		let reporter2 = 4;
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);
		<Test as Config>::Currency::make_free_balance_be(&reporter, 10000);
		<Test as Config>::Currency::make_free_balance_be(&reporter2, 10000);

		let miner_cml_id = 1;
		let miner_tea_id = [1u8; 32];
		let reporter_cml_id = 2;
		let reporter_tea_id = [2u8; 32];
		let reporter_cml_id2 = 3;
		let reporter_tea_id2 = [3u8; 32];

		let mut miner_cml = CML::from_genesis_seed(seed_from_type(miner_cml_id, 100, CmlType::B));
		miner_cml.set_owner(&miner);
		UserCmlStore::<Test>::insert(miner, miner_cml_id, ());
		CmlStore::<Test>::insert(miner_cml_id, miner_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			miner_cml_id,
			miner_tea_id,
			miner,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		let mut reporter_cml =
			CML::from_genesis_seed(seed_from_type(reporter_cml_id, 100, CmlType::B));
		reporter_cml.set_owner(&reporter);
		UserCmlStore::<Test>::insert(reporter, reporter_cml_id, ());
		CmlStore::<Test>::insert(reporter_cml_id, reporter_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(reporter),
			reporter_cml_id,
			reporter_tea_id,
			reporter,
			b"miner_ip2".to_vec(),
			Some(b"orbit_id2".to_vec()),
		));

		let mut reporter_cml2 =
			CML::from_genesis_seed(seed_from_type(reporter_cml_id2, 100, CmlType::B));
		reporter_cml2.set_owner(&reporter2);
		UserCmlStore::<Test>::insert(reporter2, reporter_cml_id2, ());
		CmlStore::<Test>::insert(reporter_cml_id2, reporter_cml2);
		assert_ok!(Cml::start_mining(
			Origin::signed(reporter2),
			reporter_cml_id2,
			reporter_tea_id2,
			reporter2,
			b"miner_ip3".to_vec(),
			Some(b"orbit_id3".to_vec()),
		));

		let current_height = 55;
		frame_system::Pallet::<Test>::set_block_number(current_height);

		assert_ok!(Tea::commit_offline_evidence(
			Origin::signed(reporter),
			reporter_tea_id,
			miner_tea_id,
			vec![]
		));
		assert!(OfflineEvidences::<Test>::contains_key(miner_tea_id));
		let evidences = OfflineEvidences::<Test>::get(miner_tea_id);
		assert_eq!(evidences.len(), 1);
		assert_eq!(evidences[0].height, current_height);
		assert_eq!(evidences[0].tea_id, reporter_tea_id);

		let current_height2 = 66;
		frame_system::Pallet::<Test>::set_block_number(current_height2);

		assert_ok!(Tea::commit_offline_evidence(
			Origin::signed(reporter2),
			reporter_tea_id2,
			miner_tea_id,
			vec![]
		));
		assert!(!OfflineEvidences::<Test>::contains_key(miner_tea_id));
		assert_eq!(
			MinerItemStore::<Test>::get(miner_tea_id).status,
			MinerStatus::Offline
		);
	})
}

#[test]
fn commit_offline_evidence_works_if_commit_multi_times_and_not_suspend_the_node() {
	new_test_ext().execute_with(|| {
		let miner = 2;
		let reporter = 3;
		let reporter2 = 4;
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);
		<Test as Config>::Currency::make_free_balance_be(&reporter, 10000);
		<Test as Config>::Currency::make_free_balance_be(&reporter2, 10000);

		let miner_cml_id = 1;
		let miner_tea_id = [1u8; 32];
		let reporter_cml_id = 2;
		let reporter_tea_id = [2u8; 32];
		let reporter_cml_id2 = 3;
		let reporter_tea_id2 = [3u8; 32];

		let mut miner_cml = CML::from_genesis_seed(seed_from_type(miner_cml_id, 100, CmlType::B));
		miner_cml.set_owner(&miner);
		UserCmlStore::<Test>::insert(miner, miner_cml_id, ());
		CmlStore::<Test>::insert(miner_cml_id, miner_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			miner_cml_id,
			miner_tea_id,
			miner,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		let mut reporter_cml =
			CML::from_genesis_seed(seed_from_type(reporter_cml_id, 100, CmlType::B));
		reporter_cml.set_owner(&reporter);
		UserCmlStore::<Test>::insert(reporter, reporter_cml_id, ());
		CmlStore::<Test>::insert(reporter_cml_id, reporter_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(reporter),
			reporter_cml_id,
			reporter_tea_id,
			reporter,
			b"miner_ip2".to_vec(),
			Some(b"orbit_id2".to_vec()),
		));

		let mut reporter_cml2 =
			CML::from_genesis_seed(seed_from_type(reporter_cml_id2, 100, CmlType::B));
		reporter_cml2.set_owner(&reporter2);
		UserCmlStore::<Test>::insert(reporter2, reporter_cml_id2, ());
		CmlStore::<Test>::insert(reporter_cml_id2, reporter_cml2);
		assert_ok!(Cml::start_mining(
			Origin::signed(reporter2),
			reporter_cml_id2,
			reporter_tea_id2,
			reporter2,
			b"miner_ip3".to_vec(),
			Some(b"orbit_id3".to_vec()),
		));

		let current_height = 55;
		frame_system::Pallet::<Test>::set_block_number(current_height);

		assert_ok!(Tea::commit_offline_evidence(
			Origin::signed(reporter),
			reporter_tea_id,
			miner_tea_id,
			vec![]
		));
		assert!(OfflineEvidences::<Test>::contains_key(miner_tea_id));
		let evidences = OfflineEvidences::<Test>::get(miner_tea_id);
		assert_eq!(evidences.len(), 1);
		assert_eq!(evidences[0].height, current_height);
		assert_eq!(evidences[0].tea_id, reporter_tea_id);

		let current_height2 = current_height + OFFLINE_VALID_DURATION as u64 + 1;
		frame_system::Pallet::<Test>::set_block_number(current_height2);

		assert_ok!(Tea::commit_offline_evidence(
			Origin::signed(reporter2),
			reporter_tea_id2,
			miner_tea_id,
			vec![]
		));
		assert!(OfflineEvidences::<Test>::contains_key(miner_tea_id));
		let evidences = OfflineEvidences::<Test>::get(miner_tea_id);
		assert_eq!(evidences.len(), 1);
		assert_eq!(evidences[0].height, current_height2);
		assert_eq!(evidences[0].tea_id, reporter_tea_id2);
	})
}

#[test]
fn commit_offline_evidence_should_fail_if_same_cml_commit_multi_times_in_a_short_time() {
	new_test_ext().execute_with(|| {
		let miner = 2;
		let reporter = 3;
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);
		<Test as Config>::Currency::make_free_balance_be(&reporter, 10000);

		let miner_cml_id = 1;
		let miner_tea_id = [1u8; 32];
		let reporter_cml_id = 2;
		let reporter_tea_id = [2u8; 32];

		let mut miner_cml = CML::from_genesis_seed(seed_from_type(miner_cml_id, 100, CmlType::B));
		miner_cml.set_owner(&miner);
		UserCmlStore::<Test>::insert(miner, miner_cml_id, ());
		CmlStore::<Test>::insert(miner_cml_id, miner_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			miner_cml_id,
			miner_tea_id,
			miner,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		let mut reporter_cml =
			CML::from_genesis_seed(seed_from_type(reporter_cml_id, 100, CmlType::B));
		reporter_cml.set_owner(&reporter);
		UserCmlStore::<Test>::insert(reporter, reporter_cml_id, ());
		CmlStore::<Test>::insert(reporter_cml_id, reporter_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(reporter),
			reporter_cml_id,
			reporter_tea_id,
			reporter,
			b"miner_ip2".to_vec(),
			Some(b"orbit_id2".to_vec()),
		));

		let current_height = 55;
		frame_system::Pallet::<Test>::set_block_number(current_height);

		assert_ok!(Tea::commit_offline_evidence(
			Origin::signed(reporter),
			reporter_tea_id,
			miner_tea_id,
			vec![]
		));
		assert!(OfflineEvidences::<Test>::contains_key(miner_tea_id));
		let evidences = OfflineEvidences::<Test>::get(miner_tea_id);
		assert_eq!(evidences.len(), 1);
		assert_eq!(evidences[0].height, current_height);
		assert_eq!(evidences[0].tea_id, reporter_tea_id);

		let current_height2 = current_height + OFFLINE_VALID_DURATION as u64 - 1;
		frame_system::Pallet::<Test>::set_block_number(current_height2);
		assert_noop!(
			Tea::commit_offline_evidence(
				Origin::signed(reporter),
				reporter_tea_id,
				miner_tea_id,
				vec![]
			),
			Error::<Test>::CanNotCommitOfflineEvidenceMultiTimes
		);

		let current_height3 = current_height + OFFLINE_VALID_DURATION as u64;
		frame_system::Pallet::<Test>::set_block_number(current_height3);
		assert_ok!(Tea::commit_offline_evidence(
			Origin::signed(reporter),
			reporter_tea_id,
			miner_tea_id,
			vec![]
		));
		assert!(OfflineEvidences::<Test>::contains_key(miner_tea_id));
		let evidences = OfflineEvidences::<Test>::get(miner_tea_id);
		assert_eq!(evidences.len(), 1);
		assert_eq!(evidences[0].height, current_height3);
		assert_eq!(evidences[0].tea_id, reporter_tea_id);
	})
}

#[test]
fn commit_offline_evidence_should_fail_if_reporter_not_exist() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Tea::commit_offline_evidence(Origin::signed(1), [1u8; 32], [2u8; 32], vec![]),
			Error::<Test>::NodeNotExist
		);
	})
}

#[test]
fn commit_offline_evidence_should_fail_if_miner_not_exist() {
	new_test_ext().execute_with(|| {
		let reporter = [1u8; 32];
		Nodes::<Test>::insert(reporter, Node::<u64>::default());

		assert_noop!(
			Tea::commit_offline_evidence(Origin::signed(1), reporter, [2u8; 32], vec![]),
			Error::<Test>::OfflineNodeNotExist
		);
	})
}

#[test]
fn commit_offline_evidence_should_fail_if_user_is_not_owner_of_reporter() {
	new_test_ext().execute_with(|| {
		let reporter = [1u8; 32];
		let miner = [2u8; 32];
		Nodes::<Test>::insert(reporter, Node::<u64>::default());
		Nodes::<Test>::insert(miner, Node::<u64>::default());

		assert_noop!(
			Tea::commit_offline_evidence(Origin::signed(1), reporter, miner, vec![]),
			Error::<Test>::InvalidTeaIdOwner
		);
	})
}

#[test]
fn commit_offline_evidence_should_fail_if_reporter_is_not_type_b_cml() {
	new_test_ext().execute_with(|| {
		let miner = 2;
		let reporter = 3;
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);
		<Test as Config>::Currency::make_free_balance_be(&reporter, 10000);

		let miner_cml_id = 1;
		let miner_tea_id = [1u8; 32];
		let reporter_cml_id = 2;
		let reporter_tea_id = [2u8; 32];

		let mut miner_cml = CML::from_genesis_seed(seed_from_type(miner_cml_id, 100, CmlType::B));
		miner_cml.set_owner(&miner);
		UserCmlStore::<Test>::insert(miner, miner_cml_id, ());
		CmlStore::<Test>::insert(miner_cml_id, miner_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			miner_cml_id,
			miner_tea_id,
			miner,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		let mut reporter_cml =
			CML::from_genesis_seed(seed_from_type(reporter_cml_id, 100, CmlType::A));
		reporter_cml.set_owner(&reporter);
		UserCmlStore::<Test>::insert(reporter, reporter_cml_id, ());
		CmlStore::<Test>::insert(reporter_cml_id, reporter_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(reporter),
			reporter_cml_id,
			reporter_tea_id,
			reporter,
			b"miner_ip2".to_vec(),
			Some(b"orbit_id2".to_vec()),
		));

		assert_noop!(
			Tea::commit_offline_evidence(
				Origin::signed(reporter),
				reporter_tea_id,
				miner_tea_id,
				vec![]
			),
			Error::<Test>::OnlyBTypeCmlCanCommitReport
		);
	})
}

#[test]
fn commit_offline_evidence_should_fail_if_offline_cml_is_c_type() {
	new_test_ext().execute_with(|| {
		let miner = 2;
		let reporter = 3;
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);
		<Test as Config>::Currency::make_free_balance_be(&reporter, 10000);

		let miner_cml_id = 1;
		let miner_tea_id = [1u8; 32];
		let reporter_cml_id = 2;
		let reporter_tea_id = [2u8; 32];

		let mut miner_cml = CML::from_genesis_seed(seed_from_type(miner_cml_id, 100, CmlType::C));
		miner_cml.set_owner(&miner);
		UserCmlStore::<Test>::insert(miner, miner_cml_id, ());
		CmlStore::<Test>::insert(miner_cml_id, miner_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			miner_cml_id,
			miner_tea_id,
			miner,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		let mut reporter_cml =
			CML::from_genesis_seed(seed_from_type(reporter_cml_id, 100, CmlType::B));
		reporter_cml.set_owner(&reporter);
		UserCmlStore::<Test>::insert(reporter, reporter_cml_id, ());
		CmlStore::<Test>::insert(reporter_cml_id, reporter_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(reporter),
			reporter_cml_id,
			reporter_tea_id,
			reporter,
			b"miner_ip2".to_vec(),
			Some(b"orbit_id2".to_vec()),
		));

		assert_noop!(
			Tea::commit_offline_evidence(
				Origin::signed(reporter),
				reporter_tea_id,
				miner_tea_id,
				vec![]
			),
			Error::<Test>::OfflineNodeCannotBeTypeC
		);
	})
}

#[test]
fn commit_offline_evidence_should_fail_if_reporter_is_inactive_already() {
	new_test_ext().execute_with(|| {
		let miner = 2;
		let reporter = 3;
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);
		<Test as Config>::Currency::make_free_balance_be(&reporter, 10000);

		let miner_cml_id = 1;
		let miner_tea_id = [1u8; 32];
		let reporter_cml_id = 2;
		let reporter_tea_id = [2u8; 32];

		let mut miner_cml = CML::from_genesis_seed(seed_from_type(miner_cml_id, 100, CmlType::B));
		miner_cml.set_owner(&miner);
		UserCmlStore::<Test>::insert(miner, miner_cml_id, ());
		CmlStore::<Test>::insert(miner_cml_id, miner_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			miner_cml_id,
			miner_tea_id,
			miner,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		let mut reporter_cml =
			CML::from_genesis_seed(seed_from_type(reporter_cml_id, 100, CmlType::B));
		reporter_cml.set_owner(&reporter);
		UserCmlStore::<Test>::insert(reporter, reporter_cml_id, ());
		CmlStore::<Test>::insert(reporter_cml_id, reporter_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(reporter),
			reporter_cml_id,
			reporter_tea_id,
			reporter,
			b"miner_ip2".to_vec(),
			Some(b"orbit_id2".to_vec()),
		));

		MinerItemStore::<Test>::mutate(&miner_tea_id, |item| {
			item.status = MinerStatus::Offline;
		});
		assert_noop!(
			Tea::commit_offline_evidence(
				Origin::signed(reporter),
				reporter_tea_id,
				miner_tea_id,
				vec![]
			),
			Error::<Test>::OfflineNodeNotActive
		);
	})
}

#[test]
fn commit_tips_evidence_works() {
	new_test_ext().execute_with(|| {
		let committer = 2;
		let committer_controller = 22;
		let reporter = 3;
		let reporter_controller = 33;
		let phisher = 4;
		let phisher_controller = 44;
		<Test as Config>::Currency::make_free_balance_be(&committer, 10000);
		<Test as Config>::Currency::make_free_balance_be(&reporter, 10000);
		<Test as Config>::Currency::make_free_balance_be(&phisher, 10000);

		let committer_cml_id = 1;
		let committer_tea_id = [1u8; 32];
		let reporter_cml_id = 2;
		let reporter_tea_id = [2u8; 32];
		let phisher_cml_id = 3;
		let phisher_tea_id = [3u8; 32];

		let mut committer_cml =
			CML::from_genesis_seed(seed_from_type(committer_cml_id, 100, CmlType::B));
		committer_cml.set_owner(&committer);
		UserCmlStore::<Test>::insert(committer, committer_cml_id, ());
		CmlStore::<Test>::insert(committer_cml_id, committer_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(committer),
			committer_cml_id,
			committer_tea_id,
			committer_controller,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		let mut reporter_cml =
			CML::from_genesis_seed(seed_from_type(reporter_cml_id, 100, CmlType::C));
		reporter_cml.set_owner(&reporter);
		UserCmlStore::<Test>::insert(reporter, reporter_cml_id, ());
		CmlStore::<Test>::insert(reporter_cml_id, reporter_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(reporter),
			reporter_cml_id,
			reporter_tea_id,
			reporter_controller,
			b"miner_ip2".to_vec(),
			None,
		));

		let mut phisher_cml =
			CML::from_genesis_seed(seed_from_type(phisher_cml_id, 100, CmlType::B));
		phisher_cml.set_owner(&phisher);
		UserCmlStore::<Test>::insert(phisher, phisher_cml_id, ());
		CmlStore::<Test>::insert(phisher_cml_id, phisher_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(phisher),
			phisher_cml_id,
			phisher_tea_id,
			phisher_controller,
			b"miner_ip3".to_vec(),
			Some(b"orbit_id2".to_vec()),
		));

		let current_height = 55;
		frame_system::Pallet::<Test>::set_block_number(current_height);

		assert_ok!(Tea::commit_tips_evidence(
			Origin::signed(committer_controller),
			committer_tea_id,
			reporter_tea_id,
			phisher_tea_id,
			vec![]
		));

		assert!(TipsEvidences::<Test>::contains_key(reporter_tea_id));
		let evidence = TipsEvidences::<Test>::get(reporter_tea_id);
		assert_eq!(evidence.height, current_height);
		assert_eq!(evidence.target, phisher_tea_id);
		assert_eq!(
			<Test as Config>::Currency::free_balance(committer_controller),
			1 + 195000000
		);
	})
}

#[test]
fn commit_tips_evidence_works_if_phishing_and_commiting_tea_id_are_same() {
	new_test_ext().execute_with(|| {
		let reporter = 3;
		let reporter_controller = 33;
		let phisher = 4;
		let phisher_controller = 44;
		<Test as Config>::Currency::make_free_balance_be(&reporter, 10000);
		<Test as Config>::Currency::make_free_balance_be(&phisher, 10000);

		let reporter_cml_id = 2;
		let reporter_tea_id = [2u8; 32];
		let phisher_cml_id = 3;
		let phisher_tea_id = [3u8; 32];

		let mut reporter_cml =
			CML::from_genesis_seed(seed_from_type(reporter_cml_id, 100, CmlType::C));
		reporter_cml.set_owner(&reporter);
		UserCmlStore::<Test>::insert(reporter, reporter_cml_id, ());
		CmlStore::<Test>::insert(reporter_cml_id, reporter_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(reporter),
			reporter_cml_id,
			reporter_tea_id,
			reporter_controller,
			b"miner_ip2".to_vec(),
			None,
		));

		let mut phisher_cml =
			CML::from_genesis_seed(seed_from_type(phisher_cml_id, 100, CmlType::B));
		phisher_cml.set_owner(&phisher);
		UserCmlStore::<Test>::insert(phisher, phisher_cml_id, ());
		CmlStore::<Test>::insert(phisher_cml_id, phisher_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(phisher),
			phisher_cml_id,
			phisher_tea_id,
			phisher_controller,
			b"miner_ip3".to_vec(),
			Some(b"orbit_id2".to_vec()),
		));

		let current_height = 55;
		frame_system::Pallet::<Test>::set_block_number(current_height);

		assert_ok!(Tea::commit_tips_evidence(
			Origin::signed(phisher_controller),
			phisher_tea_id,
			reporter_tea_id,
			phisher_tea_id,
			vec![]
		));
	})
}

#[test]
fn commit_tips_evidence_should_fail_if_commit_tea_id_not_exist() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Tea::commit_tips_evidence(Origin::signed(1), [1u8; 32], [2u8; 32], [3u8; 32], vec![]),
			Error::<Test>::NodeNotExist
		);
	})
}

#[test]
fn commit_tips_evidence_should_fail_if_report_tea_id_not_exist() {
	new_test_ext().execute_with(|| {
		let committer = [1u8; 32];
		Nodes::<Test>::insert(committer, Node::<u64>::default());

		assert_noop!(
			Tea::commit_tips_evidence(Origin::signed(1), committer, [2u8; 32], [3u8; 32], vec![]),
			Error::<Test>::ReportNodeNotExist
		);
	})
}

#[test]
fn commit_tips_evidence_should_fail_if_phishing_tea_id_not_exist() {
	new_test_ext().execute_with(|| {
		let committer = [1u8; 32];
		let reporter = [2u8; 32];
		Nodes::<Test>::insert(committer, Node::<u64>::default());
		Nodes::<Test>::insert(reporter, Node::<u64>::default());

		assert_noop!(
			Tea::commit_tips_evidence(Origin::signed(1), committer, reporter, [3u8; 32], vec![]),
			Error::<Test>::PhishingNodeNotExist
		);
	})
}

#[test]
fn commit_tips_evidence_should_fail_if_user_is_not_the_owner_of_commit_tea_id() {
	new_test_ext().execute_with(|| {
		let committer = [1u8; 32];
		let reporter = [2u8; 32];
		let phisher = [3u8; 32];
		Nodes::<Test>::insert(committer, Node::<u64>::default());
		Nodes::<Test>::insert(reporter, Node::<u64>::default());
		Nodes::<Test>::insert(phisher, Node::<u64>::default());

		assert_noop!(
			Tea::commit_tips_evidence(Origin::signed(1), committer, reporter, phisher, vec![]),
			Error::<Test>::InvalidTeaIdOwner
		);
	});

	new_test_ext().execute_with(|| {
		let committer = 2;
		let reporter = [2u8; 32];
		let phisher = [3u8; 32];
		Nodes::<Test>::insert(reporter, Node::<u64>::default());
		Nodes::<Test>::insert(phisher, Node::<u64>::default());
		<Test as Config>::Currency::make_free_balance_be(&committer, 10000);

		let committer_cml_id = 1;
		let committer_tea_id = [1u8; 32];

		let mut committer_cml =
			CML::from_genesis_seed(seed_from_type(committer_cml_id, 100, CmlType::B));
		committer_cml.set_owner(&committer);
		UserCmlStore::<Test>::insert(committer, committer_cml_id, ());
		CmlStore::<Test>::insert(committer_cml_id, committer_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(committer),
			committer_cml_id,
			committer_tea_id,
			committer,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		assert_noop!(
			Tea::commit_tips_evidence(
				Origin::signed(1),
				committer_tea_id,
				reporter,
				phisher,
				vec![]
			),
			Error::<Test>::InvalidTeaIdOwner
		);
	});
}

#[test]
fn commit_tips_evidence_should_fail_if_commit_cml_is_not_b_type() {
	new_test_ext().execute_with(|| {
		let committer = 2;
		let reporter = [2u8; 32];
		let phisher = [3u8; 32];
		Nodes::<Test>::insert(reporter, Node::<u64>::default());
		Nodes::<Test>::insert(phisher, Node::<u64>::default());
		<Test as Config>::Currency::make_free_balance_be(&committer, 10000);

		let committer_cml_id = 1;
		let committer_tea_id = [1u8; 32];

		let mut committer_cml =
			CML::from_genesis_seed(seed_from_type(committer_cml_id, 100, CmlType::A));
		committer_cml.set_owner(&committer);
		UserCmlStore::<Test>::insert(committer, committer_cml_id, ());
		CmlStore::<Test>::insert(committer_cml_id, committer_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(committer),
			committer_cml_id,
			committer_tea_id,
			committer,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		assert_noop!(
			Tea::commit_tips_evidence(
				Origin::signed(committer),
				committer_tea_id,
				reporter,
				phisher,
				vec![]
			),
			Error::<Test>::OnlyBTypeCmlCanCommitReport
		);
	})
}

#[test]
fn commit_tips_evidence_should_fail_if_report_cml_is_not_c_type() {
	new_test_ext().execute_with(|| {
		let committer = 2;
		let reporter = 3;
		let phisher = 4;
		<Test as Config>::Currency::make_free_balance_be(&committer, 10000);
		<Test as Config>::Currency::make_free_balance_be(&reporter, 10000);
		<Test as Config>::Currency::make_free_balance_be(&phisher, 10000);

		let committer_cml_id = 1;
		let committer_tea_id = [1u8; 32];
		let reporter_cml_id = 2;
		let reporter_tea_id = [2u8; 32];
		let phisher_cml_id = 3;
		let phisher_tea_id = [3u8; 32];

		let mut committer_cml =
			CML::from_genesis_seed(seed_from_type(committer_cml_id, 100, CmlType::B));
		committer_cml.set_owner(&committer);
		UserCmlStore::<Test>::insert(committer, committer_cml_id, ());
		CmlStore::<Test>::insert(committer_cml_id, committer_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(committer),
			committer_cml_id,
			committer_tea_id,
			committer,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		let mut reporter_cml =
			CML::from_genesis_seed(seed_from_type(reporter_cml_id, 100, CmlType::A));
		reporter_cml.set_owner(&reporter);
		UserCmlStore::<Test>::insert(reporter, reporter_cml_id, ());
		CmlStore::<Test>::insert(reporter_cml_id, reporter_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(reporter),
			reporter_cml_id,
			reporter_tea_id,
			reporter,
			b"miner_ip2".to_vec(),
			None,
		));

		let mut phisher_cml =
			CML::from_genesis_seed(seed_from_type(phisher_cml_id, 100, CmlType::B));
		phisher_cml.set_owner(&phisher);
		UserCmlStore::<Test>::insert(phisher, phisher_cml_id, ());
		CmlStore::<Test>::insert(phisher_cml_id, phisher_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(phisher),
			phisher_cml_id,
			phisher_tea_id,
			phisher,
			b"miner_ip3".to_vec(),
			Some(b"orbit_id2".to_vec()),
		));

		assert_noop!(
			Tea::commit_tips_evidence(
				Origin::signed(committer),
				committer_tea_id,
				reporter_tea_id,
				phisher_tea_id,
				vec![]
			),
			Error::<Test>::OnlyCTypeCmlCanReport
		);
	})
}

#[test]
fn commit_tips_evidence_should_fail_if_phishing_cml_is_c_type() {
	new_test_ext().execute_with(|| {
		let committer = 2;
		let reporter = 3;
		let phisher = 4;
		<Test as Config>::Currency::make_free_balance_be(&committer, 10000);
		<Test as Config>::Currency::make_free_balance_be(&reporter, 10000);
		<Test as Config>::Currency::make_free_balance_be(&phisher, 10000);

		let committer_cml_id = 1;
		let committer_tea_id = [1u8; 32];
		let reporter_cml_id = 2;
		let reporter_tea_id = [2u8; 32];
		let phisher_cml_id = 3;
		let phisher_tea_id = [3u8; 32];

		let mut committer_cml =
			CML::from_genesis_seed(seed_from_type(committer_cml_id, 100, CmlType::B));
		committer_cml.set_owner(&committer);
		UserCmlStore::<Test>::insert(committer, committer_cml_id, ());
		CmlStore::<Test>::insert(committer_cml_id, committer_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(committer),
			committer_cml_id,
			committer_tea_id,
			committer,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		let mut reporter_cml =
			CML::from_genesis_seed(seed_from_type(reporter_cml_id, 100, CmlType::C));
		reporter_cml.set_owner(&reporter);
		UserCmlStore::<Test>::insert(reporter, reporter_cml_id, ());
		CmlStore::<Test>::insert(reporter_cml_id, reporter_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(reporter),
			reporter_cml_id,
			reporter_tea_id,
			reporter,
			b"miner_ip2".to_vec(),
			None,
		));

		let mut phisher_cml =
			CML::from_genesis_seed(seed_from_type(phisher_cml_id, 100, CmlType::C));
		phisher_cml.set_owner(&phisher);
		UserCmlStore::<Test>::insert(phisher, phisher_cml_id, ());
		CmlStore::<Test>::insert(phisher_cml_id, phisher_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(phisher),
			phisher_cml_id,
			phisher_tea_id,
			phisher,
			b"miner_ip3".to_vec(),
			Some(b"orbit_id2".to_vec()),
		));

		assert_noop!(
			Tea::commit_tips_evidence(
				Origin::signed(committer),
				committer_tea_id,
				reporter_tea_id,
				phisher_tea_id,
				vec![]
			),
			Error::<Test>::PhishingNodeCannotBeTypeC
		);
	})
}

#[test]
fn commit_tips_evidence_should_fail_if_repoted_not_long_ago() {
	new_test_ext().execute_with(|| {
		let committer = 2;
		let reporter = 3;
		let phisher = 4;
		<Test as Config>::Currency::make_free_balance_be(&committer, 10000);
		<Test as Config>::Currency::make_free_balance_be(&reporter, 10000);
		<Test as Config>::Currency::make_free_balance_be(&phisher, 10000);

		let committer_cml_id = 1;
		let committer_tea_id = [1u8; 32];
		let reporter_cml_id = 2;
		let reporter_tea_id = [2u8; 32];
		let phisher_cml_id = 3;
		let phisher_tea_id = [3u8; 32];

		let mut committer_cml =
			CML::from_genesis_seed(seed_from_type(committer_cml_id, 100, CmlType::B));
		committer_cml.set_owner(&committer);
		UserCmlStore::<Test>::insert(committer, committer_cml_id, ());
		CmlStore::<Test>::insert(committer_cml_id, committer_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(committer),
			committer_cml_id,
			committer_tea_id,
			committer,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		let mut reporter_cml =
			CML::from_genesis_seed(seed_from_type(reporter_cml_id, 100, CmlType::C));
		reporter_cml.set_owner(&reporter);
		UserCmlStore::<Test>::insert(reporter, reporter_cml_id, ());
		CmlStore::<Test>::insert(reporter_cml_id, reporter_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(reporter),
			reporter_cml_id,
			reporter_tea_id,
			reporter,
			b"miner_ip2".to_vec(),
			None,
		));

		let mut phisher_cml =
			CML::from_genesis_seed(seed_from_type(phisher_cml_id, 100, CmlType::B));
		phisher_cml.set_owner(&phisher);
		UserCmlStore::<Test>::insert(phisher, phisher_cml_id, ());
		CmlStore::<Test>::insert(phisher_cml_id, phisher_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(phisher),
			phisher_cml_id,
			phisher_tea_id,
			phisher,
			b"miner_ip3".to_vec(),
			Some(b"orbit_id2".to_vec()),
		));

		let current_height = 55;
		frame_system::Pallet::<Test>::set_block_number(current_height);

		assert_ok!(Tea::commit_tips_evidence(
			Origin::signed(committer),
			committer_tea_id,
			reporter_tea_id,
			phisher_tea_id,
			vec![]
		));
		assert_eq!(
			TipsEvidences::<Test>::get(reporter_tea_id).height,
			current_height
		);

		frame_system::Pallet::<Test>::set_block_number(
			current_height + TIPS_ALLOWED_DURATION as u64,
		);
		assert_noop!(
			Tea::commit_tips_evidence(
				Origin::signed(committer),
				committer_tea_id,
				reporter_tea_id,
				phisher_tea_id,
				vec![]
			),
			Error::<Test>::RedundantTips
		);

		frame_system::Pallet::<Test>::set_block_number(
			current_height + TIPS_ALLOWED_DURATION as u64 + 1,
		);
		assert_ok!(Tea::commit_tips_evidence(
			Origin::signed(committer),
			committer_tea_id,
			reporter_tea_id,
			phisher_tea_id,
			vec![]
		));
	})
}

#[test]
fn report_node_expired_works() {
	new_test_ext().execute_with(|| {
		let owner = 2;
		let owner_controller = 22;
		<Test as Config>::Currency::make_free_balance_be(&owner, 10000);
		frame_system::Pallet::<Test>::set_block_number(100);

		let (node, tea_id, _, _) = new_node();
		Nodes::<Test>::insert(&tea_id, node);

		let cml_id = 1;
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		cml.set_owner(&owner);
		UserCmlStore::<Test>::insert(owner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(owner),
			cml_id,
			tea_id,
			owner_controller,
			b"miner_ip".to_vec(),
			None,
		));

		let block_height = 100;
		frame_system::Pallet::<Test>::set_block_number(block_height);
		assert_ok!(Tea::report_node_expired(
			Origin::signed(owner_controller),
			tea_id.clone(),
		));

		assert!(VersionExpiredNodes::<Test>::contains_key(&tea_id));
		assert_eq!(VersionExpiredNodes::<Test>::get(&tea_id), block_height);
	})
}

#[test]
fn report_node_expired_should_fail_if_use_stash_account() {
	new_test_ext().execute_with(|| {
		let owner = 2;
		let owner_controller = 22;
		<Test as Config>::Currency::make_free_balance_be(&owner, 10000);
		frame_system::Pallet::<Test>::set_block_number(100);

		let (node, tea_id, _, _) = new_node();
		Nodes::<Test>::insert(&tea_id, node);

		let cml_id = 1;
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		cml.set_owner(&owner);
		UserCmlStore::<Test>::insert(owner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(owner),
			cml_id,
			tea_id,
			owner_controller,
			b"miner_ip".to_vec(),
			None,
		));

		assert_noop!(
			Tea::report_node_expired(Origin::signed(owner), tea_id.clone(),),
			Error::<Test>::InvalidTeaIdOwner
		);
	})
}

#[test]
fn reset_expired_state_works() {
	new_test_ext().execute_with(|| {
		let owner = 2;
		let owner_controller = 22;
		<Test as Config>::Currency::make_free_balance_be(&owner, 10000);
		frame_system::Pallet::<Test>::set_block_number(100);

		let (node, tea_id, _, _) = new_node();
		Nodes::<Test>::insert(&tea_id, node);

		let cml_id = 1;
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		cml.set_owner(&owner);
		UserCmlStore::<Test>::insert(owner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(owner),
			cml_id,
			tea_id,
			owner_controller,
			b"miner_ip".to_vec(),
			None,
		));

		assert_ok!(Tea::report_node_expired(
			Origin::signed(owner_controller),
			tea_id.clone(),
		));
		assert!(VersionExpiredNodes::<Test>::contains_key(&tea_id));

		assert_ok!(Tea::reset_expired_state(
			Origin::signed(owner),
			tea_id.clone(),
		));
		assert!(!VersionExpiredNodes::<Test>::contains_key(&tea_id));
	})
}

#[test]
fn reset_expired_state_should_fail_if_use_controller_account() {
	new_test_ext().execute_with(|| {
		let owner = 2;
		let owner_controller = 22;
		<Test as Config>::Currency::make_free_balance_be(&owner, 10000);
		frame_system::Pallet::<Test>::set_block_number(100);

		let (node, tea_id, _, _) = new_node();
		Nodes::<Test>::insert(&tea_id, node);

		let cml_id = 1;
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		cml.set_owner(&owner);
		UserCmlStore::<Test>::insert(owner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(owner),
			cml_id,
			tea_id,
			owner_controller,
			b"miner_ip".to_vec(),
			None,
		));

		assert_ok!(Tea::report_node_expired(
			Origin::signed(owner_controller),
			tea_id.clone(),
		));

		assert_noop!(
			Tea::reset_expired_state(Origin::signed(owner_controller), tea_id.clone(),),
			Error::<Test>::InvalidTeaIdOwner
		);
	})
}

#[test]
fn report_self_offline_works() {
	new_test_ext().execute_with(|| {
		let miner = 2;
		let miner_controller = 3;
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);

		let miner_cml_id = 1;
		let miner_tea_id = [1u8; 32];

		let mut miner_cml = CML::from_genesis_seed(seed_from_type(miner_cml_id, 100, CmlType::B));
		miner_cml.set_owner(&miner);
		UserCmlStore::<Test>::insert(miner, miner_cml_id, ());
		CmlStore::<Test>::insert(miner_cml_id, miner_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			miner_cml_id,
			miner_tea_id,
			miner_controller,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		let current_height = 66;
		frame_system::Pallet::<Test>::set_block_number(current_height);

		assert_eq!(
			<Test as Config>::Currency::free_balance(miner_controller),
			1
		);

		assert_ok!(Tea::report_self_offline(
			Origin::signed(miner_controller),
			miner_tea_id,
			vec![]
		));
		assert_eq!(
			MinerItemStore::<Test>::get(miner_tea_id).status,
			MinerStatus::Offline
		);
		assert_eq!(
			<Test as Config>::Currency::free_balance(miner_controller),
			195000000 + 1
		);
	})
}

#[test]
fn report_self_offline_failed_if_node_not_exist() {
	new_test_ext().execute_with(|| {
		let miner = 2;
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);

		let miner_tea_id = [1u8; 32];
		assert_noop!(
			Tea::report_self_offline(Origin::signed(miner), miner_tea_id, vec![]),
			Error::<Test>::NodeNotExist
		);
	})
}

#[test]
fn report_self_offline_should_fail_if_user_not_controller_account() {
	new_test_ext().execute_with(|| {
		let miner = 2;
		let miner_controller = 3;
		<Test as Config>::Currency::make_free_balance_be(&miner, 10000);

		let miner_cml_id = 1;
		let miner_tea_id = [1u8; 32];

		let mut miner_cml = CML::from_genesis_seed(seed_from_type(miner_cml_id, 100, CmlType::B));
		miner_cml.set_owner(&miner);
		UserCmlStore::<Test>::insert(miner, miner_cml_id, ());
		CmlStore::<Test>::insert(miner_cml_id, miner_cml);
		assert_ok!(Cml::start_mining(
			Origin::signed(miner),
			miner_cml_id,
			miner_tea_id,
			miner_controller,
			b"miner_ip1".to_vec(),
			Some(b"orbit_id1".to_vec()),
		));

		assert_noop!(
			Tea::report_self_offline(Origin::signed(miner), miner_tea_id, vec![]),
			Error::<Test>::InvalidTeaIdOwner
		);
	})
}

fn new_node<T>() -> (Node<T>, TeaPubKey, TeaPubKey, PeerId)
where
	T: Default + AtLeast32BitUnsigned,
{
	let tea_id = hex!("df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b");
	let ephemeral_id = hex!("ba9147ba50faca694452db7c458e33a9a0322acbaac24bf35db7bb5165dff3ac");
	let peer_id = "12D3KooWLCU9sscGSP7GySktL2awwNouPwrqvZECLaDafpwLKKvt";

	let mut node = Node::default();
	node.tea_id = tea_id.clone();
	(node, tea_id, ephemeral_id, peer_id.as_bytes().to_vec())
}

fn generate_pk_and_signature(
	tea_id: &TeaPubKey,
	target_tea_id: &TeaPubKey,
	is_pass: bool,
) -> ([u8; 32], Signature) {
	use crate::utils::encode_ra_request_content;
	use ed25519_dalek::ed25519::signature::Signature;
	use ed25519_dalek::{Keypair, Signer};
	use rand::rngs::OsRng;

	let mut csprng = OsRng {};
	let kp = Keypair::generate(&mut csprng);
	let signature = kp.sign(encode_ra_request_content(tea_id, target_tea_id, is_pass).as_slice());

	(kp.public.as_bytes().clone(), signature.as_bytes().to_vec())
}

pub fn new_genesis_seed(id: CmlId) -> Seed {
	Seed {
		id,
		cml_type: CmlType::A,
		defrost_schedule: Some(DefrostScheduleType::Team),
		defrost_time: Some(0),
		lifespan: 0,
		performance: 0,
	}
}

pub fn seed_from_lifespan(id: CmlId, lifespan: u32) -> Seed {
	let mut seed = new_genesis_seed(id);
	seed.lifespan = lifespan;
	seed
}

pub fn seed_from_type(id: CmlId, lifespan: u32, cml_type: CmlType) -> Seed {
	let mut seed = new_genesis_seed(id);
	seed.lifespan = lifespan;
	seed.cml_type = cml_type;
	seed
}
