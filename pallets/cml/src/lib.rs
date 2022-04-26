#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

mod functions;
pub mod generator;
mod rpc;
mod types;

pub use cml::*;
pub use param::*;
pub use types::*;

use frame_support::{pallet_prelude::*, traits::Currency};
use frame_system::pallet_prelude::*;
use pallet_utils::{CommonUtils, CurrencyOperations};
use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_std::prelude::*;

/// The balance type of this module.
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod cml {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type Currency: Currency<Self::AccountId>;

		/// Common utils trait
		type CommonUtils: CommonUtils<AccountId = Self::AccountId>;

		/// Operations about currency that used in Tea Camellia.
		type CurrencyOperations: CurrencyOperations<
			AccountId = Self::AccountId,
			Balance = BalanceOf<Self>,
		>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Used to allocate CML ID of new created DAO CML.
	#[pallet::storage]
	#[pallet::getter(fn last_cml_id)]
	pub type LastCmlId<T: Config> = StorageValue<_, CmlId, ValueQuery>;

	/// Storage of all valid CMLs, invalid CMLs (dead CML or fresh seed that over the fresh duration) will be
	/// cleaned every staking period begins.
	#[pallet::storage]
	#[pallet::getter(fn cml_store)]
	pub type CmlStore<T: Config> =
		StorageMap<_, Twox64Concat, CmlId, CML<T::AccountId, T::BlockNumber>, ValueQuery>;

	/// Double map about user and related cml ID of him.
	#[pallet::storage]
	#[pallet::getter(fn user_cml_store)]
	pub type UserCmlStore<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::AccountId, Twox64Concat, CmlId, ()>;

	#[pallet::storage]
	#[pallet::getter(fn npc_account)]
	pub type NPCAccount<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub genesis_seeds: GenesisSeeds,
		pub phantom: PhantomData<T>,
	}
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				genesis_seeds: GenesisSeeds::default(),
				phantom: PhantomData,
			}
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			crate::functions::init_from_genesis_seeds::<T>(&self.genesis_seeds);
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {}

	#[pallet::error]
	pub enum Error<T> {
		/// User have no coupons to draw cmls.
		WithoutCoupon,
		/// User have not enough coupons to transfer.
		NotEnoughCoupon,
		/// User transfer the coupons amount is over than he really has.
		InvalidCouponAmount,
		/// It's forbidden to transfer coupon.
		ForbiddenTransferCoupon,

		/// There is not enough draw seeds left in luck draw box (this error indicate the genesis draw seeds count
		/// and coupons amount not matched).
		NotEnoughDrawSeeds,
		/// Seeds is not outdated that can't be clean out with `clean_outdated_seeds` extrinsic.
		SeedsNotOutdatedYet,
		/// Coupons has outdated that can't transfer or be used to transfer to others.
		CouponsHasOutdated,
		/// Lucky draw box is already empty so there is no need to clean out.
		NoNeedToCleanOutdatedSeeds,

		/// Could not find CML in the cml store, indicates that the specified CML not existed.
		NotFoundCML,
		/// Trying to operate a CML not belongs to the user.
		CMLOwnerInvalid,
		/// Cml is not seed so there is no need to active (to tree) or do something else.
		CmlIsNotSeed,

		/// User account free balance is not enoungh.
		InsufficientFreeBalance,
		/// The specified machine ID is already mining, that should not be used to start mining again.
		MinerAlreadyExist,
		/// The specified machine ID is not found in the machine store, indicates that the specified machin
		/// ID not existed.
		NotFoundMiner,
		/// Specified CML in not valid to operate as a mining tree.
		InvalidMiner,
		/// Sepcified miner IP is not a valid format of IPv4.
		InvalidMinerIp,
		/// Mining tree is offline already, no need to suspend
		NoNeedToSuspend,
		/// Mining tree is already active, no need to resume
		NoNeedToResume,
		/// Insufficient free balance to append pledge
		InsufficientFreeBalanceToAppendPledge,
		/// The given IP address is already registerd
		MinerIpAlreadyExist,
		/// Type B cml start mining should have orbit id
		CmlBStartMiningShouldHaveOrbitId,
		/// Can not schedule down when cml is not active state
		CanNotScheduleDownWhenInactive,
		/// Mining tree is not schedule down, no need to schedule up
		NoNeedToScheduleUp,
		/// Can not migrate when cml is active, should schedule down first
		CannotMigrateWhenActive,

		/// Specified staking index is over than the max length of current staking slots.
		InvalidStakingIndex,
		/// The first staking slot cannot be unstake, if do want to unstake it please stop mining instead.
		CannotUnstakeTheFirstSlot,
		/// User is not the owner of specified CML.
		InvalidStakingOwner,
		/// User has no reward of staking that can't to withdraw the reward.
		NotFoundRewardAccount,
		/// Specified CML has been staked by too much users that can't be append staking anymore.
		StakingSlotsOverTheMaxLength,
		/// User specfied max acceptable slot length and current staking index has over than that.
		StakingSlotsOverAcceptableIndex,
		/// It is forbidden to stake a cml that is in aution.
		CannotStakeWhenCmlIsInAuction,
		/// Not allowed type C cml to be staked.
		NotAllowedCToBeStaked,

		/// Defrost time should have value when defrost.
		CmlDefrostTimeIsNone,
		/// Cml should be frozen seed.
		CmlShouldBeFrozenSeed,
		/// Cml is still in frozen locked period that cannot be defrosted.
		CmlStillInFrozenLockedPeriod,
		/// Cml should be fresh seed.
		CmlShouldBeFreshSeed,
		/// Cml in fresh seed state and have expired the fresh duration.
		CmlFreshSeedExpired,
		/// Cml is tree means that can't be frozen seed or fresh seed.
		CmlShouldBeTree,
		/// Cml has over the lifespan
		CmlShouldDead,
		/// Cml is mining that can start mining again.
		CmlIsMiningAlready,
		/// Cml is staking that can't staking again or start mining.
		CmlIsStaking,
		/// Before start mining staking slot should be empty.
		CmlStakingSlotNotEmpty,
		/// Means we cannot decide staking type from given params.
		ConfusedStakingType,
		/// Cml is not mining that can't stake to.
		CmlIsNotMining,
		/// Cml is not staking to current miner that can't unstake.
		CmlIsNotStakingToCurrentMiner,
		/// Cml staking index over than staking slot length, that means point to not exist staking.
		CmlStakingIndexOverflow,
		/// Cml staking item owner is none, that can't identify staking belongs.
		CmlOwnerIsNone,
		/// Cml staking item owner and the owner field of cml item not match.
		CmlOwnerMismatch,
		/// Cml is not staking that can't unstake.
		CmlIsNotStaking,
		/// Some status that can't convert to another status.
		CmlInvalidStatusConversion,
		/// Not enough free balance to pay for each of the staking accounts.
		InsufficientFreeBalanceToPayForPunishment,
		/// Can not stop mining when hosting tapp
		CannotStopMiningWhenHostingTApp,
		/// C type cmls are not allowed to stake
		CTypeCmlCanNotStake,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(_n: BlockNumberFor<T>) {}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {}
}
