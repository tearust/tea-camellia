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
pub enum CmlError {
	/// Defrost time should have value when defrost.
	CmlDefrostTimeIsNone,
	/// Cml should be frozen seed.
	CmlShouldBeFrozenSeed,
	/// Cml is still in frozen locked period that cannot be defrosted.
	CmlStillInFrozenLockedPeriod,
	/// Cml should be fresh seed.
	CmlShouldBeFreshSeed,
	/// Cml in fresh seed state and have expired the fresh duration.
	CmlFreshSeedExpired,
	/// Cml is tree means that can't be frozen seed or fresh seed.
	CmlShouldBeTree,
	/// Cml has over the lifespan.
	CmlShouldDead,
	/// Cml is mining that can start mining again or staking with.
	CmlIsMiningAlready,
	/// Cml is staking that can't staking again or start mining.
	CmlIsStaking,
	/// Before start mining staking slot should be empty.
	CmlStakingSlotNotEmpty,
	/// Means we cannot decide staking type from given params.
	ConfusedStakingType,
	/// Cml is not mining that can't stake to.
	CmlIsNotMining,
	/// Cml is not staking to current miner that can't unstake.
	CmlIsNotStakingToCurrentMiner,
	/// Cml staking index over than staking slot length, that means point to not exist staking.
	CmlStakingIndexOverflow,
	/// Cml staking item owner is none, that can't identify staking belongs.
	CmlOwnerIsNone,
	/// Cml staking item owner and the owner field of cml item not match.
	CmlOwnerMismatch,
	/// Cml is not staking that can't unstake.
	CmlIsNotStaking,
	/// Some status that can't convert to another status.
	CmlInvalidStatusConversion,
}
pub type CmlResult = Result<(), CmlError>;

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
	pub fn from_genesis_seed(intrinsic: Seed) -> Self {
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

	pub(crate) fn seed_or_tree_valid(&self, current_height: &BlockNumber) -> CmlResult {
		if self.is_seed() {
			self.check_seed_validity(current_height)?;
		} else {
			self.check_tree_validity(current_height)?;
		}
		Ok(())
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

	fn check_defrost(&self, height: &BlockNumber) -> CmlResult {
		if self.intrinsic.defrost_time.is_none() {
			return Err(CmlError::CmlDefrostTimeIsNone);
		}
		if !self.is_frozen_seed() {
			return Err(CmlError::CmlShouldBeFrozenSeed);
		}
		if *height < self.intrinsic.defrost_time.unwrap().into() {
			return Err(CmlError::CmlStillInFrozenLockedPeriod);
		}

		Ok(())
	}

	fn defrost(&mut self, height: &BlockNumber) {
		if self.check_defrost(height).is_err() {
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

	fn check_convert_to_tree(&self, height: &BlockNumber) -> CmlResult {
		if !self.is_fresh_seed() {
			return Err(CmlError::CmlShouldBeFreshSeed);
		}
		if self.has_expired(height) {
			return Err(CmlError::CmlFreshSeedExpired);
		}
		Ok(())
	}

	fn convert_to_tree(&mut self, height: &BlockNumber) {
		if self.check_convert_to_tree(height).is_err() {
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

	fn check_tree_validity(&self, height: &BlockNumber) -> CmlResult {
		if self.is_seed() {
			return Err(CmlError::CmlShouldBeTree);
		}

		if self.should_dead(height) {
			return Err(CmlError::CmlShouldDead);
		}
		Ok(())
	}

	fn should_dead(&self, height: &BlockNumber) -> bool {
		if self.is_seed() {
			return false;
		}
		// planted at is none should never happen
		if self.planted_at.is_none() {
			return true;
		}

		*height >= self.planted_at.as_ref().unwrap().clone() + self.intrinsic.lifespan.into()
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

	fn staking_index(&self) -> Option<(CmlId, StakingIndex)> {
		match self.status {
			CmlStatus::Staking(id, index) => Some((id, index)),
			_ => None,
		}
	}

	fn shift_staking_index(&mut self, index: StakingIndex) {
		match self.status {
			CmlStatus::Staking(id, _) => {
				self.status = CmlStatus::Staking(id, index);
			}
			_ => {}
		}
	}

	fn staking_slots(&self) -> &Vec<StakingItem<AccountId, Balance>> {
		self.staking_slot.as_ref()
	}

	fn staking_slots_mut(&mut self) -> &mut Vec<StakingItem<AccountId, Balance>> {
		self.staking_slot.as_mut()
	}

	fn check_can_be_stake(
		&self,
		current_height: &BlockNumber,
		amount: &Option<Balance>,
		cml: &Option<CmlId>,
	) -> CmlResult {
		if (amount.is_some() && cml.is_some()) || (amount.is_none() && cml.is_none()) {
			return Err(CmlError::ConfusedStakingType);
		}
		if !self.is_mining() {
			return Err(CmlError::CmlIsNotMining);
		}
		if self.should_dead(current_height) {
			return Err(CmlError::CmlShouldDead);
		}
		Ok(())
	}

	fn check_can_stake_to(&self, current_height: &BlockNumber) -> CmlResult {
		self.seed_or_tree_valid(current_height)?;
		if self.is_staking() {
			return Err(CmlError::CmlIsStaking);
		}
		if self.is_mining() {
			return Err(CmlError::CmlIsMiningAlready);
		}
		Ok(())
	}

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
			+ UtilsProperties<BlockNumber>,
	{
		let cml_id = cml.as_ref().map(|cml| cml.id());
		if self
			.check_can_be_stake(current_height, &amount, &cml_id)
			.is_err()
		{
			return None;
		}
		if let Some(ref cml) = cml {
			if cml.check_can_stake_to(current_height).is_err() {
				return None;
			}
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
			cml: cml_id,
		};
		if let Some(cml) = cml {
			cml.try_convert_to_tree(current_height);
			cml.convert(CmlStatus::Staking(self.id(), staking_index));
		}
		self.staking_slots_mut().push(staking_item.clone());

		Some(staking_index)
	}

	fn check_unstake<StakeEntity>(
		&self,
		index: &Option<StakingIndex>,
		cml: &Option<&StakeEntity>,
	) -> CmlResult
	where
		StakeEntity: StakingProperties<AccountId, BlockNumber, Balance>
			+ TreeProperties<AccountId, BlockNumber, Balance>
			+ UtilsProperties<BlockNumber>,
	{
		if (index.is_some() && cml.is_some()) || (index.is_none() && cml.is_none()) {
			return Err(CmlError::ConfusedStakingType);
		}

		if !self.is_mining() {
			return Err(CmlError::CmlIsNotMining);
		}

		if let Some(cml) = cml {
			match cml.status() {
				CmlStatus::Staking(stakee_id, index) => {
					// not stake to me
					if *stakee_id != self.id() {
						return Err(CmlError::CmlIsNotStakingToCurrentMiner);
					}

					if self.staking_slots().len() <= *index as usize {
						return Err(CmlError::CmlStakingIndexOverflow);
					}

					if cml.owner().is_none() {
						return Err(CmlError::CmlOwnerIsNone);
					}

					if !self.staking_slots()[*index as usize]
						.owner
						.eq(cml.owner().unwrap())
					{
						return Err(CmlError::CmlOwnerMismatch);
					}
				}
				_ => return Err(CmlError::CmlIsNotStaking),
			}
		}

		if let Some(index) = index {
			if self.staking_slots().len() <= *index as usize {
				return Err(CmlError::CmlStakingIndexOverflow);
			}
		}

		Ok(())
	}

	fn unstake<StakeEntity>(
		&mut self,
		index: Option<StakingIndex>,
		cml: Option<&mut StakeEntity>,
	) -> Option<StakingIndex>
	where
		StakeEntity: StakingProperties<AccountId, BlockNumber, Balance>
			+ TreeProperties<AccountId, BlockNumber, Balance>
			+ UtilsProperties<BlockNumber>,
	{
		// todo improve the map of cml if possible later
		if self
			.check_unstake(&index, &cml.as_ref().map(|c| &**c))
			.is_err()
		{
			return None;
		}

		if let Some(index) = index {
			let _ = self.staking_slots_mut().remove(index as usize);
			return Some(index);
		}

		if let Some(cml) = cml {
			match cml.status().clone() {
				CmlStatus::Staking(_, staking_index) => {
					let _staking_item = self.staking_slots_mut().remove(staking_index as usize);
					cml.convert(CmlStatus::Tree);
					return Some(staking_index);
				}
				_ => {} // should never happen
			}
		}
		None
	}
}

impl<AccountId, BlockNumber, Balance, FreshDuration>
	MiningProperties<AccountId, BlockNumber, Balance>
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

	fn check_start_mining(&self, current_height: &BlockNumber) -> CmlResult {
		self.seed_or_tree_valid(current_height)?;

		if self.is_mining() {
			return Err(CmlError::CmlIsMiningAlready);
		}
		if self.is_staking() {
			return Err(CmlError::CmlIsStaking);
		}
		if !self.staking_slot.is_empty() {
			return Err(CmlError::CmlStakingSlotNotEmpty);
		}
		Ok(())
	}

	fn start_mining(
		&mut self,
		machine_id: MachineId,
		staking_item: StakingItem<AccountId, Balance>,
		current_height: &BlockNumber,
	) {
		if self.check_start_mining(current_height).is_err() {
			return;
		}
		self.try_convert_to_tree(current_height);
		self.machine_id = Some(machine_id);
		self.staking_slot = vec![staking_item];
	}

	fn stop_mining(&mut self) {
		if !self.is_mining() {
			return;
		}

		self.machine_id = None;
		self.staking_slot.clear();
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

	fn check_convert(&self, to_status: &CmlStatus<BlockNumber>) -> CmlResult {
		if !self.status.valid_conversion(to_status) {
			return Err(CmlError::CmlInvalidStatusConversion);
		}
		Ok(())
	}

	fn convert(&mut self, to_status: CmlStatus<BlockNumber>) {
		if self.check_convert(&to_status).is_err() {
			return;
		}

		self.status = to_status;
	}

	fn try_convert_to_tree(&mut self, current_height: &BlockNumber) {
		if self.is_frozen_seed() {
			self.defrost(current_height);
		}
		if self.is_fresh_seed() {
			self.convert_to_tree(current_height);
		}
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
	use crate::tests::seed_from_lifespan;
	use crate::{
		CmlId, MiningProperties, SeedProperties, StakingCategory, StakingItem, StakingProperties,
		CML,
	};
	use frame_support::traits::ConstU32;

	mod seed_properties_tests {
		use crate::tests::{new_genesis_seed, seed_from_lifespan};
		use crate::types::cml::tests::default_miner;
		use crate::{CmlStatus, SeedProperties, StakingProperties, TreeProperties, CML};
		use frame_support::traits::ConstU32;

		#[test]
		fn genesis_seed_works() {
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
			assert!(cml.check_defrost(&DEFROST_AT).is_ok());
		}

		#[test]
		fn defrost_seed_works() {
			const DEFROST_AT: u32 = 100;
			let mut seed = new_genesis_seed(10);
			seed.defrost_time = Some(DEFROST_AT);
			let mut cml = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(seed);

			assert!(cml.check_defrost(&0).is_err());
			assert!(cml.check_defrost(&DEFROST_AT).is_ok());

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

			assert!(cml.check_defrost(&u32::MAX).is_err());
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

			assert!(cml.check_defrost(&(DEFROST_AT - 1)).is_err());
			cml.defrost(&(DEFROST_AT - 1));
			assert!(cml.is_frozen_seed());
			assert_eq!(cml.get_sprout_at(), None);
		}

		#[test]
		fn genesis_seed_defrost_at_initial() {
			let cml = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(new_genesis_seed(1));
			assert_eq!(cml.intrinsic.defrost_time, Some(0));
			assert!(cml.check_defrost(&0).is_ok());
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
		fn seed_valid_works() {
			const DEFROST_AT: u32 = 100;
			const LIFESPAN: u32 = 100;
			let mut seed = seed_from_lifespan(10, LIFESPAN);
			seed.defrost_time = Some(DEFROST_AT);
			let mut cml = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(seed);

			assert!(cml.check_defrost(&(DEFROST_AT - 1)).is_err());
			assert!(cml.check_seed_validity(&(DEFROST_AT - 1)).is_err());

			cml.defrost(&DEFROST_AT);
			assert!(cml.is_fresh_seed());
			assert!(!cml.has_expired(&DEFROST_AT));
			assert!(cml.check_seed_validity(&DEFROST_AT).is_ok());

			assert!(cml.has_expired(&(DEFROST_AT + LIFESPAN)));
			assert!(cml.check_seed_validity(&(DEFROST_AT + LIFESPAN)).is_err());
		}

		#[test]
		fn cml_that_not_fresh_seed_will_never_expire() {
			let mut cml =
				CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(seed_from_lifespan(1, 100));

			assert!(!cml.is_fresh_seed()); // frozen seed is not fresh seed
			assert!(!cml.has_expired(&u32::MAX));

			cml.defrost(&0);
			cml.convert_to_tree(&0);

			assert!(!cml.is_fresh_seed()); // tree is not fresh seed
			assert!(!cml.has_expired(&u32::MAX));

			let mut miner_cml = default_miner(2, 100);
			let index =
				miner_cml.stake::<CML<u32, u32, u128, ConstU32<10>>>(&1, &0, None, Some(&mut cml));
			assert_eq!(index, Some(1));

			assert!(!cml.is_fresh_seed()); // staking tree is not fresh seed
			assert!(!cml.has_expired(&u32::MAX));
		}

		#[test]
		fn convert_to_tree_works() {
			let mut cml =
				CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(new_genesis_seed(1));

			let defrost_at = 100;
			cml.defrost(&defrost_at);

			assert!(cml.check_convert_to_tree(&(defrost_at + 1)).is_ok());
			cml.convert_to_tree(&(defrost_at + 1));
			assert_eq!(cml.status, CmlStatus::Tree);
			assert_eq!(cml.get_plant_at(), Some(&(defrost_at + 1)));
		}

		#[test]
		fn convert_to_tree_failed_if_frozen() {
			let mut cml =
				CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(new_genesis_seed(1));
			assert!(cml.is_frozen_seed());

			assert!(cml.check_convert_to_tree(&0).is_err());
			cml.convert_to_tree(&0);
			assert!(cml.is_frozen_seed());
		}

		#[test]
		fn convert_to_tree_failed_if_seed_has_expired() {
			let mut cml =
				CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(new_genesis_seed(1));
			cml.defrost(&0);

			let fresh_duration = cml.get_fresh_duration();
			assert!(cml.check_convert_to_tree(&fresh_duration).is_err());
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

			assert!(cml.check_convert_to_tree(&1).is_err());
			cml.convert_to_tree(&1);
			assert_ne!(cml.get_plant_at(), Some(&1));
		}
	}

	mod tree_properties_tests {
		use crate::tests::seed_from_lifespan;
		use crate::{CmlStatus, SeedProperties, TreeProperties, CML};
		use frame_support::traits::ConstU32;

		#[test]
		fn should_dead_works() {
			let lifespan = 10;
			let mut cml = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(
				seed_from_lifespan(1, lifespan),
			);
			assert!(cml.is_seed());
			assert!(!cml.should_dead(&lifespan)); // frozen seed cannot dead

			cml.defrost(&0);
			assert!(!cml.should_dead(&lifespan)); // fresh seed cannot dead

			cml.convert_to_tree(&0);
			assert_eq!(cml.get_plant_at(), Some(&0));

			assert!(!cml.should_dead(&(lifespan - 1)));
			assert!(cml.should_dead(&lifespan));
		}

		#[test]
		fn tree_will_always_dead_if_plant_at_is_none() {
			let lifespan = 10;
			let mut cml = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(
				seed_from_lifespan(1, lifespan),
			);
			assert!(cml.planted_at.is_none());

			cml.status = CmlStatus::Tree;
			assert!(cml.should_dead(&0));
			assert!(cml.should_dead(&lifespan));

			cml.status = CmlStatus::Staking(2, 1);
			assert!(cml.should_dead(&0));
			assert!(cml.should_dead(&lifespan));
		}

		#[test]
		fn seed_should_never_dead() {
			let lifespan = 10;
			let mut cml = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(
				seed_from_lifespan(1, lifespan),
			);

			assert!(!cml.should_dead(&lifespan)); // frozen seed never dead
			assert!(!cml.should_dead(&u32::MAX)); // frozen seed never dead

			cml.defrost(&0);

			assert!(!cml.should_dead(&lifespan)); // fresh seed never dead
			assert!(!cml.should_dead(&u32::MAX)); // fresh seed never dead
		}
	}

	mod staking_properties_tests {
		use crate::tests::seed_from_lifespan;
		use crate::types::cml::tests::{default_miner, default_staking_cml_pair};
		use crate::{
			CmlStatus, MiningProperties, SeedProperties, StakingCategory, StakingProperties,
			TreeProperties, CML,
		};
		use frame_support::traits::ConstU32;

		#[test]
		fn stake_with_balance_works() {
			let mut miner = default_miner(1, 100);
			assert!(miner.check_can_be_stake(&0, &Some(1), &None).is_ok());

			let amount = 1000;
			let account_id = 1;
			let index = miner.stake::<CML<u32, u32, u128, ConstU32<10>>>(
				&account_id,
				&0,
				Some(amount),
				None,
			);
			assert_eq!(index, Some(1));

			assert_eq!(miner.staking_slots().len(), 2);
			let staking_item = miner.staking_slots().get(1).unwrap();
			assert_eq!(staking_item.amount, Some(amount));
			assert_eq!(staking_item.owner, account_id);
			assert_eq!(staking_item.category, StakingCategory::Tea);
			assert_eq!(staking_item.cml, None);
		}

		#[test]
		fn stake_with_cml_works() {
			let account_id = 10;
			let cml_id = 11;
			let mut staker = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(
				seed_from_lifespan(cml_id, 100),
			);
			staker.defrost(&0);
			staker.convert_to_tree(&0);
			assert_eq!(staker.status, CmlStatus::Tree);
			assert!(staker.check_can_stake_to(&0).is_ok());

			let miner_id = 22;
			let mut miner = default_miner(miner_id, 100);
			assert!(miner
				.check_can_be_stake(&0, &None, &Some(staker.id()))
				.is_ok());

			let index = miner.stake::<CML<u32, u32, u128, ConstU32<10>>>(
				&account_id,
				&0,
				None,
				Some(&mut staker),
			);
			assert_eq!(index, Some(1));

			assert_eq!(miner.staking_slots().len(), 2);
			let staking_item = miner.staking_slots().get(1).unwrap();
			assert_eq!(staking_item.amount, None);
			assert_eq!(staking_item.owner, account_id);
			assert_eq!(staking_item.category, StakingCategory::Cml);
			assert_eq!(staking_item.cml, Some(cml_id));

			assert!(staker.is_staking());
			match staker.status {
				CmlStatus::Staking(id, staking_index) => {
					assert_eq!(id, miner_id);
					assert_eq!(staking_index, index.unwrap());
				}
				_ => {
					assert!(false); // should not happen
				}
			}
		}

		#[test]
		fn stake_with_frozen_seed_works() {
			let mut staker =
				CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(seed_from_lifespan(1, 100));
			assert!(staker.is_frozen_seed());
			assert!(staker.check_can_stake_to(&0).is_ok());

			let mut miner = default_miner(2, 100);
			assert!(miner
				.check_can_be_stake(&0, &None, &Some(staker.id()))
				.is_ok());

			let index =
				miner.stake::<CML<u32, u32, u128, ConstU32<10>>>(&1, &0, None, Some(&mut staker));
			assert_eq!(index, Some(1));
		}

		#[test]
		fn stake_with_fresh_seed_works() {
			let mut staker =
				CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(seed_from_lifespan(1, 100));
			staker.defrost(&0);
			assert!(staker.is_fresh_seed());
			assert!(staker.check_can_stake_to(&0).is_ok());

			let mut miner = default_miner(2, 100);
			assert!(miner
				.check_can_be_stake(&0, &None, &Some(staker.id()))
				.is_ok());

			let index =
				miner.stake::<CML<u32, u32, u128, ConstU32<10>>>(&1, &0, None, Some(&mut staker));
			assert_eq!(index, Some(1));
		}

		#[test]
		fn cannot_be_stake_if_not_mining() {
			let mut miner =
				CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(seed_from_lifespan(2, 100));

			assert!(!miner.is_mining()); // frozen seed cannot be stake
			assert!(miner.check_can_be_stake(&0, &Some(1), &None).is_err());
			assert_eq!(
				miner.stake::<CML<u32, u32, u128, ConstU32<10>>>(&1, &0, Some(1), None),
				None
			);

			miner.defrost(&0);
			assert!(!miner.is_mining()); // fresh seed cannot be stake
			assert!(miner.check_can_be_stake(&0, &Some(1), &None).is_err());
			assert_eq!(
				miner.stake::<CML<u32, u32, u128, ConstU32<10>>>(&1, &0, Some(1), None),
				None
			);

			miner.convert_to_tree(&0);
			assert!(!miner.is_mining()); // tree that not mining cannot be stake
			assert!(miner.check_can_be_stake(&0, &Some(1), &None).is_err());
			assert_eq!(
				miner.stake::<CML<u32, u32, u128, ConstU32<10>>>(&1, &0, Some(1), None),
				None
			);
		}

		#[test]
		fn skaking_amount_and_cml_should_have_one_and_only_one() {
			let mut staker =
				CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(seed_from_lifespan(1, 100));
			assert!(staker.check_can_stake_to(&0).is_ok());

			let mut miner = default_miner(2, 100);

			// amount and cml both none
			assert!(miner.check_can_be_stake(&0, &None, &None).is_err());
			assert_eq!(
				miner.stake::<CML<u32, u32, u128, ConstU32<10>>>(&1, &0, None, None),
				None
			);

			// amount and cml both have value
			assert!(miner
				.check_can_be_stake(&0, &Some(1), &Some(staker.id()))
				.is_err());
			assert_eq!(
				miner.stake::<CML<u32, u32, u128, ConstU32<10>>>(
					&1,
					&0,
					Some(1),
					Some(&mut staker)
				),
				None
			);
		}

		#[test]
		fn cannot_be_stake_if_cml_is_dead() {
			let lifespan = 100;
			let mut miner = default_miner(1, lifespan);

			assert!(miner.should_dead(&lifespan));
			assert!(miner
				.check_can_be_stake(&lifespan, &Some(1), &None)
				.is_err());

			let index =
				miner.stake::<CML<u32, u32, u128, ConstU32<10>>>(&1, &lifespan, Some(1), None);
			assert_eq!(index, None);
		}

		#[test]
		fn cannot_stake_to_if_cml_is_invalid() {
			let mut miner = default_miner(2, u32::MAX);

			let lifespan = 100;
			let mut staker = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(
				seed_from_lifespan(1, lifespan),
			);
			staker.defrost(&0);

			// fresh seed cannot stake if has expired fresh duration
			let fresh_duration = staker.get_fresh_duration();
			assert!(staker.check_seed_validity(&fresh_duration).is_err());
			assert!(staker.check_can_stake_to(&fresh_duration).is_err());
			assert_eq!(
				miner.stake::<CML<u32, u32, u128, ConstU32<10>>>(
					&1,
					&fresh_duration,
					None,
					Some(&mut staker)
				),
				None
			);

			// tree cannot stake if is dead already
			staker.convert_to_tree(&0);
			assert!(staker.should_dead(&lifespan));
			assert!(staker.check_can_stake_to(&lifespan).is_err());
			assert_eq!(
				miner.stake::<CML<u32, u32, u128, ConstU32<10>>>(
					&1,
					&lifespan,
					None,
					Some(&mut staker)
				),
				None
			);
		}

		#[test]
		fn cannot_stake_to_if_cml_is_mining() {
			let mut miner = default_miner(2, 100);
			let mut staker = default_miner(1, 100);

			assert!(staker.is_mining());
			assert!(staker.check_can_stake_to(&0).is_err());
			assert_eq!(
				miner.stake::<CML<u32, u32, u128, ConstU32<10>>>(&1, &0, None, Some(&mut staker)),
				None
			);
		}

		#[test]
		fn can_stake_to_if_cml_is_staking_already() {
			let mut staker =
				CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(seed_from_lifespan(1, 100));
			let mut miner = default_miner(2, 100);
			assert_eq!(
				miner.stake::<CML<u32, u32, u128, ConstU32<10>>>(&1, &0, None, Some(&mut staker)),
				Some(1)
			);

			assert!(staker.is_staking());
			assert!(staker.check_can_stake_to(&0).is_err());
			assert_eq!(
				miner.stake::<CML<u32, u32, u128, ConstU32<10>>>(&1, &0, None, Some(&mut staker)),
				None
			);
		}

		#[test]
		fn unstake_with_balance_works() {
			let mut miner = default_miner(1, 100);
			assert!(miner.check_can_be_stake(&0, &Some(1), &None).is_ok());

			let index = miner.stake::<CML<u32, u32, u128, ConstU32<10>>>(&1, &0, Some(1000), None);
			assert_eq!(index, Some(1));

			assert!(miner
				.check_unstake::<CML<u32, u32, u128, ConstU32<10>>>(&index, &None)
				.is_ok());
			miner.unstake::<CML<u32, u32, u128, ConstU32<10>>>(index, None);

			assert_eq!(miner.staking_slots().len(), 1);
		}

		#[test]
		fn unstake_with_cml_works() {
			let account_id = 10;
			let mut staker =
				CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(seed_from_lifespan(11, 100));
			staker.owner = Some(account_id);

			let mut miner = default_miner(22, 100);
			let index = miner.stake::<CML<u32, u32, u128, ConstU32<10>>>(
				&account_id,
				&0,
				None,
				Some(&mut staker),
			);
			assert_eq!(index, Some(1));
			assert!(staker.is_staking());

			assert!(miner.check_unstake(&None, &Some(&staker)).is_ok());
			miner.unstake(None, Some(&mut staker));

			assert_eq!(miner.staking_slots().len(), 1);
			assert!(!staker.is_staking());
		}

		#[test]
		fn unskaking_amount_and_cml_should_have_one_and_only_one() {
			let (mut staker, mut miner) = default_staking_cml_pair(10, 1, 2);
			assert_eq!(miner.staking_slots().len(), 2);

			// amount and cml both none
			assert!(miner
				.check_unstake::<CML<u32, u32, u128, ConstU32<10>>>(&None, &None)
				.is_err());
			miner.unstake::<CML<u32, u32, u128, ConstU32<10>>>(None, None);
			assert_eq!(miner.staking_slots().len(), 2);

			// amount and cml both have value
			assert!(miner.check_unstake(&Some(1), &Some(&staker)).is_err());
			miner.unstake(Some(1), Some(&mut staker));
			assert_eq!(miner.staking_slots().len(), 2);
		}

		#[test]
		fn cannot_unstake_if_not_mining() {
			let miner =
				CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(seed_from_lifespan(2, 100));

			assert!(!miner.is_mining());
			assert!(miner
				.check_unstake::<CML<u32, u32, u128, ConstU32<10>>>(&Some(1), &None)
				.is_err());
		}

		#[test]
		fn cannot_unstake_balance_item_if_staking_index_larger_than_slots_length() {
			let mut miner = default_miner(1, 100);
			miner.stake::<CML<u32, u32, u128, ConstU32<10>>>(&1, &0, Some(1000), None);
			assert_eq!(miner.staking_slots().len(), 2);

			assert!(miner
				.check_unstake::<CML<u32, u32, u128, ConstU32<10>>>(&Some(2), &None)
				.is_err());
			miner.unstake::<CML<u32, u32, u128, ConstU32<10>>>(Some(2), None);
			assert_eq!(miner.staking_slots().len(), 2);
		}

		#[test]
		fn cannot_unstake_cml_item_if_status_is_not_staking() {
			let (mut staker, mut miner) = default_staking_cml_pair(10, 1, 2);
			assert_eq!(miner.staking_slots().len(), 2);

			staker.status = CmlStatus::Tree; // force changing status to tree
			assert!(miner
				.check_unstake::<CML<u32, u32, u128, ConstU32<10>>>(&None, &Some(&staker))
				.is_err());
			miner.unstake::<CML<u32, u32, u128, ConstU32<10>>>(None, Some(&mut staker));
			assert_eq!(miner.staking_slots().len(), 2);
		}

		#[test]
		fn cannot_unstake_cml_item_if_cml_id_not_matched() {
			let (mut staker, mut miner) = default_staking_cml_pair(10, 1, 2);
			assert_eq!(miner.staking_slots().len(), 2);

			staker.status = CmlStatus::Staking(3, 1); // force changing miner_id to 3
			assert!(miner
				.check_unstake::<CML<u32, u32, u128, ConstU32<10>>>(&None, &Some(&staker))
				.is_err());
			miner.unstake::<CML<u32, u32, u128, ConstU32<10>>>(None, Some(&mut staker));
			assert_eq!(miner.staking_slots().len(), 2);
		}

		#[test]
		fn cannot_unstake_cml_item_if_staking_index_larger_than_slots_length() {
			let (mut staker, mut miner) = default_staking_cml_pair(10, 1, 2);
			assert_eq!(miner.staking_slots().len(), 2);

			staker.status = CmlStatus::Staking(2, 2); // force changing index to 2
			assert!(miner
				.check_unstake::<CML<u32, u32, u128, ConstU32<10>>>(&None, &Some(&staker))
				.is_err());
			miner.unstake::<CML<u32, u32, u128, ConstU32<10>>>(None, Some(&mut staker));
			assert_eq!(miner.staking_slots().len(), 2);
		}

		#[test]
		fn cannot_unstake_cml_item_if_owner_is_none() {
			let (mut staker, mut miner) = default_staking_cml_pair(10, 1, 2);
			assert_eq!(miner.staking_slots().len(), 2);

			staker.owner = None; // force changing staker owner to none
			assert!(miner
				.check_unstake::<CML<u32, u32, u128, ConstU32<10>>>(&None, &Some(&staker))
				.is_err());
			miner.unstake::<CML<u32, u32, u128, ConstU32<10>>>(None, Some(&mut staker));
			assert_eq!(miner.staking_slots().len(), 2);
		}

		#[test]
		fn cannot_unstake_cml_item_if_owner_not_matched() {
			let account1 = 11;
			let account2 = 22;
			let (staker1, mut miner1) = default_staking_cml_pair(account1, 1, 2);
			let (mut staker2, _) = default_staking_cml_pair(account2, 3, 4);
			assert_eq!(miner1.staking_slots().len(), 2);

			assert_eq!(staker1.owner, Some(account1));
			assert_eq!(staker2.owner, Some(account2));
			assert!(miner1.check_unstake(&None, &Some(&staker2)).is_err());
			miner1.unstake(None, Some(&mut staker2));
			assert_eq!(miner1.staking_slots().len(), 2);
		}
	}

	mod mining_properties_test {
		use crate::tests::seed_from_lifespan;
		use crate::types::cml::tests::default_miner;
		use crate::{
			MiningProperties, SeedProperties, StakingCategory, StakingItem, StakingProperties,
			TreeProperties, CML,
		};
		use frame_support::traits::ConstU32;

		#[test]
		fn start_mining_works() {
			let cml_id = 3;
			let mut miner = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(
				seed_from_lifespan(cml_id, 100),
			);
			miner.defrost(&0);
			miner.convert_to_tree(&0);

			assert!(miner.check_start_mining(&0).is_ok());
			miner.start_mining(
				[1u8; 32],
				StakingItem {
					owner: 1,
					category: StakingCategory::Cml,
					amount: Some(1),
					cml: None,
				},
				&0,
			);

			assert!(miner.is_mining());
			assert_eq!(miner.staking_slots().len(), 1);
		}

		#[test]
		fn start_mining_works_with_frozon_seed() {
			let cml_id = 3;
			let mut miner = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(
				seed_from_lifespan(cml_id, 100),
			);

			assert!(miner.is_frozen_seed());
			assert!(miner.check_start_mining(&0).is_ok());
			miner.start_mining(
				[1u8; 32],
				StakingItem {
					owner: 1,
					category: StakingCategory::Cml,
					amount: Some(1),
					cml: None,
				},
				&0,
			);

			assert!(miner.is_mining());
			assert_eq!(miner.staking_slots().len(), 1);
		}

		#[test]
		fn start_mining_works_with_fresh_seed() {
			let cml_id = 3;
			let mut miner = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(
				seed_from_lifespan(cml_id, 100),
			);
			miner.defrost(&0);

			assert!(miner.is_fresh_seed());
			assert!(miner.check_start_mining(&0).is_ok());
			miner.start_mining(
				[1u8; 32],
				StakingItem {
					owner: 1,
					category: StakingCategory::Cml,
					amount: Some(1),
					cml: None,
				},
				&0,
			);

			assert!(miner.is_mining());
			assert_eq!(miner.staking_slots().len(), 1);
		}

		#[test]
		fn start_mining_should_fail_if_cml_is_invalid() {
			let lifespan = 100;
			let mut miner = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(
				seed_from_lifespan(11, lifespan),
			);
			miner.defrost(&0);

			let fresh_duration = miner.get_fresh_duration();
			assert!(miner.check_seed_validity(&fresh_duration).is_err());
			assert!(miner.check_start_mining(&fresh_duration).is_err());
			miner.start_mining([1u8; 32], StakingItem::default(), &fresh_duration);
			assert!(!miner.is_mining());

			miner.convert_to_tree(&0);
			assert!(miner.check_tree_validity(&lifespan).is_err());
			assert!(miner.check_start_mining(&lifespan).is_err());
			miner.start_mining([1u8; 32], StakingItem::default(), &lifespan);
			assert!(!miner.is_mining());
		}

		#[test]
		fn start_mining_should_fail_if_cml_is_mining_already() {
			let mut miner = default_miner(11, 100);

			assert!(miner.check_start_mining(&0).is_err());
			miner.start_mining([1u8; 32], StakingItem::default(), &0);
			assert_ne!(miner.staking_slots()[0].owner, 0); // owner of staking item not reset to 0
		}

		#[test]
		fn start_mining_should_fail_if_staking_slot_not_empty() {
			let mut miner =
				CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(seed_from_lifespan(11, 100));
			miner.staking_slot.push(StakingItem::default());

			assert!(miner.check_start_mining(&0).is_err());
			miner.start_mining([1u8; 32], StakingItem::default(), &0);
			assert!(!miner.is_mining());
		}

		#[test]
		fn start_mining_should_fail_if_cml_is_staking() {
			let account_id = 10;
			let cml_id = 11;
			let mut staker = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(
				seed_from_lifespan(cml_id, 100),
			);

			let miner_id = 22;
			let mut miner = default_miner(miner_id, 100);
			assert!(miner
				.check_can_be_stake(&0, &None, &Some(staker.id()))
				.is_ok());

			let index = miner.stake::<CML<u32, u32, u128, ConstU32<10>>>(
				&account_id,
				&0,
				None,
				Some(&mut staker),
			);
			assert_eq!(index, Some(1));
			assert!(staker.is_staking());

			assert!(staker.check_start_mining(&0).is_err());
			staker.start_mining([3u8; 32], StakingItem::default(), &0);
			assert!(!staker.is_mining());
		}

		#[test]
		fn stop_mining_works() {
			let mut miner = default_miner(11, 100);
			assert!(miner.is_mining());
			assert_eq!(miner.staking_slots().len(), 1);

			miner.stop_mining();
			assert!(!miner.is_mining());
			assert!(miner.staking_slots().is_empty());
		}

		#[test]
		fn stop_mining_ignore_if_is_not_ming() {
			let mut miner =
				CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(seed_from_lifespan(11, 100));
			miner.staking_slot.push(StakingItem::default());

			assert!(!miner.is_mining());
			miner.stop_mining();
			assert!(!miner.staking_slots().is_empty());
		}
	}

	fn default_miner(id: CmlId, lifespan: u32) -> CML<u32, u32, u128, ConstU32<10>> {
		let mut miner = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(seed_from_lifespan(
			id, lifespan,
		));
		miner.defrost(&0);
		miner.convert_to_tree(&0);
		miner.start_mining(
			[1u8; 32],
			StakingItem {
				owner: 1,
				category: StakingCategory::Cml,
				amount: Some(1),
				cml: None,
			},
			&0,
		);

		miner
	}

	fn default_staking_cml_pair(
		account_id: u32,
		staker_id: CmlId,
		miner_id: CmlId,
	) -> (
		CML<u32, u32, u128, ConstU32<10>>,
		CML<u32, u32, u128, ConstU32<10>>,
	) {
		let mut staker = CML::<u32, u32, u128, ConstU32<10>>::from_genesis_seed(
			seed_from_lifespan(staker_id, 100),
		);
		staker.owner = Some(account_id);

		let mut miner = default_miner(miner_id, 100);
		miner.stake::<CML<u32, u32, u128, ConstU32<10>>>(&account_id, &0, None, Some(&mut staker));

		(staker, miner)
	}
}
