use crate as pallet_tea;
use frame_support::parameter_types;
use frame_system as system;
use node_primitives::Balance;
use pallet_cml::{MachineId, ServiceTaskPoint, Task};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

pub const RUNTIME_ACTIVITY_THRESHOLD: u32 = 6 * 60 * 10;

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
		Tea: pallet_tea::{Pallet, Call, Storage, Event<T>},
		Utils: pallet_utils::{Pallet, Call, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
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

parameter_types! {
	pub const RuntimeActivityThreshold: u32 = RUNTIME_ACTIVITY_THRESHOLD;
	pub const MinRaPassedThreshold: u32 = 3;
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

impl pallet_tea::Config for Test {
	type Event = Event;
	type RuntimeActivityThreshold = RuntimeActivityThreshold;
	type MinRaPassedThreshold = MinRaPassedThreshold;
	type WeightInfo = ();
	type CommonUtils = Utils;
	type TaskService = MockTaskService;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}

pub struct MockTaskService {}

impl Default for MockTaskService {
	fn default() -> Self {
		MockTaskService {}
	}
}

impl Task for MockTaskService {
	/// Called after a miner has complete a RA task.
	fn complete_ra_task(_machine_id: MachineId, _task_point: ServiceTaskPoint) {
		// do nothing here
	}
}
