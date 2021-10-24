use crate as pallet_tea;
use auction_interface::AuctionOperation;
use bonding_curve_interface::BondingCurveOperation;
use codec::{Decode, Encode};
use frame_benchmarking::Zero;
use frame_support::parameter_types;
use frame_support::traits::{Everything, Get};
use frame_system as system;
use genesis_exchange_interface::MiningOperation;
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

	fn is_cml_in_auction(_cml_id: u64) -> bool {
		false
	}

	fn create_new_bid(_sender: &Self::AccountId, _auction_id: &u64, _price: Self::Balance) {}

	fn update_current_winner(_auction_id: &u64, _bid_user: &Self::AccountId) {}

	fn get_window_block() -> (Self::BlockNumber, Self::BlockNumber) {
		(Zero::zero(), Zero::zero())
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

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Tea: pallet_tea::{Pallet, Call, Storage, Event<T>},
		Cml: pallet_cml::{Pallet, Call, Storage, Event<T>},
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

impl pallet_tea::Config for Test {
	type Event = Event;
	type Currency = Balances;
	type RuntimeActivityThreshold = RuntimeActivityThreshold;
	type MinRaPassedThreshold = MinRaPassedThreshold;
	type UpdateValidatorsDuration = UpdateValidatorsDuration;
	type MaxGroupMemberCount = MaxGroupMemberCount;
	type MinGroupMemberCount = MinGroupMemberCount;
	type WeightInfo = ();
	type CommonUtils = Utils;
	type TaskService = Cml;
	type CmlOperation = Cml;
	type PerRaTaskPoint = PerRaTaskPoint;
}

pub const STAKING_PRICE: Balance = 1000;

parameter_types! {
	pub const MinRaPassedThreshold: u32 = 3;
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
	type TeaOperation = Tea;
	type CommonUtils = Utils;
	type CurrencyOperations = Utils;
	type StakingEconomics = Cml;
	type AuctionOperation = AuctionOperationMock;
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

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}
