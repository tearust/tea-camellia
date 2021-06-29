use crate::CmlId;
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
pub struct MinerItem<Balance>
where
	Balance: Clone,
{
	pub cml_id: CmlId,
	pub id: MachineId,
	pub ip: Vec<u8>,
	pub status: MinerStatus,
	pub credit_amount: Option<Balance>,
}
