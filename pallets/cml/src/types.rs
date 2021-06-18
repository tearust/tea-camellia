// use super::*;
use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub use seeds::{GenesisSeeds, Seed};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

pub mod param;
pub mod seeds;

pub type CmlId = u64;

pub type MachineId = [u8; 32];

#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CmlType {
	A,
	B,
	C,
}

#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum VoucherUnlockType {
	CoreTeam,
	SeedRound,
	ARound,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug)]
pub struct Voucher {
	pub amount: u32,
	pub lock: Option<u32>,
	pub unlock_type: Option<VoucherUnlockType>,
	pub group: CmlType,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug)]
pub enum CmlStatus {
	SeedLive,
	SeedFrozen,
	CmlLive,
	Staking,
	Dead,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug)]
pub enum MinerStatus {
	Active,
	Offline,
	// ...
}

#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug)]
pub enum CmlGroup {
	Nitro,
	Tpm,
}

#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug)]
pub enum StakingCategory {
	Tea,
	Cml,
}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct StakingItem<AccountId, CmlId, Balance> {
	pub owner: AccountId,
	pub category: StakingCategory,
	pub amount: Option<Balance>,
	pub cml: Option<CmlId>,
}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct MinerItem {
	pub id: Vec<u8>,
	pub ip: Vec<u8>,
	pub status: MinerStatus,
}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct CML<AccountId, BlockNumber, Balance> {
	pub id: CmlId,
	pub group: CmlGroup,
	pub status: CmlStatus,
	pub life_time: BlockNumber, // whole life time for CML
	pub lock_time: BlockNumber,
	pub mining_rate: u8, // 8 - 12, default 10
	pub staking_slot: Vec<StakingItem<AccountId, CmlId, Balance>>,
	pub created_at: BlockNumber,
	pub miner_id: Vec<u8>,
}
