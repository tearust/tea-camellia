use frame_support::parameter_types;
use frame_system as system;
use node_primitives::Balance;
use pallet_cml::{generator::init_genesis, GenesisSeeds};
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
pub const SEED_ROTTEN_DURATION: u32 = 7 * 24 * 60 * 10;

parameter_types! {
	pub const MinRaPassedThreshold: u32 = 3;
	pub const StakingPrice: Balance = 1000;
	pub const SeedsTimeoutHeight: u32 = SEEDS_TIMEOUT_HEIGHT;
	pub const StakingPeriodLength: u32 = STAKING_PERIOD_LENGTH;
	pub const SeedFreshDuration: u32 = SEED_ROTTEN_DURATION;
	pub const StakingSlotsMaxLength: u32 = 1024;
}

impl pallet_cml::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type StakingPrice = StakingPrice;
	type VoucherTimoutHeight = SeedsTimeoutHeight;
	type StakingPeriodLength = StakingPeriodLength;
	type CommonUtils = Utils;
	type CurrencyOperations = Utils;
	type SeedFreshDuration = SeedFreshDuration;
	// todo replace value with StakingEconomics later
	type StakingEconomics = Cml;
	type StakingSlotsMaxLength = StakingSlotsMaxLength;
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

pub struct ExtBuilder {
	account_number: u32,
	initial_balance: Balance,
	pub seeds: GenesisSeeds,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			account_number: 50,
			initial_balance: 10_000,
			seeds: init_genesis(),
		}
	}
}

impl ExtBuilder {
	pub fn set_account_number(&mut self, account_number: u32) {
		self.account_number = account_number;
	}

	#[allow(dead_code)]
	pub fn set_initial_balances(&mut self, initial_balance: Balance) {
		self.initial_balance = initial_balance;
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();

		let initial_balance = self.initial_balance;
		let balances = (1..=self.account_number)
			.map(|account| (account as u64, initial_balance))
			.collect();
		pallet_balances::GenesisConfig::<Test> { balances }
			.assimilate_storage(&mut t)
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
