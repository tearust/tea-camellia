#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>
pub use bonding_curve::*;
pub use types::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

mod functions;
mod migrations;
mod rpc;
mod types;

use bonding_curve_impl::square_root::UnsignedSquareRoot;
use bonding_curve_interface::{BondingCurveInterface, BondingCurveOperation};
use codec::{Decode, Encode};
use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement},
};
use frame_system::pallet_prelude::*;
use pallet_cml::{
	CmlId, CmlOperation, CmlType, MachineId, MinerStatus, MiningProperties, Performance,
	SeedProperties, TreeProperties,
};
use pallet_utils::{extrinsic_procedure, CurrencyOperations};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, CheckedAdd, CheckedSub, Saturating, Zero},
	RuntimeDebug,
};
use sp_std::{collections::btree_set::BTreeSet, prelude::*};

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

		type CmlOperation: CmlOperation<
			AccountId = Self::AccountId,
			Balance = BalanceOf<Self>,
			BlockNumber = Self::BlockNumber,
		>;

		#[pallet::constant]
		type TAppNameMaxLength: Get<u32>;

		#[pallet::constant]
		type TAppTickerMaxLength: Get<u32>;

		#[pallet::constant]
		type TAppTickerMinLength: Get<u32>;

		#[pallet::constant]
		type TAppDetailMaxLength: Get<u32>;

		#[pallet::constant]
		type TAppLinkMaxLength: Get<u32>;

		#[pallet::constant]
		type PoolBalanceReversePrecision: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type ConsumeNoteMaxLength: Get<u32>;

		/// duration to arrange (mainly reduce hosting TApps according performance) cml
		#[pallet::constant]
		type HostArrangeDuration: Get<Self::BlockNumber>;

		#[pallet::constant]
		type HostCostCollectionDuration: Get<Self::BlockNumber>;

		#[pallet::constant]
		type CidMaxLength: Get<u32>;

		#[pallet::constant]
		type TotalSupplyMaxValue: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type MinTappHostsCount: Get<u32>;

		#[pallet::constant]
		type HostLockingBlockHeight: Get<Self::BlockNumber>;

		#[pallet::constant]
		type TAppLinkDescriptionMaxLength: Get<u32>;

		#[pallet::constant]
		type DefaultBuyCurveTheta: Get<u32>;

		#[pallet::constant]
		type DefaultSellCurveTheta: Get<u32>;

		#[pallet::constant]
		type HostPledgeAmount: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type ReservedLinkRentAmount: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type NotificationsArrangeDuration: Get<Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn account_table)]
	pub type AccountTable<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		T::AccountId,
		Twox64Concat,
		TAppId,
		BalanceOf<T>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn total_supply_table)]
	pub type TotalSupplyTable<T: Config> =
		StorageMap<_, Twox64Concat, TAppId, BalanceOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tapp_bonding_curve)]
	pub type TAppBondingCurve<T: Config> = StorageMap<
		_,
		Twox64Concat,
		TAppId,
		TAppItem<T::AccountId, BalanceOf<T>, T::BlockNumber>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn tapp_names)]
	pub type TAppNames<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, TAppId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tapp_tickers)]
	pub type TAppTickers<T: Config> = StorageMap<_, Twox64Concat, Vec<u8>, TAppId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn last_cml_id)]
	pub type LastTAppId<T: Config> = StorageValue<_, TAppId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn operation_account)]
	pub type ReservedBalanceAccount<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn enable_user_create_tapp)]
	pub type EnableUserCreateTApp<T: Config> = StorageValue<_, bool, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn npc_account)]
	pub type NPCAccount<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn cml_hosting_tapps)]
	pub type TAppCurrentHosts<T: Config> =
		StorageDoubleMap<_, Twox64Concat, TAppId, Twox64Concat, CmlId, T::BlockNumber, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tapp_current_hosted_cmls)]
	pub type CmlHostingTApps<T: Config> =
		StorageMap<_, Twox64Concat, CmlId, Vec<TAppId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tapp_resource_map)]
	pub type TAppResourceMap<T: Config> = StorageMap<_, Twox64Concat, TAppId, Vec<u8>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tapp_approved_links)]
	pub type TAppApprovedLinks<T: Config> =
		StorageMap<_, Twox64Concat, Vec<u8>, ApprovedLinkInfo<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tapp_last_activity)]
	pub type TAppLastActivity<T: Config> =
		StorageMap<_, Twox64Concat, TAppId, (u64, T::BlockNumber)>;

	#[pallet::storage]
	#[pallet::getter(fn tapp_reserved_balance)]
	pub type TAppReservedBalance<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		TAppId,
		Twox64Concat,
		T::AccountId,
		Vec<(BalanceOf<T>, CmlId)>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn tapp_host_pledge)]
	pub type TAppHostPledge<T: Config> =
		StorageDoubleMap<_, Twox64Concat, TAppId, Twox64Concat, CmlId, BalanceOf<T>, ValueQuery>;

	/// Storage version of the pallet.
	///
	/// New networks start with latest version, as determined by the genesis build.
	#[pallet::storage]
	pub(crate) type StorageVersion<T: Config> = StorageValue<_, Releases, ValueQuery>;

	#[pallet::storage]
	pub(crate) type NotificationAccount<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

	#[pallet::storage]
	pub(crate) type UserNotifications<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Vec<T::BlockNumber>, ValueQuery>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub reserved_balance_account: T::AccountId,
		pub npc_account: T::AccountId,
		pub notification_account: T::AccountId,
		pub user_create_tapp: bool,
	}
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				reserved_balance_account: Default::default(),
				npc_account: Default::default(),
				notification_account: Default::default(),
				user_create_tapp: false,
			}
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			// Genesis uses the latest storage version.
			StorageVersion::<T>::put(Releases::V1);

			ReservedBalanceAccount::<T>::set(self.reserved_balance_account.clone());
			NPCAccount::<T>::set(self.npc_account.clone());
			NotificationAccount::<T>::set(self.notification_account.clone());

			EnableUserCreateTApp::<T>::set(self.user_create_tapp);
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub fn deposit_event)]
	pub enum Event<T: Config> {
		/// Fired after Tapp created successfully, event parameters:
		/// 1. TApp Id
		/// 2. TApp name array encoded with UTF8
		/// 3. TApp owner Account Id
		TAppCreated(TAppId, Vec<u8>, T::AccountId),

		/// Fired after TApp token bought successfully, event parameters:
		/// 1. TApp Id
		/// 2. Bought Account Id
		/// 3. Bought TEA amount
		/// 4. Token amount
		/// 5. Buy price
		/// 6. Sell price
		/// 7. Total supply
		TokenBought(
			TAppId,
			T::AccountId,
			BalanceOf<T>,
			BalanceOf<T>,
			BalanceOf<T>,
			BalanceOf<T>,
			BalanceOf<T>,
		),

		/// Fired after TApp token sold successfully, event parameters:
		/// 1. TApp Id
		/// 2. Sold Account Id
		/// 3. Sold TEA amount
		/// 4. Token amount
		/// 5. Buy price
		/// 6. Sell price
		/// 7. Total supply
		TokenSold(
			TAppId,
			T::AccountId,
			BalanceOf<T>,
			BalanceOf<T>,
			BalanceOf<T>,
			BalanceOf<T>,
			BalanceOf<T>,
		),

		/// Fired after TApp consume successfully, event parameters:
		/// 1. TApp Id
		/// 2. Consume Account Id
		/// 3. Consumed TEA amount
		/// 4. Token amount
		/// 5. Consumed notes
		/// 6. Buy price
		/// 7. Sell price
		/// 8. Total supply
		TAppConsume(
			TAppId,
			T::AccountId,
			BalanceOf<T>,
			BalanceOf<T>,
			Option<Vec<u8>>,
			BalanceOf<T>,
			BalanceOf<T>,
			BalanceOf<T>,
		),

		/// Fired after TApp consume successfully, event parameters:
		/// 1. TApp Id
		/// 2. Consume statements, each item including three items:
		///		a. Account Id
		///		b. Token reward balance
		///		c. Is from investor, if false means rewards is from miner hosting token reward
		///		d. If from miner hosting token reward return the mining CmlId, otherwise return none
		TAppConsumeRewardStatements(
			TAppId,
			Vec<(T::AccountId, BalanceOf<T>, bool, Option<CmlId>)>,
		),

		/// Fired after TApp expensed successfully, event parameters:
		/// 1. TApp Id
		/// 2. Statements of miner rewards
		/// 3. Buy price
		/// 4. Sell price
		/// 5. Total supply
		/// 6. Is fix token mode
		TAppExpense(
			TAppId,
			Vec<(T::AccountId, CmlId, BalanceOf<T>)>,
			BalanceOf<T>,
			BalanceOf<T>,
			BalanceOf<T>,
			bool,
		),

		/// Fired after TApp has bankrupted
		TAppBankrupted(TAppId),

		/// Fired after host successfully, automatically unhosted lists:
		/// - Host tapp id
		/// - Host CML id
		/// - Host CML machine id
		/// - TApp became active
		TAppsHosted(TAppId, CmlId, MachineId, bool),

		/// Fired after host successfully, automatically unhosted lists:
		/// - Unhost tapp id
		/// - Unhost CML id
		/// - TApp became pending
		/// - Unreserved balance
		TAppsUnhosted(TAppId, CmlId, bool, BalanceOf<T>),

		/// Fired after each host arrange duration, automatically unhosted lists:
		/// - Unhost tapp id
		/// - Unhost CML id
		TAppsAutoUnhosted(Vec<(TAppId, CmlId)>),

		/// Fired after topuped successfully, event parameters:
		/// 1. TApp Id
		/// 2. From account
		/// 3. To account
		/// 4. Topup amount
		TAppTopup(
			TAppId,
			T::AccountId,
			T::AccountId,
			BalanceOf<T>,
			T::BlockNumber,
		),

		/// Fired after tapp actived, event parameters:
		/// 1. TApp Id
		/// 2. Block height
		/// 3. Host count
		TAppBecomeActived(TAppId, T::BlockNumber, u32),

		/// Fired after tapp actived, event parameters:
		/// 1. TApp Id
		/// 2. Block height
		/// 3. Host count
		TAppBecomePending(TAppId, T::BlockNumber, u32),
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
		/// It is forbidden for NPC to create tapp
		NotAllowedNPCCreateTApp,
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
		/// Link description is too long
		LinkDescriptionIsTooLong,
		/// Only registered link are allowed to create tapp
		LinkNotInApprovedList,
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
		/// Notification list should at least have one message
		NotificationListIsEmpty,
		/// Notification list and account list should be matched
		NotificationAndAccountListNotMatched,
		/// Not found the given notification user
		NotFoundNotificationUser,
		/// No user notification to read
		NoUserNotificationToRead,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_finalize(n: BlockNumberFor<T>) {
			if Self::need_arrange_host(n) {
				Self::arrange_host();
			}
			if Self::need_collect_host_cost(n) {
				Self::collect_host_cost();
			}
			if Self::need_arrange_notifications(n) {
				Self::arrange_notificatioins();
			}
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<(), &'static str> {
			migrations::v1::pre_migrate::<T>()
		}

		fn on_runtime_upgrade() -> Weight {
			if StorageVersion::<T>::get() == Releases::V0 {
				StorageVersion::<T>::put(Releases::V1);
				migrations::v1::migrate::<T>().saturating_add(T::DbWeight::get().reads_writes(1, 1))
			} else {
				T::DbWeight::get().reads(1)
			}
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade() -> Result<(), &'static str> {
			migrations::v1::post_migrate::<T>()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(195_000_000)]
		pub fn tapp_creation_settings(
			sender: OriginFor<T>,
			enable_create: Option<bool>,
			npc_account: Option<T::AccountId>,
		) -> DispatchResult {
			let who = ensure_root(sender)?;

			extrinsic_procedure(
				&who,
				|_who| Ok(()),
				|_who| {
					if let Some(enable_create) = enable_create {
						EnableUserCreateTApp::<T>::set(enable_create);
					}

					if let Some(ref npc_account) = npc_account {
						NPCAccount::<T>::set(npc_account.clone());
					}
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn update_tapp_last_activity(
			sender: OriginFor<T>,
			tapp_id: TAppId,
			activity_data: u64,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						who.eq(&NPCAccount::<T>::get()),
						Error::<T>::OnlyNPCAccountAllowedToUpdateActivity
					);
					ensure!(
						TAppBondingCurve::<T>::contains_key(tapp_id),
						Error::<T>::TAppIdNotExist
					);
					Ok(())
				},
				|_who| {
					let current_block = frame_system::Pallet::<T>::block_number();
					TAppLastActivity::<T>::insert(tapp_id, (activity_data, current_block));
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn register_tapp_link(
			sender: OriginFor<T>,
			link_url: Vec<u8>,
			link_description: Vec<u8>,
			creator: Option<T::AccountId>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						who.eq(&NPCAccount::<T>::get()),
						Error::<T>::OnlyNPCAccountAllowedToRegisterLinkUrl
					);
					ensure!(
						link_url.len() <= T::TAppLinkMaxLength::get() as usize,
						Error::<T>::TAppLinkIsTooLong
					);
					ensure!(
						link_description.len() <= T::TAppLinkDescriptionMaxLength::get() as usize,
						Error::<T>::LinkDescriptionIsTooLong
					);
					ensure!(
						!TAppApprovedLinks::<T>::contains_key(&link_url),
						Error::<T>::LinkUrlAlreadyExist
					);
					Ok(())
				},
				|_who| {
					TAppApprovedLinks::<T>::insert(
						&link_url,
						ApprovedLinkInfo {
							tapp_id: None,
							description: link_description.clone(),
							creator: creator.clone(),
						},
					);
				},
			)
		}

		/// Create a new tapp
		///
		/// - `tapp_name`
		/// - `ticker`
		/// - `init_fund`
		/// - `detail`
		/// - `link`
		/// - `max_allowed_hosts`
		/// - `tapp_type`
		/// - `fixed_token_mode`: is "fixed token mode", false will be "fixed fee mode"
		/// - `reward_per_1k_performance`: reward fee (in TEA) per 1000 performance requests, note this is
		/// 	only works in "fixed fee mode"
		/// - `stake_token_amount`: only works in "fixed token mode", specify reserved token amount of
		/// 	eath miner
		/// - `buy_curve_k`: represents parameter of "y = k√x" curve, and `buy_curve_k` is
		///		100 times of `k`. If `buy_curve_k` is none, will use the default value of
		/// 	`DefaultBuyCurveTheta`
		/// - `sell_curve_k`: represents parameter of "y = k√x" curve, and `sell_curve_k` is
		///		100 times of `k`. If `sell_curve_k` is none, will use the default value of
		/// 	`DefaultSellCurveTheta`
		#[pallet::weight(195_000_000)]
		pub fn create_new_tapp(
			sender: OriginFor<T>,
			tapp_name: Vec<u8>,
			ticker: Vec<u8>,
			init_fund: BalanceOf<T>,
			detail: Vec<u8>,
			link: Vec<u8>,
			max_allowed_hosts: u32,
			tapp_type: TAppType,
			fixed_token_mode: bool,
			reward_per_1k_performance: Option<BalanceOf<T>>,
			stake_token_amount: Option<BalanceOf<T>>,
			buy_curve_k: Option<u32>,
			sell_curve_k: Option<u32>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					if !EnableUserCreateTApp::<T>::get() {
						ensure!(
							who.eq(&NPCAccount::<T>::get()),
							Error::<T>::NotAllowedNormalUserCreateTApp
						);
					} else {
						ensure!(
							!who.eq(&NPCAccount::<T>::get()),
							Error::<T>::NotAllowedNPCCreateTApp
						);
					}

					Self::check_tapp_fields_length(&tapp_name, &ticker, &detail, &link)?;
					if let Some(buy_curve_k) = buy_curve_k {
						ensure!(
							!buy_curve_k.is_zero(),
							Error::<T>::BuyCurveThetaCanNotBeZero
						);
					}
					if let Some(sell_curve_k) = sell_curve_k {
						ensure!(
							!sell_curve_k.is_zero(),
							Error::<T>::SellCurveThetaCanNotBeZero
						);
					}
					ensure!(
						buy_curve_k.unwrap_or(T::DefaultBuyCurveTheta::get())
							>= sell_curve_k.unwrap_or(T::DefaultSellCurveTheta::get()),
						Error::<T>::BuyCurveThetaShouldLargerEqualThanSellCurveTheta
					);

					let link_related = tapp_type != TAppType::Bbs;
					if link_related {
						ensure!(
							!TAppNames::<T>::contains_key(&tapp_name),
							Error::<T>::TAppNameAlreadyExist
						);
					}
					ensure!(
						!TAppTickers::<T>::contains_key(&ticker),
						Error::<T>::TAppTickerAlreadyExist
					);
					ensure!(
						!init_fund.is_zero(),
						Error::<T>::OperationAmountCanNotBeZero
					);

					if link_related {
						ensure!(
							TAppApprovedLinks::<T>::contains_key(&link),
							Error::<T>::LinkNotInApprovedList
						);
					}

					let link_info = TAppApprovedLinks::<T>::get(&link);
					if link_related {
						ensure!(link_info.tapp_id.is_none(), Error::<T>::LinkAlreadyBeUsed);
						if let Some(ref creator) = link_info.creator {
							ensure!(creator.eq(who), Error::<T>::UserReservedLink);
						}
					}

					let mut deposit_tea_amount =
						Self::calculate_increase_amount_from_raise_curve_total_supply(
							buy_curve_k.unwrap_or(T::DefaultBuyCurveTheta::get()),
							0u32.into(),
							init_fund,
						)?;
					ensure!(
						!deposit_tea_amount.is_zero(),
						Error::<T>::BuyTeaAmountCanNotBeZero
					);
					if link_related && link_info.creator.is_none() {
						deposit_tea_amount += T::ReservedLinkRentAmount::get();
					}
					ensure!(
						T::CurrencyOperations::free_balance(who) >= deposit_tea_amount,
						Error::<T>::InsufficientFreeBalance,
					);
					Self::check_host_creating(
						max_allowed_hosts,
						fixed_token_mode,
						&reward_per_1k_performance,
						&stake_token_amount,
					)?;
					Ok(())
				},
				|who| {
					let link_related = tapp_type != TAppType::Bbs;
					let id = Self::next_id();
					if link_related {
						TAppNames::<T>::insert(&tapp_name, id);
						TAppApprovedLinks::<T>::mutate(&link, |link_info| {
							link_info.tapp_id = Some(id)
						});
					}
					TAppTickers::<T>::insert(&ticker, id);

					let buy_curve_k = buy_curve_k.unwrap_or(T::DefaultBuyCurveTheta::get());
					let sell_curve_k = sell_curve_k.unwrap_or(T::DefaultSellCurveTheta::get());
					let billing_mode = match fixed_token_mode {
						true => BillingMode::FixedHostingToken(
							stake_token_amount.unwrap_or(Zero::zero()),
						),
						false => BillingMode::FixedHostingFee(
							reward_per_1k_performance.unwrap_or(Zero::zero()),
						),
					};
					TAppBondingCurve::<T>::insert(
						id,
						TAppItem {
							id,
							name: tapp_name.clone(),
							ticker: ticker.clone(),
							owner: who.clone(),
							buy_curve_k,
							sell_curve_k,
							detail: detail.clone(),
							link: link.clone(),
							max_allowed_hosts,
							tapp_type,
							billing_mode,
							..Default::default()
						},
					);
					Self::buy_token_inner(who, id, init_fund);
					if link_related && TAppApprovedLinks::<T>::get(&link).creator.is_none() {
						T::CurrencyOperations::slash(who, T::ReservedLinkRentAmount::get());
					}

					Self::deposit_event(Event::TAppCreated(id, tapp_name.clone(), who.clone()));
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn buy_token(
			sender: OriginFor<T>,
			tapp_id: TAppId,
			tapp_amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						TAppBondingCurve::<T>::contains_key(tapp_id),
						Error::<T>::TAppIdNotExist
					);
					ensure!(
						!tapp_amount.is_zero(),
						Error::<T>::OperationAmountCanNotBeZero
					);
					let deposit_tea_amount =
						Self::calculate_buy_amount(Some(tapp_id), tapp_amount, None)?;
					ensure!(
						!deposit_tea_amount.is_zero(),
						Error::<T>::BuyTeaAmountCanNotBeZero
					);
					ensure!(
						T::CurrencyOperations::free_balance(who) >= deposit_tea_amount,
						Error::<T>::InsufficientFreeBalance,
					);
					ensure!(
						TotalSupplyTable::<T>::get(tapp_id)
							.checked_add(&tapp_amount)
							.ok_or(Error::<T>::AddOverflow)?
							< T::TotalSupplyMaxValue::get(),
						Error::<T>::TotalSupplyOverTheMaxValue
					);
					Ok(())
				},
				|who| {
					let deposit_tea_amount = Self::buy_token_inner(who, tapp_id, tapp_amount);

					let (buy_price, sell_price) = Self::query_price(tapp_id);
					Self::deposit_event(Event::TokenBought(
						tapp_id,
						who.clone(),
						deposit_tea_amount,
						tapp_amount,
						buy_price,
						sell_price,
						TotalSupplyTable::<T>::get(tapp_id),
					));
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn sell_token(
			sender: OriginFor<T>,
			tapp_id: TAppId,
			tapp_amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						TAppBondingCurve::<T>::contains_key(tapp_id),
						Error::<T>::TAppIdNotExist
					);
					ensure!(
						AccountTable::<T>::get(who, tapp_id) >= tapp_amount,
						Error::<T>::InsufficientTAppToken,
					);
					ensure!(
						!tapp_amount.is_zero(),
						Error::<T>::OperationAmountCanNotBeZero
					);
					let tea_amount = Self::calculate_sell_amount(tapp_id, tapp_amount)?;
					ensure!(!tea_amount.is_zero(), Error::<T>::SellTeaAmountCanNotBeZero);
					Ok(())
				},
				|who| {
					let sold_amount = Self::sell_token_inner(who, tapp_id, tapp_amount);

					let (buy_price, sell_price) = Self::query_price(tapp_id);
					Self::deposit_event(Event::TokenSold(
						tapp_id,
						who.clone(),
						sold_amount,
						tapp_amount,
						buy_price,
						sell_price,
						TotalSupplyTable::<T>::get(tapp_id),
					));

					Self::try_bankrupt_tapp(tapp_id);
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn consume(
			sender: OriginFor<T>,
			tapp_id: TAppId,
			tea_amount: BalanceOf<T>,
			note: Option<Vec<u8>>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						TAppBondingCurve::<T>::contains_key(tapp_id),
						Error::<T>::TAppIdNotExist
					);
					ensure!(
						!tea_amount.is_zero(),
						Error::<T>::OperationAmountCanNotBeZero
					);
					ensure!(
						T::CurrencyOperations::free_balance(who) >= tea_amount,
						Error::<T>::InsufficientFreeBalance,
					);
					if let Some(ref note) = note {
						ensure!(
							note.len() <= T::ConsumeNoteMaxLength::get() as usize,
							Error::<T>::ConsumeNoteIsTooLong
						);
					}
					Ok(())
				},
				|who| {
					match Self::calculate_given_increase_tea_how_much_token_mint(
						tapp_id,
						tea_amount.clone(),
					) {
						Ok(deposit_tapp_amount) => {
							if let Err(e) =
								Self::allocate_buy_tea_amount(who, tapp_id, deposit_tapp_amount)
							{
								// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
								log::error!("allocate buy tea amount failed: {:?}", e);
								return;
							}
							Self::distribute_to_investors(tapp_id, deposit_tapp_amount);

							let (buy_price, sell_price) = Self::query_price(tapp_id);
							Self::deposit_event(Event::TAppConsume(
								tapp_id,
								who.clone(),
								tea_amount,
								deposit_tapp_amount,
								note.clone(),
								buy_price,
								sell_price,
								TotalSupplyTable::<T>::get(tapp_id),
							));
						}
						Err(e) => {
							// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
							log::error!(
								"calculate given increase tea how much token mint failed: {:?}",
								e
							);
							return;
						}
					}
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn expense(sender: OriginFor<T>, tapp_id: TAppId) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						TAppBondingCurve::<T>::contains_key(tapp_id),
						Error::<T>::TAppIdNotExist
					);
					let tapp = TAppBondingCurve::<T>::get(tapp_id);
					ensure!(
						who.eq(&tapp.owner),
						Error::<T>::OnlyTAppOwnerAllowedToExpense
					);

					ensure!(
						!tapp.current_cost.is_zero(),
						Error::<T>::OperationAmountCanNotBeZero
					);

					let (withdraw_tapp_amount, _) =
						Self::calculate_given_received_tea_how_much_seller_give_away(
							tapp_id,
							tapp.current_cost,
						)?;
					ensure!(
						TotalSupplyTable::<T>::get(tapp_id) >= withdraw_tapp_amount,
						Error::<T>::InsufficientTotalSupply
					);
					Ok(())
				},
				|_who| Self::expense_inner(tapp_id),
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn host(sender: OriginFor<T>, cml_id: CmlId, tapp_id: TAppId) -> DispatchResult {
			let who = ensure_signed(sender)?;
			let current_block = frame_system::Pallet::<T>::block_number();

			extrinsic_procedure(
				&who,
				|who| {
					T::CmlOperation::check_belongs(&cml_id, who)?;
					ensure!(
						TAppBondingCurve::<T>::contains_key(tapp_id),
						Error::<T>::TAppIdNotExist
					);
					let tapp_item = TAppBondingCurve::<T>::get(tapp_id);
					ensure!(
						!TAppCurrentHosts::<T>::contains_key(tapp_id, cml_id),
						Error::<T>::CmlIsAlreadyHosting
					);
					ensure!(
						TAppCurrentHosts::<T>::iter_prefix(tapp_id).count()
							< tapp_item.max_allowed_hosts as usize,
						Error::<T>::TAppHostsIsFull
					);

					let cml = T::CmlOperation::cml_by_id(&cml_id)?;
					ensure!(cml.is_mining(), Error::<T>::OnlyMiningCmlCanHost);
					ensure!(
						!cml.should_dead(&current_block),
						Error::<T>::DeadedCmlCanNotHost
					);
					ensure!(cml.machine_id().is_some(), Error::<T>::CmlMachineIdIsNone);
					let (_, cml_status) = T::CmlOperation::mining_status(cml_id);
					ensure!(
						cml_status.eq(&MinerStatus::Active),
						Error::<T>::MiningCmlStatusShouldBeActive
					);
					ensure!(
						cml.cml_type() != CmlType::C,
						Error::<T>::NotAllowedTypeCHostingTApp
					);

					let (current_performance, _) =
						T::CmlOperation::miner_performance(cml_id, &current_block);
					ensure!(
						current_performance.unwrap_or(0)
							>= Self::cml_total_used_performance(cml_id)
								.saturating_add(tapp_item.host_performance()),
						Error::<T>::CmlMachineIsFullLoad
					);
					ensure!(
						T::CurrencyOperations::can_reserve(who, T::HostPledgeAmount::get()),
						Error::<T>::InsufficientFreeBalance
					);
					Ok(())
				},
				|who| {
					if let Err(e) = T::CurrencyOperations::reserve(who, T::HostPledgeAmount::get())
					{
						log::error!("reserve balance failed: {:?}", e);
						// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
						return;
					}
					TAppHostPledge::<T>::mutate(tapp_id, cml_id, |amount| {
						*amount = amount.saturating_add(T::HostPledgeAmount::get());
					});

					TAppCurrentHosts::<T>::insert(tapp_id, cml_id, current_block);
					let became_active = Self::try_active_tapp(tapp_id);
					CmlHostingTApps::<T>::mutate(cml_id, |tapp_ids| tapp_ids.push(tapp_id));

					match TAppBondingCurve::<T>::get(tapp_id).billing_mode {
						BillingMode::FixedHostingToken(token_amount) => {
							TAppReservedBalance::<T>::mutate(tapp_id, who, |amount| {
								amount.push((token_amount, cml_id));
							});
						}
						_ => {}
					}

					if let Ok(cml) = T::CmlOperation::cml_by_id(&cml_id) {
						Self::deposit_event(Event::TAppsHosted(
							tapp_id,
							cml_id,
							cml.machine_id().unwrap().clone(),
							became_active,
						));
					}
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn unhost(sender: OriginFor<T>, cml_id: CmlId, tapp_id: TAppId) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					T::CmlOperation::check_belongs(&cml_id, who)?;
					ensure!(
						TAppBondingCurve::<T>::contains_key(tapp_id),
						Error::<T>::TAppIdNotExist
					);
					ensure!(
						TAppCurrentHosts::<T>::contains_key(tapp_id, cml_id),
						Error::<T>::CmlNotHostTheTApp
					);

					let current_block = frame_system::Pallet::<T>::block_number();
					ensure!(
						current_block
							>= TAppCurrentHosts::<T>::get(tapp_id, cml_id)
								+ T::HostLockingBlockHeight::get(),
						Error::<T>::HostLockingBlockHeightNotReached
					);

					Ok(())
				},
				|who| {
					let pledge_amount = TAppHostPledge::<T>::take(tapp_id, cml_id);
					let unreserved_balance = T::CurrencyOperations::unreserve(who, pledge_amount);

					let became_pending = Self::unhost_tapp(tapp_id, cml_id, false);

					Self::deposit_event(Event::TAppsUnhosted(
						tapp_id,
						cml_id,
						became_pending,
						unreserved_balance,
					));
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn update_tapp_resource(
			sender: OriginFor<T>,
			tapp_id: TAppId,
			cid: Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						cid.len() <= T::CidMaxLength::get() as usize,
						Error::<T>::CidIsToLong
					);
					ensure!(
						TAppBondingCurve::<T>::contains_key(tapp_id),
						Error::<T>::TAppIdNotExist
					);
					let tapp = TAppBondingCurve::<T>::get(tapp_id);
					ensure!(
						who.eq(&tapp.owner),
						Error::<T>::OnlyTAppOwnerAllowedToExpense
					);

					Ok(())
				},
				|_who| {
					TAppResourceMap::<T>::insert(tapp_id, cid.clone());
				},
			)
		}

		/// This is basically a normal transfer balance extrinsic, except emit a topup event
		#[pallet::weight(195_000_000)]
		pub fn topup(
			sender: OriginFor<T>,
			tapp_operation_account: T::AccountId,
			tapp_id: TAppId,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						TAppBondingCurve::<T>::contains_key(tapp_id),
						Error::<T>::TAppIdNotExist
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
						tapp_id,
						who.clone(),
						tapp_operation_account.clone(),
						amount,
						current_height,
					));
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn push_notifications(
			sender: OriginFor<T>,
			accounts: Vec<T::AccountId>,
			expired_heights: Vec<T::BlockNumber>,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						NotificationAccount::<T>::get().eq(who),
						Error::<T>::NotAllowedPushNotification
					);
					ensure!(
						accounts.len() == expired_heights.len(),
						Error::<T>::NotificationAndAccountListNotMatched
					);
					ensure!(!accounts.is_empty(), Error::<T>::NotificationListIsEmpty);
					Ok(())
				},
				|who| {
					for i in 0..accounts.len() {
						UserNotifications::<T>::mutate(&accounts[i], |notification_list| {
							notification_list.push(expired_heights[i])
						});
					}
					T::CurrencyOperations::deposit_creating(who, 195000000u32.into());
				},
			)
		}

		#[pallet::weight(195_000_000)]
		pub fn read_notification(
			sender: OriginFor<T>,
			account: T::AccountId,
			expired_height: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(sender)?;

			extrinsic_procedure(
				&who,
				|who| {
					ensure!(
						NotificationAccount::<T>::get().eq(who),
						Error::<T>::NotAllowedPushNotification
					);
					ensure!(
						UserNotifications::<T>::contains_key(&account),
						Error::<T>::NotFoundNotificationUser
					);
					ensure!(
						UserNotifications::<T>::get(&account)
							.iter()
							.position(|x| x.eq(&expired_height))
							.is_some(),
						Error::<T>::NoUserNotificationToRead
					);
					Ok(())
				},
				|who| {
					UserNotifications::<T>::mutate(&account, |notification_list| {
						// try remove the first matched element
						if let Some(index) =
							notification_list.iter().position(|x| x.eq(&expired_height))
						{
							notification_list.remove(index);
						}
					});
					T::CurrencyOperations::deposit_creating(who, 195000000u32.into());
				},
			)
		}
	}
}
