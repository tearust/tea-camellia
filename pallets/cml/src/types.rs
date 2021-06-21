// use super::*;
use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub use seeds::{GenesisSeeds, Seed};
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;
pub use vouchers::{GenesisVouchers, VoucherConfig};

pub mod param;
pub mod seeds;
pub mod vouchers;

pub type CmlId = u64;

pub type MachineId = [u8; 32];

#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CmlType {
	A,
	B,
	C,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug)]
pub struct Voucher {
	pub amount: u32,
	pub cml_type: CmlType,
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
	pub id: MachineId,
	pub ip: Vec<u8>,
	pub status: MinerStatus,
}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct CML<AccountId, BlockNumber, Balance>
where
	AccountId: Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
{
	pub intrinsic: Seed,
	pub group: CmlGroup,
	pub status: CmlStatus,
	pub mining_rate: u8, // 8 - 12, default 10
	pub staking_slot: Vec<StakingItem<AccountId, CmlId, Balance>>,
	pub planted_at: BlockNumber,
	pub machine_id: Option<MachineId>,
}

impl<AccountId, BlockNumber, Balance> CML<AccountId, BlockNumber, Balance>
where
	AccountId: Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
{
	pub(crate) fn new(intrinsic: Seed) -> Self {
		CML {
			intrinsic,
			group: CmlGroup::Nitro, // todo init from intrinsic
			status: CmlStatus::SeedFrozen,
			mining_rate: 10,
			staking_slot: vec![],
			planted_at: BlockNumber::default(),
			machine_id: None,
		}
	}

	pub fn is_seed(&self) -> bool {
		match self.status {
			CmlStatus::SeedFrozen | CmlStatus::SeedLive => true,
			_ => false,
		}
	}

	pub fn id(&self) -> CmlId {
		self.intrinsic.id
	}

	pub fn should_death(&self, height: BlockNumber) -> bool {
		height > self.planted_at.clone() + self.intrinsic.lifespan.into()
	}

	pub fn owner(&self) -> Option<AccountId> {
		self.staking_slot.get(0).map(|slot| slot.owner.clone())
	}
}
