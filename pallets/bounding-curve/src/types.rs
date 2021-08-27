use super::*;

pub type TAppId = u64;

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub enum CurveType {
	UnsignedLinear,
	#[allow(non_camel_case_types)]
	UnsignedSquareRoot_10,
	#[allow(non_camel_case_types)]
	UnsignedSquareRoot_7,
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
	pub host_performance: Option<Performance>,
	pub max_allowed_hosts: Option<u32>,
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
			host_performance: Default::default(),
			max_allowed_hosts: Default::default(),
		}
	}
}
