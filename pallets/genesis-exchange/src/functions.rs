use super::*;

impl<T: genesis_exchange::Config> genesis_exchange::Pallet<T> {
	pub(crate) fn check_tea_to_usd(
		who: &T::AccountId,
		withdraw_amount: &BalanceOf<T>,
		exchange_remains_usd: &BalanceOf<T>,
		exchange_remains_tea: &BalanceOf<T>,
	) -> DispatchResult {
		ensure!(
			*withdraw_amount < *exchange_remains_usd,
			Error::<T>::ExchangeInsufficientUSD
		);
		ensure!(
			!withdraw_amount.is_zero(),
			Error::<T>::WithdrawAmountShouldNotBeZero
		);
		let need_deposit =
			Self::delta_deposit_amount(withdraw_amount, exchange_remains_usd, exchange_remains_tea);
		ensure!(!need_deposit.is_zero(), Error::<T>::InvalidDepositAmount);
		ensure!(
			T::CurrencyOperations::free_balance(who) >= need_deposit,
			Error::<T>::UserInsufficientTEA
		);

		Ok(())
	}

	pub(crate) fn exchange_tea_to_usd(
		who: &T::AccountId,
		withdraw_amount: &BalanceOf<T>,
		exchange_remains_usd: &BalanceOf<T>,
		exchange_remains_tea: &BalanceOf<T>,
	) {
		let exchange_account = OperationAccount::<T>::get();
		let need_deposit =
			Self::delta_deposit_amount(withdraw_amount, exchange_remains_usd, exchange_remains_tea);
		if let Err(e) = T::CurrencyOperations::transfer(
			who,
			&exchange_account,
			need_deposit,
			ExistenceRequirement::AllowDeath,
		) {
			// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
			error!("transfer balance failed: {:?}", e);
			return;
		}

		if let Err(e) = Self::transfer_usd(&exchange_account, who, *withdraw_amount) {
			error!("transfer usd failed: {:?}", e);
			return;
		}

		Self::deposit_event(Event::SellTeaSuccess(
			who.clone(),
			need_deposit,
			*withdraw_amount,
		))
	}

	pub(crate) fn check_usd_to_tea(
		who: &T::AccountId,
		withdraw_amount: &BalanceOf<T>,
		exchange_remains_usd: &BalanceOf<T>,
		exchange_remains_tea: &BalanceOf<T>,
	) -> DispatchResult {
		ensure!(
			*withdraw_amount < *exchange_remains_tea,
			Error::<T>::ExchangeInsufficientTEA
		);
		ensure!(
			!withdraw_amount.is_zero(),
			Error::<T>::WithdrawAmountShouldNotBeZero
		);
		let need_deposit =
			Self::delta_deposit_amount(withdraw_amount, exchange_remains_tea, exchange_remains_usd);
		ensure!(!need_deposit.is_zero(), Error::<T>::InvalidDepositAmount);
		ensure!(
			USDStore::<T>::get(who) >= need_deposit,
			Error::<T>::UserInsufficientUSD
		);

		Ok(())
	}

	pub(crate) fn exchange_usd_to_tea(
		who: &T::AccountId,
		withdraw_amount: &BalanceOf<T>,
		exchange_remains_usd: &BalanceOf<T>,
		exchange_remains_tea: &BalanceOf<T>,
	) {
		let exchange_account = OperationAccount::<T>::get();
		let need_deposit =
			Self::delta_deposit_amount(withdraw_amount, exchange_remains_tea, exchange_remains_usd);

		if let Err(e) = Self::transfer_usd(who, &exchange_account, need_deposit) {
			// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
			error!("transfer usd failed: {:?}", e);
			return;
		}

		if let Err(e) = T::CurrencyOperations::transfer(
			&exchange_account,
			who,
			*withdraw_amount,
			ExistenceRequirement::AllowDeath,
		) {
			error!("transfer balance failed: {:?}", e);
			return;
		}

		Self::deposit_event(Event::BuyTeaSuccess(
			who.clone(),
			*withdraw_amount,
			need_deposit,
		))
	}

	pub(crate) fn delta_deposit_amount(
		withdraw_delta: &BalanceOf<T>,
		withdraw_total: &BalanceOf<T>,
		deposit_total: &BalanceOf<T>,
	) -> BalanceOf<T> {
		if *withdraw_total <= *withdraw_delta {
			return Zero::zero();
		}

		AMMCurveKCoefficient::<T>::get() / (*withdraw_total - *withdraw_delta) - *deposit_total
	}

	pub(crate) fn transfer_usd(
		source: &T::AccountId,
		dest: &T::AccountId,
		value: BalanceOf<T>,
	) -> DispatchResult {
		let mut source_amount = USDStore::<T>::get(source);
		let mut dest_amount = USDStore::<T>::get(dest);
		source_amount = source_amount
			.checked_sub(&value)
			.ok_or(Error::<T>::InvalidTransferUSDAmount)?;
		dest_amount = dest_amount
			.checked_add(&value)
			.ok_or(Error::<T>::InvalidTransferUSDAmount)?;

		USDStore::<T>::insert(source, source_amount);
		USDStore::<T>::insert(dest, dest_amount);
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use crate::mock::*;
	use crate::*;
	use frame_benchmarking::Zero;
	use frame_support::{assert_noop, assert_ok};

	#[test]
	fn delta_deposit_amount_with_small_withdraw_delta_works() {
		new_test_ext().execute_with(|| {
			let withdraw_delta = 100;
			let deposit_delta = GenesisExchange::delta_deposit_amount(
				&withdraw_delta,
				&OPERATION_USD_AMOUNT,
				&OPERATION_TEA_AMOUNT,
			);
			assert_eq!(withdraw_delta, deposit_delta);
		})
	}

	#[test]
	fn delta_deposit_amount_with_large_withdraw_delta_works() {
		new_test_ext().execute_with(|| {
			let withdraw_delta = OPERATION_USD_AMOUNT - 100;
			let deposit_delta = GenesisExchange::delta_deposit_amount(
				&withdraw_delta,
				&OPERATION_USD_AMOUNT,
				&OPERATION_TEA_AMOUNT,
			);
			assert!(withdraw_delta < deposit_delta);
		})
	}

	#[test]
	fn delta_deposit_amount_return_zero_if_withdraw_delta_larger_equal_than_withdraw_amount() {
		new_test_ext().execute_with(|| {
			let deposit_delta = GenesisExchange::delta_deposit_amount(
				&OPERATION_USD_AMOUNT,
				&OPERATION_USD_AMOUNT,
				&OPERATION_TEA_AMOUNT,
			);
			assert!(deposit_delta.is_zero());

			let deposit_delta = GenesisExchange::delta_deposit_amount(
				&(OPERATION_USD_AMOUNT + 1),
				&OPERATION_USD_AMOUNT,
				&OPERATION_TEA_AMOUNT,
			);
			assert!(deposit_delta.is_zero());
		})
	}

	#[test]
	fn delta_deposit_amount_return_zero_if_withdraw_delta_is_zero() {
		new_test_ext().execute_with(|| {
			let deposit_delta = GenesisExchange::delta_deposit_amount(
				&0,
				&OPERATION_USD_AMOUNT,
				&OPERATION_TEA_AMOUNT,
			);
			assert!(deposit_delta.is_zero());
		})
	}

	#[test]
	fn transfer_usd_works() {
		new_test_ext().execute_with(|| {
			let user1 = 1;
			let user2 = 2;
			let user1_amount = 10000;
			let user2_amount = 10000;

			USDStore::<Test>::insert(user1, user1_amount);
			USDStore::<Test>::insert(user2, user2_amount);

			let amount = 1000;
			assert_ok!(GenesisExchange::transfer_usd(&user1, &user2, amount));

			assert_eq!(USDStore::<Test>::get(user1), user1_amount - amount);
			assert_eq!(USDStore::<Test>::get(user2), user2_amount + amount);
		})
	}

	#[test]
	fn transfer_usd_works_if_dest_user_not_exist() {
		new_test_ext().execute_with(|| {
			let user1 = 1;
			let user2 = 2;
			let user1_amount = 10000;

			USDStore::<Test>::insert(user1, user1_amount);
			assert!(!USDStore::<Test>::contains_key(user2));

			let amount = 1000;
			assert_ok!(GenesisExchange::transfer_usd(&user1, &user2, amount));

			assert_eq!(USDStore::<Test>::get(user1), user1_amount - amount);
			assert_eq!(USDStore::<Test>::get(user2), amount);
		})
	}

	#[test]
	fn transfer_usd_fail_if_source_user_amount_is_not_enough() {
		new_test_ext().execute_with(|| {
			let user1 = 1;
			let user2 = 2;
			let user1_amount = 10000;

			USDStore::<Test>::insert(user1, user1_amount);

			let amount = user1_amount + 1;
			assert_noop!(
				GenesisExchange::transfer_usd(&user1, &user2, amount),
				Error::<Test>::InvalidTransferUSDAmount
			);

			assert_eq!(USDStore::<Test>::get(user1), user1_amount);
			assert_eq!(USDStore::<Test>::get(user2), 0);
		})
	}
}
