use sp_std::prelude::*;

mod cml;
mod miner;
pub mod param;
mod seeds;
mod staking;
mod vouchers;

pub use cml::{CmlError, CmlId, CmlStatus, CmlType, CML};
pub use miner::{MachineId, MinerItem, MinerStatus};
pub use seeds::{DefrostScheduleType, GenesisSeeds, Seed};
pub use staking::{
	MinerStakingPoint, ServiceTaskPoint, StakingCategory, StakingIndex, StakingItem,
	StakingSnapshotItem, StakingWeight,
};
pub use vouchers::{GenesisVouchers, Voucher, VoucherConfig};

pub trait SeedProperties<BlockNumber> {
	fn id(&self) -> CmlId;

	fn is_seed(&self) -> bool;

	fn is_frozen_seed(&self) -> bool;

	fn is_fresh_seed(&self) -> bool;

	fn can_be_defrost(&self, height: &BlockNumber) -> bool;

	fn defrost(&mut self, height: &BlockNumber);

	fn get_sprout_at(&self) -> Option<&BlockNumber>;

	fn get_fresh_duration(&self) -> BlockNumber;

	fn convert_to_tree(&mut self, height: &BlockNumber) -> Result<(), CmlError>;

	/// expire happens when a `FreshSeed` not planted after a certain period.
	fn has_expired(&self, height: &BlockNumber) -> Result<bool, CmlError>;

	fn seed_valid(&self, height: &BlockNumber) -> Result<bool, CmlError> {
		Ok(self.can_be_defrost(height) || !self.has_expired(height)?)
	}

	fn is_from_genesis(&self) -> bool;
}

pub trait TreeProperties<AccountId, BlockNumber, Balance> {
	fn get_plant_at(&self) -> Option<&BlockNumber>;

	fn tree_valid(&self, height: &BlockNumber) -> Result<bool, CmlError>;

	fn should_dead(&self, height: &BlockNumber) -> Result<bool, CmlError>;

	fn owner(&self) -> Option<&AccountId>;
}

pub trait StakingProperties<AccountId, BlockNumber, Balance>
where
	BlockNumber: Default + sp_runtime::traits::AtLeast32BitUnsigned + Clone,
{
	fn is_staking(&self) -> bool;

	fn staking_weight(&self) -> StakingWeight;

	fn staking_slots(&self) -> &Vec<StakingItem<AccountId, Balance>>;

	fn staking_slots_mut(&mut self) -> &mut Vec<StakingItem<AccountId, Balance>>;

	fn stake<StakeEntity>(
		&mut self,
		account: &AccountId,
		amount: Option<Balance>,
		cml: Option<&mut StakeEntity>,
	) -> Result<StakingIndex, CmlError>
	where
		StakeEntity: StakingProperties<AccountId, BlockNumber, Balance>
			+ SeedProperties<BlockNumber>
			+ UtilsProperties<BlockNumber>;

	fn unstake<StakeEntity>(
		&mut self,
		index: Option<StakingIndex>,
		cml: Option<&mut StakeEntity>,
	) -> Result<(), CmlError>
	where
		BlockNumber: Default + sp_runtime::traits::AtLeast32BitUnsigned + Clone,
		StakeEntity: StakingProperties<AccountId, BlockNumber, Balance>
			+ TreeProperties<AccountId, BlockNumber, Balance>
			+ UtilsProperties<BlockNumber>;
}

pub trait MiningProperties<AccountId, Balance> {
	fn is_mining(&self) -> bool;

	fn machine_id(&self) -> Option<&MachineId>;

	fn get_performance(&self) -> param::Performance;

	fn swap_first_slot(&mut self, staking_item: StakingItem<AccountId, Balance>);

	fn start_mining(
		&mut self,
		machine_id: MachineId,
		staking_item: StakingItem<AccountId, Balance>,
	) -> Result<(), CmlError>;

	fn stop_mining(&mut self) -> Result<(), CmlError>;
}

pub trait UtilsProperties<BlockNumber>
where
	BlockNumber: Default + sp_runtime::traits::AtLeast32BitUnsigned + Clone,
{
	fn status(&self) -> &CmlStatus<BlockNumber>;

	fn can_convert(&self, to_status: &CmlStatus<BlockNumber>) -> bool;

	fn convert(&mut self, to_status: CmlStatus<BlockNumber>);
}
