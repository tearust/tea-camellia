//! Benchmarking setup for pallet-tea

use super::*;
#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
    add_new_node {
        let public = hex_to_key("e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44");
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(caller), public)

    update_node_profile {
        let (node, tea_id, ephemeral_id, peer_id) = dummy_new_node();
        Nodes::<T>::insert(&tea_id, node);
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(caller), tea_id, ephemeral_id, Vec::new(), Vec::new(), peer_id)

    remote_attestation {
        let (mut node, tea_id, ephemeral_id, peer_id) = dummy_new_node();
        let ra_nodes = generate_ra_nodes();

        let validator_id = ra_nodes.last().unwrap().0.clone();
        let validator_ephemeral_id = [223, 29, 70, 56, 216, 225, 70, 148, 24, 223, 187, 97, 37, 233, 158, 213, 178, 176, 90, 82, 52, 111, 18, 139, 243, 175, 205, 28, 41, 224, 109, 54];
        let signature = [135, 89, 133, 152, 88, 167, 186, 93, 214, 178, 213, 158, 106, 253, 244, 178, 12, 190, 182, 190, 218, 164, 197, 70, 94, 137, 118, 159, 199, 205, 143, 204, 139, 247, 71, 90, 21, 72, 151, 58, 93, 212, 148, 7, 49, 199, 101, 53, 254, 153, 144, 194, 22, 166, 234, 181, 31, 48, 7, 90, 185, 78, 210, 11];

        node.ra_nodes = ra_nodes;
        Nodes::<T>::insert(&tea_id, node);

        let mut validator_node = Node::default();
        validator_node.ephemeral_id = validator_ephemeral_id;
        Nodes::<T>::insert(&validator_id, validator_node);

        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(caller), validator_id, tea_id, true, signature.to_vec())

    update_runtime_activity {
        let (node, tea_id, _, _) = dummy_new_node();
        Nodes::<T>::insert(&tea_id, node);

        let ephemeral_id = [165, 58, 163, 23, 13, 97, 185, 160, 186, 118, 53, 125, 233, 94, 151, 57, 7, 247, 104, 108, 190, 115, 86, 119, 36, 182, 201, 201, 236, 59, 199, 1];
        let signature = [234, 207, 225, 211, 4, 214, 65, 93, 178, 30, 245, 64, 118, 189, 228, 151, 196, 15, 211, 0, 205, 132, 74, 20, 246, 250, 210, 24, 137, 5, 105, 1, 62, 126, 246, 130, 22, 226, 14, 230, 67, 113, 192, 50, 238, 185, 170, 89, 240, 136, 46, 130, 103, 117, 240, 3, 177, 6, 26, 54, 19, 217, 136, 9];
        let caller: T::AccountId = whitelisted_caller();
    }: _(RawOrigin::Signed(caller), tea_id, None, ephemeral_id, signature.to_vec())
}

impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test,);

fn dummy_new_node<T>() -> (Node<T>, TeaPubKey, TeaPubKey, PeerId)
where
    T: Default,
{
    let tea_id = hex_to_key("df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b");
    let ephemeral_id =
        hex_to_key("ba9147ba50faca694452db7c458e33a9a0322acbaac24bf35db7bb5165dff3ac");
    let peer_id = "12D3KooWLCU9sscGSP7GySktL2awwNouPwrqvZECLaDafpwLKKvt";

    let mut node = Node::default();
    node.tea_id = tea_id.clone();
    (node, tea_id, ephemeral_id, peer_id.as_bytes().to_vec())
}

fn generate_ra_nodes() -> Vec<(TeaPubKey, bool)> {
    vec![
        "e9889b1c54ccd6cf184901ded892069921d76f7749b6f73bed6cf3b9be1a8a44",
        "c7e016fad0796bb68594e49a6ef1942cf7e73497e69edb32d19ba2fab3696596",
        "c9380fde1ba795fc656ab08ab4ef4482cf554790fd3abcd4642418ae8fb5fd52",
        "2754d7e9c73ced5b302e12464594110850980027f8f83c469e8145eef59220b6",
    ]
    .iter()
    .map(|v| (hex_to_key(v), false))
    .collect::<Vec<_>>()
}

fn hex_to_key(s: &str) -> TeaPubKey {
    let mut key = [0; 32];
    hex::decode_to_slice(s, &mut key as &mut [u8]).unwrap();
    key
}
