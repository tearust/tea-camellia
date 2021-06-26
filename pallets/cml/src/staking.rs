use super::*;

impl<T: cml::Config> cml::Pallet<T> {
	pub(crate) fn is_staking_period_start(height: T::BlockNumber) -> bool {
		height % T::StakingPeriodLength::get() == 1u32.into()
	}

	pub(crate) fn is_staking_period_end(height: T::BlockNumber) -> bool {
		height % T::StakingPeriodLength::get() == 0u32.into()
	}

	pub(crate) fn check_balance_staking(_who: &T::AccountId) -> DispatchResult {
		// todo implement me later
		// ensure!(
		// 	T::CurrencyOperations::free_balance(&sender) > T::StakingPrice::get(),
		// 	Error::<T>::InsufficientFreeBalance,
		// );
		Ok(())
	}

	pub(crate) fn create_balance_staking(
		who: &T::AccountId,
	) -> Result<StakingItem<T::AccountId, BalanceOf<T>>, DispatchError> {
		let staking_price: BalanceOf<T> = T::StakingPrice::get();

		// todo implement me later
		// T::CurrencyOperations::reserve(who, staking_price)?;
		Ok(StakingItem {
			owner: who.clone(),
			category: StakingCategory::Tea,
			amount: Some(staking_price),
			cml: None,
		})
	}

	pub(crate) fn check_seed_staking(
		who: &T::AccountId,
		cml_id: CmlId,
		current_height: &T::BlockNumber,
	) -> DispatchResult {
		Self::check_belongs(&cml_id, who)?;
		let cml = CmlStore::<T>::get(cml_id);
		ensure!(cml.is_some(), Error::<T>::NotFoundCML);
		let cml = cml.unwrap();
		ensure!(
			cml.seed_valid(current_height)
				.map_err(|e| Error::<T>::from(e))?
				|| cml
					.tree_valid(current_height)
					.map_err(|e| Error::<T>::from(e))?,
			Error::<T>::ShouldStakingLiveTree
		);
		Ok(())
	}

	#[allow(dead_code)]
	pub(crate) fn create_seed_staking(
		who: &T::AccountId,
		cml_id: CmlId,
		current_height: &T::BlockNumber,
	) -> Result<StakingItem<T::AccountId, BalanceOf<T>>, DispatchError> {
		Self::check_belongs(&cml_id, who)?;

		CmlStore::<T>::mutate(cml_id, |cml| match cml {
			Some(cml) => {
				ensure!(
					cml.seed_valid(&current_height)?,
					Error::<T>::ShouldStakingLiveTree
				);
				cml.convert_to_tree(&current_height)?;
				Ok(())
			}
			None => Err(Error::<T>::NotFoundCML),
		})?;

		Ok(StakingItem {
			owner: who.clone(),
			category: StakingCategory::Cml,
			amount: None,
			cml: Some(cml_id),
		})
	}
}

#[cfg(test)]
mod tests {
	use crate::mock::*;

	#[test]
	fn staking_period_related_works() {
		new_test_ext().execute_with(|| {
			assert!(Cml::is_staking_period_end(0));
			assert!(Cml::is_staking_period_start(1));

			for i in 2..STAKING_PERIOD_LENGTH as u64 {
				assert!(!Cml::is_staking_period_end(i));
				assert!(!Cml::is_staking_period_start(i));
			}

			assert!(Cml::is_staking_period_end(STAKING_PERIOD_LENGTH as u64));
			assert!(Cml::is_staking_period_start(
				STAKING_PERIOD_LENGTH as u64 + 1
			));
		})
	}
}
