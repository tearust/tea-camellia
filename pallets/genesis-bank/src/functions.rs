use super::*;

impl<T: genesis_bank::Config> genesis_bank::Pallet<T> {
	pub(crate) fn is_lien_billing_period_end(height: T::BlockNumber) -> bool {
		// offset with 5 to void overlapping with staking period
		height % T::LienBillingPeriod::get() == 5u32.into()
	}

	pub(crate) fn try_clean_expired_lien() {
		let current_height = frame_system::Pallet::<T>::block_number();
		let expired_ids: Vec<AssetUniqueId> = LienStore::<T>::iter()
			.filter(|(id, _)| Self::is_lien_expired(id, &current_height))
			.map(|(id, lien)| {
				UserLienStore::<T>::remove(&lien.owner, &id);
				id
			})
			.collect();
		expired_ids.iter().for_each(|id| LienStore::<T>::remove(id));
	}

	pub(crate) fn check_pawn_asset(id: &AssetUniqueId, who: &T::AccountId) -> DispatchResult {
		match id.asset_type {
			AssetType::CML => {
				let cml_id = to_cml_id(&id.inner_id).map_err(|e| Error::<T>::from(e))?;
				T::CmlOperation::check_belongs(&cml_id, who)?;
				let cml = T::CmlOperation::cml_by_id(&cml_id)?;
				ensure!(cml.is_frozen_seed(), Error::<T>::ShouldPawnFrozenSeed);
				ensure!(cml.is_from_genesis(), Error::<T>::ShouldPawnGenesisSeed);
				T::CmlOperation::check_transfer_cml_to_other(
					who,
					&cml_id,
					&OperationAccount::<T>::get(),
				)?;

				ensure!(
					T::CurrencyOperations::free_balance(&OperationAccount::<T>::get())
						>= T::GenesisCmlLienAmount::get(),
					Error::<T>::InsufficientBalanceToPay
				);
			}
		}
		Ok(())
	}

	pub(crate) fn create_new_lien(id: &AssetUniqueId, who: &T::AccountId) {
		match id.asset_type {
			AssetType::CML => {
				if T::CurrencyOperations::transfer(
					&OperationAccount::<T>::get(),
					who,
					T::GenesisCmlLienAmount::get(),
					ExistenceRequirement::AllowDeath,
				)
				.is_err()
				{
					// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
					return;
				}

				let current_height = frame_system::Pallet::<T>::block_number();
				let cml_id = to_cml_id(&id.inner_id).unwrap();
				LienStore::<T>::insert(
					id,
					Lien {
						start_at: current_height,
						owner: who.clone(),
					},
				);
				UserLienStore::<T>::insert(who, id, ());
				T::CmlOperation::transfer_cml_to_other(who, &cml_id, &OperationAccount::<T>::get());
			}
		}
	}

	pub(crate) fn check_redeem_asset(id: &AssetUniqueId, who: &T::AccountId) -> DispatchResult {
		let current_height = frame_system::Pallet::<T>::block_number();
		Self::check_belongs(who, id)?;

		match id.asset_type {
			AssetType::CML => {
				let cml_id = to_cml_id(&id.inner_id).map_err(|e| Error::<T>::from(e))?;
				ensure!(
					!Self::is_lien_expired(id, &current_height),
					Error::<T>::LienHasExpired
				);
				ensure!(
					T::CurrencyOperations::free_balance(who)
						>= Self::cml_need_to_pay(id, &current_height),
					Error::<T>::InsufficientRedeemBalance
				);
				T::CmlOperation::check_transfer_cml_to_other(
					&OperationAccount::<T>::get(),
					&cml_id,
					who,
				)?;
			}
		}
		Ok(())
	}

	pub(crate) fn is_lien_expired(id: &AssetUniqueId, current_height: &T::BlockNumber) -> bool {
		*current_height > LienStore::<T>::get(id).start_at + T::LienTermDuration::get()
	}

	pub(crate) fn check_belongs(who: &T::AccountId, id: &AssetUniqueId) -> DispatchResult {
		ensure!(LienStore::<T>::contains_key(id), Error::<T>::AssetNotExists);
		ensure!(
			UserLienStore::<T>::contains_key(who, id),
			Error::<T>::InvalidAssetUser
		);
		Ok(())
	}

	pub(crate) fn redeem_asset_inner(id: &AssetUniqueId, who: &T::AccountId) {
		let current_height = frame_system::Pallet::<T>::block_number();

		match id.asset_type {
			AssetType::CML => {
				let cml_id = to_cml_id(&id.inner_id).unwrap();
				if T::CurrencyOperations::transfer(
					who,
					&OperationAccount::<T>::get(),
					Self::cml_need_to_pay(id, &current_height),
					ExistenceRequirement::AllowDeath,
				)
				.is_err()
				{
					// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
					return;
				}
				T::CmlOperation::transfer_cml_to_other(&OperationAccount::<T>::get(), &cml_id, who);
			}
		}
	}

	pub(crate) fn cml_need_to_pay(
		id: &AssetUniqueId,
		current_height: &T::BlockNumber,
	) -> BalanceOf<T> {
		let lien = LienStore::<T>::get(id);
		let terms: Option<u32> = ((*current_height - lien.start_at) / T::LienBillingPeriod::get())
			.try_into()
			.ok();

		let interest =
			T::GenesisCmlLienAmount::get() * terms.unwrap_or(1u32).into() * T::LendingRates::get()
				/ 10000u32.into()
				+ T::LendingRates::get();
		T::GenesisCmlLienAmount::get() + interest
	}
}
