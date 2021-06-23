use crate::param::{GENESIS_SEED_A_COUNT, GENESIS_SEED_B_COUNT, GENESIS_SEED_C_COUNT};
use crate::{MachineId, Seed, StakingItem};
use frame_support::sp_runtime::traits::AtLeast32BitUnsigned;

pub type CmlId = u64;

#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CmlType {
	A,
	B,
	C,
}

#[derive(Clone, Copy, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub enum CmlStatus {
	/// Special status about genesis seeds, `FrozenSeed` can't rot.
	FrozenSeed,
	/// DAO generated seed initial state, or defrost from genesis seeds.
	/// Seed will rot if not plant during a certain period (about 1 week).
	FreshSeed,
	/// Seed grow up and become tree, the tree have `lifespan` and shall dead if it lived over
	/// than the lifespan (start calculate at `plant_at` block height).
	/// A tree can be planted into a machine, then `machine_id` should not be `None`, and staking
	/// slot should at least have one item.
	Tree,
	/// Tree can staking instead of running on a machine (aka mining), a staking tree will consume
	/// life same as tree, if becomes dead should not staking anymore.
	Staking,
}

impl CmlStatus {
	pub fn valid_conversion(&self, to: CmlStatus) -> bool {
		match *self {
			/// Allowed status transfer:
			/// `FrozenSeed` => `FreshSeed`
			CmlStatus::FrozenSeed => to == CmlStatus::FreshSeed,

			/// Allowed status transfer:
			/// `FreshSeed` => `Tree`
			CmlStatus::FreshSeed => to == CmlStatus::Tree,

			///	Allowed status transfer:
			/// `Tree` => `Staking`
			CmlStatus::Tree => to == CmlStatus::Staking,

			/// Allowed status transfer:
			/// `Staking` => `Tree`
			CmlStatus::Staking => to == CmlStatus::Tree,
		}
	}
}

pub enum CmlError {
	OwnerIsNone,
	SproutAtIsNone,
	PlantAtIsNone,
	StakingSlotIsEmpty,
	MachineIdIsNone,
	DefrostScheduleIsNone,
	DefrostTimeIsNone,
	DefrostFailed,
	CmlStatusConvertFailed,
}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct CML<AccountId, BlockNumber, Balance>
where
	AccountId: Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
{
	pub intrinsic: Seed,
	status: CmlStatus,
	owner: Option<AccountId>,
	/// The time a fresh seed created (or converted from `FrozenSeed`)
	pub sprout_at: Option<BlockNumber>,
	/// The time a tree created
	pub planted_at: Option<BlockNumber>,
	pub staking_slot: Vec<StakingItem<AccountId, Balance>>,
	pub machine_id: Option<MachineId>,
}

impl<AccountId, BlockNumber, Balance> CML<AccountId, BlockNumber, Balance>
where
	AccountId: Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
{
	pub fn is_from_genesis(seed: &Seed) -> bool {
		seed.id < GENESIS_SEED_A_COUNT + GENESIS_SEED_B_COUNT + GENESIS_SEED_C_COUNT
	}

	pub(crate) fn new(intrinsic: Seed) -> Self {
		let status = if Self::is_from_genesis(&intrinsic) {
			CmlStatus::FrozenSeed
		} else {
			CmlStatus::FreshSeed
		};

		CML {
			intrinsic,
			status,
			owner: None,
			sprout_at: None,
			planted_at: None,
			staking_slot: vec![],
			machine_id: None,
		}
	}

	pub fn id(&self) -> CmlId {
		self.intrinsic.id
	}

	pub fn is_seed(&self) -> bool {
		match self.status {
			CmlStatus::FrozenSeed | CmlStatus::FreshSeed => true,
			_ => false,
		}
	}

	pub fn should_dead(&self, height: BlockNumber) -> bool {
		!self.is_seed() && height > self.planted_at.clone() + self.intrinsic.lifespan.into()
	}

	pub fn can_be_defrost(&self, height: BlockNumber) -> Result<bool, CmlError> {
		Ok(self.status == CmlStatus::FrozenSeed
			&& height
				> self
					.intrinsic
					.defrost_time
					.ok_or(CmlError::DefrostTimeIsNone)?
					.into())
	}

	pub fn defrost(&mut self, height: BlockNumber) -> Result<(), CmlError> {
		if self.can_be_defrost(height) {
			return Err(CmlError::DefrostFailed);
		}

		self.try_to_convert(CmlStatus::FreshSeed)
	}

	/// rotten means a `FreshSeed` not planted after a certain period.
	pub fn has_rotten(
		&self,
		height: BlockNumber,
		rotten_duration: BlockNumber,
	) -> Result<bool, CmlError> {
		Ok(self.status == CmlStatus::FreshSeed
			&& height > self.sprout_at.ok_or(CmlError::SproutAtIsNone)? + rotten_duration)
	}

	pub fn owner(&self) -> Option<&AccountId> {
		self.owner.as_ref()
	}

	pub fn seed_valid(
		&self,
		height: BlockNumber,
		rotten_duration: BlockNumber,
	) -> Result<bool, CmlError> {
		Ok(self.can_be_defrost(height.clone())? || self.has_rotten(height, rotten_duration)?)
	}

	pub fn tree_valid(&self, height: BlockNumber) -> bool {
		!self.is_seed() && !self.should_dead(height)
	}

	pub fn convert_to_tree(&mut self, height: BlockNumber, rotten_duration: BlockNumber) {
		if !self.seed_valid(height, rotten_duration) {
			return;
		}
		self.try_to_convert(CmlStatus::Tree);
	}

	pub fn try_to_convert(&mut self, to_status: CmlStatus) -> Result<(), CmlError> {
		if !self.status.valid_conversion(to_status) {
			return Err(CmlError::CmlStatusConvertFailed);
		}

		self.status = to_status;
		Ok(())
	}
}
