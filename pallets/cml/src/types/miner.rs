use crate::CmlId;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

pub type MachineId = [u8; 32];

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub enum MinerStatus {
	Active,
	Offline,
	ScheduleDown,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct MinerItem<BlockNumber>
where
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
{
	pub cml_id: CmlId,
	pub id: MachineId,
	pub ip: Vec<u8>,
	pub status: MinerStatus,
	pub orbitdb_id: Option<Vec<u8>>,
	pub suspend_height: Option<BlockNumber>,
	pub schedule_down_height: Option<BlockNumber>,
}

impl<BlockNumber> Default for MinerItem<BlockNumber>
where
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
{
	fn default() -> Self {
		MinerItem {
			cml_id: 0,
			id: [0; 32],
			ip: vec![],
			orbitdb_id: None,
			status: MinerStatus::Offline,
			suspend_height: None,
			schedule_down_height: None,
		}
	}
}
