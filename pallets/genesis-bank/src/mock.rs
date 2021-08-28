use crate as pallet_genesis_bank;
use auction_interface::AuctionOperation;
use frame_benchmarking::frame_support::pallet_prelude::GenesisBuild;
use frame_benchmarking::frame_support::sp_runtime::DispatchResult;
use frame_support::{parameter_types, traits::Currency};
use frame_system as system;
use genesis_exchange_interface::MiningOperation;
use node_primitives::{Balance, BlockNumber};
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
pub const IN_AUCTION_CML_ID: u64 = 99;

pub const OPERATION_ACCOUNT: u64 = 100;
pub const BANK_INITIAL_BALANCE: Balance = 100_000;
pub const BANK_INITIAL_INTEREST_RATE: Balance = 10;

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

	fn is_cml_in_auction(cml_id: u64) -> bool {
		cml_id == IN_AUCTION_CML_ID
	}

	fn create_new_bid(_sender: &Self::AccountId, _auction_id: &u64, _price: Self::Balance) {
		todo!()
	}

	fn update_current_winner(_auction_id: &u64, _bid_user: &Self::AccountId) {
		todo!()
	}

	fn get_window_block() -> (Self::BlockNumber, Self::BlockNumber) {
		todo!()
	}
}

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
	) -> DispatchResult {
		Ok(())
	}

	fn redeem_coupons(_who: &Self::AccountId, _a_coupon: u32, _b_coupon: u32, _c_coupon: u32) {}
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
		Cml: pallet_cml::{Pallet, Call, Storage, Event<T>},
		GenesisBank: pallet_genesis_bank::{Pallet, Call, Storage, Event<T>},
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

pub const LOAN_TERM_DURATION: BlockNumber = 10000;
pub const LOAN_BILLING_CYCLE: BlockNumber = 1000;
pub const CML_A_LOAN_AMOUNT: Balance = 2000;
pub const CML_B_LOAN_AMOUNT: Balance = 1000;
pub const CML_C_LOAN_AMOUNT: Balance = 500;

parameter_types! {
	pub const LoanTermDuration: BlockNumber = LOAN_TERM_DURATION;
	pub const BillingCycle: BlockNumber = LOAN_BILLING_CYCLE;
	pub const CmlALoanAmount: Balance = CML_A_LOAN_AMOUNT;
	pub const CmlBLoanBmount: Balance = CML_B_LOAN_AMOUNT;
	pub const CmlCLoanCmount: Balance = CML_C_LOAN_AMOUNT;
}

impl pallet_genesis_bank::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type CurrencyOperations = Utils;
	type CmlOperation = Cml;
	type AuctionOperation = AuctionOperationMock;
	type LoanTermDuration = LoanTermDuration;
	type BillingCycle = BillingCycle;
	type CmlALoanAmount = CmlALoanAmount;
	type CmlBLoanAmount = CmlBLoanBmount;
	type CmlCLoanAmount = CmlCLoanCmount;
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
	let mut t = system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	pallet_genesis_bank::GenesisConfig::<Test> {
		operation_account: OPERATION_ACCOUNT,
		bank_initial_balance: BANK_INITIAL_BALANCE,
		bank_initial_interest_rate: BANK_INITIAL_INTEREST_RATE,
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
		<Test as pallet_genesis_bank::Config>::Currency::make_free_balance_be(
			&OPERATION_ACCOUNT,
			BANK_INITIAL_BALANCE,
		);
	});
	ext
}
