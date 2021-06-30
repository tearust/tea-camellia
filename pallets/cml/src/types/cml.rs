use crate::param::{Performance, GENESIS_SEED_A_COUNT, GENESIS_SEED_B_COUNT, GENESIS_SEED_C_COUNT};
use crate::{
	MachineId, MiningProperties, Seed, SeedProperties, StakingCategory, StakingIndex, StakingItem,
	StakingProperties, StakingWeight, TreeProperties, UtilsProperties,
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
	Staking(CmlId, StakingIndex),
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
pub struct CML<AccountId, BlockNumber, Balance, FreshDuration>
where
	AccountId: PartialEq + Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
	Balance: Clone,
	FreshDuration: Get<BlockNumber>,
{
	intrinsic: Seed,
	status: CmlStatus<BlockNumber>,
	owner: Option<AccountId>,
	/// The time a tree created
	planted_at: Option<BlockNumber>,
	staking_slot: Vec<StakingItem<AccountId, Balance>>,
	machine_id: Option<MachineId>,
	fresh_duration: PhantomData<FreshDuration>,
}

impl<AccountId, BlockNumber, Balance, FreshDuration>
	CML<AccountId, BlockNumber, Balance, FreshDuration>
where
	AccountId: PartialEq + Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
	Balance: Clone,
	FreshDuration: Get<BlockNumber>,
{
	pub(crate) fn from_genesis_seed(intrinsic: Seed) -> Self {
		CML {
			intrinsic,
			status: CmlStatus::FrozenSeed,
			owner: None,
			planted_at: None,
			staking_slot: vec![],
			machine_id: None,
			fresh_duration: PhantomData,
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
			fresh_duration: PhantomData,
		}
	}

	pub fn set_owner(&mut self, account: &AccountId) {
		self.owner = Some(account.clone());
	}
}

impl<AccountId, BlockNumber, Balance, FreshDuration> SeedProperties<BlockNumber>
	for CML<AccountId, BlockNumber, Balance, FreshDuration>
where
	AccountId: PartialEq + Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
	Balance: Clone,
	FreshDuration: Get<BlockNumber>,
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

	fn can_be_defrost(&self, height: &BlockNumber) -> bool {
		if self.intrinsic.defrost_time.is_none() {
			return false;
		}

		self.is_frozen_seed() && *height >= self.intrinsic.defrost_time.unwrap().into()
	}

	fn defrost(&mut self, height: &BlockNumber) {
		if !self.can_be_defrost(height) {
			return;
		}
		self.convert(CmlStatus::FreshSeed(height.clone()))
	}

	fn get_sprout_at(&self) -> Option<&BlockNumber> {
		match &self.status {
			CmlStatus::FreshSeed(height) => Some(height),
			_ => None,
		}
	}

	fn get_fresh_duration(&self) -> BlockNumber {
		FreshDuration::get()
	}

	fn can_convert_to_tree(&self, height: &BlockNumber) -> bool {
		self.is_fresh_seed() && self.seed_valid(height)
	}

	fn convert_to_tree(&mut self, height: &BlockNumber) {
		if !self.can_convert_to_tree(height) {
			return;
		}

		self.convert(CmlStatus::Tree);
		self.planted_at = Some(height.clone());
	}

	fn has_expired(&self, height: &BlockNumber) -> bool {
		if self.get_sprout_at().is_none() {
			return false;
		}

		self.is_fresh_seed()
			&& *height >= self.get_sprout_at().unwrap().clone() + self.get_fresh_duration()
	}

	fn is_from_genesis(&self) -> bool {
		self.id() < GENESIS_SEED_A_COUNT + GENESIS_SEED_B_COUNT + GENESIS_SEED_C_COUNT
	}
}

impl<AccountId, BlockNumber, Balance, FreshDuration> TreeProperties<AccountId, BlockNumber, Balance>
	for CML<AccountId, BlockNumber, Balance, FreshDuration>
where
	AccountId: PartialEq + Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
	Balance: Clone,
	FreshDuration: Get<BlockNumber>,
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

impl<AccountId, BlockNumber, Balance, FreshDuration>
	StakingProperties<AccountId, BlockNumber, Balance>
	for CML<AccountId, BlockNumber, Balance, FreshDuration>
where
	AccountId: PartialEq + Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
	Balance: Clone,
	FreshDuration: Get<BlockNumber>,
{
	fn is_staking(&self) -> bool {
		match self.status {
			CmlStatus::Staking(_, _) => true,
			_ => false,
		}
	}

	fn staking_weight(&self) -> StakingWeight {
		match self.intrinsic.cml_type {
			CmlType::A => 3,
			CmlType::B => 2,
			CmlType::C => 1,
		}
	}

	fn staking_slots(&self) -> &Vec<StakingItem<AccountId, Balance>> {
		self.staking_slot.as_ref()
	}

	fn staking_slots_mut(&mut self) -> &mut Vec<StakingItem<AccountId, Balance>> {
		self.staking_slot.as_mut()
	}

	fn stake<StakeEntity>(
		&mut self,
		account: &AccountId,
		amount: Option<Balance>,
		cml: Option<&mut StakeEntity>,
	) -> Result<StakingIndex, CmlError>
	where
		StakeEntity: StakingProperties<AccountId, BlockNumber, Balance>
			+ SeedProperties<BlockNumber>
			+ UtilsProperties<BlockNumber>,
	{
		if (amount.is_some() && cml.is_some()) || (amount.is_none() && cml.is_none()) {
			return Err(CmlError::ConfusedStakingType);
		}

		let staking_index = self.staking_slots().len() as StakingIndex;

		let staking_item = StakingItem {
			owner: account.clone(),
			category: if amount.is_some() {
				StakingCategory::Tea
			} else {
				StakingCategory::Cml
			},
			amount,
			cml: cml.as_ref().map(|cml| cml.id()),
		};
		if let Some(cml) = cml {
			cml.convert(CmlStatus::Staking(self.id(), staking_index));
		}
		self.staking_slots_mut().push(staking_item.clone());

		Ok(staking_index)
	}

	fn unstake<StakeEntity>(
		&mut self,
		index: Option<StakingIndex>,
		cml: Option<&mut StakeEntity>,
	) -> Result<(), CmlError>
	where
		StakeEntity: StakingProperties<AccountId, BlockNumber, Balance>
			+ TreeProperties<AccountId, BlockNumber, Balance>
			+ UtilsProperties<BlockNumber>,
	{
		if !self.is_mining() {
			return Err(CmlError::CmlIsNotMining);
		}
		if (index.is_some() && cml.is_some()) || (index.is_none() && cml.is_none()) {
			return Err(CmlError::ConfusedStakingType);
		}
		if cml.is_some() && !cml.as_ref().unwrap().is_staking() {
			return Err(CmlError::CmlIsNotStaking);
		}

		if let Some(index) = index {
			let _ = self.staking_slots_mut().remove(index as usize);
		}

		if let Some(cml) = cml {
			match cml.status() {
				CmlStatus::Staking(_, staking_index) => {
					let staking_item = self.staking_slots_mut().remove(*staking_index as usize);
					if !staking_item
						.owner
						.eq(cml.owner().ok_or(CmlError::CmlOwnerIsNone)?)
					{
						return Err(CmlError::UnstakingSlotOwnerMismatch);
					}
					cml.convert(CmlStatus::Tree);
				}
				_ => {} // should never happen
			}
		}

		Ok(())
	}
}

impl<AccountId, BlockNumber, Balance, FreshDuration> MiningProperties<AccountId, Balance>
	for CML<AccountId, BlockNumber, Balance, FreshDuration>
where
	AccountId: PartialEq + Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
	Balance: Clone,
	FreshDuration: Get<BlockNumber>,
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

impl<AccountId, BlockNumber, Balance, FreshDuration> UtilsProperties<BlockNumber>
	for CML<AccountId, BlockNumber, Balance, FreshDuration>
where
	AccountId: PartialEq + Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
	Balance: Clone,
	FreshDuration: Get<BlockNumber>,
{
	fn status(&self) -> &CmlStatus<BlockNumber> {
		&self.status
	}

	fn can_convert(&self, to_status: &CmlStatus<BlockNumber>) -> bool {
		self.status.valid_conversion(to_status)
	}

	fn convert(&mut self, to_status: CmlStatus<BlockNumber>) {
		if !self.can_convert(&to_status) {
			return;
		}

		self.status = to_status;
	}
}

impl<AccountId, BlockNumber, Balance, FreshDuration> Default
	for CML<AccountId, BlockNumber, Balance, FreshDuration>
where
	AccountId: PartialEq + Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
	Balance: Clone,
	FreshDuration: Get<BlockNumber>,
{
	fn default() -> Self {
		CML {
			intrinsic: Seed::default(),
			status: CmlStatus::FrozenSeed,
			owner: None,
			planted_at: None,
			staking_slot: vec![],
			machine_id: None,
			fresh_duration: PhantomData,
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::{CmlId, CmlType, DefrostScheduleType, Seed};

	mod seed_properties_tests {
		use super::new_genesis_seed;
		use crate::{CmlError, CmlStatus, SeedProperties, StakingProperties, TreeProperties, CML};
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
			assert!(cml.can_be_defrost(&DEFROST_AT));

			Ok(())
		}

		#[test]
		fn defrost_seed_works() {
			const DEFROST_AT: u32 = 100;
			let mut seed = new_genesis_seed(10);
			seed.defrost_time = Some(DEFROST_AT);
			let mut cml = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(seed);

			assert!(!cml.can_be_defrost(&0));
			assert!(cml.can_be_defrost(&DEFROST_AT));

			assert!(cml.is_frozen_seed());
			cml.defrost(&DEFROST_AT);
			assert!(cml.is_fresh_seed());
			assert_eq!(cml.get_sprout_at(), Some(&DEFROST_AT));
		}

		#[test]
		fn defrost_failed_when_defrost_time_is_none() {
			let mut seed = new_genesis_seed(10);
			seed.defrost_time = None;
			let mut cml = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(seed);

			assert!(!cml.can_be_defrost(&u32::MAX));
			assert!(cml.is_frozen_seed());
			cml.defrost(&u32::MAX);
			assert!(cml.is_frozen_seed());
			assert_eq!(cml.get_sprout_at(), None);
		}

		#[test]
		fn defrost_failed_before_defrost_time() {
			const DEFROST_AT: u32 = 100;
			let mut seed = new_genesis_seed(10);
			seed.defrost_time = Some(DEFROST_AT);
			let mut cml = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(seed);

			assert!(!cml.can_be_defrost(&(DEFROST_AT - 1)));
			cml.defrost(&(DEFROST_AT - 1));
			assert!(cml.is_frozen_seed());
			assert_eq!(cml.get_sprout_at(), None);
		}

		#[test]
		fn genesis_seed_defrost_at_initial() {
			let cml = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(new_genesis_seed(1));
			assert_eq!(cml.intrinsic.defrost_time, Some(0));
			assert!(cml.can_be_defrost(&0));
		}

		#[test]
		fn seed_expire_works() {
			let mut cml =
				CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(new_genesis_seed(1));
			let sprout_at = 100;
			let fresh_duration = cml.get_fresh_duration();
			cml.defrost(&sprout_at);

			assert!(cml.is_fresh_seed());
			assert!(!cml.has_expired(&sprout_at));
			assert!(!cml.has_expired(&(sprout_at + fresh_duration - 1)));
			assert!(cml.has_expired(&(sprout_at + fresh_duration)));
		}

		#[test]
		fn cml_that_not_fresh_seed_will_never_expire() {
			let mut cml =
				CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(new_genesis_seed(1));

			assert!(!cml.is_fresh_seed()); // frozen seed is not fresh seed
			assert!(!cml.has_expired(&u32::MAX));

			cml.defrost(&0);
			cml.convert_to_tree(&0);

			assert!(!cml.is_fresh_seed()); // tree is not fresh seed
			assert!(!cml.has_expired(&u32::MAX));

			cml.stake::<CML<u32, u32, u128, ConstU32<10>>>(&1, Some(1), None)
				.unwrap();

			assert!(!cml.is_fresh_seed()); // staking tree is not fresh seed
			assert!(!cml.has_expired(&u32::MAX));
		}

		#[test]
		fn convert_to_tree_works() {
			let mut cml =
				CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(new_genesis_seed(1));

			let defrost_at = 100;
			cml.defrost(&defrost_at);

			assert!(cml.can_convert_to_tree(&(defrost_at + 1)));
			cml.convert_to_tree(&(defrost_at + 1));
			assert_eq!(cml.status, CmlStatus::Tree);
			assert_eq!(cml.get_plant_at(), Some(&(defrost_at + 1)));
		}

		#[test]
		fn convert_to_tree_failed_if_frozen() {
			let mut cml =
				CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(new_genesis_seed(1));
			assert!(cml.is_frozen_seed());

			assert!(!cml.can_convert_to_tree(&0));
			cml.convert_to_tree(&0);
			assert!(cml.is_frozen_seed());
		}

		#[test]
		fn convert_to_tree_failed_if_seed_has_expired() {
			let mut cml =
				CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(new_genesis_seed(1));
			cml.defrost(&0);

			let fresh_duration = cml.get_fresh_duration();
			assert!(!cml.can_convert_to_tree(&fresh_duration));
			cml.convert_to_tree(&fresh_duration);
			assert!(cml.is_fresh_seed());
		}

		#[test]
		fn convert_to_tree_failed_if_it_is_tree_already() {
			let mut cml =
				CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(new_genesis_seed(1));

			cml.defrost(&0);
			cml.convert_to_tree(&0);

			assert_eq!(cml.status, CmlStatus::Tree);
			assert_eq!(cml.get_plant_at(), Some(&0));

			assert!(!cml.can_convert_to_tree(&1));
			cml.convert_to_tree(&1);
			assert_ne!(cml.get_plant_at(), Some(&1));
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

			cml.defrost(&0);
			assert!(!cml.should_dead(&lifespan)?); // fresh seed cannot dead

			cml.convert_to_tree(&0);
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
