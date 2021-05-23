
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

mod impl_stored_map;
mod functions;
mod types;
use types::*;

use sp_std::{prelude::*, borrow::Borrow};
use sp_runtime::{
		RuntimeDebug, TokenError, traits::{
		AtLeast32BitUnsigned, Zero, One, StaticLookup, Saturating, CheckedSub, CheckedAdd, Bounded,
		StoredMapError,
	}
};
use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
};
use frame_system::pallet_prelude::*;
use codec::{HasCompact};
use frame_support::{ensure, dispatch::{DispatchError}};
use frame_support::traits::{Currency, ReservableCurrency, BalanceStatus, StoredMap, Get,};
use frame_support::traits::tokens::{WithdrawConsequence, DepositConsequence, fungibles};
// use frame_system::Config as SystemConfig;
pub use cml::*;


// type BalanceOf<T> = 
// 	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod cml {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Identifier for the class of asset.
		type AssetId: Member + Parameter + Default + Copy + HasCompact;

		type Currency: Currency<Self::AccountId>;

		// type Unit: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type StakingPrice: Get<u32>;

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// #[pallet::storage]
	// pub(super) type LastAssetId<T: Config> = StorageValue<
	// 	_,
	// 	T::AssetId,
	// 	ValueQuery,
	// >;

	// #[pallet::storage]
	// #[pallet::getter(fn cml_store)]
	// pub(super) type CmlStore<T: Config> = StorageMap<
	// 	_,
	// 	Twox64Concat,
	// 	T::AccountId,
	// 	Vec<CML<T::AssetId, T::AccountId, T::BlockNumber>>,
	// >;

	#[pallet::storage]
	#[pallet::getter(fn dai_store)]
	pub(super) type DaiStore<T: Config> = StorageMap<
		_, 
		Twox64Concat, 
		T::AccountId,
		Dai,
	>;

	// #[pallet::storage]
	// pub(super) type MinerItemStore<T: Config> = StorageMap<
	// 	_,
	// 	identity,
	// 	Vec<u8>,
	// 	MinerItem,
	// >;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub dai_list: Vec<(T::AccountId, Dai)>,
	}
	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				dai_list: vec![],
			}
		}
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (account, amount) in self.dai_list.iter() {
        Pallet::<T>::set_dai(&account, *amount);
      }
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	#[pallet::metadata(
		T::AccountId = "AccountId",
		T::AssetId = "AssetId"
	)]
	pub enum Event<T: Config> {
		// Issued(T::AssetId, T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		NotEnoughDai,
		NotFoundCML,
		CMLNotLive,
		NotEnoughTeaToStaking,
		MinerAlreadyExist,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		// TODO
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(1_000)]
		fn transfer_dai(
			sender: OriginFor<T>,
			target: T::AccountId,
			#[pallet::compact] amount: Dai,
		) -> DispatchResult {
			let sender = ensure_signed(sender)?;

			let _sender_dai = Self::get_dai(&sender);
			let _target_dai = Self::get_dai(&target);
			
			ensure!(_sender_dai >= amount, Error::<T>::NotEnoughDai);

			Self::set_dai(&sender, _sender_dai-amount);
			Self::set_dai(&target, _target_dai+amount);
			
			Ok(())
		}
	}
}


