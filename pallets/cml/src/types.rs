
// use super::*;
use codec::{Decode, Encode};
use sp_std::prelude::*;
use sp_runtime::RuntimeDebug;

pub type Dai = u32;

#[derive(Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct StakingItem<AccountId, AssetId> {
	owner: AccountId,
	category: Vec<u8>,   // seed, tea
	amount: u32,  // amount of tea
	cml: AssetId,
}

#[derive(Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct MinerItem {
	id: Vec<u8>,
	group: Vec<u8>,
	ip: Vec<u8>,
	status: Vec<u8>,
}

#[derive(Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct CML<AssetId, AccountId, BlockNumber> {
  id: AssetId,
  group: Vec<u8>,   // nitro
	status: Vec<u8>,  // Seed_Live, Seed_Frozen, Seed_Planting, CML_Live
	life_time: BlockNumber, // whole life time for CML
	lock_time: BlockNumber, 
	mining_rate: u8, // 8 - 12, default 10
	staking_slot: Vec<StakingItem<AccountId, AssetId>>,
	created_at: BlockNumber,
	miner_id: Vec<u8>,
}