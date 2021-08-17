use codec::{Decode, Encode};
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

pub type TAppId = u64;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum BuyCurveType {
	Linear,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum SellCurveType {
	Linear,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct TAppItem {
	pub id: TAppId,
	pub name: Vec<u8>,
	pub buy_curve: BuyCurveType,
	pub sell_curve: SellCurveType,
}

impl Default for TAppItem {
	fn default() -> Self {
		TAppItem {
			id: 0,
			name: vec![],
			buy_curve: BuyCurveType::Linear,
			sell_curve: SellCurveType::Linear,
		}
	}
}
