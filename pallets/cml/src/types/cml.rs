use crate::param::{Performance, GENESIS_SEED_A_COUNT, GENESIS_SEED_B_COUNT, GENESIS_SEED_C_COUNT};
use crate::{
	MachineId, MiningProperties, Seed, SeedProperties, StakingCategory, StakingItem,
	StakingProperties, TreeProperties,
};
use codec::{Decode, Encode};
use frame_support::sp_runtime::traits::AtLeast32BitUnsigned;
use frame_support::traits::Get;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;
use sp_std::marker::PhantomData;
use sp_std::prelude::*;

pub type CmlId = u64;

#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CmlType {
	A,
	B,
	C,
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub enum CmlStatus<BlockNumber>
where
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
{
	/// Special status about genesis seeds, `FrozenSeed` can't rot.
	FrozenSeed,
	/// DAO generated seed initial state, or defrost from genesis seeds.
	/// Seed will rot if not plant during a certain period (about 1 week).
	///
	/// The only parameter is block number of the seed sprout at.
	FreshSeed(BlockNumber),
	/// Seed grow up and become tree, the tree have `lifespan` and shall dead if it lived over
	/// than the lifespan (start calculate at `plant_at` block height).
	/// A tree can be planted into a machine, then `machine_id` should not be `None`, and staking
	/// slot should at least have one item.
	Tree,
	/// Tree can staking instead of running on a machine (aka mining), a staking tree will consume
	/// life same as tree, if becomes dead should not staking anymore. Note that a staking cml
	/// can't be auctioned.
	///
	/// The first parameter is the CmlId staked to, and the second parameter is the index in staking
	/// slot.
	Staking(CmlId, u64),
}

impl<BlockNumber> CmlStatus<BlockNumber>
where
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
{
	pub fn valid_conversion(&self, to: &CmlStatus<BlockNumber>) -> bool {
		match *self {
			// Allowed status transfer:
			// `FrozenSeed` => `FreshSeed`
			CmlStatus::FrozenSeed => match *to {
				CmlStatus::FreshSeed(_) => true,
				_ => false,
			},

			// Allowed status transfer:
			// `FreshSeed` => `Tree`
			CmlStatus::FreshSeed(_) => *to == CmlStatus::Tree,

			//	Allowed status transfer:
			// `Tree` => `Staking`
			CmlStatus::Tree => match *to {
				CmlStatus::Staking(_, _) => true,
				_ => false,
			},

			// Allowed status transfer:
			// `Staking` => `Tree`
			CmlStatus::Staking(_, _) => *to == CmlStatus::Tree,
		}
	}
}

#[derive(Debug)]
pub enum CmlError {
	SproutAtIsNone,
	PlantAtIsNone,
	DefrostTimeIsNone,
	DefrostFailed,
	CmlStatusConvertFailed,
	NotValidFreshSeed,
	SlotShouldBeEmpty,
	CmlOwnerIsNone,
	ConfusedStakingType,
	CmlIsNotStaking,
	UnstakingSlotOwnerMismatch,
	InvalidStatusToMine,
	AlreadyHasMachineId,
	CmlIsNotMining,
}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct CML<AccountId, BlockNumber, Balance, RottenDuration>
where
	AccountId: PartialEq + Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
	Balance: Clone,
	RottenDuration: Get<BlockNumber>,
{
	pub intrinsic: Seed,
	status: CmlStatus<BlockNumber>,
	owner: Option<AccountId>,
	/// The time a tree created
	planted_at: Option<BlockNumber>,
	staking_slot: Vec<StakingItem<AccountId, Balance>>,
	machine_id: Option<MachineId>,
	rotten_duration: PhantomData<RottenDuration>,
}

impl<AccountId, BlockNumber, Balance, RottenDuration>
	CML<AccountId, BlockNumber, Balance, RottenDuration>
where
	AccountId: PartialEq + Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
	Balance: Clone,
	RottenDuration: Get<BlockNumber>,
{
	pub(crate) fn from_genesis_seed(intrinsic: Seed) -> Self {
		CML {
			intrinsic,
			status: CmlStatus::FrozenSeed,
			owner: None,
			planted_at: None,
			staking_slot: vec![],
			machine_id: None,
			rotten_duration: PhantomData,
		}
	}

	#[allow(dead_code)]
	pub(crate) fn from_dao_seed(intrinsic: Seed, height: BlockNumber) -> Self {
		CML {
			intrinsic,
			status: CmlStatus::FreshSeed(height),
			owner: None,
			planted_at: None,
			staking_slot: vec![],
			machine_id: None,
			rotten_duration: PhantomData,
		}
	}

	pub fn can_convert(&self, to_status: &CmlStatus<BlockNumber>) -> bool {
		self.status.valid_conversion(to_status)
	}

	pub fn set_owner(&mut self, account: &AccountId) {
		self.owner = Some(account.clone());
	}

	pub fn try_to_convert(&mut self, to_status: CmlStatus<BlockNumber>) -> Result<(), CmlError> {
		if !self.can_convert(&to_status) {
			return Err(CmlError::CmlStatusConvertFailed);
		}

		self.status = to_status;
		Ok(())
	}
}

impl<AccountId, BlockNumber, Balance, RottenDuration> SeedProperties<BlockNumber>
	for CML<AccountId, BlockNumber, Balance, RottenDuration>
where
	AccountId: PartialEq + Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
	Balance: Clone,
	RottenDuration: Get<BlockNumber>,
{
	fn id(&self) -> CmlId {
		self.intrinsic.id
	}

	fn is_seed(&self) -> bool {
		self.is_frozen_seed() || self.is_fresh_seed()
	}

	fn is_frozen_seed(&self) -> bool {
		self.status == CmlStatus::FrozenSeed
	}

	fn is_fresh_seed(&self) -> bool {
		match self.status {
			CmlStatus::FreshSeed(_) => true,
			_ => false,
		}
	}

	fn can_be_defrost(&self, height: &BlockNumber) -> Result<bool, CmlError> {
		Ok(self.is_frozen_seed()
			&& *height
				>= self
					.intrinsic
					.defrost_time
					.ok_or(CmlError::DefrostTimeIsNone)?
					.into())
	}

	fn defrost(&mut self, height: &BlockNumber) -> Result<(), CmlError> {
		if !self.can_be_defrost(height)? {
			return Err(CmlError::DefrostFailed);
		}

		self.try_to_convert(CmlStatus::FreshSeed(height.clone()))
	}

	fn get_sprout_at(&self) -> Option<&BlockNumber> {
		match &self.status {
			CmlStatus::FreshSeed(height) => Some(height),
			_ => None,
		}
	}

	fn get_rotten_duration(&self) -> BlockNumber {
		RottenDuration::get()
	}

	fn convert_to_tree(&mut self, height: &BlockNumber) -> Result<(), CmlError> {
		if !self.is_fresh_seed() || !self.seed_valid(height)? {
			return Err(CmlError::NotValidFreshSeed);
		}
		self.try_to_convert(CmlStatus::Tree)?;
		self.planted_at = Some(height.clone());
		Ok(())
	}

	fn has_rotten(&self, height: &BlockNumber) -> Result<bool, CmlError> {
		Ok(self.is_fresh_seed()
			&& *height
				>= self
					.get_sprout_at()
					.ok_or(CmlError::SproutAtIsNone)?
					.clone() + self.get_rotten_duration())
	}

	fn is_from_genesis(&self) -> bool {
		self.id() < GENESIS_SEED_A_COUNT + GENESIS_SEED_B_COUNT + GENESIS_SEED_C_COUNT
	}
}

impl<AccountId, BlockNumber, Balance, RottenDuration>
	TreeProperties<AccountId, BlockNumber, Balance>
	for CML<AccountId, BlockNumber, Balance, RottenDuration>
where
	AccountId: PartialEq + Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
	Balance: Clone,
	RottenDuration: Get<BlockNumber>,
{
	fn get_plant_at(&self) -> Option<&BlockNumber> {
		self.planted_at.as_ref()
	}

	fn tree_valid(&self, height: &BlockNumber) -> Result<bool, CmlError> {
		Ok(!self.is_seed() && !self.should_dead(height)?)
	}

	fn should_dead(&self, height: &BlockNumber) -> Result<bool, CmlError> {
		Ok(!self.is_seed()
			&& *height
				>= self
					.planted_at
					.as_ref()
					.ok_or(CmlError::PlantAtIsNone)?
					.clone() + self.intrinsic.lifespan.into())
	}

	fn owner(&self) -> Option<&AccountId> {
		self.owner.as_ref()
	}
}

impl<AccountId, BlockNumber, Balance, RottenDuration> StakingProperties<AccountId, Balance>
	for CML<AccountId, BlockNumber, Balance, RottenDuration>
where
	AccountId: PartialEq + Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
	Balance: Clone,
	RottenDuration: Get<BlockNumber>,
{
	fn is_staking(&self) -> bool {
		match self.status {
			CmlStatus::Staking(_, _) => true,
			_ => false,
		}
	}

	fn staking_slots(&self) -> &Vec<StakingItem<AccountId, Balance>> {
		self.staking_slot.as_ref()
	}

	fn staking_slots_mut(&mut self) -> &mut Vec<StakingItem<AccountId, Balance>> {
		self.staking_slot.as_mut()
	}

	fn stake<StakeTo, StakeBlockNumber>(
		&mut self,
		stake_to: &mut StakeTo,
		amount: Option<Balance>,
		cml: Option<CmlId>,
	) -> Result<StakingItem<AccountId, Balance>, CmlError>
	where
		StakeTo: StakingProperties<AccountId, Balance> + SeedProperties<StakeBlockNumber>,
	{
		if (amount.is_some() && cml.is_some()) || (amount.is_none() && cml.is_none()) {
			return Err(CmlError::ConfusedStakingType);
		}

		let staking_item = StakingItem {
			owner: self.owner().ok_or(CmlError::CmlOwnerIsNone)?.clone(),
			category: if amount.is_some() {
				StakingCategory::Tea
			} else {
				StakingCategory::Cml
			},
			amount,
			cml,
		};
		stake_to.staking_slots_mut().push(staking_item.clone());
		self.try_to_convert(CmlStatus::Staking(
			stake_to.id(),
			stake_to.staking_slots().len() as u64 - 1,
		))?;

		Ok(staking_item)
	}

	fn unstake<StakeTo, StakeBlockNumber>(&mut self, stake_to: &mut StakeTo) -> Result<(), CmlError>
	where
		StakeTo: StakingProperties<AccountId, Balance> + SeedProperties<StakeBlockNumber>,
	{
		match self.status {
			CmlStatus::Staking(_, staking_index) => {
				let staking_item = stake_to.staking_slots_mut().remove(staking_index as usize);
				if !staking_item
					.owner
					.eq(self.owner.as_ref().ok_or(CmlError::CmlOwnerIsNone)?)
				{
					return Err(CmlError::UnstakingSlotOwnerMismatch);
				}
				Ok(())
			}
			_ => Err(CmlError::CmlIsNotStaking),
		}
	}
}

impl<AccountId, BlockNumber, Balance, RottenDuration> MiningProperties<AccountId, Balance>
	for CML<AccountId, BlockNumber, Balance, RottenDuration>
where
	AccountId: PartialEq + Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
	Balance: Clone,
	RottenDuration: Get<BlockNumber>,
{
	fn is_mining(&self) -> bool {
		self.machine_id.is_some()
	}

	fn machine_id(&self) -> Option<&MachineId> {
		self.machine_id.as_ref()
	}

	fn get_performance(&self) -> Performance {
		self.intrinsic.performance
	}

	fn swap_first_slot(&mut self, staking_item: StakingItem<AccountId, Balance>) {
		self.staking_slot.remove(0);
		self.staking_slot.insert(0, staking_item);
	}

	fn start_mining(
		&mut self,
		machine_id: MachineId,
		staking_item: StakingItem<AccountId, Balance>,
	) -> Result<(), CmlError> {
		if self.status != CmlStatus::Tree {
			return Err(CmlError::InvalidStatusToMine);
		}

		if self.machine_id.is_some() {
			return Err(CmlError::AlreadyHasMachineId);
		}
		self.machine_id = Some(machine_id);

		if !self.staking_slot.is_empty() {
			return Err(CmlError::SlotShouldBeEmpty);
		}
		self.staking_slot = vec![staking_item];

		Ok(())
	}

	fn stop_mining(&mut self) -> Result<(), CmlError> {
		if !self.is_mining() {
			return Err(CmlError::CmlIsNotMining);
		}

		self.machine_id = None;
		self.staking_slot.clear();
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use crate::{CmlId, CmlType, DefrostScheduleType, Seed};

	mod seed_properties_tests {
		use super::new_genesis_seed;
		use crate::{CmlError, SeedProperties, CML};
		use frame_support::traits::ConstU32;

		#[test]
		fn genesis_seed_works() -> Result<(), CmlError> {
			let id = 10;
			const DEFROST_AT: u32 = 100;
			let mut seed = new_genesis_seed(id);
			seed.defrost_time = Some(DEFROST_AT);
			let cml = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(new_genesis_seed(id));

			assert_eq!(cml.id(), id);
			assert!(cml.is_from_genesis());

			assert!(cml.is_seed());
			assert!(cml.is_frozen_seed());
			assert!(!cml.is_fresh_seed());
			assert!(cml.can_be_defrost(&DEFROST_AT)?);

			Ok(())
		}

		#[test]
		fn defrost_seed_works() -> Result<(), CmlError> {
			const DEFROST_AT: u32 = 100;
			let mut seed = new_genesis_seed(10);
			seed.defrost_time = Some(DEFROST_AT);
			let mut cml = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(seed);

			assert!(!cml.can_be_defrost(&0)?);
			assert!(cml.can_be_defrost(&DEFROST_AT)?);

			assert!(cml.is_frozen_seed());
			cml.defrost(&DEFROST_AT)?;
			assert!(cml.is_fresh_seed());
			Ok(())
		}

		#[test]
		fn genesis_seed_defrost_at_initial() -> Result<(), CmlError> {
			let cml = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(new_genesis_seed(1));
			assert_eq!(cml.intrinsic.defrost_time, Some(0));
			assert!(cml.can_be_defrost(&0)?);
			Ok(())
		}
	}

	mod tree_properties_tests {
		use super::seed_from_lifespan;
		use crate::{CmlError, SeedProperties, TreeProperties, CML};
		use frame_support::traits::ConstU32;

		#[test]
		fn should_dead_works() -> Result<(), CmlError> {
			let lifespan = 10;
			let mut cml = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(
				seed_from_lifespan(1, lifespan),
			);
			assert!(!cml.should_dead(&lifespan)?); // frozen seed cannot dead

			cml.defrost(&0)?;
			assert!(!cml.should_dead(&lifespan)?); // fresh seed cannot dead

			cml.convert_to_tree(&0)?;
			assert_eq!(cml.get_plant_at(), Some(&0));

			assert!(!cml.should_dead(&(lifespan - 1))?);
			assert!(cml.should_dead(&lifespan)?);

			Ok(())
		}
	}

	pub fn new_genesis_seed(id: CmlId) -> Seed {
		Seed {
			id,
			cml_type: CmlType::A,
			defrost_schedule: Some(DefrostScheduleType::Team),
			defrost_time: Some(0),
			lifespan: 0,
			performance: 0,
		}
	}

	fn seed_from_lifespan(id: CmlId, lifespan: u32) -> Seed {
		let mut seed = new_genesis_seed(id);
		seed.lifespan = lifespan;
		seed
	}
}
