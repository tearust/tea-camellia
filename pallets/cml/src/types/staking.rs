use crate::CmlId;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

/// Types of weight:
/// - Balance: 1
/// - A class cml: 3
/// - B class cml: 2
/// - C class cml: 1
pub type StakingWeight = u64;

pub type ServiceTaskPoint = u64;

pub type MinerStakingPoint = u64;

pub type StakingIndex = u64;

#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug)]
pub enum StakingCategory {
	Tea,
	Cml,
}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct StakingItem<AccountId, Balance> {
	pub owner: AccountId,
	pub category: StakingCategory,
	pub amount: Option<Balance>,
	pub cml: Option<CmlId>,
}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct StakingSnapshotItem<AccountId> {
	pub owner: AccountId,
	pub weight: StakingWeight,
	pub staking_at: StakingIndex,
}

impl<AccountId, Balance> Default for StakingItem<AccountId, Balance>
where
	AccountId: Default,
	Balance: Default,
{
	fn default() -> Self {
		StakingItem {
			owner: AccountId::default(),
			category: StakingCategory::Tea,
			amount: Some(Balance::default()),
			cml: None,
		}
	}
}

impl<AccountId> Default for StakingSnapshotItem<AccountId>
where
	AccountId: Default,
{
	fn default() -> Self {
		StakingSnapshotItem {
			owner: AccountId::default(),
			weight: 0,
			staking_at: 0,
		}
	}
}
