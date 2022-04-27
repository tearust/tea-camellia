use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::prelude::*;

pub type CmlId = u64;
pub type IssuerId = u64;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, Default)]
pub struct Issuer<Account>
where
	Account: Default,
{
	pub id: IssuerId,
	pub url: Vec<u8>,
	pub owner: Account,
}

/// Tea public key generated from the TEA secure module (Tpm, Aws Nitro etc.) used to identify
/// the TEA node.
pub type TeaPubKey = [u8; 32];
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, Default)]
pub struct Machine<Account>
where
	Account: Default,
{
	pub tea_id: TeaPubKey,
	pub issuer_id: IssuerId,
	pub owner: Account,
}
