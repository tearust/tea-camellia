use codec::{Decode, Encode};
use pallet_cml::CmlId;
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

// all types of ID should encode as Vec<u8>
pub type AssetId = Vec<u8>;

#[derive(Clone, Copy, Encode, Decode, PartialEq, RuntimeDebug)]
pub enum AssetType {
	CML,
}

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug)]
pub struct AssetUniqueId {
	pub asset_type: AssetType,
	pub inner_id: AssetId,
}

#[derive(Clone, Encode, Decode, RuntimeDebug)]
pub struct Lien<AccountId, BlockNumber>
where
	AccountId: Default + PartialEq + Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
{
	pub start_at: BlockNumber,
	pub owner: AccountId,
}

impl<AccountId, BlockNumber> Default for Lien<AccountId, BlockNumber>
where
	AccountId: Default + PartialEq + Clone,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone,
{
	fn default() -> Self {
		Lien {
			start_at: BlockNumber::default(),
			owner: Default::default(),
		}
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub enum BankError {
	/// Asset id convert to cml id with invalid length.
	ConvertToCmlIdLengthMismatch,
}

pub fn from_cml_id(cml_id: CmlId) -> AssetId {
	cml_id.to_le_bytes().to_vec()
}

pub fn to_cml_id(id: &AssetId) -> Result<CmlId, BankError> {
	// asset id length should be 8 bytes
	if id.len() != 8 {
		return Err(BankError::ConvertToCmlIdLengthMismatch);
	}

	let mut buf: [u8; 8] = Default::default();
	buf.copy_from_slice(&id.as_slice()[0..8]);
	Ok(u64::from_le_bytes(buf))
}

#[cfg(test)]
mod tests {
	use crate::types::{from_cml_id, to_cml_id};
	use rand::{thread_rng, Rng};

	#[test]
	fn convert_between_asset_id_and_cml_id_works() {
		let mut rng = thread_rng();
		let cml_id: u64 = rng.gen();

		let asset_id = from_cml_id(cml_id);
		assert!(!asset_id.is_empty());
		let cml_id2 = to_cml_id(&asset_id).unwrap();
		assert_eq!(cml_id, cml_id2);
	}
}
