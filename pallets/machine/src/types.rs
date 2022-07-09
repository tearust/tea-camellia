use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_std::prelude::*;

pub type CmlId = u64;
pub type IssuerId = u64;

pub const BUILTIN_ISSURE: IssuerId = 0;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct Issuer<Account>
where
	Account: MaxEncodedLen,
{
	pub id: IssuerId,
	pub owner: Account,
}

/// Tea public key generated from the TEA secure module (Tpm, Aws Nitro etc.) used to identify
/// the TEA node.
pub type TeaPubKey = [u8; 32];
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, TypeInfo, MaxEncodedLen)]
pub struct Machine<Account>
where
	Account: MaxEncodedLen,
{
	pub tea_id: TeaPubKey,
	pub issuer_id: IssuerId,
	pub owner: Account,
}
