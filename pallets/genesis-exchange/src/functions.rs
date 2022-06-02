use super::*;

impl<T: genesis_exchange::Config> genesis_exchange::Pallet<T> {
	pub(crate) fn is_interest_period_end(height: T::BlockNumber) -> bool {
		// offset with `InterestPeriodLength` - 1 to void overlapping with staking period
		height % T::InterestPeriodLength::get() == T::InterestPeriodLength::get() - 1u32.into()
	}

	pub(crate) fn accumulate_usd_interest() {
		USDStore::<T>::iter()
			.filter(|(user, _)| !user.eq(&OperationAccount::<T>::get()))
			.for_each(|(user, _)| {
				USDStore::<T>::mutate(user, |balance| {
					*balance = balance
						.saturating_add(*balance * USDInterestRate::<T>::get() / 10000u32.into());
				});
			});
	}

	pub(crate) fn check_buy_tea_to_usd(
		who: &T::AccountId,
		buy_usd_amount: &BalanceOf<T>,
		exchange_remains_usd: &BalanceOf<T>,
		exchange_remains_tea: &BalanceOf<T>,
	) -> DispatchResult {
		ensure!(!buy_usd_amount.is_zero(), Error::<T>::AmountShouldNotBeZero);
		ensure!(
			*buy_usd_amount < *exchange_remains_usd,
			Error::<T>::ExchangeInsufficientUSD
		);

		let deposit_tea_amount =
			Self::delta_deposit_amount(buy_usd_amount, exchange_remains_usd, exchange_remains_tea);
		// The following error should never happen, otherwise there will be an calculation
		//	parameter error.
		ensure!(
			!deposit_tea_amount.is_zero(),
			Error::<T>::InvalidCalculationAmount
		);

		ensure!(
			T::CurrencyOperations::free_balance(who) >= deposit_tea_amount,
			Error::<T>::UserInsufficientTEA
		);

		Ok(())
	}

	pub(crate) fn exchange_buy_tea_to_usd(
		who: &T::AccountId,
		buy_usd_amount: &BalanceOf<T>,
		exchange_remains_usd: &BalanceOf<T>,
		exchange_remains_tea: &BalanceOf<T>,
	) {
		let exchange_account = OperationAccount::<T>::get();
		let deposit_tea_amount =
			Self::delta_deposit_amount(buy_usd_amount, exchange_remains_usd, exchange_remains_tea);
		if let Err(e) = T::CurrencyOperations::transfer(
			who,
			&exchange_account,
			deposit_tea_amount,
			ExistenceRequirement::AllowDeath,
		) {
			// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
			error!("transfer balance failed: {:?}", e);
			return;
		}

		if let Err(e) = Self::transfer_usd_inner(&exchange_account, who, *buy_usd_amount) {
			error!("transfer usd failed: {:?}", e);
			return;
		}

		let (tea_rate, usd_rate, _, _, _) = Self::current_exchange_rate();
		Self::deposit_event(Event::ExchangeSuccess(
			who.clone(),
			deposit_tea_amount,
			*buy_usd_amount,
			tea_rate,
			usd_rate,
		))
	}

	pub(crate) fn check_sell_tea_to_usd(
		who: &T::AccountId,
		sell_tea_amount: &BalanceOf<T>,
		exchange_remains_usd: &BalanceOf<T>,
		exchange_remains_tea: &BalanceOf<T>,
	) -> DispatchResult {
		ensure!(
			!sell_tea_amount.is_zero(),
			Error::<T>::AmountShouldNotBeZero
		);
		ensure!(
			T::CurrencyOperations::free_balance(who) >= *sell_tea_amount,
			Error::<T>::UserInsufficientTEA
		);

		let withdraw_usd_amount = Self::delta_withdraw_amount(
			sell_tea_amount,
			exchange_remains_tea,
			exchange_remains_usd,
		);
		// The following two ensure errors should never happen, otherwise there will be an calculation
		//	parameter error.
		ensure!(
			!withdraw_usd_amount.is_zero(),
			Error::<T>::InvalidCalculationAmount
		);
		ensure!(
			*exchange_remains_usd >= withdraw_usd_amount,
			Error::<T>::InvalidCalculationAmount
		);

		Ok(())
	}

	pub(crate) fn exchange_sell_tea_to_usd(
		who: &T::AccountId,
		sell_tea_amount: &BalanceOf<T>,
		exchange_remains_usd: &BalanceOf<T>,
		exchange_remains_tea: &BalanceOf<T>,
	) {
		let exchange_account = OperationAccount::<T>::get();
		let withdraw_usd_amount = Self::delta_withdraw_amount(
			sell_tea_amount,
			exchange_remains_tea,
			exchange_remains_usd,
		);
		if let Err(e) = T::CurrencyOperations::transfer(
			who,
			&exchange_account,
			*sell_tea_amount,
			ExistenceRequirement::AllowDeath,
		) {
			// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
			error!("transfer balance failed: {:?}", e);
			return;
		}

		if let Err(e) = Self::transfer_usd_inner(&exchange_account, who, withdraw_usd_amount) {
			error!("transfer usd failed: {:?}", e);
			return;
		}

		let (tea_rate, usd_rate, _, _, _) = Self::current_exchange_rate();
		Self::deposit_event(Event::ExchangeSuccess(
			who.clone(),
			*sell_tea_amount,
			withdraw_usd_amount,
			tea_rate,
			usd_rate,
		))
	}

	pub(crate) fn check_buy_usd_to_tea(
		who: &T::AccountId,
		buy_tea_amount: &BalanceOf<T>,
		exchange_remains_usd: &BalanceOf<T>,
		exchange_remains_tea: &BalanceOf<T>,
	) -> DispatchResult {
		ensure!(!buy_tea_amount.is_zero(), Error::<T>::AmountShouldNotBeZero);
		ensure!(
			*buy_tea_amount < *exchange_remains_tea,
			Error::<T>::ExchangeInsufficientTEA
		);

		let deposit_usd_amount =
			Self::delta_deposit_amount(buy_tea_amount, exchange_remains_tea, exchange_remains_usd);
		// The following error should never happen, otherwise there will be an calculation
		//	parameter error.
		ensure!(
			!deposit_usd_amount.is_zero(),
			Error::<T>::InvalidCalculationAmount
		);

		ensure!(
			USDStore::<T>::get(who) >= deposit_usd_amount,
			Error::<T>::UserInsufficientUSD
		);

		Ok(())
	}

	pub(crate) fn exchange_buy_usd_to_tea(
		who: &T::AccountId,
		buy_tea_amount: &BalanceOf<T>,
		exchange_remains_usd: &BalanceOf<T>,
		exchange_remains_tea: &BalanceOf<T>,
	) {
		let exchange_account = OperationAccount::<T>::get();
		let deposit_usd_amount =
			Self::delta_deposit_amount(buy_tea_amount, exchange_remains_tea, exchange_remains_usd);

		if let Err(e) = Self::transfer_usd_inner(who, &exchange_account, deposit_usd_amount) {
			// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
			error!("transfer usd failed: {:?}", e);
			return;
		}

		if let Err(e) = T::CurrencyOperations::transfer(
			&exchange_account,
			who,
			*buy_tea_amount,
			ExistenceRequirement::AllowDeath,
		) {
			error!("transfer balance failed: {:?}", e);
			return;
		}

		let (tea_rate, usd_rate, _, _, _) = Self::current_exchange_rate();
		Self::deposit_event(Event::ExchangeSuccess(
			who.clone(),
			*buy_tea_amount,
			deposit_usd_amount,
			tea_rate,
			usd_rate,
		))
	}

	pub(crate) fn check_sell_usd_to_tea(
		who: &T::AccountId,
		sell_usd_amount: &BalanceOf<T>,
		exchange_remains_usd: &BalanceOf<T>,
		exchange_remains_tea: &BalanceOf<T>,
	) -> DispatchResult {
		ensure!(
			!sell_usd_amount.is_zero(),
			Error::<T>::AmountShouldNotBeZero
		);
		ensure!(
			USDStore::<T>::get(who) >= *sell_usd_amount,
			Error::<T>::UserInsufficientUSD
		);

		let withdraw_tea_amount = Self::delta_withdraw_amount(
			sell_usd_amount,
			exchange_remains_usd,
			exchange_remains_tea,
		);
		// The following two errors should never happen, otherwise there will be an calculation
		//	parameter error.
		ensure!(
			!withdraw_tea_amount.is_zero(),
			Error::<T>::InvalidCalculationAmount
		);
		ensure!(
			*exchange_remains_tea >= withdraw_tea_amount,
			Error::<T>::InvalidCalculationAmount
		);

		Ok(())
	}

	pub(crate) fn exchange_sell_usd_to_tea(
		who: &T::AccountId,
		sell_usd_amount: &BalanceOf<T>,
		exchange_remains_usd: &BalanceOf<T>,
		exchange_remains_tea: &BalanceOf<T>,
	) {
		let exchange_account = OperationAccount::<T>::get();
		let withdraw_tea_amount = Self::delta_withdraw_amount(
			sell_usd_amount,
			exchange_remains_usd,
			exchange_remains_tea,
		);

		if let Err(e) = Self::transfer_usd_inner(who, &exchange_account, *sell_usd_amount) {
			// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
			error!("transfer usd failed: {:?}", e);
			return;
		}

		if let Err(e) = T::CurrencyOperations::transfer(
			&exchange_account,
			who,
			withdraw_tea_amount,
			ExistenceRequirement::AllowDeath,
		) {
			error!("transfer balance failed: {:?}", e);
			return;
		}

		let (tea_rate, usd_rate, _, _, _) = Self::current_exchange_rate();
		Self::deposit_event(Event::ExchangeSuccess(
			who.clone(),
			withdraw_tea_amount,
			*sell_usd_amount,
			tea_rate,
			usd_rate,
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

	pub(crate) fn delta_withdraw_amount(
		deposit_delta: &BalanceOf<T>,
		deposit_total: &BalanceOf<T>,
		withdraw_total: &BalanceOf<T>,
	) -> BalanceOf<T> {
		*withdraw_total - AMMCurveKCoefficient::<T>::get() / (*deposit_total + *deposit_delta)
	}

	pub(crate) fn transfer_usd_inner(
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
			let withdraw_delta = 30_000 * 10_000_000_000 * 100;
			let deposit_delta = GenesisExchange::delta_deposit_amount(
				&withdraw_delta,
				&OPERATION_USD_AMOUNT,
				&OPERATION_TEA_AMOUNT,
			);
			assert_eq!(deposit_delta, 120_000 * 10_000_000_000 * 100);
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
	fn delta_withdraw_amount_with_small_withdraw_delta_works() {
		new_test_ext().execute_with(|| {
			let deposit_delta = 100;
			let withdraw_delta = GenesisExchange::delta_withdraw_amount(
				&deposit_delta,
				&OPERATION_USD_AMOUNT,
				&OPERATION_TEA_AMOUNT,
			);
			assert_eq!(deposit_delta, withdraw_delta);
		})
	}

	#[test]
	fn delta_withdraw_amount_with_large_withdraw_delta_works() {
		new_test_ext().execute_with(|| {
			let deposit_delta = 120_000 * 10_000_000_000 * 100;
			let withdraw_delta = GenesisExchange::delta_withdraw_amount(
				&deposit_delta,
				&OPERATION_USD_AMOUNT,
				&OPERATION_TEA_AMOUNT,
			);
			assert_eq!(withdraw_delta, 30_000 * 10_000_000_000 * 100);
		})
	}

	#[test]
	fn delta_withdraw_amount_return_zero_if_deposit_delta_is_zero() {
		new_test_ext().execute_with(|| {
			let deposit_delta = GenesisExchange::delta_withdraw_amount(
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
			assert_ok!(GenesisExchange::transfer_usd_inner(&user1, &user2, amount));

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
			assert_ok!(GenesisExchange::transfer_usd_inner(&user1, &user2, amount));

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
				GenesisExchange::transfer_usd_inner(&user1, &user2, amount),
				Error::<Test>::InvalidTransferUSDAmount
			);

			assert_eq!(USDStore::<Test>::get(user1), user1_amount);
			assert_eq!(USDStore::<Test>::get(user2), 0);
		})
	}
}
