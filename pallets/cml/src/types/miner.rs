use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

pub type MachineId = [u8; 32];

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug)]
pub enum MinerStatus {
	Active,
	Offline,
	// ...
}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct MinerItem {
	pub id: MachineId,
	pub ip: Vec<u8>,
	pub status: MinerStatus,
}
