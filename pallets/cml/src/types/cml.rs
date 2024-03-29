use crate::param::{Performance, GENESIS_SEED_A_COUNT, GENESIS_SEED_B_COUNT, GENESIS_SEED_C_COUNT};
use crate::Seed;
use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::sp_runtime::traits::AtLeast32BitUnsigned;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;
use sp_std::marker::PhantomData;
use sp_std::prelude::*;

pub type CmlId = u64;

#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CmlType {
	A,
	B,
	C,
}

#[derive(Clone, Encode, Decode, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct CML<AccountId, BlockNumber>
where
	AccountId: PartialEq + Clone + MaxEncodedLen,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone + MaxEncodedLen,
{
	intrinsic: Seed,
	owner: AccountId,
	phantom: PhantomData<BlockNumber>,
}

impl<AccountId, BlockNumber> CML<AccountId, BlockNumber>
where
	AccountId: PartialEq + Clone + MaxEncodedLen,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone + MaxEncodedLen,
{
	pub fn from_seed(intrinsic: Seed, account: AccountId) -> Self {
		CML {
			intrinsic,
			owner: account,
			phantom: PhantomData,
		}
	}

	/// CML identity.
	pub fn id(&self) -> CmlId {
		self.intrinsic.id
	}

	pub fn cml_type(&self) -> CmlType {
		self.intrinsic.cml_type
	}

	pub fn lifespan(&self) -> BlockNumber {
		self.intrinsic.lifespan.into()
	}

	pub fn is_from_genesis(&self) -> bool {
		self.id() < GENESIS_SEED_A_COUNT + GENESIS_SEED_B_COUNT + GENESIS_SEED_C_COUNT
	}

	pub fn owner(&self) -> &AccountId {
		&self.owner
	}

	pub fn get_peak_performance(&self) -> Performance {
		self.intrinsic.performance
	}

	pub fn set_owner(&mut self, account: AccountId) {
		self.owner = account;
	}
}
