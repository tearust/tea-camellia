use crate as pallet_bonding_curve;
use auction_interface::AuctionOperation;
use bonding_curve_impl::linear::UnsignedLinearCurve;
use bonding_curve_impl::square_root::UnsignedSquareRoot;
use frame_support::parameter_types;
use frame_system as system;
use genesis_exchange_interface::MiningOperation;
use node_primitives::Balance;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup, Zero},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub struct MiningOperationMock {}

impl Default for MiningOperationMock {
	fn default() -> Self {
		MiningOperationMock {}
	}
}

impl MiningOperation for MiningOperationMock {
	type AccountId = u64;

	fn check_buying_mining_machine(
		_who: &Self::AccountId,
		_cml_id: u64,
	) -> sp_runtime::DispatchResult {
		Ok(())
	}

	fn buy_mining_machine(_who: &Self::AccountId, _cml_id: u64) {}

	fn check_redeem_coupons(
		_who: &Self::AccountId,
		_a_coupon: u32,
		_b_coupon: u32,
		_c_coupon: u32,
	) -> sp_runtime::DispatchResult {
		Ok(())
	}

	fn redeem_coupons(_who: &Self::AccountId, _a_coupon: u32, _b_coupon: u32, _c_coupon: u32) {}
}

pub struct AuctionOperationMock {}

impl Default for AuctionOperationMock {
	fn default() -> Self {
		AuctionOperationMock {}
	}
}
impl AuctionOperation for AuctionOperationMock {
	type AccountId = u64;
	type Balance = Balance;
	type BlockNumber = u64;

	fn is_cml_in_auction(_cml_id: u64) -> bool {
		false
	}

	fn create_new_bid(_sender: &Self::AccountId, _auction_id: &u64, _price: Self::Balance) {}

	fn update_current_winner(_auction_id: &u64, _bid_user: &Self::AccountId) {}

	fn get_window_block() -> (Self::BlockNumber, Self::BlockNumber) {
		(Zero::zero(), Zero::zero())
	}
}

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		BondingCurve: pallet_bonding_curve::{Pallet, Call, Storage, Event<T>},
		Utils: pallet_utils::{Pallet, Call, Storage, Event<T>},
		Cml: pallet_cml::{Pallet, Call, Storage, Event<T>},
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
pub const HOST_ARRANGE_DURATION: u64 = 1000;
pub const HOST_COST_COLLECTION_DURATION: u64 = 100;
pub const HOST_COST_COEFFICIENT: Balance = 10000;
pub const CONSUME_NOTE_MAX_LENGTH: u32 = 140;
pub const CID_MAX_LENGTH: u32 = 100;

parameter_types! {
	pub const TAppNameMaxLength: u32 = TAPP_NAME_MAX_LENGTH;
	pub const TAppDetailMaxLength: u32 = TAPP_DETAIL_MAX_LENGTH;
	pub const TAppLinkMaxLength: u32 = TAPP_LINK_MAX_LENGTH;
	pub const TAppTickerMaxLength: u32 = TAPP_TICKER_MAX_LENGTH;
	pub const TAppTickerMinLength: u32 = TAPP_TICKER_MIN_LENGTH;
	pub const PoolBalanceReversePrecision: Balance = POOL_BALANCE_REVERSE_PRECISION;
	pub const HostArrangeDuration: u64 = HOST_ARRANGE_DURATION;
	pub const HostCostCollectionDuration: u64 = HOST_COST_COLLECTION_DURATION;
	pub const HostCostCoefficient: Balance = HOST_COST_COEFFICIENT;
	pub const ConsumeNoteMaxLength: u32 = CONSUME_NOTE_MAX_LENGTH;
	pub const CidMaxLength: u32 = CID_MAX_LENGTH;
}

impl pallet_bonding_curve::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type CurrencyOperations = Utils;
	type CmlOperation = Cml;
	type TAppNameMaxLength = TAppNameMaxLength;
	type TAppTickerMinLength = TAppTickerMinLength;
	type TAppTickerMaxLength = TAppTickerMaxLength;
	type TAppDetailMaxLength = TAppDetailMaxLength;
	type TAppLinkMaxLength = TAppLinkMaxLength;
	type LinearCurve = UnsignedLinearCurve<Balance, 100>;
	type UnsignedSquareRoot_10 = UnsignedSquareRoot<Balance, 10>;
	type UnsignedSquareRoot_7 = UnsignedSquareRoot<Balance, 7>;
	type PoolBalanceReversePrecision = PoolBalanceReversePrecision;
	type HostArrangeDuration = HostArrangeDuration;
	type HostCostCollectionDuration = HostCostCollectionDuration;
	type HostCostCoefficient = HostCostCoefficient;
	type ConsumeNoteMaxLength = ConsumeNoteMaxLength;
	type CidMaxLength = CidMaxLength;
}

pub const STAKING_PRICE: Balance = 1000;

parameter_types! {
	pub const MinRaPassedThreshold: u32 = 3;
	pub const StakingPrice: Balance = STAKING_PRICE;
	pub const SeedsTimeoutHeight: u32 = 1 * 30 * 24 * 60 * 10;
	pub const StakingPeriodLength: u32 = 100;
	pub const SeedFreshDuration: u32 = 7 * 30 * 24 * 60 * 10;
	pub const StakingSlotsMaxLength: u32 = 1024;
	pub const StopMiningPunishment: Balance = 100;
}

impl pallet_cml::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type StakingPrice = StakingPrice;
	type CouponTimoutHeight = SeedsTimeoutHeight;
	type StakingPeriodLength = StakingPeriodLength;
	type SeedFreshDuration = SeedFreshDuration;
	type CommonUtils = Utils;
	type CurrencyOperations = Utils;
	type StakingEconomics = Cml;
	type AuctionOperation = AuctionOperationMock;
	type StakingSlotsMaxLength = StakingSlotsMaxLength;
	type StopMiningPunishment = StopMiningPunishment;
	type MiningOperation = MiningOperationMock;
	type WeightInfo = ();
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
