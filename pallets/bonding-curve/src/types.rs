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

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub enum TAppType {
	YouTube,
	Reddit,
	Twitter,
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub enum BillingMode<Balance> {
	FixedHostingFee(Balance),
	FixedHostingToken(Balance),
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug)]
pub enum TAppStatus {
	Active,
	Pending,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct TAppItem<AccountId, Balance> {
	pub id: TAppId,
	pub name: Vec<u8>,
	pub ticker: Vec<u8>,
	pub owner: AccountId,
	pub buy_curve: CurveType,
	pub sell_curve: CurveType,
	pub detail: Vec<u8>,
	pub link: Vec<u8>,
	pub max_allowed_hosts: u32,
	pub current_cost: Balance,
	pub status: TAppStatus,
	pub tapp_type: TAppType,
	pub billing_mode: BillingMode<Balance>,
}

impl<AccountId, Balance> TAppItem<AccountId, Balance> {
	pub fn host_performance(&self) -> Performance {
		match self.tapp_type {
			TAppType::YouTube => 3000,
			TAppType::Reddit => 2000,
			TAppType::Twitter => 1000,
		}
	}
}

impl<AccountId, Balance> Default for TAppItem<AccountId, Balance>
where
	AccountId: Default,
	Balance: AtLeast32BitUnsigned + Default,
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
			max_allowed_hosts: Default::default(),
			current_cost: Default::default(),
			status: TAppStatus::Pending,
			tapp_type: TAppType::Twitter,
			billing_mode: BillingMode::FixedHostingToken(Default::default()),
		}
	}
}
