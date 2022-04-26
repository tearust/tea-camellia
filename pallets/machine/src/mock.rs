use crate as pallet_machine;
use codec::{Decode, Encode};
use frame_support::parameter_types;
use frame_support::traits::{Everything, Get};
use frame_system as system;
use node_primitives::Balance;
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

pub const RUNTIME_ACTIVITY_THRESHOLD: u32 = 6 * 60 * 10;
pub const UPDATE_VALIDATORS_DURATION: u32 = 10 * 60 * 10;
pub const MAX_GROUP_MEMBER_COUNT: u32 = 10;
pub const MIN_GROUP_MEMBER_COUNT: u32 = 5;
pub const MAX_ALLOWED_RA_COMMIT_DURATION: u32 = 10;
pub const PHISHING_ALLOWED_DURATION: u32 = 100;
pub const TIPS_ALLOWED_DURATION: u32 = 100;
pub const OFFLINE_VALID_DURATION: u32 = 150;
pub const OFFLINE_EFFECT_THRESHOLD: u32 = 2;
pub const REPORT_RAWARD_DURATION: u32 = 200;
pub const MINING_NODES_ACTIVITY_CHECK_DURATION: u32 = 500;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

pub const SEED_FRESH_DURATION: u64 = 7 * 24 * 60 * 10;

#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
pub struct SeedFreshDuration {
	duration: u64,
}

impl Get<u64> for SeedFreshDuration {
	fn get() -> u64 {
		SEED_FRESH_DURATION
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
		Machine: pallet_machine::{Pallet, Call, Storage, Event<T>},
		Utils: pallet_utils::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
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
}

parameter_types! {
	pub const RuntimeActivityThreshold: u32 = RUNTIME_ACTIVITY_THRESHOLD;
	pub const PerRaTaskPoint: u32 = 100;
	pub const UpdateValidatorsDuration: u32 = UPDATE_VALIDATORS_DURATION;
	pub const MaxGroupMemberCount: u32 = MAX_GROUP_MEMBER_COUNT;
	pub const MinGroupMemberCount: u32 = MIN_GROUP_MEMBER_COUNT;
	pub const MaxAllowedRaCommitDuration: u32 = MAX_ALLOWED_RA_COMMIT_DURATION;
	pub const PhishingAllowedDuration: u32 = PHISHING_ALLOWED_DURATION;
	pub const TipsAllowedDuration: u32 = TIPS_ALLOWED_DURATION;
	pub const OfflineValidDuration: u32 = OFFLINE_VALID_DURATION;
	pub const OfflineEffectThreshold: u32 = OFFLINE_EFFECT_THRESHOLD;
	pub const ReportRawardDuration: u32 = REPORT_RAWARD_DURATION;
	pub const MiningNodesActivityCheckDuration: u32 = MINING_NODES_ACTIVITY_CHECK_DURATION;
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

impl pallet_machine::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type CommonUtils = Utils;
	type CurrencyOperations = Utils;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}
