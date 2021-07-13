use super::*;

impl<T: cml::Config> CmlOperation for cml::Pallet<T> {
	type AccountId = T::AccountId;
	type Balance = BalanceOf<T>;
	type BlockNumber = T::BlockNumber;
	type FreshDuration = T::SeedFreshDuration;

	fn cml_by_id(
		cml_id: &CmlId,
	) -> Result<
		CML<Self::AccountId, Self::BlockNumber, Self::Balance, Self::FreshDuration>,
		DispatchError,
	> {
		ensure!(CmlStore::<T>::contains_key(cml_id), Error::<T>::NotFoundCML);
		Ok(CmlStore::<T>::get(cml_id))
	}

	fn check_belongs(cml_id: &u64, who: &Self::AccountId) -> DispatchResult {
		ensure!(CmlStore::<T>::contains_key(cml_id), Error::<T>::NotFoundCML);
		ensure!(
			UserCmlStore::<T>::contains_key(who, cml_id),
			Error::<T>::CMLOwnerInvalid
		);
		Ok(())
	}

	fn check_transfer_cml_to_other(
		from_account: &Self::AccountId,
		cml_id: &CmlId,
		target_account: &Self::AccountId,
	) -> DispatchResult {
		Self::check_belongs(cml_id, from_account)?;
		ensure!(
			Self::user_credit_amount(from_account).is_zero(),
			Error::<T>::CannotTransferCmlWithCredit
		);

		let cml = CmlStore::<T>::get(cml_id);
		if cml.is_mining() {
			ensure!(
				T::CurrencyOperations::free_balance(target_account) >= T::StakingPrice::get(),
				Error::<T>::InsufficientFreeBalance
			);
			ensure!(
				T::CurrencyOperations::reserved_balance(from_account) >= T::StakingPrice::get(),
				Error::<T>::InsufficientReservedBalance
			);
		}
		Ok(())
	}

	fn transfer_cml_to_other(
		from_account: &Self::AccountId,
		cml_id: &CmlId,
		target_account: &Self::AccountId,
	) {
		if Self::check_transfer_cml_to_other(from_account, cml_id, target_account).is_err() {
			return;
		}

		let success = CmlStore::<T>::mutate(&cml_id, |cml| {
			cml.set_owner(target_account);
			if cml.is_mining() {
				let amount = match cml.staking_slots().get(0) {
					Some(item) => item.amount.clone(),
					None => None,
				};
				if amount.is_none() {
					return false;
				}

				return if let Ok(staking_item) =
					Self::create_balance_staking(target_account, amount.unwrap())
				{
					T::CurrencyOperations::unreserve(from_account, amount.unwrap());
					cml.swap_first_slot(staking_item);
					true
				} else {
					false
				};
			}
			true
		});
		// see https://github.com/tearust/tea-camellia/issues/13
		if !success {
			return;
		}

		// remove from from UserCmlStore
		UserCmlStore::<T>::remove(from_account, cml_id);
		UserCmlStore::<T>::insert(target_account, cml_id, ());
	}

	fn cml_deposit_price(cml_id: &CmlId) -> Option<Self::Balance> {
		if !CmlStore::<T>::contains_key(cml_id) {
			return None;
		}

		let cml = CmlStore::<T>::get(cml_id);
		if cml.is_mining() {
			if let Some(staking_item) = cml.staking_slots().get(0) {
				return staking_item.amount.clone();
			}
		}
		None
	}

	fn user_credit_amount(account_id: &Self::AccountId) -> Self::Balance {
		if !GenesisMinerCreditStore::<T>::contains_key(account_id) {
			return Zero::zero();
		}

		GenesisMinerCreditStore::<T>::get(account_id)
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		mock::*, tests::seed_from_lifespan, CmlOperation, CmlStore, Config, StakingProperties,
		TreeProperties, UserCmlStore, CML,
	};
	use frame_support::{assert_ok, traits::Currency};

	#[test]
	fn transfer_cml_to_other_works() {
		new_test_ext().execute_with(|| {
			let user_id = 11;
			let owner = 22;
			let user_origin_balance = 100 * 1000;
			let owner_origin_balance = 100 * 1000;
			<Test as Config>::Currency::make_free_balance_be(&user_id, user_origin_balance);
			<Test as Config>::Currency::make_free_balance_be(&owner, owner_origin_balance);

			let cml_id = 1;
			let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
			UserCmlStore::<Test>::insert(owner, cml_id, ());
			CmlStore::<Test>::insert(cml_id, cml);

			assert_ok!(Cml::start_mining(
				Origin::signed(owner),
				cml_id,
				[1u8; 32],
				b"miner_ip".to_vec()
			));
			assert!(UserCmlStore::<Test>::contains_key(owner, cml_id));
			let cml = CmlStore::<Test>::get(cml_id);
			assert_eq!(cml.staking_slots()[0].owner, owner);

			assert_ok!(Cml::check_transfer_cml_to_other(&owner, &cml_id, &user_id));
			Cml::transfer_cml_to_other(&owner, &cml_id, &user_id);

			assert!(UserCmlStore::<Test>::contains_key(user_id, cml_id));
			let cml = CmlStore::<Test>::get(cml_id);
			assert_eq!(cml.owner(), Some(&user_id));
			assert_eq!(cml.staking_slots()[0].owner, user_id);

			assert_eq!(
				<Test as Config>::Currency::free_balance(&user_id),
				user_origin_balance - STAKING_PRICE
			);
			assert_eq!(
				<Test as Config>::Currency::reserved_balance(&user_id),
				STAKING_PRICE
			);

			assert_eq!(
				<Test as Config>::Currency::free_balance(&owner),
				user_origin_balance
			);
			assert_eq!(<Test as Config>::Currency::reserved_balance(&owner), 0);
		})
	}
}