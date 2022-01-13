#![cfg_attr(not(feature = "std"), no_std)]

pub trait TeaOperation {
	type AccountId: Default + Clone;

	fn add_new_node(machine_id: [u8; 32], sender: &Self::AccountId);

	fn update_node_key(old: [u8; 32], new: [u8; 32], sender: &Self::AccountId);

	fn remove_node(machine_id: [u8; 32], sender: &Self::AccountId);
}
