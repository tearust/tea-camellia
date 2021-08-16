use codec::{Decode, Encode};
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

pub type TAppId = u64;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum BoundingCurveType {
	Linear,
}
