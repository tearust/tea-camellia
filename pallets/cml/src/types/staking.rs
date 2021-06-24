use crate::CmlId;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

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
