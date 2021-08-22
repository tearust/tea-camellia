use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

pub type TAppId = u64;

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub enum CurveType {
	UnsignedLinear,
	#[allow(non_camel_case_types)]
	UnsignedSquareRoot_1000,
	#[allow(non_camel_case_types)]
	UnsignedSquareRoot_700,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct TAppItem<AccountId> {
	pub id: TAppId,
	pub name: Vec<u8>,
	pub ticker: Vec<u8>,
	pub owner: AccountId,
	pub buy_curve: CurveType,
	pub sell_curve: CurveType,
	pub detail: Vec<u8>,
	pub link: Vec<u8>,
}

impl<AccountId> Default for TAppItem<AccountId>
where
	AccountId: Default,
{
	fn default() -> Self {
		TAppItem {
			id: 0,
			name: vec![],
			ticker: vec![],
			owner: Default::default(),
			buy_curve: CurveType::UnsignedLinear,
			sell_curve: CurveType::UnsignedLinear,
			detail: vec![],
			link: vec![],
		}
	}
}
