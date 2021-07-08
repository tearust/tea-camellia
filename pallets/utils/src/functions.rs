use super::*;
use sp_runtime::traits::Zero;

impl<T: utils::Config> CommonUtils for utils::Pallet<T> {
	type AccountId = T::AccountId;

	fn generate_random(sender: Self::AccountId, salt: &RandomSalt) -> U256 {
		let random_seed = <pallet_randomness_collective_flip::Module<T>>::random_seed();
		let payload = (
			random_seed,
			sender,
			salt,
			frame_system::Pallet::<T>::block_number(),
		);
		payload.using_encoded(blake2_256).into()
	}
}

impl<T: utils::Config> CurrencyOperations for utils::Pallet<T> {
	type AccountId = T::AccountId;
	type Balance = <<T as utils::Config>::Currency as Currency<
		<T as frame_system::Config>::AccountId,
	>>::Balance;

	fn total_issuance() -> Self::Balance {
		T::Currency::total_issuance()
	}

	fn minimum_balance() -> Self::Balance {
		T::Currency::minimum_balance()
	}

	fn total_balance(who: &Self::AccountId) -> Self::Balance {
		T::Currency::total_balance(who)
	}

	fn free_balance(who: &Self::AccountId) -> Self::Balance {
		T::Currency::free_balance(who)
	}

	fn transfer(
		source: &Self::AccountId,
		dest: &Self::AccountId,
		value: Self::Balance,
		existence_requirement: ExistenceRequirement,
	) -> DispatchResult {
		T::Currency::transfer(source, dest, value, existence_requirement)
	}

	fn reserved_balance(who: &Self::AccountId) -> Self::Balance {
		T::Currency::reserved_balance(who)
	}

	fn reserve(who: &Self::AccountId, amount: Self::Balance) -> DispatchResult {
		T::Currency::reserve(who, amount)
	}

	fn unreserve(who: &Self::AccountId, value: Self::Balance) -> Self::Balance {
		T::Currency::unreserve(who, value)
	}

	fn slash(who: &Self::AccountId, value: Self::Balance) -> Self::Balance {
		let (imbalance, balance) = T::Currency::slash(who, value);
		T::Slash::on_unbalanced(imbalance);
		balance
	}

	fn slash_reserved(who: &Self::AccountId, value: Self::Balance) -> Self::Balance {
		let (imbalance, balance) = T::Currency::slash_reserved(who, value);
		T::Slash::on_unbalanced(imbalance);
		balance
	}

	fn repatriate_reserved(
		slashed: &Self::AccountId,
		beneficiary: &Self::AccountId,
		value: Self::Balance,
	) -> Result<Self::Balance, DispatchError> {
		T::Currency::repatriate_reserved(slashed, beneficiary, value, BalanceStatus::Free)
	}

	fn repatriate_reserved_batch(
		slashed: &Self::AccountId,
		beneficiary_list: &Vec<Self::AccountId>,
		value_list: &Vec<Self::Balance>,
	) -> DispatchResult {
		ensure!(
			beneficiary_list.len() == value_list.len(),
			Error::<T>::MismatchedRepatriateBatchList
		);

		let mut total_repatriate = Self::Balance::zero();
		for value in value_list {
			total_repatriate += *value;
		}
		ensure!(
			Self::reserved_balance(slashed) >= total_repatriate,
			Error::<T>::InsufficientRepatriateBalance
		);
		// prevent repatriate to accounts not exist
		for account in beneficiary_list {
			ensure!(
				Self::free_balance(account) != BalanceOf::<T>::zero(),
				Error::<T>::AccountNotExist
			);
		}

		for i in 0..beneficiary_list.len() {
			Self::repatriate_reserved(
				slashed,
				beneficiary_list.get(i).ok_or(DispatchError::Other(
					"failed to get beneficiary from beneficiary_list",
				))?,
				*value_list
					.get(i)
					.ok_or(DispatchError::Other("failed to get value from value_list"))?,
			)?;
		}
		Ok(())
	}

	fn deposit_creating(who: &Self::AccountId, value: Self::Balance) {
		let imbalance = T::Currency::deposit_creating(who, value);
		T::Reward::on_unbalanced(imbalance);
	}
}

#[cfg(test)]
mod tests {
	use crate::{mock::*, CommonUtils, CurrencyOperations, Error};
	use frame_support::{
		assert_noop, assert_ok,
		traits::{
			Currency,
			ExistenceRequirement::{AllowDeath, KeepAlive},
		},
	};
	use pallet_balances::Error as BalanceError;

	#[test]
	fn generate_random_works() {
		new_test_ext().execute_with(|| {
			frame_system::Pallet::<Test>::set_block_number(100);

			// same (account + block_number + salt)
			let random1 = Utils::generate_random(1, &vec![1]);
			let random2 = Utils::generate_random(1, &vec![1]);
			assert_eq!(random1, random2);

			// different salt
			let random1 = Utils::generate_random(1, &vec![1]);
			let random2 = Utils::generate_random(1, &vec![2]);
			assert_ne!(random1, random2);

			// different account
			let random1 = Utils::generate_random(1, &vec![1]);
			let random2 = Utils::generate_random(2, &vec![1]);
			assert_ne!(random1, random2);

			// different block height
			frame_system::Pallet::<Test>::set_block_number(100);
			let random1 = Utils::generate_random(1, &vec![1]);
			frame_system::Pallet::<Test>::set_block_number(101);
			let random2 = <Utils as CommonUtils>::generate_random(1, &vec![1]);
			assert_ne!(random1, random2);
		})
	}

	#[test]
	fn all_kinds_balances_works() {
		new_test_ext().execute_with(|| {
			let _ = Balances::deposit_creating(&1, 10);
			let _ = Balances::deposit_creating(&2, 10);

			assert_ok!(Utils::reserve(&1, 4));

			assert_eq!(Utils::total_issuance(), 20);
			assert_eq!(Utils::minimum_balance(), EXISTENTIAL_DEPOSIT);

			assert_eq!(Utils::total_balance(&1), 10);
			assert_eq!(Utils::free_balance(&1), 6);
			assert_eq!(Utils::reserved_balance(&1), 4);

			assert_eq!(Utils::total_balance(&2), 10);
			assert_eq!(Utils::free_balance(&2), 10);
			assert_eq!(Utils::reserved_balance(&2), 0);

			// not exist account balances should be zero
			assert!(!System::account_exists(&3));
			assert_eq!(Utils::free_balance(&3), 0);
			assert_eq!(Utils::reserved_balance(&3), 0);
		})
	}

	#[test]
	fn free_balance_only_works() {
		// basic free balance operations
		new_test_ext().execute_with(|| {
			let _ = Balances::deposit_creating(&1, 10);

			assert!(!System::account_exists(&2));
			assert_ok!(Utils::transfer(&1, &2, 4, AllowDeath));
			assert!(System::account_exists(&2));

			assert_eq!(Utils::free_balance(&1), 6);
			assert_eq!(Utils::free_balance(&2), 4);
		});

		// transfer to let source account lower than existence requirement
		new_test_ext().execute_with(|| {
			let _ = Balances::deposit_creating(&1, 10);

			assert!(System::account_exists(&1));
			assert!(!System::account_exists(&2));
			assert_ok!(Utils::transfer(&1, &2, 10, AllowDeath));
			assert!(!System::account_exists(&1));
			assert!(System::account_exists(&2));

			assert_eq!(Utils::free_balance(&2), 10);
		});

		// transfer should fail if free balance lower than transfer value
		new_test_ext().execute_with(|| {
			let _ = Balances::deposit_creating(&1, 10);

			assert_noop!(
				Utils::transfer(&1, &2, 11, AllowDeath),
				BalanceError::<Test>::InsufficientBalance
			);
			assert_eq!(Utils::free_balance(&1), 10);
		});

		// transfer should fail if left free balance lower than existence requirement
		new_test_ext().execute_with(|| {
			let _ = Balances::deposit_creating(&1, 10);

			assert_noop!(
				Utils::transfer(&1, &2, 10, KeepAlive),
				BalanceError::<Test>::KeepAlive
			);
			assert_eq!(Utils::free_balance(&1), 10);
		});
	}

	#[test]
	fn normal_reserve_unreserve_works() {
		new_test_ext().execute_with(|| {
			let _ = Balances::deposit_creating(&1, 10);

			assert_ok!(Utils::reserve(&1, 4));
			assert_eq!(Utils::free_balance(&1), 6);
			assert_eq!(Utils::reserved_balance(&1), 4);

			assert_eq!(Utils::unreserve(&1, 3), 0);
			assert_eq!(Utils::free_balance(&1), 9);
			assert_eq!(Utils::reserved_balance(&1), 1);

			assert_eq!(Utils::unreserve(&1, 1), 0);
			assert_eq!(Utils::free_balance(&1), 10);
			assert_eq!(Utils::reserved_balance(&1), 0);
		});
	}

	#[test]
	fn reserve_to_left_free_balance_be_zero_works() {
		new_test_ext().execute_with(|| {
			let _ = Balances::deposit_creating(&1, 10);

			assert_ok!(Utils::reserve(&1, 10));
			assert_eq!(Utils::free_balance(&1), 0);
			assert_eq!(Utils::reserved_balance(&1), 10);

			assert!(System::account_exists(&1));
		});
	}

	#[test]
	fn reserve_amount_larger_than_free_balance_should_fail() {
		new_test_ext().execute_with(|| {
			let _ = Balances::deposit_creating(&1, 10);

			assert_noop!(
				Utils::reserve(&1, 11),
				BalanceError::<Test>::InsufficientBalance
			);
			assert_eq!(Utils::free_balance(&1), 10);
			assert_eq!(Utils::reserved_balance(&1), 0);
		});
	}

	#[test]
	fn unreserve_amount_larger_than_reserved_balance_should_failed() {
		new_test_ext().execute_with(|| {
			let _ = Balances::deposit_creating(&1, 10);

			assert_ok!(Utils::reserve(&1, 4));
			assert_eq!(Utils::free_balance(&1), 6);
			assert_eq!(Utils::reserved_balance(&1), 4);

			assert_eq!(Utils::unreserve(&1, 9), 5);

			assert_eq!(Utils::free_balance(&1), 10);
			assert_eq!(Utils::reserved_balance(&1), 0);
		});
	}

	#[test]
	fn slash_works() {
		new_test_ext().execute_with(|| {
			let _ = Balances::deposit_creating(&1, 10);

			assert_eq!(Utils::slash(&1, 3), 0);
			assert_eq!(Utils::free_balance(&1), 7);
			assert_eq!(Utils::reserved_balance(&1), 0);
		})
	}

	#[test]
	fn slash_can_delete_slashed_account() {
		new_test_ext().execute_with(|| {
			let _ = Balances::deposit_creating(&1, 10);

			assert_eq!(Utils::slash(&1, 10), 0);
			assert!(!System::account_exists(&1));
		})
	}

	#[test]
	fn slash_amount_more_than_free_balance_works() {
		new_test_ext().execute_with(|| {
			let _ = Balances::deposit_creating(&1, 10);

			assert_eq!(Utils::slash(&1, 15), 5);
			assert_eq!(Utils::free_balance(&1), 0);
			assert!(!System::account_exists(&1));
		})
	}

	#[test]
	fn slash_can_effect_both_free_and_reserve_balance() {
		new_test_ext().execute_with(|| {
			let _ = Balances::deposit_creating(&1, 10);

			assert_ok!(Utils::reserve(&1, 4));
			assert_eq!(Utils::slash(&1, 9), 0);

			assert_eq!(Utils::free_balance(&1), 0);
			assert_eq!(Utils::reserved_balance(&1), 1);
		})
	}

	#[test]
	fn slash_reserved_works() {
		new_test_ext().execute_with(|| {
			let _ = Balances::deposit_creating(&1, 10);

			assert_ok!(Utils::reserve(&1, 4));
			assert_eq!(Utils::slash_reserved(&1, 3), 0);

			assert_eq!(Utils::free_balance(&1), 6);
			assert_eq!(Utils::reserved_balance(&1), 1);
		})
	}

	#[test]
	fn slash_reserved_can_delete_slashed_account() {
		new_test_ext().execute_with(|| {
			let _ = Balances::deposit_creating(&1, 10);

			assert_ok!(Utils::reserve(&1, 10));
			assert_eq!(Utils::slash_reserved(&1, 10), 0);

			assert!(!System::account_exists(&1));
		})
	}

	#[test]
	fn slash_amount_more_than_reserved_balance_works() {
		new_test_ext().execute_with(|| {
			let _ = Balances::deposit_creating(&1, 10);

			assert_ok!(Utils::reserve(&1, 4));
			assert_eq!(Utils::slash_reserved(&1, 5), 1);

			assert_eq!(Utils::free_balance(&1), 6);
			assert_eq!(Utils::reserved_balance(&1), 0);
		})
	}

	#[test]
	fn repatriate_reserved_works() {
		new_test_ext().execute_with(|| {
			let _ = Balances::deposit_creating(&1, 10);
			let _ = Balances::deposit_creating(&2, 10);

			assert_ok!(Utils::reserve(&1, 4));
			assert_eq!(Utils::repatriate_reserved(&1, &2, 3), Ok(0));

			assert_eq!(Utils::free_balance(&1), 6);
			assert_eq!(Utils::reserved_balance(&1), 1);

			assert_eq!(Utils::free_balance(&2), 13);
			assert_eq!(Utils::reserved_balance(&2), 0);
		})
	}

	#[test]
	fn repatriate_reserved_to_not_exist_account_should_fail() {
		new_test_ext().execute_with(|| {
			let _ = Balances::deposit_creating(&1, 10);

			assert_ok!(Utils::reserve(&1, 4));
			assert_noop!(
				Utils::repatriate_reserved(&1, &2, 3),
				BalanceError::<Test>::DeadAccount
			);

			assert_eq!(Utils::free_balance(&1), 6);
			assert_eq!(Utils::reserved_balance(&1), 4);
			assert!(!System::account_exists(&2));
		})
	}

	#[test]
	fn repatriate_reserved_batch_works() {
		new_test_ext().execute_with(|| {
			let _ = Balances::deposit_creating(&1, 10);
			let _ = Balances::deposit_creating(&2, 10);
			let _ = Balances::deposit_creating(&3, 10);

			assert_ok!(Utils::reserve(&1, 9));
			assert_ok!(Utils::repatriate_reserved_batch(
				&1,
				&vec![2, 3],
				&vec![3, 4]
			));

			assert_eq!(Utils::free_balance(&1), 1);
			assert_eq!(Utils::reserved_balance(&1), 2);

			assert_eq!(Utils::free_balance(&2), 13);
			assert_eq!(Utils::free_balance(&3), 14);
		})
	}

	#[test]
	fn repatriate_reserved_batch_with_mismatched_length_should_fail() {
		new_test_ext().execute_with(|| {
			assert_noop!(
				Utils::repatriate_reserved_batch(&1, &vec![2, 3], &vec![3, 4, 5]),
				Error::<Test>::MismatchedRepatriateBatchList,
			);
		})
	}

	#[test]
	fn repatriate_reserved_batch_with_insufficient_reserved_balance_should_fail() {
		new_test_ext().execute_with(|| {
			let _ = Balances::deposit_creating(&1, 10);
			assert_ok!(Utils::reserve(&1, 9));

			assert_noop!(
				Utils::repatriate_reserved_batch(&1, &vec![2, 3], &vec![5, 5]),
				Error::<Test>::InsufficientRepatriateBalance,
			);
			assert_eq!(Utils::reserved_balance(&1), 9);
		})
	}

	#[test]
	fn repatriate_reserved_batch_with_not_exist_account_should_fail() {
		new_test_ext().execute_with(|| {
			let _ = Balances::deposit_creating(&1, 10);
			let _ = Balances::deposit_creating(&2, 10);

			assert_ok!(Utils::reserve(&1, 9));

			// not exist account at the first
			assert_noop!(
				Utils::repatriate_reserved_batch(&1, &vec![3, 2], &vec![3, 4]),
				Error::<Test>::AccountNotExist,
			);
			assert_eq!(Utils::reserved_balance(&1), 9);

			// not exist account at the second
			assert_noop!(
				Utils::repatriate_reserved_batch(&1, &vec![2, 3], &vec![3, 4]),
				Error::<Test>::AccountNotExist,
			);
			assert_eq!(Utils::reserved_balance(&1), 9);
		})
	}

	#[test]
	fn deposit_creating_works() {
		new_test_ext().execute_with(|| {
			Utils::deposit_creating(&1, 10);
			Utils::deposit_creating(&2, 10);

			assert!(System::account_exists(&1));
			assert!(System::account_exists(&2));

			assert_eq!(Utils::free_balance(&1), 10);
			assert_eq!(Utils::free_balance(&2), 10);

			assert_eq!(Utils::total_issuance(), 20);
		})
	}
}
