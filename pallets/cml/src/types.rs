
// use super::*;
use codec::{Decode, Encode};
use sp_std::prelude::*;
use sp_runtime::RuntimeDebug;
use crate::cml;

pub type Dai = u32;

pub type CmlId<T> = <T as cml::Config>::AssetId;

#[derive(Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct StakingItem<AccountId, AssetId> {
	pub owner: AccountId,
	pub category: Vec<u8>,   // seed, tea
	pub amount: u32,  // amount of tea
	pub cml: Option<AssetId>,
}

#[derive(Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct MinerItem {
	pub id: Vec<u8>,
	pub group: Vec<u8>,
	pub ip: Vec<u8>,
	pub status: Vec<u8>,
}

#[derive(Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct CML<AssetId, AccountId, BlockNumber> {
  pub id: AssetId,
  pub group: Vec<u8>,   // nitro
	pub status: Vec<u8>,  // Seed_Live, Seed_Frozen, Seed_Planting, CML_Live
	pub life_time: BlockNumber, // whole life time for CML
	pub lock_time: BlockNumber, 
	pub mining_rate: u8, // 8 - 12, default 10
	pub staking_slot: Vec<StakingItem<AccountId, AssetId>>,
	pub created_at: BlockNumber,
	pub miner_id: Vec<u8>,
}