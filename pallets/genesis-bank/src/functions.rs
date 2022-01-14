use super::*;

impl<T: genesis_bank::Config> genesis_bank::Pallet<T> {
	pub(crate) fn is_time_for_reset_interest_rate(height: T::BlockNumber) -> bool {
		// offset with `InterestPeriodLength` - 2 to void overlapping with staking period
		height % T::BillingCycle::get() == T::BillingCycle::get() - 2u32.into()
	}

	pub(crate) fn try_clean_default_loan() -> Vec<AssetUniqueId> {
		let current_height = frame_system::Pallet::<T>::block_number();
		let expired_ids: Vec<AssetUniqueId> = CollateralStore::<T>::iter()
			.filter(|(id, _)| Self::is_loan_in_default(id, &current_height))
			.map(|(id, loan)| {
				match id.asset_type {
					AssetType::CML => {
						if let Ok(cml_id) = to_cml_id(&id.inner_id) {
							match Self::cml_loan_initial_amount(cml_id) {
								Ok(compensation_amount) => {
									T::CurrencyOperations::deposit_creating(
										&OperationAccount::<T>::get(),
										compensation_amount,
									);
								}
								Err(e) => {
									log::warn!("get cml loan initial amount failed: {:?}", e);
								}
							}

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

		let expired_cmls = expired_ids
			.iter()
			.map(|id| match id.asset_type {
				AssetType::CML => to_cml_id(&id.inner_id).ok(),
			})
			.filter(|id| id.is_some())
			.map(|id| id.unwrap())
			.collect();
		Self::deposit_event(Event::BurnedCmlList(expired_cmls));

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
				#[cfg(not(feature = "fast"))]
				ensure!(cml.is_from_genesis(), Error::<T>::ShouldPawnGenesisSeed);
				T::CmlOperation::check_transfer_cml_to_other(
					who,
					&cml_id,
					&OperationAccount::<T>::get(),
				)?;
				ensure!(
					!T::AuctionOperation::is_cml_in_auction(cml_id),
					Error::<T>::CannotPawnWhenCmlIsInAuction
				);

				let amount = Self::cml_loan_initial_amount(cml_id)?;
				ensure!(
					T::CurrencyOperations::free_balance(&OperationAccount::<T>::get()) >= amount,
					Error::<T>::GenesisBankInsufficientFreeBalance,
				);
			}
		}
		Ok(())
	}

	pub(crate) fn create_new_collateral(id: &AssetUniqueId, who: &T::AccountId) {
		match id.asset_type {
			AssetType::CML => {
				let current_height = frame_system::Pallet::<T>::block_number();
				let cml_id = to_cml_id(&id.inner_id).unwrap();

				if let Ok(cml) = T::CmlOperation::cml_by_id(&cml_id) {
					match Self::cml_loan_initial_amount(cml_id) {
						Ok(initial_amount) => {
							if let Err(e) = T::CurrencyOperations::transfer(
								&OperationAccount::<T>::get(),
								who,
								initial_amount,
								ExistenceRequirement::AllowDeath,
							) {
								// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
								log::error!("genesis bank transfer free balance failed: {:?}", e);
							}

							CollateralStore::<T>::insert(
								id,
								Loan {
									start_at: current_height,
									term_update_at: current_height,
									owner: who.clone(),
									loan_type: cml.cml_type(),
									principal: initial_amount,
									interest: Zero::zero(),
								},
							);
							UserCollateralStore::<T>::insert(who, id, ());
							T::CmlOperation::transfer_cml_to_other(
								who,
								&cml_id,
								&OperationAccount::<T>::get(),
							);
						}
						Err(e) => {
							// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
							log::error!("get cml loan initial amount failed: {:?}", e);
						}
					}
				}
			}
		}
	}

	pub(crate) fn check_before_payoff_loan(
		id: &AssetUniqueId,
		who: &T::AccountId,
		amount: BalanceOf<T>,
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
				ensure!(!amount.is_zero(), Error::<T>::RepayAmountCanNotBeZero);
				ensure!(
					T::CurrencyOperations::free_balance(who) >= amount,
					Error::<T>::InsufficientRepayBalance
				);

				if amount >= Self::cml_need_to_pay(id, false) {
					T::CmlOperation::check_transfer_cml_to_other(
						&OperationAccount::<T>::get(),
						&cml_id,
						who,
					)?;
				}
			}
		}
		Ok(())
	}

	pub(crate) fn is_loan_in_default(id: &AssetUniqueId, current_height: &T::BlockNumber) -> bool {
		*current_height > CollateralStore::<T>::get(id).term_update_at + T::LoanTermDuration::get()
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

	pub(crate) fn classify_amount_usage(
		id: &AssetUniqueId,
		amount: BalanceOf<T>,
		update_term_start: &mut bool,
		transfer_cml: &mut bool,
	) -> (BalanceOf<T>, BalanceOf<T>) {
		let need_to_pay_all = Self::cml_need_to_pay(id, false);
		let need_to_pay_interest = Self::cml_need_to_pay(id, true);

		if amount >= need_to_pay_all {
			*update_term_start = true;
			*transfer_cml = true;
			(
				need_to_pay_all.saturating_sub(need_to_pay_interest.clone()),
				need_to_pay_interest,
			)
		} else if amount >= need_to_pay_interest {
			*update_term_start = true;
			*transfer_cml = false;
			(
				amount.saturating_sub(need_to_pay_interest.clone()),
				need_to_pay_interest,
			)
		} else {
			*update_term_start = false;
			*transfer_cml = false;
			(Zero::zero(), amount)
		}
	}

	pub(crate) fn payoff_loan_inner(id: &AssetUniqueId, who: &T::AccountId, amount: BalanceOf<T>) {
		match id.asset_type {
			AssetType::CML => {
				let cml_id = to_cml_id(&id.inner_id).unwrap();
				let mut update_term_start = false;
				let mut transfer_cml = false;

				let (need_transfer, need_slash) = Self::classify_amount_usage(
					id,
					amount,
					&mut update_term_start,
					&mut transfer_cml,
				);

				if !need_transfer.is_zero() {
					if let Err(e) = T::CurrencyOperations::transfer(
						who,
						&OperationAccount::<T>::get(),
						need_transfer.clone(),
						ExistenceRequirement::AllowDeath,
					) {
						// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
						log::error!("transfer balance to bank failed: {:?}", e);
						return;
					}
					CollateralStore::<T>::mutate(id, |loan| {
						loan.principal = loan.principal.saturating_sub(need_transfer);
					});
				}

				if !need_slash.is_zero() {
					T::CurrencyOperations::slash(who, need_slash.clone());
					CollateralStore::<T>::mutate(id, |loan| {
						loan.interest = loan
							.interest
							.saturating_add(Self::calculate_interest(&loan))
							.saturating_sub(need_slash);
					});
				}

				if transfer_cml {
					Self::clean_cml_loan(id, who, cml_id);
				} else if update_term_start {
					CollateralStore::<T>::mutate(id, |loan| {
						loan.term_update_at = frame_system::Pallet::<T>::block_number();
					})
				}
			}
		}
	}

	fn clean_cml_loan(id: &AssetUniqueId, who: &T::AccountId, cml_id: CmlId) {
		T::CmlOperation::transfer_cml_to_other(&OperationAccount::<T>::get(), &cml_id, who);
		CollateralStore::<T>::remove(id);
		UserCollateralStore::<T>::remove(who, id);
	}

	pub(crate) fn cml_need_to_pay(id: &AssetUniqueId, pay_interest_only: bool) -> BalanceOf<T> {
		if !CollateralStore::<T>::contains_key(id) {
			return Zero::zero();
		}

		match id.asset_type {
			AssetType::CML => {
				let loan = CollateralStore::<T>::get(id);
				if !pay_interest_only {
					loan.principal
						.saturating_add(loan.interest)
						.saturating_add(Self::calculate_interest(&loan))
				} else {
					loan.interest
						.saturating_add(Self::calculate_interest(&loan))
				}
			}
		}
	}

	pub(crate) fn reset_all_loan_amounts() {
		CollateralStore::<T>::iter().for_each(|(id, _)| {
			CollateralStore::<T>::mutate(&id, |loan| {
				loan.interest += Self::calculate_interest(loan);
			});
		});
	}

	pub fn calculate_interest(
		loan: &Loan<T::AccountId, T::BlockNumber, BalanceOf<T>>,
	) -> BalanceOf<T> {
		loan.principal.saturating_add(loan.interest) * InterestRate::<T>::get() / 10000u32.into()
	}

	pub(crate) fn reset_interest_rate() {
		InterestRate::<T>::set(
			AMMCurveKCoefficient::<T>::get()
				/ T::CurrencyOperations::free_balance(&OperationAccount::<T>::get()),
		);
	}

	pub(crate) fn cml_loan_initial_amount(cml_id: CmlId) -> Result<BalanceOf<T>, DispatchError> {
		let cml = T::CmlOperation::cml_by_id(&cml_id)?;
		let amount = match cml.cml_type() {
			CmlType::A => T::CmlALoanAmount::get(),
			CmlType::B => T::CmlBLoanAmount::get(),
			CmlType::C => T::CmlCLoanAmount::get(),
		};
		Ok(amount)
	}
}

#[cfg(test)]
mod tests {
	use crate::mock::*;
	use crate::*;
	use pallet_cml::{CmlId, CmlStore, DefrostScheduleType, Seed, CML};

	#[test]
	fn cml_need_to_pay_works() {
		new_test_ext().execute_with(|| {
			let id = AssetUniqueId {
				asset_type: AssetType::CML,
				inner_id: from_cml_id(1),
			};
			let loan = Loan {
				start_at: 0,
				term_update_at: 0,
				owner: 1,
				loan_type: CmlType::B,
				principal: 100_000,
				interest: Zero::zero(),
			};
			CollateralStore::<Test>::insert(&id, loan);

			assert_eq!(
				GenesisBank::cml_need_to_pay(&id, false),
				100_000 + 100_000 * BANK_INITIAL_INTEREST_RATE / 10000
			);
			assert_eq!(
				GenesisBank::cml_need_to_pay(&id, true),
				100_000 * BANK_INITIAL_INTEREST_RATE / 10000
			);
		})
	}

	#[test]
	fn reset_interest_rate_works() {
		new_test_ext().execute_with(|| {
			assert_eq!(
				<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
				BANK_INITIAL_BALANCE
			);
			assert_eq!(InterestRate::<Test>::get(), BANK_INITIAL_INTEREST_RATE);

			<Test as Config>::Currency::make_free_balance_be(
				&OperationAccount::<Test>::get(),
				BANK_INITIAL_BALANCE / 2,
			);
			GenesisBank::reset_interest_rate();
			assert_eq!(InterestRate::<Test>::get(), BANK_INITIAL_INTEREST_RATE * 2);

			<Test as Config>::Currency::make_free_balance_be(
				&OperationAccount::<Test>::get(),
				BANK_INITIAL_BALANCE * 2,
			);
			GenesisBank::reset_interest_rate();
			assert_eq!(InterestRate::<Test>::get(), BANK_INITIAL_INTEREST_RATE / 2);

			<Test as Config>::Currency::make_free_balance_be(
				&OperationAccount::<Test>::get(),
				BANK_INITIAL_BALANCE * 2 / 10,
			);
			GenesisBank::reset_interest_rate();
			assert_eq!(InterestRate::<Test>::get(), BANK_INITIAL_INTEREST_RATE * 5);
		})
	}

	#[test]
	fn calculate_interest_works() {
		new_test_ext().execute_with(|| {
			let unit_interest = CML_A_LOAN_AMOUNT * BANK_INITIAL_INTEREST_RATE / 10000;
			let mut loan = new_lien(1, 0);

			loan.principal = CML_A_LOAN_AMOUNT;
			assert_eq!(GenesisBank::calculate_interest(&loan), unit_interest);

			loan.principal = CML_A_LOAN_AMOUNT * 10;
			assert_eq!(GenesisBank::calculate_interest(&loan), unit_interest * 10);

			loan.interest = CML_A_LOAN_AMOUNT;
			assert_eq!(
				GenesisBank::calculate_interest(&loan),
				unit_interest * 10 + unit_interest
			);
		});
	}

	#[test]
	fn try_clean_default_loan_works() {
		new_test_ext().execute_with(|| {
			assert_eq!(
				<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
				BANK_INITIAL_BALANCE
			);

			let user1 = 11;
			let user2 = 22;
			let cml1 = 1;
			let cml2 = 2;
			CmlStore::<Test>::insert(
				cml1,
				CML::from_genesis_seed(seed_from_type(cml1, CmlType::A)),
			);
			CmlStore::<Test>::insert(
				cml2,
				CML::from_genesis_seed(seed_from_type(cml2, CmlType::B)),
			);

			let id1 = new_id(cml1);
			let id2 = new_id(cml2);
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
			assert!(!CmlStore::<Test>::contains_key(&cml1));
			assert!(CmlStore::<Test>::contains_key(&cml2));
			assert_eq!(
				<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
				BANK_INITIAL_BALANCE + CML_A_LOAN_AMOUNT
			);

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
			assert!(!CmlStore::<Test>::contains_key(&cml2));
			assert_eq!(
				<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
				BANK_INITIAL_BALANCE + CML_A_LOAN_AMOUNT + CML_B_LOAN_AMOUNT
			);
		})
	}

	fn new_id(cml_id: CmlId) -> AssetUniqueId {
		AssetUniqueId {
			asset_type: AssetType::CML,
			inner_id: from_cml_id(cml_id),
		}
	}

	fn new_lien(owner: u64, start_at: u64) -> Loan<u64, u64, u128> {
		Loan {
			owner,
			start_at,
			term_update_at: start_at,
			..Default::default()
		}
	}

	fn seed_from_type(id: CmlId, cml_type: CmlType) -> Seed {
		Seed {
			id,
			cml_type,
			defrost_schedule: Some(DefrostScheduleType::Team),
			defrost_time: Some(0),
			lifespan: 100,
			performance: 0,
		}
	}
}
