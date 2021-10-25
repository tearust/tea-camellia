use crate::{mock::*, types::*, BuiltinMiners, BuiltinNodes, Config, Error, Nodes};
use frame_support::{assert_noop, assert_ok, traits::Currency};
use hex_literal::hex;
use pallet_cml::{CmlId, CmlStore, CmlType, DefrostScheduleType, Seed, UserCmlStore, CML};
use sp_runtime::traits::AtLeast32BitUnsigned;
use tea_interface::TeaOperation;

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

		assert_ok!(Tea::update_node_profile(
			Origin::signed(builtin_miner),
			tea_id.clone(),
			ephemeral_id.clone(),
			Vec::new(),
			Vec::new(),
			peer_id,
		));
		assert!(Tea::is_builtin_node(&tea_id));

		let new_node = Nodes::<Test>::get(&tea_id);
		assert_eq!(ephemeral_id, new_node.ephemeral_id);
		assert_eq!(NodeStatus::Active, new_node.status);
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
				Vec::new(),
				peer_id,
			),
			Error::<Test>::InvalidBuiltinMiner
		);
	})
}

#[test]
fn normal_node_update_node_profile_works() {
	new_test_ext().execute_with(|| {
		let owner = 2;
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
			b"miner_ip".to_vec(),
			None,
		));

		assert_ok!(Tea::update_node_profile(
			Origin::signed(owner),
			tea_id.clone(),
			ephemeral_id.clone(),
			Vec::new(),
			Vec::new(),
			peer_id,
		));
		assert!(!Tea::is_builtin_node(&tea_id));

		let new_node = Nodes::<Test>::get(&tea_id);
		assert_eq!(ephemeral_id, new_node.ephemeral_id);
		assert_eq!(NodeStatus::Pending, new_node.status);
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
				Vec::new(),
				peer_id,
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
		let (mut node, tea_id, _, _) = new_node();

		let mut ra_nodes: Vec<(TeaPubKey, bool)> = Vec::new();
		let validator_1 = hex!("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
		let validator_2 = hex!("c7e016fad0796bb68594e49a6ef1942cf7e73497e69edb32d19ba2fab3696596");
		let validator_3 = hex!("c9380fde1ba795fc656ab08ab4ef4482cf554790fd3abcd4642418ae8fb5fd52");
		let validator_4 = hex!("2754d7e9c73ced5b302e12464594110850980027f8f83c469e8145eef59220b6");

		let owner1 = 2;
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
			b"miner_ip1".to_vec(),
			None,
		));

		let owner2 = 2;
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
			b"miner_ip2".to_vec(),
			None,
		));

		let owner3 = 3;
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
			b"miner_ip3".to_vec(),
			None,
		));

		let owner4 = 4;
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
		Tea::update_validator_groups_count();

		assert_ok!(Tea::remote_attestation(
			Origin::signed(owner1),
			validator_1,
			tea_id.clone(),
			true,
			signature1
		));
		assert_eq!(Nodes::<Test>::get(&tea_id).status, NodeStatus::Pending);

		assert_ok!(Tea::remote_attestation(
			Origin::signed(owner2),
			validator_2,
			tea_id.clone(),
			false,
			signature2
		));
		assert_eq!(Nodes::<Test>::get(&tea_id).status, NodeStatus::Pending);

		assert_ok!(Tea::remote_attestation(
			Origin::signed(owner3),
			validator_3,
			tea_id.clone(),
			true,
			signature3
		));
		assert_eq!(Nodes::<Test>::get(&tea_id).status, NodeStatus::Active);

		assert_ok!(Tea::remote_attestation(
			Origin::signed(owner4),
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
			b"miner_ip".to_vec(),
			None,
		));

		frame_system::Pallet::<Test>::set_block_number(
			last_update_height + MAX_ALLOWED_RA_COMMIT_DURATION as u64 + 1,
		);
		assert_noop!(
			Tea::remote_attestation(
				Origin::signed(owner),
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
			b"miner_ip".to_vec(),
			None,
		));

		assert_noop!(
			Tea::remote_attestation(
				Origin::signed(owner),
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
