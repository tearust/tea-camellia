
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
pub use types::*;

use sp_std::{prelude::*, borrow::Borrow};
use sp_runtime::{
	RuntimeDebug, TokenError, ArithmeticError, traits::{
		AtLeast32BitUnsigned, Zero, One, StaticLookup, Saturating, CheckedSub, CheckedAdd, Bounded,
		StoredMapError,
	}
};
use codec::{Encode, Decode, HasCompact};
use frame_support::{ensure, dispatch::{DispatchError, DispatchResult}};
use frame_support::traits::{Currency, ReservableCurrency, BalanceStatus, StoredMap, Get,};
use frame_support::traits::tokens::{WithdrawConsequence, DepositConsequence, fungibles};
use frame_system::Config as SystemConfig;

pub use weights::WeightInfo;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::DispatchResult,
		pallet_prelude::*,
	};
	use frame_system::pallet_prelude::*;
	use super::*;

	

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	/// The module configuration trait.
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Identifier for the class of asset.
		type AssetId: Member + Parameter + Default + Copy + HasCompact;

		// Id coin for pre-sale, convert to CML when main-net onboard.
		type Dai: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

		type Currency: Currency<Self::AccountId>;

		#[pallet::constant]
		type Unit: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type StakingPrice: Get<u32>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	pub(super) type LastAssetId<T: Config> = T::AssetId;

	#[pallet::storage]
	#[pallet::getter(fn cml_store)]
	pub(super) type CmlStore<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		Vec<CML<T::AssetId, T::AccountId, T::BlockNumber>>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn dai_store)]
	pub(super) type DaiStore<T: Config> = StorageMap<
		_, 
		Twox64Concat, 
		T::AccountId,
		T::Dai,
	>;

	#[pallet::storage]
	pub(super) type MinerItemStore<T: Config> = StorageMap<
		_,
		identity,
		Vec<u8>,
		MinerItem,
	>;

	#[pallet::genesis_config]
	#[derive(Default)]
	pub struct GenesisConfig {
		pub dai_list: Vec<T::AccountId, T::Dai>
	}
	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig {
		fn build(&self) {
			for (account, amount) in config.dai_list.iter() {
        Module::<T>::set_dai(&account, *amount);
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
		Issued(T::AssetId, T::AccountId, T::Balance),
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
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
		// TODO
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)]
		fn transfer_dai(
			sender: OriginFor<T>,
			target: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] amount: T::Dai,
		) {
			let sender = ensure_signed(origin)?;

			let _sender_dai = Self::get_dai(&sender);
			let _target_dai = Self::get_dai(&target);
			
			ensure!(_sender_dai >= amount, Error::<T>::NotEnoughDai);

			Self::set_dai(&sender, _sender_dai-amount);
      Self::set_dai(&target, _target_dai+amount);
		}
	}
}
