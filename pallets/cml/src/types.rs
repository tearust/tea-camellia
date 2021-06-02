
// use super::*;
use codec::{Decode, Encode};
use sp_std::prelude::*;
use sp_runtime::RuntimeDebug;

pub type Dai = u32;

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug)]
pub enum CmlStatus {
	SeedLive, 
	SeedFrozen, 
	CmlLive, 
	Staking, 
	Dead,
}


#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct StakingItem<AccountId, CmlId> {
	pub owner: AccountId,
	pub category: Vec<u8>,   // seed, tea
	pub amount: u32,  // amount of tea
	pub cml: Option<CmlId>,
}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct MinerItem {
	pub id: Vec<u8>,
	pub group: Vec<u8>,
	pub ip: Vec<u8>,
	pub status: Vec<u8>,
}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct CML<CmlId, AccountId, BlockNumber> {
  pub id: CmlId,
  pub group: Vec<u8>,   // nitro
	pub status: CmlStatus,
	pub life_time: BlockNumber, // whole life time for CML
	pub lock_time: BlockNumber, 
	pub mining_rate: u8, // 8 - 12, default 10
	pub staking_slot: Vec<StakingItem<AccountId, CmlId>>,
	pub created_at: BlockNumber,
	pub miner_id: Vec<u8>,
}