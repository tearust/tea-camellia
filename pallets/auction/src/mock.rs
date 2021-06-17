use crate as pallet_auction;
use frame_support::parameter_types;
use frame_system as system;
use node_primitives::{Balance, BlockNumber};
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
		Auction: pallet_auction::{Pallet, Call, Storage, Event<T>},
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

parameter_types! {
	pub const MinRaPassedThreshold: u32 = 3;
	pub const StakingPrice: Balance = 1000;
}

impl pallet_cml::Config for Test {
	type Event = Event;
	type CmlId = u64;
	type Currency = Balances;
	type StakingPrice = StakingPrice;
}

parameter_types! {
	pub const AuctionDealWindowBLock: BlockNumber = 50;
	pub const BidDeposit: Balance = 100;
	pub const MinPriceForBid: Balance = 1;
	pub const AuctionOwnerPenaltyForEachBid: Balance = 1;
}

impl pallet_auction::Config for Test {
	type Event = Event;
	type AuctionId = u64;
	type Currency = Balances;
	type AuctionDealWindowBLock = AuctionDealWindowBLock;
	type BidDeposit = BidDeposit;
	type MinPriceForBid = MinPriceForBid;
	type AuctionOwnerPenaltyForEachBid = AuctionOwnerPenaltyForEachBid;
	type CurrencyOperations = Utils;
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
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}
