use crate as pallet_cml;
use crate::generator::init_genesis;
use crate::{CouponConfig, GenesisCoupons, GenesisSeeds};
use auction_interface::AuctionOperation;
use bonding_curve_interface::BondingCurveOperation;
use codec::{Decode, Encode};
use frame_benchmarking::Zero;
use frame_support::pallet_prelude::*;
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
use tea_interface::TeaOperation;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// same as mock implementation of StakingEconomics in "staking.rs" file
pub const DOLLARS: node_primitives::Balance = 100000;
pub const INVALID_MINING_CML_ID: u64 = 99;
pub const HOSTING_CML_ID: u64 = 98;
pub const INSUFFICIENT_CML_ID: u64 = 97;
pub const NPC_ACCOUNT: u64 = 100;

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

	fn is_cml_hosting(cml_id: u64) -> bool {
		HOSTING_CML_ID == cml_id
	}

	fn transfer_reserved_tokens(_from: &Self::AccountId, _to: &Self::AccountId, _cml_id: u64) {}

	fn npc_account() -> Self::AccountId {
		NPC_ACCOUNT
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

	fn can_append_pledge(cml_id: u64) -> bool {
		cml_id != INSUFFICIENT_CML_ID
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

pub struct TeaOperationMock {}

impl Default for TeaOperationMock {
	fn default() -> Self {
		TeaOperationMock {}
	}
}

impl TeaOperation for TeaOperationMock {
	type AccountId = u64;

	fn add_new_node(_machine_id: [u8; 32], _who: &Self::AccountId) {}

	fn update_node_key(_old: [u8; 32], _new: [u8; 32], _sender: &Self::AccountId) {}

	fn remove_node(_machine_id: [u8; 32], _sender: &Self::AccountId) {}
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
	type StakingPrice = StakingPrice;
	type MachineAccountTopUpAmount = MachineAccountTopUpAmount;
	type StakingPeriodLength = StakingPeriodLength;
	type CouponTimoutHeight = SeedsTimeoutHeight;
	type SeedFreshDuration = SeedFreshDuration;
	type TeaOperation = TeaOperationMock;
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
	coupons: GenesisCoupons<u64>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			seeds: GenesisSeeds::default(),
			coupons: GenesisCoupons::default(),
		}
	}
}

impl ExtBuilder {
	pub fn init_seeds(mut self) -> Self {
		self.seeds = init_genesis([1; 32]);
		self
	}

	pub fn coupons(mut self, coupons: Vec<CouponConfig<u64>>) -> Self {
		self.coupons.coupons = coupons;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();

		pallet_cml::GenesisConfig::<Test> {
			genesis_seeds: self.seeds,
			genesis_coupons: self.coupons,
			initial_task_point_base: 10000,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
