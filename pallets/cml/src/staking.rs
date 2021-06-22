use super::*;

impl<T: cml::Config> cml::Pallet<T> {
	pub(crate) fn create_balance_staking(
		who: &T::AccountId,
	) -> Result<StakingItem<T::AccountId, BalanceOf<T>>, DispatchError> {
		let staking_price: BalanceOf<T> = T::StakingPrice::get();

		T::CurrencyOperations::reserve(who, staking_price)?;
		Ok(StakingItem {
			owner: who.clone(),
			category: StakingCategory::Tea,
			amount: Some(staking_price),
			cml: None,
		})
	}

	pub(crate) fn create_seed_staking(
		who: &T::AccountId,
		cml_id: CmlId,
		current_height: T::BlockNumber,
	) -> Result<StakingItem<T::AccountId, BalanceOf<T>>, DispatchError> {
		Self::check_belongs(&cml_id, who)?;
		let cml = CmlStore::<T>::get(cml_id).ok_or(Error::<T>::NotFoundCML)?;
		ensure!(
			cml.seed_valid(current_height),
			Error::<T>::ShouldStakingLiveSeed
		);
		CmlStore::<T>::mutate(cml_id, |cml| {
			cml.as_mut().unwrap().status = CmlStatus::SeedStaking
		});

		Ok(StakingItem {
			owner: who.clone(),
			category: StakingCategory::Cml,
			amount: None,
			cml: Some(cml_id),
		})
	}

	pub fn check_miner_staking_slot(
		cml: &CML<T::AccountId, T::BlockNumber, BalanceOf<T>>,
	) -> Result<(), DispatchError> {
		// todo implement me
		Ok(())
	}

	pub fn staking_to_cml(
		staking_item: StakingItem<T::AccountId, BalanceOf<T>>,
		target_cml_id: &CmlId,
		height: T::BlockNumber,
	) -> DispatchResult {
		let mut cml = CmlStore::<T>::get(&target_cml_id).ok_or(Error::<T>::NotFoundCML)?;

		ensure!(cml.should_dead(height), Error::<T>::CMLNotLive);
		cml.staking_slot.push(staking_item);

		Self::update_cml(cml.clone());

		Ok(())
	}
}
