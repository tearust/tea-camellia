use crate as pallet_cml;
use crate::generator::init_genesis;
use crate::GenesisSeeds;
use frame_support::pallet_prelude::*;
use frame_support::parameter_types;
use frame_support::traits::Everything;
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
pub const INVALID_MINING_CML_ID: u64 = 99;
pub const HOSTING_CML_ID: u64 = 98;
pub const INSUFFICIENT_CML_ID: u64 = 97;
pub const NPC_ACCOUNT: u64 = 100;

pub const SEED_FRESH_DURATION: u64 = 7 * 24 * 60 * 10;

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
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
	}
);

impl pallet_randomness_collective_flip::Config for Test {}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const SS58Prefix: u8 = 42;
}

impl system::Config for Test {
	type BaseCallFilter = Everything;
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
	type MaxConsumers = ConstU32<16>;
}

pub const SEEDS_TIMEOUT_HEIGHT: u32 = 1 * 30 * 24 * 60 * 10;
pub const STAKING_PERIOD_LENGTH: u32 = 100;
pub const STAKING_PRICE: Balance = 1000;
pub const MACHINE_ACCOUNT_TOP_UP_AMOUNT: Balance = 1;
pub const STAKING_SLOTS_MAX_LENGTH: u32 = 100;
pub const STOP_MINING_PUNISHMENT: Balance = 100;
pub const MAX_ALLOWED_SUSPEND_HEIGHT: u32 = 1000;
pub const CML_A_MINING_REWARD_RATE: Balance = 0;
pub const CML_B_MINING_REWARD_RATE: Balance = 5000;
pub const CML_C_MINING_REWARD_RATE: Balance = 0;

parameter_types! {
	pub const StakingPrice: Balance = STAKING_PRICE;
	pub const MachineAccountTopUpAmount: Balance = MACHINE_ACCOUNT_TOP_UP_AMOUNT;
	pub const StakingPeriodLength: u32 = STAKING_PERIOD_LENGTH;
	pub const SeedsTimeoutHeight: u32 = SEEDS_TIMEOUT_HEIGHT;
	pub const StakingSlotsMaxLength: u32 = STAKING_SLOTS_MAX_LENGTH;
	pub const StopMiningPunishment: Balance = STOP_MINING_PUNISHMENT;
	pub const MaxAllowedSuspendHeight: u32 = MAX_ALLOWED_SUSPEND_HEIGHT;
	pub const CmlAMiningRewardRate: Balance = CML_A_MINING_REWARD_RATE;
	pub const CmlBMiningRewardRate: Balance = CML_B_MINING_REWARD_RATE;
	pub const CmlCMiningRewardRate: Balance = CML_C_MINING_REWARD_RATE;
}

impl pallet_cml::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type CommonUtils = Utils;
	type CurrencyOperations = Utils;
}

impl pallet_utils::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type Reward = ();
	type Slash = ();
	type RandomnessSource = RandomnessCollectiveFlip;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
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
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			seeds: GenesisSeeds::default(),
		}
	}
}

impl ExtBuilder {
	pub fn init_seeds(mut self) -> Self {
		self.seeds = init_genesis([1; 32]);
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();

		pallet_cml::GenesisConfig::<Test> {
			npc_account: Default::default(),
			startup_account: Default::default(),
			genesis_seeds: self.seeds,
			startup_cmls: Default::default(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
