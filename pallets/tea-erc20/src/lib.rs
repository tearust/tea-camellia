#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use bonding_curve::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod functions;

use codec::Encode;
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement},
};
use frame_system::pallet_prelude::*;
use pallet_utils::{extrinsic_procedure, CurrencyOperations};
use sp_core::H256;
use sp_io::hashing::blake2_256;
use sp_std::prelude::*;

/// The balance type of this module.
pub type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod bonding_curve {
	use super::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The lockable currency type
		type Currency: Currency<Self::AccountId>;
		/// Currency operations trait defined in utils trait.
		type CurrencyOperations: CurrencyOperations<
			AccountId = Self::AccountId,
			Balance = BalanceOf<Self>,
		>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn withdraw_storage)]
	pub(crate) type WithdrawStorage<T: Config> =
		StorageMap<_, Twox64Concat, H256, T::BlockNumber, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn consume_storage)]
	pub(crate) type ConsumeStorage<T: Config> =
		StorageMap<_, Twox64Concat, H256, T::BlockNumber, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub reserved_balance_account: T::AccountId,
		pub npc_account: T::AccountId,
		pub user_create_tapp: bool,
	}
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				reserved_balance_account: Default::default(),
				npc_account: Default::default(),
				user_create_tapp: false,
			}
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub fn deposit_event)]
	pub enum Event<T: Config> {
		/// Fired after topuped successfully, event parameters:
		/// 2. From account
		/// 3. To account
		/// 4. Topup amount
		/// 5. Curent block number
		TAppTopup(T::AccountId, T::AccountId, BalanceOf<T>, T::BlockNumber),

		/// Fired after topuped successfully, event parameters:
		/// 2. From account
		/// 3. To account
		/// 4. Topup amount
		/// 5. Curent block number
		/// 6. Tsid
		TAppWithdraw(
			T::AccountId,
			T::AccountId,
			BalanceOf<T>,
			T::BlockNumber,
			Vec<u8>,
		),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The length of the tapp name is over than required
		TAppNameIsTooLong,
		/// The length of the tapp ticker is over than required
		TAppTickerIsTooLong,
		/// The length of the tapp ticker is less than required
		TAppTickerIsTooShort,
		/// The length of the tapp detail is over than required
		TAppDetailIsTooLong,
		/// The length of the tapp link is over than required
		TAppLinkIsTooLong,
		/// Tapp name already exists
		TAppNameAlreadyExist,
		/// Tapp ticker already exists
		TAppTickerAlreadyExist,
		/// TEA free balance is not enough
		InsufficientFreeBalance,
		/// Tapp token is not enough
		InsufficientTAppToken,
		/// Sell amount more than total supply
		InsufficientTotalSupply,
		/// The given tapp id not exists in the tapp store
		TAppIdNotExist,
		/// Sell amount more than total reserved pool tea token
		TAppInsufficientFreeBalance,
		/// All operation amount should greater than 0
		OperationAmountCanNotBeZero,
		/// Buy tea amount should greater than 0
		BuyTeaAmountCanNotBeZero,
		/// Sell tea amount should greater than 0
		SellTeaAmountCanNotBeZero,
		/// Subtraction operation has overflowed
		SubtractionOverflow,
		/// Add operation has overflowed
		AddOverflow,
		/// It is forbidden for normal user to create tapp
		NotAllowedNormalUserCreateTApp,
		/// Only the tapp owner is allowed to submit the `expense` extrinsic
		OnlyTAppOwnerAllowedToExpense,
		/// Performance value should greater than 0
		PerformanceValueShouldNotBeZero,
		/// If specify the maximum allowed host count, it should be greater than 0
		MaxAllowedHostShouldLargerEqualThanMinAllowedHosts,
		/// The tapp already has desired count of hosts that can not be hosted anymore
		TAppHostsIsFull,
		/// The CML machine is full loan that can not host anymore
		CmlMachineIsFullLoad,
		/// The CML not hosting the given tapp
		CmlNotHostTheTApp,
		/// Cml owner is none
		CmlOwnerIsNone,
		/// It's not allowed for the CML that not start mining to host
		OnlyMiningCmlCanHost,
		/// It's not allowed for the CML that already dead to host
		DeadedCmlCanNotHost,
		/// Cml machine id not exist
		CmlMachineIdIsNone,
		/// The CML is already hosting the given tapp
		CmlIsAlreadyHosting,
		/// There is no miner hosting the tapp so no need to distribute
		NoHostingToDistributeMiner,
		/// Consume note should not over the max length limitation
		ConsumeNoteIsTooLong,
		/// Only the tapp owner is allowed to submit the `update_tapp_resource` extrinsic
		OnlyTAppOwnerAllowedToUpdateResource,
		/// The length of the cid parameter is longer than required
		CidIsToLong,
		/// Total supply will over the max value if buy given amount of tapp token
		TotalSupplyOverTheMaxValue,
		/// Reward per performance should not be zero
		RewardPerPerformanceShouldNotBeZero,
		/// Stake token should not be zero
		StakeTokenShouldNotBeZero,
		/// Stake token should not be none in fixed token mode
		StakeTokenIsNoneInFixedTokenMode,
		/// Reward per performance should not be none in fixed fee mode
		RewardPerPerformanceIsNoneInFixedFeeMode,
		/// Should unlock after host locking block height
		HostLockingBlockHeightNotReached,
		/// Only NPC account allowed to register link url
		OnlyNPCAccountAllowedToRegisterLinkUrl,
		/// Link url already registered
		LinkUrlAlreadyExist,
		/// Link url not registered
		LinkUrlNotExist,
		/// Link description is too long
		LinkDescriptionIsTooLong,
		/// Link already used by other tapp
		LinkAlreadyBeUsed,
		/// Only NPC account allowed to update activity
		OnlyNPCAccountAllowedToUpdateActivity,
		/// Stake token amount and reward per performance should not both exist
		StakeTokenAmountAndRewardPerPerformanceCannotBothExist,
		/// Link that created by a user can only be used to created by the user himself
		UserReservedLink,
		/// Theta of buy bonding curve should be not be zero
		BuyCurveThetaCanNotBeZero,
		/// Theta of sell bonding curve should be not be zero
		SellCurveThetaCanNotBeZero,
		/// Theta of buy bonding curve should larger equal than sell's
		BuyCurveThetaShouldLargerEqualThanSellCurveTheta,
		/// Can host tapp only when cml is active
		MiningCmlStatusShouldBeActive,
		/// Not allowed type C cml to host tapp
		NotAllowedTypeCHostingTApp,
		/// Only notification account allowed to push notification
		NotAllowedPushNotification,
		/// Only notification account allowed to clear notification
		NotAllowedClearNotification,
		/// Notification list should at least have one message
		NotificationListIsEmpty,
		/// Notification list and account list should be matched
		NotificationAndAccountListNotMatched,
		/// Not found the given notification user
		NotFoundNotificationUser,
		/// No user notification to read
		NoUserNotificationToRead,
		/// Withdraw tsid already exist
		WithdrawTsidAlreadyExist,
		/// Only NPC account can use batch transfer
		OnlyNpcCanBatchTransfer,
		/// NPC has not enough free balance to do batch transfer
		BatchTransferInsufficientBalance,
		/// Only NPC allowed to mint
		OnlyNpcCanMint,
		/// Notification tsid already exist
		NotificationTsidAlreadyExist,
		/// Clear notification should larger than last clear notification height
		InvalidClearNotificationHeight,
		/// Notification account has not been initialized yet
		NotificationAccountNotInit,
		/// Consume tsid already exist
		ConsumeTsidAlreadyExist,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(_n: BlockNumberFor<T>) {}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// This is basically a normal transfer balance extrinsic, except emit a topup event
		#[pallet::weight(195_000_000)]
		pub fn topup(
			sender: OriginFor<T>,
			tapp_operation_account: T::AccountId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						T::CurrencyOperations::free_balance(who) >= amount,
						Error::<T>::InsufficientFreeBalance
					);
					Ok(())
				},
				|who| {
					if let Err(e) = T::CurrencyOperations::transfer(
						who,
						&tapp_operation_account,
						amount,
						ExistenceRequirement::AllowDeath,
					) {
						// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
						log::error!("tapp topup transfer free balance failed: {:?}", e);
						return;
					}

					let current_height = frame_system::Pallet::<T>::block_number();
					Self::deposit_event(Event::TAppTopup(
						who.clone(),
						tapp_operation_account.clone(),
						amount,
						current_height,
					));
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn withdraw(
			sender: OriginFor<T>,
			to_account: T::AccountId,
			amount: BalanceOf<T>,
			tsid: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			let withdraw_hash = Self::tsid_hash(&tsid);
			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						!WithdrawStorage::<T>::contains_key(&withdraw_hash),
						Error::<T>::WithdrawTsidAlreadyExist
					);
					ensure!(
						T::CurrencyOperations::free_balance(who) >= amount,
						Error::<T>::InsufficientFreeBalance
					);
					Ok(())
				},
				|who| {
					if let Err(e) = T::CurrencyOperations::transfer(
						who,
						&to_account,
						amount,
						ExistenceRequirement::AllowDeath,
					) {
						// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
						log::error!("tapp withdraw transfer free balance failed: {:?}", e);
						return;
					}

					let current_block = frame_system::Pallet::<T>::block_number();
					WithdrawStorage::<T>::insert(&withdraw_hash, current_block);

					Self::deposit_event(Event::TAppWithdraw(
						who.clone(),
						to_account,
						amount,
						current_block,
						tsid,
					));
				},
			)
		}
	}
}
