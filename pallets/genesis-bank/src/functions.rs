use super::*;

impl<T: genesis_bank::Config> genesis_bank::Pallet<T> {
	pub(crate) fn is_time_for_collateral_check(height: T::BlockNumber) -> bool {
		// offset with 5 to void overlapping with staking period
		height % T::BillingCycle::get() == 5u32.into()
	}

	pub(crate) fn try_clean_default_loan() -> Vec<AssetUniqueId> {
		let current_height = frame_system::Pallet::<T>::block_number();
		let expired_ids: Vec<AssetUniqueId> = CollateralStore::<T>::iter()
			.filter(|(id, _)| Self::is_loan_in_default(id, &current_height))
			.map(|(id, loan)| {
				match id.asset_type {
					AssetType::CML => {
						if let Ok(cml_id) = to_cml_id(&id.inner_id) {
							T::CmlOperation::remove_cml(cml_id);
						}
					}
				}

				UserCollateralStore::<T>::remove(&loan.owner, &id);
				id
			})
			.collect();
		expired_ids
			.iter()
			.for_each(|id| CollateralStore::<T>::remove(id));

		expired_ids
	}

	pub(crate) fn check_before_collateral(
		id: &AssetUniqueId,
		who: &T::AccountId,
	) -> DispatchResult {
		let current_height = frame_system::Pallet::<T>::block_number();
		ensure!(
			current_height < CloseHeight::<T>::get().unwrap_or(u32::MAX.into()),
			Error::<T>::CannotApplyLoanAfterClosed
		);

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
			}
		}
		Ok(())
	}

	pub(crate) fn create_new_collateral(id: &AssetUniqueId, who: &T::AccountId) {
		match id.asset_type {
			AssetType::CML => {
				let current_height = frame_system::Pallet::<T>::block_number();
				let cml_id = to_cml_id(&id.inner_id).unwrap();
				CollateralStore::<T>::insert(
					id,
					Loan {
						start_at: current_height,
						owner: who.clone(),
					},
				);
				UserCollateralStore::<T>::insert(who, id, ());
				T::CmlOperation::transfer_cml_to_other(who, &cml_id, &OperationAccount::<T>::get());

				T::CurrencyOperations::deposit_creating(who, T::GenesisCmlLoanAmount::get());
			}
		}
	}

	pub(crate) fn check_before_payoff_loan(
		id: &AssetUniqueId,
		who: &T::AccountId,
	) -> DispatchResult {
		let current_height = frame_system::Pallet::<T>::block_number();
		Self::check_belongs(who, id)?;

		match id.asset_type {
			AssetType::CML => {
				let cml_id = to_cml_id(&id.inner_id).map_err(|e| Error::<T>::from(e))?;
				ensure!(
					!Self::is_loan_in_default(id, &current_height),
					Error::<T>::LoanInDefault
				);
				ensure!(
					T::CurrencyOperations::free_balance(who)
						>= Self::cml_need_to_pay(id, &current_height),
					Error::<T>::InsufficientRepayBalance
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

	pub(crate) fn is_loan_in_default(id: &AssetUniqueId, current_height: &T::BlockNumber) -> bool {
		*current_height > CollateralStore::<T>::get(id).start_at + T::LoanTermDuration::get()
	}

	pub(crate) fn check_belongs(who: &T::AccountId, id: &AssetUniqueId) -> DispatchResult {
		ensure!(
			CollateralStore::<T>::contains_key(id),
			Error::<T>::LoanNotExists
		);
		ensure!(
			UserCollateralStore::<T>::contains_key(who, id),
			Error::<T>::InvalidBorrower
		);
		Ok(())
	}

	pub(crate) fn payoff_loan_inner(id: &AssetUniqueId, who: &T::AccountId) {
		let current_height = frame_system::Pallet::<T>::block_number();

		match id.asset_type {
			AssetType::CML => {
				let cml_id = to_cml_id(&id.inner_id).unwrap();
				T::CurrencyOperations::slash(who, Self::cml_need_to_pay(id, &current_height));
				T::CmlOperation::transfer_cml_to_other(&OperationAccount::<T>::get(), &cml_id, who);

				CollateralStore::<T>::remove(id);
				UserCollateralStore::<T>::remove(who, id);
			}
		}
	}

	pub(crate) fn cml_need_to_pay(
		id: &AssetUniqueId,
		current_height: &T::BlockNumber,
	) -> BalanceOf<T> {
		if !CollateralStore::<T>::contains_key(id) {
			return Zero::zero();
		}

		let loan = CollateralStore::<T>::get(id);
		T::GenesisCmlLoanAmount::get() + Self::calculate_interest(current_height, &loan.start_at)
	}

	pub fn calculate_interest(
		current_height: &T::BlockNumber,
		start_at: &T::BlockNumber,
	) -> BalanceOf<T> {
		if *current_height < *start_at {
			return Zero::zero();
		}

		let terms: Option<u32> = ((*current_height - *start_at) / T::BillingCycle::get())
			.try_into()
			.ok();

		(T::GenesisCmlLoanAmount::get() * (terms.unwrap_or(1u32) + 1u32).into() / 10000u32.into())
			* T::InterestRate::get()
	}
}

#[cfg(test)]
mod tests {
	use crate::mock::*;
	use crate::*;
	use pallet_cml::CmlId;

	#[test]
	fn calculate_interest_works() {
		new_test_ext().execute_with(|| {
			let unit_interest = GENESIS_CML_LOAN_AMOUNT * INTEREST_RATE / 10000;
			// return unit leading rate if current_height equals to start_at, or difference lower than BillingCycle
			assert_eq!(GenesisBank::calculate_interest(&0, &0), unit_interest);
			assert_eq!(
				GenesisBank::calculate_interest(&10000, &10000),
				unit_interest
			);
			assert_eq!(
				GenesisBank::calculate_interest(&(LOAN_BILLING_CYCLE as u64 - 1), &0),
				unit_interest
			);

			assert_eq!(
				GenesisBank::calculate_interest(&(LOAN_BILLING_CYCLE as u64), &0),
				unit_interest * 2
			);
			assert_eq!(
				GenesisBank::calculate_interest(&(2 * LOAN_BILLING_CYCLE as u64 - 1), &0),
				unit_interest * 2
			);

			assert_eq!(
				GenesisBank::calculate_interest(&(2 * LOAN_BILLING_CYCLE as u64), &0),
				unit_interest * 3
			);
		});
	}

	#[test]
	fn try_clean_default_loan_works() {
		new_test_ext().execute_with(|| {
			let user1 = 11;
			let user2 = 22;
			let id1 = new_id(1);
			let id2 = new_id(2);
			let start_height1 = 0;
			let start_height2 = 1000;
			CollateralStore::<Test>::insert(&id1, new_lien(user1, start_height1));
			CollateralStore::<Test>::insert(&id2, new_lien(user2, start_height2));
			UserCollateralStore::<Test>::insert(user1, &id1, ());
			UserCollateralStore::<Test>::insert(user2, &id2, ());

			frame_system::Pallet::<Test>::set_block_number(0);
			assert_eq!(GenesisBank::try_clean_default_loan().len(), 0);

			frame_system::Pallet::<Test>::set_block_number(LOAN_TERM_DURATION as u64 - 1);
			assert_eq!(GenesisBank::try_clean_default_loan().len(), 0);

			frame_system::Pallet::<Test>::set_block_number(LOAN_TERM_DURATION as u64);
			assert_eq!(GenesisBank::try_clean_default_loan().len(), 0);

			frame_system::Pallet::<Test>::set_block_number(LOAN_TERM_DURATION as u64 + 1);
			let cleaned_ids = GenesisBank::try_clean_default_loan();
			assert_eq!(cleaned_ids.len(), 1);
			assert_eq!(cleaned_ids[0], id1);
			assert!(!CollateralStore::<Test>::contains_key(&id1));
			assert!(!UserCollateralStore::<Test>::contains_key(&user1, &id1));
			assert!(CollateralStore::<Test>::contains_key(&id2));
			assert!(UserCollateralStore::<Test>::contains_key(&user2, &id2));

			frame_system::Pallet::<Test>::set_block_number(
				LOAN_TERM_DURATION as u64 + start_height2,
			);
			assert_eq!(GenesisBank::try_clean_default_loan().len(), 0);

			frame_system::Pallet::<Test>::set_block_number(
				LOAN_TERM_DURATION as u64 + start_height2 + 1,
			);
			let cleaned_ids = GenesisBank::try_clean_default_loan();
			assert_eq!(cleaned_ids.len(), 1);
			assert_eq!(cleaned_ids[0], id2);
			assert!(!CollateralStore::<Test>::contains_key(&id2));
			assert!(!UserCollateralStore::<Test>::contains_key(&user2, &id2));
		})
	}

	fn new_id(cml_id: CmlId) -> AssetUniqueId {
		AssetUniqueId {
			asset_type: AssetType::CML,
			inner_id: from_cml_id(cml_id),
		}
	}

	fn new_lien(owner: u64, start_at: u64) -> Loan<u64, u64> {
		Loan { owner, start_at }
	}
}
