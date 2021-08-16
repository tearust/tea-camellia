use super::*;

impl<T: cml::Config> CmlOperation for cml::Pallet<T> {
	type AccountId = T::AccountId;
	type Balance = BalanceOf<T>;
	type BlockNumber = T::BlockNumber;
	type FreshDuration = T::SeedFreshDuration;

	/// Get cml with given cml ID, if not exist will throw the `NotFoundCML` error.
	fn cml_by_id(
		cml_id: &CmlId,
	) -> Result<
		CML<Self::AccountId, Self::BlockNumber, Self::Balance, Self::FreshDuration>,
		DispatchError,
	> {
		ensure!(CmlStore::<T>::contains_key(cml_id), Error::<T>::NotFoundCML);
		Ok(CmlStore::<T>::get(cml_id))
	}

	/// Check if the given CML not belongs to specified account.
	fn check_belongs(cml_id: &u64, who: &Self::AccountId) -> DispatchResult {
		ensure!(CmlStore::<T>::contains_key(cml_id), Error::<T>::NotFoundCML);
		ensure!(
			UserCmlStore::<T>::contains_key(who, cml_id),
			Error::<T>::CMLOwnerInvalid
		);
		Ok(())
	}

	/// Check if `from_account` can transfer the specifying CML to `target_account`.
	fn check_transfer_cml_to_other(
		from_account: &Self::AccountId,
		cml_id: &CmlId,
		target_account: &Self::AccountId,
	) -> DispatchResult {
		Self::check_belongs(cml_id, from_account)?;
		ensure!(
			Self::user_credit_amount(from_account, cml_id).is_zero(),
			Error::<T>::OperationForbiddenWithCredit
		);

		let cml = CmlStore::<T>::get(cml_id);
		if cml.is_mining() {
			ensure!(
				T::CurrencyOperations::free_balance(target_account) >= T::StakingPrice::get(),
				Error::<T>::InsufficientFreeBalance
			);
		}
		Ok(())
	}

	/// Transfer `from_account` the specifying CML to `target_account`.
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

	/// Get the deposit price if CML is mining, or `None` otherwise.
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

	/// Get credit amount of the given user.
	fn user_credit_amount(account_id: &Self::AccountId, cml_id: &CmlId) -> Self::Balance {
		if !GenesisMinerCreditStore::<T>::contains_key(account_id, cml_id) {
			return Zero::zero();
		}

		GenesisMinerCreditStore::<T>::get(account_id, cml_id)
	}

	fn user_credits(who: &Self::AccountId) -> Vec<(CmlId, Self::Balance)> {
		GenesisMinerCreditStore::<T>::iter_prefix(who).collect()
	}

	/// Add a cml into `CmlStore` and bind the CML with the given user.
	fn add_cml(
		who: &Self::AccountId,
		cml: CML<Self::AccountId, Self::BlockNumber, Self::Balance, Self::FreshDuration>,
	) {
		let cml_id = cml.id();
		CmlStore::<T>::insert(cml_id, cml);
		UserCmlStore::<T>::insert(who, cml_id, ());
	}

	/// Remove cml from `CmlStore` and unbind cml with its owner.
	fn remove_cml(cml_id: CmlId) {
		let cml = CmlStore::<T>::take(cml_id);
		if let Some(owner) = cml.owner() {
			UserCmlStore::<T>::remove(owner, cml_id);
		}
	}

	/// Get all user owned cml list.
	fn user_owned_cmls(who: &Self::AccountId) -> Vec<CmlId> {
		Self::user_cml_list(who)
	}

	/// Estimate reward according to total task point and miner task point
	///
	/// Both `total_point` and `miner_task_point` returns points in milli-unit, that means
	///  each 1000 milli-points will be treated as 1 task point in reward calculation.
	fn estimate_reward_statements<X, Y>(
		total_point: X,
		miner_task_point: Y,
	) -> Vec<(Self::AccountId, CmlId, Self::Balance)>
	where
		X: FnOnce() -> ServiceTaskPoint,
		Y: Fn(CmlId) -> ServiceTaskPoint,
	{
		let current_height = frame_system::Pallet::<T>::block_number();
		let total_task_point = total_point();

		let mut reward_statements = Vec::new();

		let snapshots: Vec<(CmlId, Vec<StakingSnapshotItem<T::AccountId>>)> =
			ActiveStakingSnapshot::<T>::iter().collect();
		for (cml_id, snapshot_items) in snapshots {
			let miner_task_point = miner_task_point(cml_id);
			let (performance, _) = Self::miner_performance(cml_id, &current_height);
			let miner_total_reward = T::StakingEconomics::total_staking_rewards_of_miner(
				miner_task_point,
				total_task_point,
				performance,
			);
			let total_staking_point =
				T::StakingEconomics::miner_total_staking_weight(&snapshot_items);

			for item in snapshot_items.iter() {
				let reward = T::StakingEconomics::single_staking_reward(
					miner_total_reward,
					total_staking_point,
					item,
				);
				reward_statements.push((item.owner.clone(), cml_id, reward));
			}
		}

		reward_statements
	}

	fn current_mining_cmls() -> Vec<CmlId> {
		MinerItemStore::<T>::iter()
			.map(|(_, miner_item)| miner_item.cml_id)
			.collect()
	}
}

#[cfg(test)]
mod tests {
	use crate::{
		mock::*, tests::seed_from_lifespan, CmlOperation, CmlStore, Config, StakingProperties,
		TreeProperties, UserCmlStore, CML,
	};
	use frame_support::{assert_ok, traits::Currency};
	use pallet_utils::CurrencyOperations;

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

	#[test]
	fn transfer_cml_to_other_works_if_sender_reserved_balance_has_been_slashed() {
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

			assert_eq!(
				<Test as Config>::Currency::free_balance(&owner),
				user_origin_balance - STAKING_PRICE
			);
			assert_eq!(
				<Test as Config>::Currency::reserved_balance(&owner),
				STAKING_PRICE
			);

			let slashed_amount = STAKING_PRICE / 2;
			Utils::slash_reserved(&owner, slashed_amount);

			assert_ok!(Cml::check_transfer_cml_to_other(&owner, &cml_id, &user_id));
			Cml::transfer_cml_to_other(&owner, &cml_id, &user_id);

			assert_eq!(
				<Test as Config>::Currency::free_balance(&owner),
				user_origin_balance - slashed_amount
			);
			assert_eq!(<Test as Config>::Currency::reserved_balance(&owner), 0);
		})
	}
}
