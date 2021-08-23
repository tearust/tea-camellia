use super::*;

pub(crate) const CALCULATION_PRECISION: u32 = 100000000;

impl<T: bounding_curve::Config> bounding_curve::Pallet<T> {
	pub fn next_id() -> TAppId {
		LastTAppId::<T>::mutate(|id| {
			if *id < u64::MAX {
				*id += 1;
			} else {
				*id = 1;
			}

			*id
		})
	}

	pub fn allocate_buy_tea_amount(
		who: &T::AccountId,
		tapp_id: TAppId,
		tapp_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		let deposit_tea_amount = Self::calculate_buy_amount(tapp_id, tapp_amount);
		let reserved_tea_amount = Self::calculate_reserve_amount(tapp_id, tapp_amount);

		if let Err(e) = T::CurrencyOperations::transfer(
			who,
			&OperationAccount::<T>::get(),
			reserved_tea_amount,
			ExistenceRequirement::AllowDeath,
		) {
			// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
			log::error!("transfer free balance failed: {:?}", e);
			return Zero::zero();
		}

		if let Err(e) = T::CurrencyOperations::transfer(
			who,
			&TAppBoundingCurve::<T>::get(tapp_id).owner,
			deposit_tea_amount - reserved_tea_amount,
			ExistenceRequirement::AllowDeath,
		) {
			// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
			log::error!("transfer free balance failed: {:?}", e);
			return Zero::zero();
		}

		deposit_tea_amount
	}

	pub fn buy_token_inner(
		who: &T::AccountId,
		tapp_id: TAppId,
		tapp_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		let deposit_tea_amount = Self::allocate_buy_tea_amount(who, tapp_id, tapp_amount);

		AccountTable::<T>::mutate(who, tapp_id, |amount| {
			*amount = amount.saturating_add(tapp_amount);
		});
		TotalSupplyTable::<T>::mutate(tapp_id, |amount| {
			*amount = amount.saturating_add(tapp_amount);
		});

		deposit_tea_amount
	}

	pub fn sell_token_inner(
		who: &T::AccountId,
		tapp_id: TAppId,
		tapp_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		if let Err(e) = AccountTable::<T>::mutate(who, tapp_id, |amount| {
			match amount.checked_sub(&tapp_amount) {
				Some(a) => {
					*amount = a;
					Ok(())
				}
				None => Err("account tapp token is not enough"),
			}
		}) {
			// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
			log::error!("{}", e);
			return Zero::zero();
		}

		match Self::calculate_sell_amount(tapp_id, tapp_amount) {
			Ok(deposit_tea_amount) => {
				if let Err(e) = T::CurrencyOperations::transfer(
					&OperationAccount::<T>::get(),
					who,
					deposit_tea_amount,
					ExistenceRequirement::AllowDeath,
				) {
					// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
					log::error!("transfer free balance failed: {:?}", e);
					return Zero::zero();
				}

				TotalSupplyTable::<T>::mutate(tapp_id, |amount| {
					*amount = amount.saturating_sub(tapp_amount);
				});
				Self::try_clean_tapp_related(who, tapp_id);

				deposit_tea_amount
			}
			Err(e) => {
				// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
				log::error!("calculating sell amount failed: {:?}", e);
				return Zero::zero();
			}
		}
	}

	pub fn try_clean_tapp_related(who: &T::AccountId, tapp_id: TAppId) {
		if AccountTable::<T>::get(who, tapp_id).is_zero() {
			AccountTable::<T>::remove(who, tapp_id);
		}
		if TotalSupplyTable::<T>::get(tapp_id).is_zero() {
			TotalSupplyTable::<T>::remove(tapp_id);
			let item = TAppBoundingCurve::<T>::take(tapp_id);
			TAppNames::<T>::remove(item.name);
			TAppTickers::<T>::remove(item.ticker);
		}
	}

	pub(crate) fn distribute_to_investors(tapp_id: TAppId, distributing_amount: BalanceOf<T>) {
		// todo replace total amount with total supply later if calculation result is correct
		let (investors, total_amount) = Self::tapp_investors(tapp_id);
		if !approximately_equals::<T>(
			total_amount,
			TotalSupplyTable::<T>::get(tapp_id),
			CALCULATION_PRECISION.into(),
		) {
			log::error!(
				"distributing calculate total amount error: calculated result is: {:?}, \
				total supply is {:?}",
				total_amount,
				TotalSupplyTable::<T>::get(tapp_id)
			);
		}

		investors.iter().for_each(|account| {
			AccountTable::<T>::mutate(account, tapp_id, |user_amount| {
				*user_amount =
					user_amount.saturating_add(distributing_amount * (*user_amount) / total_amount);
			});
		});

		TotalSupplyTable::<T>::mutate(tapp_id, |amount| {
			*amount = amount.saturating_add(distributing_amount);
		});
	}

	pub(crate) fn collect_with_investors(tapp_id: TAppId, collecting_amount: BalanceOf<T>) {
		// todo replace total amount with total supply later if calculation result is correct
		let (investors, total_amount) = Self::tapp_investors(tapp_id);
		if !approximately_equals::<T>(
			total_amount,
			TotalSupplyTable::<T>::get(tapp_id),
			CALCULATION_PRECISION.into(),
		) {
			log::error!(
				"collecting calculate total amount error: calculated result is: {:?}, \
				total supply is {:?}",
				total_amount,
				TotalSupplyTable::<T>::get(tapp_id)
			);
		}

		investors.iter().for_each(|account| {
			AccountTable::<T>::mutate(account, tapp_id, |user_amount| {
				*user_amount =
					user_amount.saturating_sub(collecting_amount * (*user_amount) / total_amount);
			});
		});

		TotalSupplyTable::<T>::mutate(tapp_id, |amount| {
			*amount = amount.saturating_sub(collecting_amount);
		});
	}

	pub(crate) fn tapp_investors(tapp_id: TAppId) -> (BTreeSet<T::AccountId>, BalanceOf<T>) {
		let mut investors = BTreeSet::new();
		let mut total_amount: BalanceOf<T> = Zero::zero();
		for (account, id, amount) in AccountTable::<T>::iter() {
			if id != tapp_id {
				continue;
			}
			total_amount = total_amount.saturating_add(amount);
			investors.insert(account);
		}

		(investors, total_amount)
	}

	pub(crate) fn calculate_buy_amount(tapp_id: TAppId, tapp_amount: BalanceOf<T>) -> BalanceOf<T> {
		let tapp_item = TAppBoundingCurve::<T>::get(tapp_id);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);
		Self::calculate_increase_amount_from_curve_total_supply(
			tapp_item.buy_curve,
			total_supply,
			tapp_amount,
		)
	}

	pub(crate) fn calculate_reserve_amount(
		tapp_id: TAppId,
		tapp_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		let tapp_item = TAppBoundingCurve::<T>::get(tapp_id);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);
		Self::calculate_increase_amount_from_curve_total_supply(
			tapp_item.sell_curve,
			total_supply,
			tapp_amount,
		)
	}

	pub(crate) fn calculate_increase_amount_from_curve_total_supply(
		curve_type: CurveType,
		total_supply: BalanceOf<T>,
		tapp_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		let current_pool_balance = match curve_type {
			CurveType::UnsignedLinear => T::LinearCurve::pool_balance(total_supply),
			CurveType::UnsignedSquareRoot_10 => {
				T::UnsignedSquareRoot_10::pool_balance(total_supply)
			}
			CurveType::UnsignedSquareRoot_7 => T::UnsignedSquareRoot_7::pool_balance(total_supply),
		};

		let after_buy_pool_balance = match curve_type {
			CurveType::UnsignedLinear => T::LinearCurve::pool_balance(total_supply + tapp_amount),
			CurveType::UnsignedSquareRoot_10 => {
				T::UnsignedSquareRoot_10::pool_balance(total_supply + tapp_amount)
			}
			CurveType::UnsignedSquareRoot_7 => {
				T::UnsignedSquareRoot_7::pool_balance(total_supply + tapp_amount)
			}
		};
		after_buy_pool_balance - current_pool_balance
	}

	pub(crate) fn calculate_given_increase_tea_how_much_token_mint(
		tapp_id: TAppId,
		tea_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		let tapp_item = TAppBoundingCurve::<T>::get(tapp_id);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);
		let current_buy_area_tea_amount = match tapp_item.buy_curve {
			CurveType::UnsignedLinear => T::LinearCurve::pool_balance(total_supply),
			CurveType::UnsignedSquareRoot_10 => {
				T::UnsignedSquareRoot_10::pool_balance(total_supply)
			}
			CurveType::UnsignedSquareRoot_7 => T::UnsignedSquareRoot_7::pool_balance(total_supply),
		};
		let after_increase_tea_amount = current_buy_area_tea_amount + tea_amount;
		let total_supply_after_increase = match tapp_item.buy_curve {
			CurveType::UnsignedLinear => T::LinearCurve::pool_balance_reverse(
				after_increase_tea_amount,
				T::PoolBalanceReversePrecision::get(),
			),
			CurveType::UnsignedSquareRoot_10 => T::UnsignedSquareRoot_10::pool_balance_reverse(
				after_increase_tea_amount,
				T::PoolBalanceReversePrecision::get(),
			),
			CurveType::UnsignedSquareRoot_7 => T::UnsignedSquareRoot_7::pool_balance_reverse(
				after_increase_tea_amount,
				T::PoolBalanceReversePrecision::get(),
			),
		};
		total_supply_after_increase - total_supply
	}

	/// If user want to sell tapp_amount of tapp_id token, how many T token seller receive after sale
	pub(crate) fn calculate_sell_amount(
		tapp_id: TAppId,
		tapp_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let tapp_item = TAppBoundingCurve::<T>::get(tapp_id);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);
		if tapp_amount > total_supply {
			return Err(Error::<T>::InsufficientTotalSupply.into());
		}

		let current_pool_balance = match tapp_item.sell_curve {
			CurveType::UnsignedLinear => T::LinearCurve::pool_balance(total_supply),
			CurveType::UnsignedSquareRoot_10 => {
				T::UnsignedSquareRoot_10::pool_balance(total_supply)
			}
			CurveType::UnsignedSquareRoot_7 => T::UnsignedSquareRoot_7::pool_balance(total_supply),
		};
		let after_sell_pool_balance = match tapp_item.sell_curve {
			CurveType::UnsignedLinear => T::LinearCurve::pool_balance(total_supply - tapp_amount),
			CurveType::UnsignedSquareRoot_10 => {
				T::UnsignedSquareRoot_10::pool_balance(total_supply - tapp_amount)
			}
			CurveType::UnsignedSquareRoot_7 => {
				T::UnsignedSquareRoot_7::pool_balance(total_supply - tapp_amount)
			}
		};
		Ok(current_pool_balance - after_sell_pool_balance)
	}

	/// calcualte given seller receive tea_amount of TEA, how much of tapp token this seller will give away
	pub(crate) fn calculate_given_received_tea_how_much_seller_give_away(
		tapp_id: TAppId,
		tea_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let tapp_item = TAppBoundingCurve::<T>::get(tapp_id);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);
		let current_reserve_pool_tea = match tapp_item.sell_curve {
			CurveType::UnsignedLinear => T::LinearCurve::pool_balance(total_supply),
			CurveType::UnsignedSquareRoot_10 => {
				T::UnsignedSquareRoot_10::pool_balance(total_supply)
			}
			CurveType::UnsignedSquareRoot_7 => T::UnsignedSquareRoot_7::pool_balance(total_supply),
		};
		if tea_amount > current_reserve_pool_tea {
			return Err(Error::<T>::TAppInsufficientFreeBalance.into());
		}
		let total_supply_after_sell_tapp_token = match tapp_item.sell_curve {
			CurveType::UnsignedLinear => T::LinearCurve::pool_balance_reverse(
				current_reserve_pool_tea - tea_amount,
				T::PoolBalanceReversePrecision::get(),
			),
			CurveType::UnsignedSquareRoot_10 => T::UnsignedSquareRoot_10::pool_balance_reverse(
				current_reserve_pool_tea - tea_amount,
				T::PoolBalanceReversePrecision::get(),
			),
			CurveType::UnsignedSquareRoot_7 => T::UnsignedSquareRoot_7::pool_balance_reverse(
				current_reserve_pool_tea - tea_amount,
				T::PoolBalanceReversePrecision::get(),
			),
		};
		Ok(total_supply - total_supply_after_sell_tapp_token)
	}

	pub(crate) fn check_tapp_fields_length(
		tapp_name: &Vec<u8>,
		ticker: &Vec<u8>,
		detail: &Vec<u8>,
		link: &Vec<u8>,
	) -> DispatchResult {
		ensure!(
			tapp_name.len() <= T::TAppNameMaxLength::get() as usize,
			Error::<T>::TAppNameIsTooLong
		);
		ensure!(
			ticker.len() <= T::TAppTickerMaxLength::get() as usize,
			Error::<T>::TAppTickerIsTooLong
		);
		ensure!(
			ticker.len() >= T::TAppTickerMinLength::get() as usize,
			Error::<T>::TAppTickerIsTooShort
		);
		ensure!(
			detail.len() <= T::TAppDetailMaxLength::get() as usize,
			Error::<T>::TAppDetailIsTooLong
		);
		ensure!(
			link.len() <= T::TAppLinkMaxLength::get() as usize,
			Error::<T>::TAppLinkIsTooLong
		);
		Ok(())
	}
}

pub fn approximately_equals<T>(a: BalanceOf<T>, b: BalanceOf<T>, precision: BalanceOf<T>) -> bool
where
	T: bounding_curve::Config,
{
	let abs = match a >= b {
		true => a - b,
		false => b - a,
	};
	abs <= precision
}

#[cfg(test)]
mod tests {
	use crate::mock::*;
	use crate::*;

	const CENTS: node_primitives::Balance = 10_000_000_000;
	const DOLLARS: node_primitives::Balance = 100 * CENTS;

	#[test]
	fn allocate_buy_tea_amount_works() {
		new_test_ext().execute_with(|| {
			let user1 = 1;
			let user2 = 2;
			let tapp_id = 1;
			<Test as Config>::Currency::make_free_balance_be(&user1, DOLLARS);
			TAppBoundingCurve::<Test>::insert(
				tapp_id,
				TAppItem {
					id: tapp_id,
					owner: user2,
					buy_curve: CurveType::UnsignedSquareRoot_10,
					sell_curve: CurveType::UnsignedSquareRoot_7,
					..Default::default()
				},
			);

			let deposit_amount = BoundingCurve::allocate_buy_tea_amount(&user1, 1, 1000000);
			assert_eq!(deposit_amount, 66);
			assert_eq!(<Test as Config>::Currency::free_balance(&user2), 20);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&user1),
				DOLLARS - 66
			);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&OperationAccount::<Test>::get()),
				46
			);
		})
	}

	#[test]
	fn calculate_sell_amount_works() {
		new_test_ext().execute_with(|| {
			let tapp_id = 1;
			TotalSupplyTable::<Test>::insert(tapp_id, 100_000_000);
			TAppBoundingCurve::<Test>::insert(
				tapp_id,
				TAppItem {
					id: tapp_id,
					buy_curve: CurveType::UnsignedSquareRoot_10,
					sell_curve: CurveType::UnsignedSquareRoot_7,
					..Default::default()
				},
			);

			let amount = BoundingCurve::calculate_sell_amount(tapp_id, 90_000_000);
			assert_eq!(amount.unwrap(), 46666 - 1475);
		})
	}
}