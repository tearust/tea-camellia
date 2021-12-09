#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use codec::{Decode, Encode};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{
		Currency, Everything, Get, Imbalance, KeyOwnerProofSystem, LockIdentifier, OnUnbalanced,
		U128CurrencyToVote,
	},
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_PER_SECOND},
		DispatchClass, IdentityFee, Weight,
	},
};
use frame_system::{
	limits::{BlockLength, BlockWeights},
	EnsureOneOf, EnsureRoot,
};
use node_primitives::{BlockNumber, Hash, Moment};
use pallet_election_provider_multi_phase::FallbackStrategy;
use pallet_grandpa::fg_primitives;
use pallet_grandpa::{AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use pallet_session::historical as pallet_session_historical;
use pallet_transaction_payment::{CurrencyAdapter, Multiplier, TargetedFeeAdjustment};
use scale_info::TypeInfo;
use sp_api::impl_runtime_apis;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_core::{
	crypto::KeyTypeId,
	u32_trait::{_1, _2, _3, _4},
	OpaqueMetadata, H256,
};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, NumberFor, OpaqueKeys},
	transaction_validity::{TransactionPriority, TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, FixedPointNumber, Perquintill,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use staking_economics::index::{STAKING_PRICE_TABLE, STAKING_SLOTS_MAX_LENGTH};

// A few exports that help ease life for downstream crates.
pub use node_primitives::{AccountId, Balance, Index, Signature};
pub use pallet_balances::Call as BalancesCall;
pub use pallet_staking::StakerStatus;
pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};

/// Constant values used within the runtime.
pub mod constants;
mod weights;
use constants::{currency::*, time::*};

pub use pallet_auction;
pub use pallet_balances;
pub use pallet_bonding_curve;
pub use pallet_cml;
pub use pallet_genesis_bank;
pub use pallet_genesis_exchange;
pub use pallet_staking;
pub use pallet_tea;
pub use pallet_utils;

/// Digest item type.
pub type DigestItem = generic::DigestItem<Hash>;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub babe: Babe,
			pub grandpa: Grandpa,
			pub im_online: ImOnline,
			pub authority_discovery: AuthorityDiscovery,
		}
	}
}

type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;

#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
pub struct SeedFreshDuration {
	duration: u32,
}

impl Get<u32> for SeedFreshDuration {
	fn get() -> u32 {
		SEED_FRESH_DURATION
	}
}

pub struct DealWithFees;
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance>) {
		if let Some(fees) = fees_then_tips.next() {
			// for fees, 100% to author
			let mut split = fees.ration(0, 100);
			if let Some(tips) = fees_then_tips.next() {
				// for tips, if any, 100% to author
				tips.ration_merge_into(0, 100, &mut split);
			}
			Author::on_unbalanced(split.1);
		}
	}
}

pub struct Author;
impl OnUnbalanced<NegativeImbalance> for Author {
	fn on_nonzero_unbalanced(amount: NegativeImbalance) {
		Balances::resolve_creating(&Authorship::author(), amount);
	}
}

// To learn more about runtime versioning and what each of the following value means:
//   https://substrate.dev/docs/en/knowledgebase/runtime/upgrades#runtime-versioning
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("tea-layer1"),
	impl_name: create_runtime_str!("tea-layer1"),
	authoring_version: 1,
	// The version of the runtime specification. A full node will not attempt to use its native
	//   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
	//   `spec_version`, and `authoring_version` are the same between Wasm and native.
	// This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
	//   the compatible custom types.
	spec_version: 103,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
};

/// The BABE epoch configuration at genesis.
pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
	sp_consensus_babe::BabeEpochConfiguration {
		c: PRIMARY_PROBABILITY,
		allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryPlainSlots,
	};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

/// We assume that ~10% of the block weight is consumed by `on_initialize` handlers.
/// This is used to limit the maximal weight of a single extrinsic.
const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time.
const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

parameter_types! {
	pub const Version: RuntimeVersion = VERSION;
	pub const BlockHashCount: BlockNumber = 2400;
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u8 = 42;
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = Everything;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = RuntimeBlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = RuntimeBlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The aggregated dispatch type that is available for extrinsics.
	type Call = Call;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	type Lookup = AccountIdLookup<AccountId, ()>;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type Event = Event;
	/// The ubiquitous origin type.
	type Origin = Origin;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Version of the runtime.
	type Version = Version;
	/// Converts a module to the index of the module in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type PalletInfo = PalletInfo;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = weights::frame_system::WeightInfo<Runtime>;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	/// The set code logic, just the default since we're not a parachain.
	type OnSetCode = ();
}

impl pallet_randomness_collective_flip::Config for Runtime {}

parameter_types! {
	// NOTE: Currently it is not possible to change the epoch duration after the chain has started.
	//       Attempting to do so will brick block production.
	pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS;
	pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
	pub const ReportLongevity: u64 =
		BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
}

impl pallet_babe::Config for Runtime {
	type EpochDuration = EpochDuration;
	type ExpectedBlockTime = ExpectedBlockTime;
	type EpochChangeTrigger = pallet_babe::ExternalTrigger;
	type DisabledValidators = Session;

	type KeyOwnerProofSystem = Historical;

	type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
		KeyTypeId,
		pallet_babe::AuthorityId,
	)>>::IdentificationTuple;

	type KeyOwnerProof = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
		KeyTypeId,
		pallet_babe::AuthorityId,
	)>>::Proof;

	type HandleEquivocation =
		pallet_babe::EquivocationHandler<Self::KeyOwnerIdentification, Offences, ReportLongevity>;

	type WeightInfo = (); // not setting because polkadot do not set either, add weight info if needed later
}

impl pallet_mmr::Config for Runtime {
	const INDEXING_PREFIX: &'static [u8] = b"mmr";
	type Hashing = <Runtime as frame_system::Config>::Hashing;
	type Hash = <Runtime as frame_system::Config>::Hash;
	type LeafData = frame_system::Pallet<Self>;
	type OnNewRoot = ();
	type WeightInfo = ();
}

impl pallet_grandpa::Config for Runtime {
	type Event = Event;
	type Call = Call;

	type KeyOwnerProofSystem = Historical;

	type KeyOwnerProof =
		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

	type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
		KeyTypeId,
		GrandpaId,
	)>>::IdentificationTuple;

	type HandleEquivocation = pallet_grandpa::EquivocationHandler<
		Self::KeyOwnerIdentification,
		Offences,
		ReportLongevity,
	>;

	type WeightInfo = (); // not setting because polkadot do not set either, add weight info if needed later
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Babe;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = weights::pallet_timestamp::WeightInfo<Runtime>;
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 1 * MILLICENTS;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const TransactionByteFee: Balance = 1;
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1, 100_000);
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
}

impl pallet_transaction_payment::Config for Runtime {
	type OnChargeTransaction = CurrencyAdapter<Balances, DealWithFees>;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate =
		TargetedFeeAdjustment<Self, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;
}

impl pallet_sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}

parameter_types! {
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(17);
}

impl pallet_session::Config for Runtime {
	type Event = Event;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_staking::StashOf<Self>;
	type ShouldEndSession = Babe;
	type NextSessionRotation = Babe;
	type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
	type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = opaque::SessionKeys;
	type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
	type WeightInfo = weights::pallet_session::WeightInfo<Runtime>;
}

impl pallet_session::historical::Config for Runtime {
	type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
	type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

parameter_types! {
	pub const SessionsPerEra: sp_staking::SessionIndex = 6;
	pub const BondingDuration: pallet_staking::EraIndex = 4;
	pub const SlashDeferDuration: pallet_staking::EraIndex = 1; // 1/4 the bonding duration.
	pub const MaxNominatorRewardedPerValidator: u32 = 256;
	pub OffchainRepeat: BlockNumber = 5;
}

use frame_election_provider_support::onchain;
impl pallet_staking::Config for Runtime {
	const MAX_NOMINATIONS: u32 = MAX_NOMINATIONS;
	type Currency = Balances;
	type UnixTime = Timestamp;
	type CurrencyToVote = U128CurrencyToVote;
	type RewardRemainder = ();
	type Event = Event;
	type Slash = ();
	type Reward = (); // rewards are minted from the void
	type SessionsPerEra = SessionsPerEra;
	type BondingDuration = BondingDuration;
	type SlashDeferDuration = SlashDeferDuration;
	/// A super-majority of the council can cancel the slash.
	type SlashCancelOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, CouncilCollective>,
	>;
	type SessionInterface = Self;
	type NextNewSession = Session;
	type MaxNominatorRewardedPerValidator = MaxNominatorRewardedPerValidator;
	type ElectionProvider = ElectionProviderMultiPhase;
	type GenesisElectionProvider = onchain::OnChainSequentialPhragmen<
		pallet_election_provider_multi_phase::OnChainConfig<Self>,
	>;
	type WeightInfo = pallet_staking::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const UncleGenerations: BlockNumber = 5;
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = (Staking, ImOnline);
}
parameter_types! {
	pub const MaxAuthorities: u32 = 100;
}

impl pallet_authority_discovery::Config for Runtime {
	type MaxAuthorities = MaxAuthorities;
}

impl pallet_offences::Config for Runtime {
	type Event = Event;
	type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
	type OnOffenceHandler = Staking;
}

parameter_types! {
	pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
	/// We prioritize im-online heartbeats over election solution submission.
	pub const StakingUnsignedPriority: TransactionPriority = TransactionPriority::max_value() / 2;
}

impl pallet_im_online::Config for Runtime {
	type AuthorityId = ImOnlineId;
	type Event = Event;
	type NextSessionRotation = Babe;
	type ValidatorSet = Historical;
	type ReportUnresponsiveness = Offences;
	type UnsignedPriority = ImOnlineUnsignedPriority;
	type WeightInfo = weights::pallet_im_online::WeightInfo<Runtime>;
}

/// Maximum number of iterations for balancing that will be executed in the embedded OCW
/// miner of election provider multi phase.
pub const MINER_MAX_ITERATIONS: u32 = 10;

/// A source of random balance for NposSolver, which is meant to be run by the OCW election miner.
pub struct OffchainRandomBalancing;
impl frame_support::pallet_prelude::Get<Option<(usize, sp_npos_elections::ExtendedBalance)>>
	for OffchainRandomBalancing
{
	fn get() -> Option<(usize, sp_npos_elections::ExtendedBalance)> {
		use sp_runtime::traits::TrailingZeroInput;
		let iters = match MINER_MAX_ITERATIONS {
			0 => 0,
			max @ _ => {
				let seed = sp_io::offchain::random_seed();
				let random = <u32>::decode(&mut TrailingZeroInput::new(&seed))
					.expect("input is padded with zeroes; qed")
					% max.saturating_add(1);
				random as usize
			}
		};

		Some((iters, 0))
	}
}

parameter_types! {
	// phase durations. 1/4 of the last session for each.
	pub const SignedPhase: u32 = EPOCH_DURATION_IN_BLOCKS / 4;
	pub const UnsignedPhase: u32 = EPOCH_DURATION_IN_BLOCKS / 4;

	pub SolutionImprovementThreshold: Perbill = Perbill::from_rational(1u32, 10_000);

	// miner configs
	pub const MultiPhaseUnsignedPriority: TransactionPriority = StakingUnsignedPriority::get() - 1u64;
	pub MinerMaxWeight: Weight = RuntimeBlockWeights::get()
		.get(DispatchClass::Normal)
		.max_extrinsic.expect("Normal extrinsics have a weight limit configured; qed")
		.saturating_sub(BlockExecutionWeight::get());
	// Solution can occupy 90% of normal block size
	pub MinerMaxLength: u32 = Perbill::from_rational(9u32, 10) *
		*RuntimeBlockLength::get()
		.max
		.get(DispatchClass::Normal);

	// signed config
	pub const SignedMaxSubmissions: u32 = 10;
	pub const SignedRewardBase: Balance = 1 * DOLLARS;
	pub const SignedDepositBase: Balance = 1 * DOLLARS;
	pub const SignedDepositByte: Balance = 1 * CENTS;

	// fallback: run election on-chain.
	pub const Fallback: FallbackStrategy = FallbackStrategy::OnChain;
}

sp_npos_elections::generate_solution_type!(
	#[compact]
	pub struct NposSolution16::<
		VoterIndex = u32,
		TargetIndex = u16,
		Accuracy = sp_runtime::PerU16,
	>(16)
);

pub const MAX_NOMINATIONS: u32 = <NposSolution16 as sp_npos_elections::NposSolution>::LIMIT as u32;

impl pallet_election_provider_multi_phase::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type EstimateCallFee = TransactionPayment;
	type SignedPhase = SignedPhase;
	type UnsignedPhase = UnsignedPhase;
	type SolutionImprovementThreshold = SolutionImprovementThreshold;
	type OffchainRepeat = OffchainRepeat;
	type MinerMaxWeight = MinerMaxWeight;
	type MinerMaxLength = MinerMaxLength;
	type MinerTxPriority = MultiPhaseUnsignedPriority;
	type DataProvider = Staking;
	type Fallback = Fallback;
	type SignedMaxSubmissions = SignedMaxSubmissions;
	type SignedMaxWeight = MinerMaxWeight;
	type SignedRewardBase = SignedRewardBase;
	type SignedDepositBase = SignedDepositBase;
	type SignedDepositByte = SignedDepositByte;
	type SignedDepositWeight = ();
	type SlashHandler = (); // burn slashes
	type RewardHandler = (); // nothing to do upon rewards
	type ForceOrigin = EnsureRootOrHalfCouncil;
	type Solution = NposSolution16;
	type Solver = frame_election_provider_support::SequentialPhragmen<
		AccountId,
		pallet_election_provider_multi_phase::SolutionAccuracyOf<Self>,
		OffchainRandomBalancing,
	>;
	type OnChainAccuracy = Perbill;
	type WeightInfo = pallet_election_provider_multi_phase::weights::SubstrateWeight<Runtime>;
	type BenchmarkingConfig = ();
}

parameter_types! {
	pub const CandidacyBond: Balance = 10 * DOLLARS;
	// 1 storage item created, key size is 32 bytes, value size is 16+16.
	pub const VotingBondBase: Balance = deposit(1, 64);
	// additional data per vote is 32 bytes (account id).
	pub const VotingBondFactor: Balance = deposit(0, 32);
	/// Weekly council elections; scaling up to monthly eventually.
	pub const TermDuration: BlockNumber = 7 * DAYS;
	pub const DesiredMembers: u32 = 13;
	pub const DesiredRunnersUp: u32 = 7;
	pub const ElectionsPhragmenPalletId: LockIdentifier = *b"phrelect";
}

impl pallet_elections_phragmen::Config for Runtime {
	type Event = Event;
	type PalletId = ElectionsPhragmenPalletId;
	type Currency = Balances;
	type ChangeMembers = Council;
	// NOTE: this implies that council's genesis members cannot be set directly and must come from
	// this module.
	type InitializeMembers = Council;
	type CurrencyToVote = U128CurrencyToVote;
	type CandidacyBond = CandidacyBond;
	type VotingBondBase = VotingBondBase;
	type VotingBondFactor = VotingBondFactor;
	type LoserCandidate = ();
	type KickedMember = ();
	type DesiredMembers = DesiredMembers;
	type DesiredRunnersUp = DesiredRunnersUp;
	type TermDuration = TermDuration;
	type WeightInfo = pallet_elections_phragmen::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 7 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = weights::pallet_collective::WeightInfo<Runtime>;
}

parameter_types! {
	pub const TechnicalMotionDuration: BlockNumber = 7 * DAYS;
	pub const TechnicalMaxProposals: u32 = 100;
	pub const TechnicalMaxMembers: u32 = 100;
}

type TechnicalCollective = pallet_collective::Instance2;
impl pallet_collective::Config<TechnicalCollective> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = TechnicalMaxProposals;
	type MaxMembers = TechnicalMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = weights::pallet_collective::WeightInfo<Runtime>;
}

type EnsureRootOrHalfCouncil = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	pallet_collective::EnsureProportionMoreThan<_1, _2, AccountId, CouncilCollective>,
>;

impl pallet_membership::Config<pallet_membership::Instance1> for Runtime {
	type Event = Event;
	type AddOrigin = EnsureRootOrHalfCouncil;
	type RemoveOrigin = EnsureRootOrHalfCouncil;
	type SwapOrigin = EnsureRootOrHalfCouncil;
	type ResetOrigin = EnsureRootOrHalfCouncil;
	type PrimeOrigin = EnsureRootOrHalfCouncil;
	type MembershipInitialized = TechnicalCommittee;
	type MembershipChanged = TechnicalCommittee;
	type MaxMembers = TechnicalMaxMembers;
	type WeightInfo = weights::pallet_membership::WeightInfo<Runtime>;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
		RuntimeBlockWeights::get().max_block;
	pub const MaxScheduledPerBlock: u32 = 50;
}

impl pallet_scheduler::Config for Runtime {
	type Event = Event;
	type Origin = Origin;
	type PalletsOrigin = OriginCaller;
	type Call = Call;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = MaxScheduledPerBlock;
	type WeightInfo = weights::pallet_scheduler::WeightInfo<Runtime>;
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type WeightInfo = weights::pallet_utility::WeightInfo<Runtime>;
}

parameter_types! {
	pub const LaunchPeriod: BlockNumber = 28 * DAYS;
	pub const VotingPeriod: BlockNumber = 28 * DAYS;
	pub const FastTrackVotingPeriod: BlockNumber = 3 * HOURS;
	pub const MinimumDeposit: Balance = 100 * DOLLARS;
	pub const EnactmentPeriod: BlockNumber = 28 * DAYS;
	pub const CooloffPeriod: BlockNumber = 7 * DAYS;
	// One cent: $10,000 / MB
	pub const PreimageByteDeposit: Balance = 1 * CENTS;
	pub const InstantAllowed: bool = true;
	pub const MaxVotes: u32 = 100;
	pub const MaxProposals: u32 = 100;
}

impl pallet_democracy::Config for Runtime {
	type Proposal = Call;
	type Event = Event;
	type Currency = Balances;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;
	type VotingPeriod = VotingPeriod;
	type VoteLockingPeriod = EnactmentPeriod; // Same as EnactmentPeriod
	type MinimumDeposit = MinimumDeposit;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin =
		pallet_collective::EnsureProportionAtLeast<_1, _2, AccountId, CouncilCollective>;
	/// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin =
		pallet_collective::EnsureProportionAtLeast<_3, _4, AccountId, CouncilCollective>;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.
	type ExternalDefaultOrigin =
		pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, CouncilCollective>;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.
	type FastTrackOrigin =
		pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, TechnicalCollective>;
	type InstantOrigin =
		pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>;
	type InstantAllowed = InstantAllowed;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin =
		pallet_collective::EnsureProportionAtLeast<_2, _3, AccountId, CouncilCollective>;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.
	type CancelProposalOrigin = EnsureOneOf<
		AccountId,
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<_1, _1, AccountId, TechnicalCollective>,
	>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cool-off period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
	type CooloffPeriod = CooloffPeriod;
	type PreimageByteDeposit = PreimageByteDeposit;
	type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
	type Slash = ();
	type Scheduler = Scheduler;
	type PalletsOrigin = OriginCaller;
	type MaxVotes = MaxVotes;
	type WeightInfo = weights::pallet_democracy::WeightInfo<Runtime>;
	type MaxProposals = MaxProposals;
}

parameter_types! {
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const DepositBase: Balance = deposit(1, 88);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = deposit(0, 32);
	pub const MaxSignatories: u16 = 100;
}

impl pallet_multisig::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = weights::pallet_multisig::WeightInfo<Runtime>;
}

parameter_types! {
	pub const BasicDeposit: Balance = 10 * DOLLARS;       // 258 bytes on-chain
	pub const FieldDeposit: Balance = 250 * CENTS;        // 66 bytes on-chain
	pub const SubAccountDeposit: Balance = 2 * DOLLARS;   // 53 bytes on-chain
	pub const MaxSubAccounts: u32 = 100;
	pub const MaxAdditionalFields: u32 = 100;
	pub const MaxRegistrars: u32 = 20;
}

impl pallet_identity::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type BasicDeposit = BasicDeposit;
	type FieldDeposit = FieldDeposit;
	type SubAccountDeposit = SubAccountDeposit;
	type MaxSubAccounts = MaxSubAccounts;
	type MaxAdditionalFields = MaxAdditionalFields;
	type MaxRegistrars = MaxRegistrars;
	type Slashed = ();
	type ForceOrigin = EnsureRootOrHalfCouncil;
	type RegistrarOrigin = EnsureRootOrHalfCouncil;
	type WeightInfo = weights::pallet_identity::WeightInfo<Runtime>;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
	Call: From<C>,
{
	type Extrinsic = UncheckedExtrinsic;
	type OverarchingCall = Call;
}

impl pallet_utils::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type Reward = ();
	type Slash = ();
	type RandomnessSource = RandomnessCollectiveFlip;
}

parameter_types! {
	/// (1 * 60 * 10) blocks equals (1 * 60 * 10 * 6secs) = 1hours
	pub const RuntimeActivityThreshold: u32 = 1 * 60 * 10;
	/// (4 * 60 * 10) blocks equals (4 * 60 * 10 * 6secs) = 4hours
	pub const UpdateValidatorsDuration: u32 = 4 * 60 * 10;
	pub const PerRaTaskPoint: u32 = 10000;
	pub const MaxGroupMemberCount: u32 = 10;
	pub const MinGroupMemberCount: u32 = 5;
	/// (10) blocks equals (10 * 6secs) = 1 minute
	pub const MaxAllowedRaCommitDuration: u32 = 10;
	pub const PhishingAllowedDuration: u32 = 100;
	pub const TipsAllowedDuration: u32 = 100;
	pub const OfflineValidDuration: u32 = 100;
	pub const OfflineEffectThreshold: u32 = 2;
	pub const ReportRawardDuration: u32 = 100;
}

impl pallet_tea::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type RuntimeActivityThreshold = RuntimeActivityThreshold;
	type UpdateValidatorsDuration = UpdateValidatorsDuration;
	type MaxGroupMemberCount = MaxGroupMemberCount;
	type MinGroupMemberCount = MinGroupMemberCount;
	type MaxAllowedRaCommitDuration = MaxAllowedRaCommitDuration;
	type PhishingAllowedDuration = PhishingAllowedDuration;
	type TipsAllowedDuration = TipsAllowedDuration;
	type OfflineValidDuration = OfflineValidDuration;
	type OfflineEffectThreshold = OfflineEffectThreshold;
	type ReportRawardDuration = ReportRawardDuration;
	type WeightInfo = weights::pallet_tea::WeightInfo<Runtime>;
	type CurrencyOperations = Utils;
	type CommonUtils = Utils;
	type TaskService = Cml;
	type CmlOperation = Cml;
	type PerRaTaskPoint = PerRaTaskPoint;
}

#[cfg(not(feature = "fast"))]
/// Timeout height is (1 * 30 * 24 * 60 * 10 * 6secs) about 1 month
const SEEDS_TIMEOUT_HEIGHT: u32 = 1 * 30 * 24 * 60 * 10;
#[cfg(feature = "fast")]
/// Timeout height is (2 * 7 * 24 * 60 * 10 * 6secs) about 2 weeks
const SEEDS_TIMEOUT_HEIGHT: u32 = 2 * 7 * 24 * 60 * 10;

#[cfg(not(feature = "fast"))]
/// Staking period length is (1 * 24 * 60 * 10) about 1 day
const STAKING_PERIOD_LENGTH: u32 = 1 * 24 * 60 * 10;
#[cfg(feature = "fast")]
/// Staking period length is 10 about 1 minutes
const STAKING_PERIOD_LENGTH: u32 = 10;

#[cfg(not(feature = "fast"))]
/// Seed fresh duration is (7 * 24 * 60 * 10) about 1 week
const SEED_FRESH_DURATION: u32 = 7 * 24 * 60 * 10;
#[cfg(feature = "fast")]
/// Seed fresh duration is (30 * 10) about 30 minuts
const SEED_FRESH_DURATION: u32 = 30 * 10;

parameter_types! {
	/// Investors need to pay StakingPrice for each staking slots of CML regardless the index number
	pub const StakingPrice: Balance = 1000 * DOLLARS;
	pub const MachineAccountTopUpAmount: Balance = 1 * DOLLARS;
	/// After SeedsTimeoutHeight, coupon will be expired
	pub const SeedsTimeoutHeight: u32 = SEEDS_TIMEOUT_HEIGHT;
	/// Every StakingPeriodLength, DAO will calculate the staking earning and pay to reward balance
	pub const StakingPeriodLength: u32 = STAKING_PERIOD_LENGTH;
	/// CML cannot have more than StakingSlotsMaxLength slots
	pub const StakingSlotsMaxLength: u32 = STAKING_SLOTS_MAX_LENGTH;
	/// Punishment amount need to pay for each staking account when stop mining.
	pub const StopMiningPunishment: Balance = 100 * DOLLARS;
	pub const MaxAllowedSuspendHeight: u32 = 1000;
	pub const CmlAMiningRewardRate: Balance = 0;
	/// Type B cml miner will get 50% reward
	pub const CmlBMiningRewardRate: Balance = 5000;
	/// Type C cml miner will get 50% reward
	pub const CmlCMiningRewardRate: Balance = 5000;
}

impl pallet_cml::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type StakingPrice = StakingPrice;
	type MachineAccountTopUpAmount = MachineAccountTopUpAmount;
	type CouponTimoutHeight = SeedsTimeoutHeight;
	type StakingPeriodLength = StakingPeriodLength;
	type SeedFreshDuration = SeedFreshDuration;
	type TeaOperation = Tea;
	type CommonUtils = Utils;
	type CurrencyOperations = Utils;
	type MiningOperation = GenesisExchange;
	type AuctionOperation = Auction;
	type BondingCurveOperation = BondingCurve;
	type StakingEconomics = staking_economics::TeaStakingEconomics;
	type StakingSlotsMaxLength = StakingSlotsMaxLength;
	type WeightInfo = weights::pallet_cml::WeightInfo<Runtime>;
	type StopMiningPunishment = StopMiningPunishment;
	type MaxAllowedSuspendHeight = MaxAllowedSuspendHeight;
	type CmlAMiningRewardRate = CmlAMiningRewardRate;
	type CmlBMiningRewardRate = CmlBMiningRewardRate;
	type CmlCMiningRewardRate = CmlCMiningRewardRate;
}

#[cfg(not(feature = "fast"))]
const LOAN_TERM_DURATION: BlockNumber = 200000;
#[cfg(feature = "fast")]
const LOAN_TERM_DURATION: BlockNumber = 33000; //about 55 hours. good for fast testing

parameter_types! {
	/// Borrower has to repay the loan before LoanTermDuration, otherwise in default
	pub const LoanTermDuration: BlockNumber = LOAN_TERM_DURATION;
	/// The Genesis Bank calculate interest every BillingCycle. If borrower repay the loan before a billing cycle ends,
	/// the interest is calculated to the end of this billing cycle.
	pub const BillingCycle: BlockNumber = 1000;
	pub const CmlALoanAmount: Balance = 2000 * DOLLARS;
	pub const CmlBLoanBmount: Balance = 3000 * DOLLARS;
	pub const CmlCLoanCmount: Balance = 1500 * DOLLARS;
}

impl pallet_genesis_bank::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type CmlOperation = Cml;
	type CurrencyOperations = Utils;
	type AuctionOperation = Auction;
	type LoanTermDuration = LoanTermDuration;
	type BillingCycle = BillingCycle;
	type CmlALoanAmount = CmlALoanAmount;
	type CmlBLoanAmount = CmlBLoanBmount;
	type CmlCLoanAmount = CmlCLoanCmount;
}

parameter_types! {
	pub const PER: Balance = 7;
	pub const InterestPeriodLength: BlockNumber = 1000;
	pub const CmlAMiningMachineCost: Balance = 0;
	pub const CmlBMiningMachineCost: Balance = 0;
	pub const CmlCMiningMachineCost: Balance = 0;
	pub const CmlARedeemCouponCost: Balance =  0;
	pub const CmlBRedeemCouponCost: Balance =  0;
	pub const CmlCRedeemCouponCost: Balance =  0;
	pub const BorrowAllowance: Balance = 10000 * DOLLARS;
}

impl pallet_genesis_exchange::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type CmlOperation = Cml;
	type CurrencyOperations = Utils;
	type GenesisBankOperation = GenesisBank;
	type BondingCurveOperation = BondingCurve;
	type PER = PER;
	type InterestPeriodLength = InterestPeriodLength;
	type CmlAMiningMachineCost = CmlAMiningMachineCost;
	type CmlBMiningMachineCost = CmlBMiningMachineCost;
	type CmlCMiningMachineCost = CmlCMiningMachineCost;
	type CmlARedeemCouponCost = CmlARedeemCouponCost;
	type CmlBRedeemCouponCost = CmlBRedeemCouponCost;
	type CmlCRedeemCouponCost = CmlCRedeemCouponCost;
	type BorrowAllowance = BorrowAllowance;
}

parameter_types! {
	pub const TAppNameMaxLength: u32 = 20;
	pub const TAppDetailMaxLength: u32 = 120;
	pub const TAppLinkMaxLength: u32 = 140;
	pub const TAppTickerMinLength: u32 = 3;
	pub const TAppTickerMaxLength: u32 = 6;
	pub const PoolBalanceReversePrecision: Balance = 10;
	pub const HostArrangeDuration: BlockNumber = 1000;
	pub const HostCostCollectionDuration: BlockNumber = 100;
	pub const ConsumeNoteMaxLength: u32 = 140;
	pub const CidMaxLength: u32 = 100;
	pub const TotalSupplyMaxValue: Balance = 1000000000000000000000000;
	pub const MinTappHostsCount: u32 = 3;
	pub const HostLockingBlockHeight: BlockNumber = 1000;
	pub const TAppLinkDescriptionMaxLength: u32 = 140;
	pub const DefaultBuyCurveTheta: u32 = 10;
	pub const DefaultSellCurveTheta: u32 = 7;
	pub const HostPledgeAmount: Balance = 0 * DOLLARS;
	pub const ReservedLinkRentAmount: Balance = 100 * DOLLARS;
	pub const NotificationsArrangeDuration: BlockNumber = 1000;
}

impl pallet_bonding_curve::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type CurrencyOperations = Utils;
	type CmlOperation = Cml;
	type TAppNameMaxLength = TAppNameMaxLength;
	type TAppTickerMinLength = TAppTickerMinLength;
	type TAppTickerMaxLength = TAppTickerMaxLength;
	type TAppDetailMaxLength = TAppDetailMaxLength;
	type TAppLinkMaxLength = TAppLinkMaxLength;
	type PoolBalanceReversePrecision = PoolBalanceReversePrecision;
	type HostArrangeDuration = HostArrangeDuration;
	type HostCostCollectionDuration = HostCostCollectionDuration;
	type ConsumeNoteMaxLength = ConsumeNoteMaxLength;
	type CidMaxLength = CidMaxLength;
	type TotalSupplyMaxValue = TotalSupplyMaxValue;
	type MinTappHostsCount = MinTappHostsCount;
	type HostLockingBlockHeight = HostLockingBlockHeight;
	type TAppLinkDescriptionMaxLength = TAppLinkDescriptionMaxLength;
	type DefaultBuyCurveTheta = DefaultBuyCurveTheta;
	type DefaultSellCurveTheta = DefaultSellCurveTheta;
	type HostPledgeAmount = HostPledgeAmount;
	type ReservedLinkRentAmount = ReservedLinkRentAmount;
	type NotificationsArrangeDuration = NotificationsArrangeDuration;
}

#[cfg(feature = "fast")]
const AUCTION_WINDOW_BLOCK: BlockNumber = 100;
#[cfg(not(feature = "fast"))]
const AUCTION_WINDOW_BLOCK: BlockNumber = 1000;

parameter_types! {
	/// Every AuctionDealWindowBLock blocks, the auction window closed. the highest bidder is winner.
	/// There is a count down in the UI so that the bidder know when the window close
	pub const AuctionDealWindowBLock: BlockNumber = AUCTION_WINDOW_BLOCK;
	/// This is Bid Increments. Bidder has to pay MinPriceForBid higher than existing top price to make new bid
	pub const MinPriceForBid: Balance = 1 * DOLLARS;
	/// When the auctioneer want to cancel an on going auction, he will need to pay such penalty to
	/// all involved bidders. Every bidder receives AuctionOwnerPenaltyForEachBid
	pub const AuctionOwnerPenaltyForEachBid: Balance = 1 * DOLLARS;
	/// The escrow deposit from auctioneer.
	pub const AuctionPledgeAmount: Balance = 100 * DOLLARS;
	/// How many bids are allowed for each item. To avoid DDoS attack
	pub const MaxUsersPerAuction: u64 = 10000;
	/// Auction fee. Every new auction window, the auctioneer needs to pay such fee if choose to renew to continue to next auction window
	pub const AuctionFeePerWindow: Balance = 1 * DOLLARS;
}
impl pallet_auction::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type CurrencyOperations = Utils;
	type CmlOperation = Cml;
	type AuctionOperation = Auction;
	type GenesisBankOperation = GenesisBank;
	type BondingCurveOperation = BondingCurve;
	type AuctionDealWindowBLock = AuctionDealWindowBLock;
	type MinPriceForBid = MinPriceForBid;
	type AuctionOwnerPenaltyForEachBid = AuctionOwnerPenaltyForEachBid;
	type AuctionPledgeAmount = AuctionPledgeAmount;
	type MaxUsersPerAuction = MaxUsersPerAuction;
	type AuctionFeePerWindow = AuctionFeePerWindow;
	type WeightInfo = weights::pallet_auction::WeightInfo<Runtime>;
}

// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Pallet, Storage},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage, Inherent},
		Babe: pallet_babe::{Pallet, Call, Storage, Config, ValidateUnsigned},
		Grandpa: pallet_grandpa::{Pallet, Call, Storage, Config, Event},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage},
		Sudo: pallet_sudo::{Pallet, Call, Config<T>, Storage, Event<T>},
		Staking: pallet_staking::{Pallet, Call, Config<T>, Storage, Event<T>},
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>},
		Historical: pallet_session_historical::{Pallet},
		Authorship: pallet_authorship::{Pallet, Call, Storage, Inherent},
		Offences: pallet_offences::{Pallet, Storage, Event},
		ImOnline: pallet_im_online::{Pallet, Call, Storage, Event<T>, ValidateUnsigned, Config<T>},
		AuthorityDiscovery: pallet_authority_discovery::{Pallet, Config},
		Elections: pallet_elections_phragmen::{Pallet, Call, Storage, Event<T>, Config<T>},
		ElectionProviderMultiPhase: pallet_election_provider_multi_phase::{Pallet, Call, Storage, Event<T>, ValidateUnsigned},
		Council: pallet_collective::<Instance1>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
		TechnicalCommittee: pallet_collective::<Instance2>::{Pallet, Call, Storage, Origin<T>, Event<T>, Config<T>},
		TechnicalMembership: pallet_membership::<Instance1>::{Pallet, Call, Storage, Event<T>, Config<T>},
		Scheduler: pallet_scheduler::{Pallet, Call, Storage, Event<T>},
		Democracy: pallet_democracy::{Pallet, Call, Storage, Config<T>, Event<T>},
		Utility: pallet_utility::{Pallet, Call, Event},
		Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>},
		Identity: pallet_identity::{Pallet, Call, Storage, Event<T>},
		Mmr: pallet_mmr::{Pallet, Storage},
		// Include the custom logic from the pallets in the runtime.
		Cml: pallet_cml::{Pallet, Call, Config<T>, Storage, Event<T>} = 100,
		Auction: pallet_auction::{Pallet, Call, Storage, Event<T>} = 101,
		Utils: pallet_utils::{Pallet, Call, Storage, Event<T>} = 103,
		GenesisBank: pallet_genesis_bank::{Pallet, Call, Config<T>, Storage, Event<T>} = 104,
		GenesisExchange: pallet_genesis_exchange::{Pallet, Call, Config<T>, Storage, Event<T>} = 105,
		BondingCurve: pallet_bonding_curve::{Pallet, Call, Config<T>, Storage, Event<T>} = 106,
		Tea: pallet_tea::{Pallet, Call, Config<T>, Storage, Event<T>} = 107,
	}
);

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPallets,
>;

/// MMR helper types.
mod mmr {
	use super::Runtime;
	pub use pallet_mmr::primitives::*;

	pub type Leaf = <<Runtime as pallet_mmr::Config>::LeafData as LeafDataProvider>::LeafData;
	pub type Hash = <Runtime as pallet_mmr::Config>::Hash;
	pub type Hashing = <Runtime as pallet_mmr::Config>::Hashing;
}

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			Runtime::metadata().into()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl pallet_mmr::primitives::MmrApi<
		Block,
		mmr::Hash,
	> for Runtime {
		fn generate_proof(leaf_index: u64)
			-> Result<(mmr::EncodableOpaqueLeaf, mmr::Proof<mmr::Hash>), mmr::Error>
		{
			Mmr::generate_proof(leaf_index)
				.map(|(leaf, proof)| (mmr::EncodableOpaqueLeaf::from_leaf(&leaf), proof))
		}

		fn verify_proof(leaf: mmr::EncodableOpaqueLeaf, proof: mmr::Proof<mmr::Hash>)
			-> Result<(), mmr::Error>
		{
			let leaf: mmr::Leaf = leaf
				.into_opaque_leaf()
				.try_decode()
				.ok_or(mmr::Error::Verify)?;
			Mmr::verify_leaf(leaf, proof)
		}

		fn verify_proof_stateless(
			root: mmr::Hash,
			leaf: mmr::EncodableOpaqueLeaf,
			proof: mmr::Proof<mmr::Hash>
		) -> Result<(), mmr::Error> {
			let node = mmr::DataOrHash::Data(leaf.into_opaque_leaf());
			pallet_mmr::verify_leaf_proof::<mmr::Hashing, _>(root, node, proof)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_consensus_babe::BabeApi<Block> for Runtime {
		fn configuration() -> sp_consensus_babe::BabeGenesisConfiguration {
			// The choice of `c` parameter (where `1 - c` represents the
			// probability of a slot being empty), is done in accordance to the
			// slot duration and expected target block time, for safely
			// resisting network delays of maximum two seconds.
			// <https://research.web3.foundation/en/latest/polkadot/BABE/Babe/#6-practical-results>
			sp_consensus_babe::BabeGenesisConfiguration {
				slot_duration: Babe::slot_duration(),
				epoch_length: EpochDuration::get(),
				c: BABE_GENESIS_EPOCH_CONFIG.c,
				genesis_authorities: Babe::authorities(),
				randomness: Babe::randomness(),
				allowed_slots: BABE_GENESIS_EPOCH_CONFIG.allowed_slots,
			}
		}

		fn current_epoch_start() -> sp_consensus_babe::Slot {
			Babe::current_epoch_start()
		}

		fn current_epoch() -> sp_consensus_babe::Epoch {
			Babe::current_epoch()
		}

		fn next_epoch() -> sp_consensus_babe::Epoch {
			Babe::next_epoch()
		}

		fn generate_key_ownership_proof(
			_slot: sp_consensus_babe::Slot,
			authority_id: sp_consensus_babe::AuthorityId,
		) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
			use codec::Encode;

			Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
			key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;

			Babe::submit_unsigned_equivocation_report(
				equivocation_proof,
				key_owner_proof,
			)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl cml_runtime_api::CmlApi<Block, AccountId> for Runtime {
		fn user_cml_list(who: &AccountId) -> Vec<u64> {
			Cml::user_cml_list(who)
		}

		fn user_staking_list(who: &AccountId) -> Vec<(u64, u64)> {
			Cml::user_staking_list(who)
		}

		fn current_mining_cml_list() -> Vec<(u64, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, u32)> {
			Cml::current_mining_cml_list()
		}

		fn staking_price_table() -> Vec<Balance> {
			STAKING_PRICE_TABLE.to_vec()
		}

		fn estimate_stop_mining_penalty(cml_id: u64) -> Balance {
			Cml::estimate_stop_mining_penalty(cml_id)
		}
	}

	impl auction_runtime_api::AuctionApi<Block, AccountId> for Runtime {
		fn user_auction_list(who: &AccountId) -> Vec<u64> {
			Auction::user_auction_list(who)
		}

		fn user_bid_list(who: &AccountId) -> Vec<u64> {
			Auction::user_bid_list(who)
		}

		fn current_auction_list() -> Vec<u64> {
			Auction::current_auction_list()
		}

		fn estimate_minimum_bid_price(auction_id: u64, who: &AccountId) -> (Balance, Option<Balance>, bool) {
			Auction::estimate_minimum_bid_price(auction_id, who)
		}

		fn penalty_amount(auction_id: u64, who: &AccountId) -> Balance {
			Auction::penalty_amount(auction_id, who)
		}
	}

	impl tea_runtime_api::TeaApi<Block, AccountId> for Runtime {
		fn is_ra_validator(
			tea_id: &[u8; 32],
			target_tea_id: &[u8; 32],
			block_number: BlockNumber,
		) -> bool {
			Tea::is_ra_validator(tea_id, target_tea_id, &block_number)
		}

		fn boot_nodes() -> Vec<[u8; 32]> {
			Tea::list_boot_nodes()
		}

		fn allowed_pcrs() -> Vec<(H256, Vec<Vec<u8>>)> {
			Tea::list_allowed_pcrs()
		}

		fn allowed_versions() -> Vec<(H256, Vec<(Vec<u8>, Vec<u8>)>, Option<BlockNumber> )> {
			Tea::list_allowed_versions()
		}

		fn find_tea_id_by_peer_id(peer_id: Vec<u8>) -> Vec<[u8; 32]> {
			Tea::find_tea_id_by_peer_id(&peer_id)
		}

		fn version_expired_nodes() -> Vec<[u8; 32]> {
			Tea::list_version_expired_nodes()
		}
	}

	impl genesis_bank_runtime_api::GenesisBankApi<Block, AccountId> for Runtime {
		fn cml_calculate_loan_amount(cml_id: u64) -> (Balance, Balance, Balance) {
			GenesisBank::cml_calculate_loan_amount(cml_id)
		}

		fn user_collateral_list(who: &AccountId) -> Vec<(u64, BlockNumber)> {
			GenesisBank::user_collateral_list(who)
		}
	}

	impl genesis_exchange_runtime_api::GenesisExchangeApi<Block, AccountId> for Runtime {
		/// Returns
		/// 1. current 1TEA equals how many USD amount
		/// 2. current 1USD equals how many TEA amount
		/// 3. exchange remains USD
		/// 4. exchange remains TEA
		/// 5. product of  exchange remains USD and exchange remains TEA
		fn current_exchange_rate() -> (
			Balance,
			Balance,
			Balance,
			Balance,
			Balance,
		) {
			GenesisExchange::current_exchange_rate()
		}

		fn estimate_amount(withdraw_amount: Balance, buy_tea: bool) -> Balance {
			GenesisExchange::estimate_amount(withdraw_amount, buy_tea)
		}

		fn user_asset_list() -> Vec<(AccountId, Balance, Balance, Balance, Balance, Balance, Balance, Balance)> {
			GenesisExchange::user_asset_list()
		}

		fn user_borrowing_usd_margin(who: &AccountId) -> Balance {
			GenesisExchange::user_borrowing_usd_margin(who)
		}
	}

	impl bonding_curve_runtime_api::BondingCurveApi<Block, AccountId> for Runtime {
		fn query_price(tapp_id: u64) -> (Balance, Balance) {
			BondingCurve::query_price(tapp_id)
		}

		fn estimate_required_tea_when_buy(tapp_id: Option<u64>, token_amount: Balance, buy_curve_k: Option<u32>) -> Balance {
			BondingCurve::estimate_required_tea_when_buy(tapp_id, token_amount, buy_curve_k)
		}

		fn estimate_receive_tea_when_sell(tapp_id: u64, token_amount: Balance) -> Balance {
			BondingCurve::estimate_receive_tea_when_sell(tapp_id, token_amount)
		}

		fn estimate_receive_token_when_buy(tapp_id: u64, tea_amount: Balance) -> Balance {
			BondingCurve::estimate_receive_token_when_buy(tapp_id, tea_amount)
		}

		fn estimate_required_token_when_sell(tapp_id: u64, tea_amount: Balance) -> Balance {
			BondingCurve::estimate_required_token_when_sell(tapp_id, tea_amount)
		}

		fn list_tapps(active_only: bool) -> Vec<(
			Vec<u8>,
			u64,
			Vec<u8>,
			Balance,
			Balance,
			Balance,
			AccountId,
			Vec<u8>,
			Vec<u8>,
			u32,
			(u32, u32),
			Option<BlockNumber>,
		)> {
			BondingCurve::list_tapps(active_only)
		}

		fn list_user_assets(who: AccountId) -> Vec<(
			Vec<u8>,
			u64,
			Vec<u8>,
			(Balance, Balance),
			Balance,
			AccountId,
			Vec<u8>,
			Vec<u8>,
			u32,
			u32,
			u32,
			Balance,
		)> {
			BondingCurve::list_user_assets(&who)
		}

		fn tapp_details(tapp_id: u64) -> (
			Vec<u8>,
			u64,
			Vec<u8>,
			AccountId,
			Vec<u8>,
			Vec<u8>,
			u32,
			u32,
			u32,
			Balance,
			Balance,
			Balance,
		) {
			BondingCurve::tapp_details(tapp_id)
		}

		fn list_candidate_miners(who: AccountId) -> Vec<(
			u64,
			u32,
			u32,
			BlockNumber,
			Vec<u64>)> {
			BondingCurve::list_candidate_miners(&who)
		}

		fn tapp_hosted_cmls(tapp_id: u64) -> Vec<(
			u64,
			Option<AccountId>,
			BlockNumber,
			Option<u32>,
			Option<u32>,
			u32)> {
			BondingCurve::tapp_hosted_cmls(tapp_id)
		}

		fn list_cml_hosting_tapps(cml_id: u64) -> Vec<(
			u64,
			Option<u32>,
			u64,
			Vec<u8>,
			Vec<u8>,
			Vec<u8>,
			Vec<u8>,
			u32,
			Balance)> {
			BondingCurve::list_cml_hosting_tapps(cml_id)
		}

		fn cml_performance(cml_id: u64) -> (Option<u32>, Option<u32>, u32) {
			BondingCurve::cml_performance(cml_id)
		}

		fn approved_links() -> Vec<(Vec<u8>, Option<u64>, Vec<u8>, Option<AccountId>)> {
			BondingCurve::approved_links()
		}

		fn user_notification_count(account: AccountId) -> u32 {
			BondingCurve::user_notification_count(account)
		}
	}

	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn current_set_id() -> fg_primitives::SetId {
			Grandpa::current_set_id()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;

			Grandpa::submit_unsigned_equivocation_report(
				equivocation_proof,
				key_owner_proof,
			)
		}

		fn generate_key_ownership_proof(
			_set_id: fg_primitives::SetId,
			authority_id: GrandpaId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			use codec::Encode;

			Historical::prove((fg_primitives::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(fg_primitives::OpaqueKeyOwnershipProof::new)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {
			System::account_nonce(account)
		}
	}

	impl sp_authority_discovery::AuthorityDiscoveryApi<Block> for Runtime {
		fn authorities() -> Vec<AuthorityDiscoveryId> {
			AuthorityDiscovery::authorities()
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade() -> Result<(Weight, Weight), sp_runtime::RuntimeString> {
			let weight = Executive::try_runtime_upgrade()?;
			Ok((weight, RuntimeBlockWeights::get().max_block))
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};

			use frame_system_benchmarking::Pallet as SystemBench;
			use pallet_session_benchmarking::Pallet as SessionBench;
			use pallet_offences_benchmarking::Pallet as OffencesBench;

			impl frame_system_benchmarking::Config for Runtime {}
			impl pallet_session_benchmarking::Config for Runtime {}
			impl pallet_offences_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmark!(params, batches, frame_system, SystemBench::<Runtime>);
			add_benchmark!(params, batches, pallet_grandpa, Grandpa);
			add_benchmark!(params, batches, pallet_timestamp, Timestamp);
			add_benchmark!(params, batches, pallet_balances, Balances);
			add_benchmark!(params, batches, pallet_babe, Babe);
			add_benchmark!(params, batches, pallet_session, SessionBench::<Runtime>);
			add_benchmark!(params, batches, pallet_staking, Staking);
			add_benchmark!(params, batches, pallet_offences, OffencesBench::<Runtime>);
			add_benchmark!(params, batches, pallet_im_online, ImOnline);
			add_benchmark!(params, batches, pallet_elections_phragmen, Elections);
			add_benchmark!(params, batches, pallet_election_provider_multi_phase, ElectionProviderMultiPhase);
			add_benchmark!(params, batches, pallet_collective, Council);
			add_benchmark!(params, batches, pallet_membership, TechnicalMembership);
			add_benchmark!(params, batches, pallet_scheduler, Scheduler);
			add_benchmark!(params, batches, pallet_democracy, Democracy);
			add_benchmark!(params, batches, pallet_utility, Utility);
			add_benchmark!(params, batches, pallet_multisig, Multisig);
			add_benchmark!(params, batches, pallet_identity, Identity);

			add_benchmark!(params, batches, pallet_tea, Tea);
			add_benchmark!(params, batches, pallet_cml, Cml);
			add_benchmark!(params, batches, pallet_auction, Auction);

			if batches.is_empty() { return Err("Benchmark not found for this pallet.".into()) }
			Ok(batches)
		}
	}
}
