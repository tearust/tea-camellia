use super::*;

pub type TAppId = u64;

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, MaxEncodedLen)]
pub enum Releases {
	V0,
	V1,
}

impl Default for Releases {
	fn default() -> Self {
		Releases::V0
	}
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum CurveType {
	UnsignedLinear,
	#[allow(non_camel_case_types)]
	UnsignedSquareRoot_10,
	#[allow(non_camel_case_types)]
	UnsignedSquareRoot_7,
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum TAppType {
	YouTube,
	Reddit,
	Twitter,
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum BillingMode<Balance> {
	FixedHostingFee(Balance),
	FixedHostingToken(Balance),
}

#[derive(Encode, Decode, Clone, Copy, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub enum TAppStatus<BlockNumber> {
	Active(BlockNumber),
	Pending,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct ApprovedLinkInfo<AccountId> {
	pub tapp_id: Option<TAppId>,
	pub description: Vec<u8>,
	pub creator: Option<AccountId>,
}

impl<AccountId> Default for ApprovedLinkInfo<AccountId> {
	fn default() -> Self {
		Self {
			tapp_id: None,
			description: Default::default(),
			creator: None,
		}
	}
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo)]
pub struct TAppItem<AccountId, Balance, BlockNumber> {
	pub id: TAppId,
	pub name: Vec<u8>,
	pub ticker: Vec<u8>,
	pub owner: AccountId,
	pub detail: Vec<u8>,
	pub link: Vec<u8>,
	pub max_allowed_hosts: u32,
	pub current_cost: Balance,
	pub status: TAppStatus<BlockNumber>,
	pub tapp_type: TAppType,
	pub billing_mode: BillingMode<Balance>,
	pub buy_curve_theta: u32,
	pub sell_curve_theta: u32,
}

impl<AccountId, Balance, BlockNumber> TAppItem<AccountId, Balance, BlockNumber> {
	pub fn host_performance(&self) -> Performance {
		match self.tapp_type {
			TAppType::YouTube => 3000,
			TAppType::Reddit => 2000,
			TAppType::Twitter => 1000,
		}
	}
}

impl<AccountId, Balance, BlockNumber> Default for TAppItem<AccountId, Balance, BlockNumber>
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
			detail: vec![],
			link: vec![],
			max_allowed_hosts: Default::default(),
			current_cost: Default::default(),
			status: TAppStatus::Pending,
			tapp_type: TAppType::Twitter,
			billing_mode: BillingMode::FixedHostingToken(Default::default()),
			buy_curve_theta: 10,
			sell_curve_theta: 7,
		}
	}
}
