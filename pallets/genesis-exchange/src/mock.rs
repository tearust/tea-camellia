use crate as pallet_genesis_exchange;
use frame_benchmarking::frame_support::pallet_prelude::GenesisBuild;
use frame_support::{parameter_types, traits::Currency};
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
		GenesisExchange: pallet_genesis_exchange::{Pallet, Call, Storage, Event<T>},
		GenesisBank: pallet_genesis_bank::{Pallet, Call, Storage, Event<T>},
		Utils: pallet_utils::{Pallet, Call, Storage, Event<T>},
	}
);

pub const LOAN_TERM_DURATION: BlockNumber = 10000;
pub const GENESIS_CML_LOAN_AMOUNT: Balance = 5000000000000;
pub const GENESIS_BANK_INTEREST_RATE: Balance = 5;
pub const LOAN_BILLING_CYCLE: BlockNumber = 1000;

parameter_types! {
	pub const LoanTermDuration: BlockNumber = LOAN_TERM_DURATION;
	pub const GenesisCmlLoanAmount: Balance = GENESIS_CML_LOAN_AMOUNT;
	pub const InterestRate: Balance = GENESIS_BANK_INTEREST_RATE;
	pub const BillingCycle: BlockNumber = LOAN_BILLING_CYCLE;
}

impl pallet_genesis_bank::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type CurrencyOperations = Utils;
	type CmlOperation = Cml;
	type AuctionOperation = Auction;
	type LoanTermDuration = LoanTermDuration;
	type GenesisCmlLoanAmount = GenesisCmlLoanAmount;
	type InterestRate = InterestRate;
	type BillingCycle = BillingCycle;
}

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
	type StakingSlotsMaxLength = StakingSlotsMaxLength;
	type StopMiningPunishment = StopMiningPunishment;
	type WeightInfo = ();
}

pub const AUCTION_DEAL_WINDOW_BLOCK: BlockNumber = 50;
pub const MIN_PRICE_FOR_BID: Balance = 1;
pub const AUCTION_PLEDGE_AMOUNT: Balance = 100;
pub const AUCTION_OWNER_PENALTY_FOR_EACH_BID: Balance = 1;
pub const MAX_USERS_PER_AUCTION: u64 = 100;
pub const AUCTION_FEE_PER_WINDOW: Balance = 1;

parameter_types! {
	pub const AuctionDealWindowBLock: BlockNumber = AUCTION_DEAL_WINDOW_BLOCK;
	pub const MinPriceForBid: Balance = MIN_PRICE_FOR_BID;
	pub const AuctionOwnerPenaltyForEachBid: Balance = AUCTION_OWNER_PENALTY_FOR_EACH_BID;
	pub const AuctionPledgeAmount: Balance = AUCTION_PLEDGE_AMOUNT;
	pub const MaxUsersPerAuction: u64 = MAX_USERS_PER_AUCTION;
	pub const AuctionFeePerWindow: Balance = AUCTION_FEE_PER_WINDOW;
}

impl pallet_auction::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type AuctionDealWindowBLock = AuctionDealWindowBLock;
	type MinPriceForBid = MinPriceForBid;
	type AuctionOwnerPenaltyForEachBid = AuctionOwnerPenaltyForEachBid;
	type CurrencyOperations = Utils;
	type CmlOperation = Cml;
	type AuctionOperation = Auction;
	type GenesisBankOperation = GenesisBank;
	type AuctionPledgeAmount = AuctionPledgeAmount;
	type MaxUsersPerAuction = MaxUsersPerAuction;
	type AuctionFeePerWindow = AuctionFeePerWindow;
	type WeightInfo = ();
}

pub const PER_RATE: Balance = 5;
pub const INTEREST_PERIOD_LENGTH: BlockNumber = 1000;

parameter_types! {
	pub const PER: Balance = PER_RATE;
	pub const InterestPeriodLength: BlockNumber = INTEREST_PERIOD_LENGTH;
}

impl pallet_genesis_exchange::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type CurrencyOperations = Utils;
	type CmlOperation = Cml;
	type PER = PER;
	type GenesisBankOperation = GenesisBank;
	type InterestPeriodLength = InterestPeriodLength;
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

pub const OPERATION_USD_AMOUNT: Balance = 40_000 * 10_000_000_000 * 100;
pub const OPERATION_TEA_AMOUNT: Balance = 40_000 * 10_000_000_000 * 100;
pub const COMPETITION_USER_USD_AMOUNT: Balance = 1000 * 10_000_000_000 * 100;

pub const OPERATION_ACCOUNT: u64 = 100;
pub const COMPETITION_USERS1: u64 = 101;
pub const COMPETITION_USERS2: u64 = 102;
pub const COMPETITION_USERS3: u64 = 103;

// Build genesis storage according to the mock runtime.
#[allow(dead_code)]
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	pallet_genesis_exchange::GenesisConfig::<Test> {
		operation_account: OPERATION_ACCOUNT,
		operation_tea_amount: OPERATION_TEA_AMOUNT,
		operation_usd_amount: OPERATION_USD_AMOUNT,
		competition_users: vec![COMPETITION_USERS1, COMPETITION_USERS2, COMPETITION_USERS3]
			.iter()
			.map(|v| (*v, COMPETITION_USER_USD_AMOUNT))
			.collect(),
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
		<Test as pallet_genesis_exchange::Config>::Currency::make_free_balance_be(
			&OPERATION_ACCOUNT,
			OPERATION_TEA_AMOUNT,
		);
	});
	ext
}
