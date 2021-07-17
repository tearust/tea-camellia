use crate as pallet_cml;
use crate::generator::init_genesis;
use crate::{CouponConfig, GenesisCoupons, GenesisSeeds};
use frame_support::pallet_prelude::*;
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

// same as mock implementation of StakingEconomics in "staking.rs" file
pub const DOLLARS: node_primitives::Balance = 100000;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Cml: pallet_cml::{Pallet, Call, Storage, Event<T>},
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

pub const SEEDS_TIMEOUT_HEIGHT: u32 = 1 * 30 * 24 * 60 * 10;
pub const STAKING_PERIOD_LENGTH: u32 = 100;
pub const STAKING_PRICE: Balance = 1000;
pub const SEED_FRESH_DURATION: u32 = 7 * 30 * 24 * 60 * 10;
pub const STAKING_SLOTS_MAX_LENGTH: u32 = 100;

parameter_types! {
	pub const MinRaPassedThreshold: u32 = 3;
	pub const StakingPrice: Balance = STAKING_PRICE;
	pub const StakingPeriodLength: u32 = STAKING_PERIOD_LENGTH;
	pub const SeedsTimeoutHeight: u32 = SEEDS_TIMEOUT_HEIGHT;
	pub const SeedFreshDuration: u32 = SEED_FRESH_DURATION;
	pub const StakingSlotsMaxLength: u32 = STAKING_SLOTS_MAX_LENGTH;
}

impl pallet_cml::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type StakingPrice = StakingPrice;
	type StakingPeriodLength = StakingPeriodLength;
	type CouponTimoutHeight = SeedsTimeoutHeight;
	type SeedFreshDuration = SeedFreshDuration;
	type CommonUtils = Utils;
	type CurrencyOperations = Utils;
	type StakingEconomics = Cml;
	type StakingSlotsMaxLength = StakingSlotsMaxLength;
	type WeightInfo = ();
}

impl pallet_utils::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type Reward = ();
	type Slash = ();
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

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}

pub struct ExtBuilder {
	seeds: GenesisSeeds,
	coupons: GenesisCoupons<u64>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			seeds: GenesisSeeds::default(),
			coupons: GenesisCoupons::default(),
		}
	}
}

impl ExtBuilder {
	pub fn init_seeds(mut self) -> Self {
		self.seeds = init_genesis([1; 32]);
		self
	}

	pub fn coupons(mut self, coupons: Vec<CouponConfig<u64>>) -> Self {
		self.coupons.coupons = coupons;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();

		pallet_cml::GenesisConfig::<Test> {
			genesis_seeds: self.seeds,
			genesis_coupons: self.coupons,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
