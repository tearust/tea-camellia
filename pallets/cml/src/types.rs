use sp_std::prelude::*;

mod cml;
mod miner;
pub mod param;
mod seeds;
mod staking;
mod vouchers;

pub use cml::{CmlError, CmlId, CmlType, CML};
pub use miner::{MachineId, MinerItem, MinerStatus};
pub use seeds::{DefrostScheduleType, GenesisSeeds, Seed};
pub use staking::{StakingCategory, StakingItem};
pub use vouchers::{GenesisVouchers, Voucher, VoucherConfig};

pub trait SeedProperties<BlockNumber> {
	fn id(&self) -> CmlId;

	fn is_seed(&self) -> bool;

	fn is_frozen_seed(&self) -> bool;

	fn is_fresh_seed(&self) -> bool;

	fn can_be_defrost(&self, height: &BlockNumber) -> Result<bool, CmlError>;

	fn defrost(&mut self, height: &BlockNumber) -> Result<(), CmlError>;

	fn get_sprout_at(&self) -> Option<&BlockNumber>;

	fn get_rotten_duration(&self) -> BlockNumber;

	fn convert_to_tree(&mut self, height: &BlockNumber) -> Result<(), CmlError>;

	/// rotten happens when a `FreshSeed` not planted after a certain period.
	fn has_rotten(&self, height: &BlockNumber) -> Result<bool, CmlError>;

	fn seed_valid(&self, height: &BlockNumber) -> Result<bool, CmlError> {
		Ok(self.can_be_defrost(height)? || !self.has_rotten(height)?)
	}
}

pub trait TreeProperties<AccountId, BlockNumber, Balance> {
	fn get_plant_at(&self) -> Option<&BlockNumber>;

	fn tree_valid(&self, height: &BlockNumber) -> Result<bool, CmlError>;

	fn should_dead(&self, height: &BlockNumber) -> Result<bool, CmlError>;
}

pub trait StakingProperties<AccountId, Balance> {
	fn is_staking(&self) -> bool;

	fn staking_slots(&self) -> &Vec<StakingItem<AccountId, Balance>>;

	fn staking_slots_mut(&mut self) -> &mut Vec<StakingItem<AccountId, Balance>>;

	fn stake<StakeTo, BlockNumber>(
		&mut self,
		stake_to: &mut StakeTo,
		amount: Option<Balance>,
		cml: Option<CmlId>,
	) -> Result<StakingItem<AccountId, Balance>, CmlError>
	where
		StakeTo: StakingProperties<AccountId, Balance> + SeedProperties<BlockNumber>;

	fn unstake<StakeTo, BlockNumber>(&mut self, stake_to: &mut StakeTo) -> Result<(), CmlError>
	where
		StakeTo: StakingProperties<AccountId, Balance> + SeedProperties<BlockNumber>;
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
