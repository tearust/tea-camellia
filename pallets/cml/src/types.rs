use sp_std::prelude::*;

mod cml;
mod coupons;
mod miner;
pub mod param;
mod seeds;
mod staking;

pub use cml::{CmlError, CmlId, CmlResult, CmlStatus, CmlType, CML};
pub use coupons::{Coupon, CouponConfig, GenesisCoupons};
pub use miner::{MachineId, MinerItem, MinerStatus};
pub use seeds::{DefrostScheduleType, GenesisSeeds, Seed};
pub use staking::{
	ServiceTaskPoint, StakingCategory, StakingIndex, StakingItem, StakingSnapshotItem,
	StakingWeight,
};

/// Trait described seed properties about a CML.
pub trait SeedProperties<BlockNumber> {
	/// CML identity.
	fn id(&self) -> CmlId;

	fn cml_type(&self) -> CmlType;

	fn lifespan(&self) -> BlockNumber;

	/// Return `true` if CML is frozen seed or fresh seed, or `false` otherwise.
	fn is_seed(&self) -> bool;

	/// Return `true` if CML is frozen seed, or `false` otherwise.
	fn is_frozen_seed(&self) -> bool;

	/// Return `true` if CML is fresh seed, or `false` otherwise.
	fn is_fresh_seed(&self) -> bool;

	/// Check if a frozen seed can defrost to fresh seed successfuly, return `Ok(())` if can defrost
	/// or `Err(CmlError)` otherwise.
	fn check_defrost(&self, height: &BlockNumber) -> CmlResult;

	/// Defrost a frozen seed to fresh seed.
	fn defrost(&mut self, height: &BlockNumber);

	/// Get sprout height of a fresh seed, if CML is not fresh seed will return `None`.
	fn get_sprout_at(&self) -> Option<&BlockNumber>;

	fn get_fresh_duration(&self) -> BlockNumber;

	fn check_convert_to_tree(&self, height: &BlockNumber) -> CmlResult;

	fn convert_to_tree(&mut self, height: &BlockNumber);

	/// expire happens when a `FreshSeed` not planted after a certain period.
	fn has_expired(&self, height: &BlockNumber) -> bool;

	fn check_seed_validity(&self, height: &BlockNumber) -> CmlResult {
		if self.is_frozen_seed() {
			self.check_defrost(height)?;
		}
		if self.is_fresh_seed() {
			self.check_convert_to_tree(height)?;
		}
		Ok(())
	}

	fn is_from_genesis(&self) -> bool;
}

pub trait TreeProperties<AccountId, BlockNumber, Balance> {
	fn get_plant_at(&self) -> Option<&BlockNumber>;

	fn check_tree_validity(&self, height: &BlockNumber) -> CmlResult;

	fn should_dead(&self, height: &BlockNumber) -> bool;

	fn owner(&self) -> Option<&AccountId>;
}

pub trait StakingProperties<AccountId, BlockNumber, Balance>
where
	BlockNumber: Default + sp_runtime::traits::AtLeast32BitUnsigned + Clone,
{
	fn is_staking(&self) -> bool;

	fn staking_weight(&self) -> StakingWeight;

	fn staking_index(&self) -> Option<(CmlId, StakingIndex)>;

	fn shift_staking_index(&mut self, index: StakingIndex);

	fn staking_slots(&self) -> &Vec<StakingItem<AccountId, Balance>>;

	fn staking_slots_mut(&mut self) -> &mut Vec<StakingItem<AccountId, Balance>>;

	/// can stake would check state of the stake to cml, not the staking cml
	fn check_can_be_stake(
		&self,
		current_height: &BlockNumber,
		amount: &Option<Balance>,
		cml: &Option<CmlId>,
	) -> CmlResult;

	fn check_can_stake_to(&self, current_height: &BlockNumber) -> CmlResult;

	fn stake<StakeEntity>(
		&mut self,
		account: &AccountId,
		current_height: &BlockNumber,
		amount: Option<Balance>,
		cml: Option<&mut StakeEntity>,
	) -> Option<StakingIndex>
	where
		StakeEntity: StakingProperties<AccountId, BlockNumber, Balance>
			+ SeedProperties<BlockNumber>
			+ TreeProperties<AccountId, BlockNumber, Balance>
			+ UtilsProperties<BlockNumber>;

	fn check_unstake<StakeEntity>(
		&self,
		index: &Option<StakingIndex>,
		cml: &Option<&StakeEntity>,
	) -> CmlResult
	where
		StakeEntity: StakingProperties<AccountId, BlockNumber, Balance>
			+ TreeProperties<AccountId, BlockNumber, Balance>
			+ UtilsProperties<BlockNumber>;

	fn unstake<StakeEntity>(
		&mut self,
		index: Option<StakingIndex>,
		cml: Option<&mut StakeEntity>,
	) -> Option<StakingIndex>
	where
		StakeEntity: StakingProperties<AccountId, BlockNumber, Balance>
			+ TreeProperties<AccountId, BlockNumber, Balance>
			+ UtilsProperties<BlockNumber>;
}

pub trait MiningProperties<AccountId, BlockNumber, Balance> {
	fn is_mining(&self) -> bool;

	fn machine_id(&self) -> Option<&MachineId>;

	fn get_peak_performance(&self) -> param::Performance;

	fn calculate_performance(&self, age_percentage: u32) -> param::Performance;

	fn swap_first_slot(&mut self, staking_item: StakingItem<AccountId, Balance>);

	fn check_start_mining(&self, current_height: &BlockNumber) -> CmlResult;

	fn start_mining(
		&mut self,
		machine_id: MachineId,
		staking_item: StakingItem<AccountId, Balance>,
		current_height: &BlockNumber,
	);

	fn stop_mining(&mut self);

	fn migrate_to(&mut self, machine_id: MachineId);
}

pub trait UtilsProperties<BlockNumber>
where
	BlockNumber: Default + sp_runtime::traits::AtLeast32BitUnsigned + Clone,
{
	fn status(&self) -> &CmlStatus<BlockNumber>;

	fn check_convert(&self, to_status: &CmlStatus<BlockNumber>) -> CmlResult;

	fn convert(&mut self, to_status: CmlStatus<BlockNumber>);

	fn try_convert_to_tree(&mut self, current_height: &BlockNumber);
}
