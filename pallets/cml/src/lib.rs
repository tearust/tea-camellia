
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

use sp_std::{
	prelude::*, 
	// borrow::Borrow
};
use sp_runtime::{
		SaturatedConversion,
		traits::{
		AtLeast32Bit, Zero, One, 
	}
};
use log::{info};

use frame_support::{
	dispatch::DispatchResult,
	pallet_prelude::*,
};
use frame_system::pallet_prelude::*;
// use codec::{HasCompact};
use frame_support::{ensure};
use frame_support::traits::{
	Currency, 
	Get,
};

pub use cml::*;


pub type BalanceOf<T> = 
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod cml {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type CmlId: Parameter + AtLeast32Bit + Default + Copy;

		type Currency: Currency<Self::AccountId>;

		#[pallet::constant]
		type StakingPrice: Get<BalanceOf<Self>>;

	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::type_value]
	pub fn DefaultAssetId<T: Config>() -> T::CmlId { <T::CmlId>::saturated_from(10000_u32) }
	#[pallet::storage]
	pub type LastCmlId<T: Config> = StorageValue<
		_,
		T::CmlId,
		ValueQuery,
		DefaultAssetId<T>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn cml_store)]
	pub type CmlStore<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::CmlId,
		CML<T::CmlId, T::AccountId, T::BlockNumber, BalanceOf<T>>,
	>;

	#[pallet::storage]
	#[pallet::getter(fn user_cml_store)]
	pub type UserCmlStore<T: Config> = StorageMap<
		_,
		Twox64Concat, T::AccountId,
		Vec<T::CmlId>
	>;

	#[pallet::storage]
	#[pallet::getter(fn dai_store)]
	pub type DaiStore<T: Config> = StorageMap<
		_, 
		Twox64Concat, 
		T::AccountId,
		Dai,
	>;

	#[pallet::storage]
	pub type MinerItemStore<T: Config> = StorageMap<
		_,
		Twox64Concat,
		Vec<u8>,
		MinerItem,
	>;

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
		T::CmlId = "CmlId"
	)]
	pub enum Event<T: Config> {
		ActiveCml(T::AccountId, T::CmlId),
	}

	#[pallet::error]
	pub enum Error<T> {
		NotEnoughDai,
		NotFoundCML,
		CMLNotLive,
		NotEnoughTeaToStaking,
		MinerAlreadyExist,
		CMLOwnerInvalid,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		// TODO
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(1_000)]
		pub fn transfer_dai(
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

		#[pallet::weight(10_000)]
		pub fn convert_cml_from_dai(
			sender: OriginFor<T>,
		) -> DispatchResult {
			let sender = ensure_signed(sender)?;

			// check sender dai
			let _sender_dai = Self::get_dai(&sender);
			ensure!(_sender_dai > Zero::zero(), Error::<T>::NotEnoughDai);

			// TODO, check dai is frozen or live
			let status = CmlStatus::SeedFrozen;

			// dai - 1
			Self::set_dai(&sender, _sender_dai.saturating_sub(1 as Dai));

			// add cml
			let cml = Self::new_cml_from_dai(CmlGroup::Nitro, status);
			Self::add_cml(&sender, cml);

			Ok(())
		}

		#[pallet::weight(10_000)]
		pub fn active_cml_for_nitro(
			sender: OriginFor<T>,
			cml_id: T::CmlId,
			miner_id: Vec<u8>,
			miner_ip: Vec<u8>,
		) -> DispatchResult {
			let sender = ensure_signed(sender)?;

			let _ = Self::get_cml_by_id(&cml_id)?;
			Self::check_belongs(&cml_id, &sender)?;

			let miner_item = MinerItem {
				id: miner_id.clone(),
				ip: miner_ip,
				status: MinerStatus::Active,
			};

			ensure!(!<MinerItemStore<T>>::contains_key(&miner_id), Error::<T>::MinerAlreadyExist);

			let balance = T::Currency::free_balance(&sender);

			let max_price: BalanceOf<T> = T::StakingPrice::get();
			ensure!(balance > max_price, Error::<T>::NotEnoughTeaToStaking);

			let staking_item = StakingItem {
				owner: sender.clone(),
				category: StakingCategory::Tea,
				amount: Some(T::StakingPrice::get()),
				cml: None,
			};

			Self::update_cml_to_active(&cml_id, miner_id.clone(), staking_item)?;
			<MinerItemStore<T>>::insert(&miner_id, miner_item);

			info!("TODO ---- lock balance");

			Self::deposit_event(Event::ActiveCml(sender.clone(), cml_id));
			Ok(())
		}
	}
}


