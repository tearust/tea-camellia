use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

// all types of ID should encode as Vec<u8>
pub type AssetId = Vec<u8>;

#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug)]
pub enum AssetType {
	CML,
}
