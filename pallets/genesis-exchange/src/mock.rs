use crate as pallet_genesis_exchange;
use codec::{Decode, Encode};
use frame_benchmarking::frame_support::pallet_prelude::GenesisBuild;
use frame_support::traits::{ConstU32, Everything, Get};
use frame_support::{parameter_types, traits::Currency};
use frame_system as system;
use node_primitives::{Balance, BlockNumber};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

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

pub struct TeaOperationMock {}

impl Default for TeaOperationMock {
	fn default() -> Self {
		TeaOperationMock {}
	}
}

pub struct BondingCurveOperationMock {}

impl Default for BondingCurveOperationMock {
	fn default() -> Self {
		BondingCurveOperationMock {}
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
		GenesisExchange: pallet_genesis_exchange::{Pallet, Call, Storage, Event<T>},
		Utils: pallet_utils::{Pallet, Call, Storage, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
	}
);

impl pallet_randomness_collective_flip::Config for Test {}

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

pub const STAKING_PRICE: Balance = 1000;

parameter_types! {
	pub const StakingPrice: Balance = STAKING_PRICE;
	pub const MachineAccountTopUpAmount: Balance = 1;
	pub const SeedsTimeoutHeight: u32 = 1 * 30 * 24 * 60 * 10;
	pub const StakingPeriodLength: u32 = 100;
	pub const StakingSlotsMaxLength: u32 = 1024;
	pub const StopMiningPunishment: Balance = 100;
	pub const MaxAllowedSuspendHeight: u32 = 1000;
	pub const CmlAMiningRewardRate: Balance = 0;
	pub const CmlBMiningRewardRate: Balance = 0;
	pub const CmlCMiningRewardRate: Balance = 0;
}

pub const PER_RATE: Balance = 5;
pub const INTEREST_PERIOD_LENGTH: BlockNumber = 1000;
pub const CML_A_MINING_MACHINE_COST: Balance = 2000;
pub const CML_B_MINING_MACHINE_COST: Balance = 1000;
pub const CML_C_MINING_MACHINE_COST: Balance = 500;
pub const CML_A_REDEEM_COUPON_COST: Balance = 2000;
pub const CML_B_REDEEM_COUPON_COST: Balance = 1000;
pub const CML_C_REDEEM_COUPON_COST: Balance = 500;
pub const BORROW_ALLOWANCE: Balance = 20000;
pub const BORROW_DEBT_RATIO_CAP: Balance = 20000;
pub const REGISTER_FOR_COMPETITION_ALLOWANCE: Balance = 10;

parameter_types! {
	pub const PER: Balance = PER_RATE;
	pub const InterestPeriodLength: BlockNumber = INTEREST_PERIOD_LENGTH;
	pub const CmlAMiningMachineCost: Balance = CML_A_MINING_MACHINE_COST;
	pub const CmlBMiningMachineCost: Balance = CML_B_MINING_MACHINE_COST;
	pub const CmlCMiningMachineCost: Balance = CML_C_MINING_MACHINE_COST;
	pub const CmlARedeemCouponCost: Balance = CML_A_REDEEM_COUPON_COST;
	pub const CmlBRedeemCouponCost: Balance = CML_B_REDEEM_COUPON_COST;
	pub const CmlCRedeemCouponCost: Balance = CML_C_REDEEM_COUPON_COST;
	pub const BorrowAllowance: Balance = BORROW_ALLOWANCE;
	pub const RegisterForCompetitionAllowance: Balance = REGISTER_FOR_COMPETITION_ALLOWANCE;
}

impl pallet_genesis_exchange::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type CurrencyOperations = Utils;
	type PER = PER;
	type InterestPeriodLength = InterestPeriodLength;
	type RegisterForCompetitionAllowance = RegisterForCompetitionAllowance;
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

impl pallet_utils::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type Reward = ();
	type Slash = ();
	type RandomnessSource = RandomnessCollectiveFlip;
}

pub const OPERATION_USD_AMOUNT: Balance = 40_000 * 10_000_000_000 * 100;
pub const OPERATION_TEA_AMOUNT: Balance = 40_000 * 10_000_000_000 * 100;

pub const OPERATION_ACCOUNT: u64 = 100;
pub const NPC_ACCOUNT: u64 = 111;

pub const BANK_OPERATION_ACCOUNT: u64 = 200;
pub const BANK_INITIAL_BALANCE: Balance = 100_000;

// Build genesis storage according to the mock runtime.
#[allow(dead_code)]
pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap();

	pallet_genesis_exchange::GenesisConfig::<Test> {
		operation_account: Some(OPERATION_ACCOUNT),
		npc_account: Some(NPC_ACCOUNT),
		operation_tea_amount: OPERATION_TEA_AMOUNT,
		operation_usd_amount: OPERATION_USD_AMOUNT,
		bonding_curve_npc: Some((Default::default(), 0)),
		initial_usd_interest_rate: 5,
		borrow_debt_ratio_cap: BORROW_DEBT_RATIO_CAP,
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
		<Test as pallet_genesis_exchange::Config>::Currency::make_free_balance_be(
			&BANK_OPERATION_ACCOUNT,
			BANK_INITIAL_BALANCE,
		);
	});
	ext
}
