use crate::CmlId;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

pub type MachineId = [u8; 32];

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub enum MinerStatus {
	Active,
	Offline,
	// ...
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct MinerItem {
	pub cml_id: CmlId,
	pub id: MachineId,
	pub ip: Vec<u8>,
	pub status: MinerStatus,
	pub orbitdb_id: Vec<u8>,
}

impl Default for MinerItem {
	fn default() -> Self {
		MinerItem {
			cml_id: 0,
			id: [0; 32],
			ip: vec![],
			orbitdb_id: vec![],
			status: MinerStatus::Offline,
		}
	}
}
