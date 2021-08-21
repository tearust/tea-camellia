use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

pub type TAppId = u64;

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub enum CurveType {
	UnsignedLinear,
	UnsignedSquareRoot,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct TAppItem<AccountId> {
	pub id: TAppId,
	pub name: Vec<u8>,
	pub owner: AccountId,
	pub buy_curve: CurveType,
	pub sell_curve: CurveType,
}

impl<AccountId> Default for TAppItem<AccountId>
where
	AccountId: Default,
{
	fn default() -> Self {
		TAppItem {
			id: 0,
			name: vec![],
			owner: Default::default(),
			buy_curve: CurveType::UnsignedLinear,
			sell_curve: CurveType::UnsignedLinear,
		}
	}
}
