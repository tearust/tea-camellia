#![cfg_attr(not(feature = "std"), no_std)]

pub trait TeaOperation {
	type AccountId: Default + Clone;

	fn add_new_node(machine_id: [u8; 32], sender: &Self::AccountId);
}
