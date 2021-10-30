use crate as pallet_auction;
use bonding_curve_interface::BondingCurveOperation;
use codec::{Decode, Encode};
use frame_support::parameter_types;
use frame_support::traits::{Everything, Get};
use frame_system as system;
use genesis_exchange_interface::MiningOperation;
use node_primitives::{Balance, BlockNumber};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use tea_interface::TeaOperation;

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

pub struct BondingCurveOperationMock {}

impl Default for BondingCurveOperationMock {
	fn default() -> Self {
		BondingCurveOperationMock {}
	}
}

impl BondingCurveOperation for BondingCurveOperationMock {
	type AccountId = u64;
	type Balance = Balance;

	fn list_tapp_ids() -> Vec<u64> {
		vec![]
	}

	fn estimate_hosting_income_statements(
		_tapp_id: u64,
	) -> Vec<(Self::AccountId, u64, Self::Balance)> {
		vec![]
	}

	fn current_price(_tapp_id: u64) -> (Self::Balance, Self::Balance) {
		(0, 0)
	}

	fn tapp_user_token_asset(_who: &Self::AccountId) -> Vec<(u64, Self::Balance)> {
		vec![]
	}

	fn is_cml_hosting(_cml_id: u64) -> bool {
		false
	}

	fn transfer_reserved_tokens(_from: &Self::AccountId, _to: &Self::AccountId, _cml_id: u64) {}

	fn npc_account() -> Self::AccountId {
		0
	}

	fn cml_host_tapps(_cml_id: u64) -> Vec<u64> {
		vec![]
	}

	fn try_active_tapp(_tapp_id: u64) -> bool {
		true
	}

	fn try_deactive_tapp(_tapp_id: u64) -> bool {
		true
	}

	fn pay_hosting_penalty(_tapp_id: u64, _cml_id: u64) {}

	fn can_append_pledge(_cml_id: u64) -> bool {
		true
	}

	fn append_pledge(_cml_id: u64) -> bool {
		true
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
	) -> sp_runtime::DispatchResult {
		Ok(())
	}

	fn redeem_coupons(_who: &Self::AccountId, _a_coupon: u32, _b_coupon: u32, _c_coupon: u32) {}
}

pub struct TeaOperationMock {}

impl Default for TeaOperationMock {
	fn default() -> Self {
		TeaOperationMock {}
	}
}

impl TeaOperation for TeaOperationMock {
	type AccountId = u64;

	fn add_new_node(_machine_id: [u8; 32], _who: &Self::AccountId) {}
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
		Auction: pallet_auction::{Pallet, Call, Storage, Event<T>},
		GenesisBank: pallet_genesis_bank::{Pallet, Call, Storage, Event<T>},
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
}

pub const STAKING_PRICE: Balance = 1000;

parameter_types! {
	pub const StakingPrice: Balance = STAKING_PRICE;
	pub const SeedsTimeoutHeight: u32 = 1 * 30 * 24 * 60 * 10;
	pub const StakingPeriodLength: u32 = 100;
	pub const StakingSlotsMaxLength: u32 = 1024;
	pub const StopMiningPunishment: Balance = 100;
	pub const MaxAllowedSuspendHeight: u32 = 1000;
	pub const CmlAMiningRewardRate: Balance = 0;
	pub const CmlBMiningRewardRate: Balance = 0;
	pub const CmlCMiningRewardRate: Balance = 0;
}

impl pallet_cml::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type StakingPrice = StakingPrice;
	type CouponTimoutHeight = SeedsTimeoutHeight;
	type StakingPeriodLength = StakingPeriodLength;
	type SeedFreshDuration = SeedFreshDuration;
	type TeaOperation = TeaOperationMock;
	type CommonUtils = Utils;
	type CurrencyOperations = Utils;
	type StakingEconomics = Cml;
	type AuctionOperation = Auction;
	type StakingSlotsMaxLength = StakingSlotsMaxLength;
	type StopMiningPunishment = StopMiningPunishment;
	type MiningOperation = MiningOperationMock;
	type BondingCurveOperation = BondingCurveOperationMock;
	type MaxAllowedSuspendHeight = MaxAllowedSuspendHeight;
	type CmlAMiningRewardRate = CmlAMiningRewardRate;
	type CmlBMiningRewardRate = CmlBMiningRewardRate;
	type CmlCMiningRewardRate = CmlCMiningRewardRate;
	type WeightInfo = ();
}

pub const AUCTION_DEAL_WINDOW_BLOCK: BlockNumber = 50;
pub const MIN_PRICE_FOR_BID: Balance = 1;
pub const AUCTION_PLEDGE_AMOUNT: Balance = 100;
pub const AUCTION_OWNER_PENALTY_FOR_EACH_BID: Balance = 1;
pub const MAX_USERS_PER_AUCTION: u64 = 100;
pub const AUCTION_FEE_PER_WINDOW: Balance = 1;
pub const NEW_BID_LIMIT_DURATION: BlockNumber = 20;

parameter_types! {
	pub const AuctionDealWindowBLock: BlockNumber = AUCTION_DEAL_WINDOW_BLOCK;
	pub const MinPriceForBid: Balance = MIN_PRICE_FOR_BID;
	pub const AuctionOwnerPenaltyForEachBid: Balance = AUCTION_OWNER_PENALTY_FOR_EACH_BID;
	pub const AuctionPledgeAmount: Balance = AUCTION_PLEDGE_AMOUNT;
	pub const MaxUsersPerAuction: u64 = MAX_USERS_PER_AUCTION;
	pub const AuctionFeePerWindow: Balance = AUCTION_FEE_PER_WINDOW;
	pub const NewBidLimitDuration: BlockNumber = NEW_BID_LIMIT_DURATION;
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
	type BondingCurveOperation = BondingCurveOperationMock;
	type NewBidLimitDuration = NewBidLimitDuration;
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
	type AuctionOperation = Auction;
	type LoanTermDuration = LoanTermDuration;
	type BillingCycle = BillingCycle;
	type CmlALoanAmount = CmlALoanAmount;
	type CmlBLoanAmount = CmlBLoanBmount;
	type CmlCLoanAmount = CmlCLoanCmount;
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

// Build genesis storage according to the mock runtime.
#[allow(dead_code)]
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}
