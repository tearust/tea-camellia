use crate as pallet_bounding_curve;
use bounding_curve_impl::linear::UnsignedLinearCurve;
use bounding_curve_impl::square_root::UnsignedSquareRoot;
use frame_support::parameter_types;
use frame_system as system;
use node_primitives::Balance;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		BoundingCurve: pallet_bounding_curve::{Pallet, Call, Storage, Event<T>},
		Utils: pallet_utils::{Pallet, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

pub const TAPP_NAME_MAX_LENGTH: u32 = 20;
pub const TAPP_TICKER_MAX_LENGTH: u32 = 6;
pub const TAPP_TICKER_MIN_LENGTH: u32 = 3;
pub const TAPP_DETAIL_MAX_LENGTH: u32 = 120;
pub const TAPP_LINK_MAX_LENGTH: u32 = 100;
pub const POOL_BALANCE_REVERSE_PRECISION: Balance = 10;

parameter_types! {
	pub const TAppNameMaxLength: u32 = TAPP_NAME_MAX_LENGTH;
	pub const TAppDetailMaxLength: u32 = TAPP_DETAIL_MAX_LENGTH;
	pub const TAppLinkMaxLength: u32 = TAPP_LINK_MAX_LENGTH;
	pub const TAppTickerMaxLength: u32 = TAPP_TICKER_MAX_LENGTH;
	pub const TAppTickerMinLength: u32 = TAPP_TICKER_MIN_LENGTH;
	pub const PoolBalanceReversePrecision: Balance = POOL_BALANCE_REVERSE_PRECISION;
}

impl pallet_bounding_curve::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type CurrencyOperations = Utils;
	type TAppNameMaxLength = TAppNameMaxLength;
	type TAppTickerMinLength = TAppTickerMinLength;
	type TAppTickerMaxLength = TAppTickerMaxLength;
	type TAppDetailMaxLength = TAppDetailMaxLength;
	type TAppLinkMaxLength = TAppLinkMaxLength;
	type LinearCurve = UnsignedLinearCurve<Balance, 100>;
	type UnsignedSquareRoot_1000 = UnsignedSquareRoot<Balance, 1000>;
	type UnsignedSquareRoot_700 = UnsignedSquareRoot<Balance, 700>;
	type PoolBalanceReversePrecision = PoolBalanceReversePrecision;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
	type MaxLocks = MaxLocks;
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

impl pallet_utils::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type Reward = ();
	type Slash = ();
}

// Build genesis storage according to the mock runtime.
#[allow(dead_code)]
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}
