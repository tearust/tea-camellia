use super::*;

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
		let deposit_tea_amount = Self::calculate_buy_amount(tapp_id, tapp_amount, true);
		let reserved_tea_amount = Self::calculate_buy_amount(tapp_id, tapp_amount, false);

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
		let deposit_tea_amount = Self::calculate_sell_amount(tapp_id, tapp_amount);
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

		if AccountTable::<T>::get(who, tapp_id).is_zero() {
			AccountTable::<T>::remove(who, tapp_id);
		}
		if TotalSupplyTable::<T>::get(tapp_id).is_zero() {
			TotalSupplyTable::<T>::remove(tapp_id);
		}

		deposit_tea_amount
	}

	pub(crate) fn distribute_to_investors(tapp_id: TAppId, distributing_amount: BalanceOf<T>) {
		let (investors, total_amount) = Self::tapp_investors(tapp_id);
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
		let (investors, total_amount) = Self::tapp_investors(tapp_id);
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

	pub(crate) fn calculate_buy_amount(
		tapp_id: TAppId,
		tapp_amount: BalanceOf<T>,
		buy_curve: bool,
	) -> BalanceOf<T> {
		let tapp_item = TAppBoundingCurve::<T>::get(tapp_id);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);

		Self::estimate_buy_amount(
			match buy_curve {
				true => tapp_item.buy_curve,
				false => tapp_item.sell_curve,
			},
			total_supply,
			tapp_amount,
		)
	}

	pub(crate) fn estimate_buy_amount(
		curve_type: CurveType,
		total_supply: BalanceOf<T>,
		tapp_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		match curve_type {
			CurveType::Linear => T::LinearCurve::pool_balance(total_supply, tapp_amount, false),
			CurveType::SquareRoot => T::LinearCurve::pool_balance(total_supply, tapp_amount, false),
		}
	}

	pub(crate) fn calculate_buy_reverse_amount(
		tapp_id: TAppId,
		tea_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		let tapp_item = TAppBoundingCurve::<T>::get(tapp_id);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);
		match tapp_item.buy_curve {
			CurveType::Linear => {
				T::LinearCurve::pool_balance_reverse(total_supply, tea_amount, false)
			}
			CurveType::SquareRoot => {
				T::SquareRootCurve::pool_balance_reverse(total_supply, tea_amount, false)
			}
		}
	}

	pub(crate) fn calculate_sell_amount(
		tapp_id: TAppId,
		tapp_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		let tapp_item = TAppBoundingCurve::<T>::get(tapp_id);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);
		match tapp_item.sell_curve {
			CurveType::Linear => T::LinearCurve::pool_balance(total_supply, tapp_amount, true),
			CurveType::SquareRoot => {
				T::SquareRootCurve::pool_balance(total_supply, tapp_amount, true)
			}
		}
	}

	pub(crate) fn calculate_sell_reverse_amount(
		tapp_id: TAppId,
		tea_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		let tapp_item = TAppBoundingCurve::<T>::get(tapp_id);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);
		match tapp_item.sell_curve {
			CurveType::Linear => {
				T::LinearCurve::pool_balance_reverse(total_supply, tea_amount, true)
			}
			CurveType::SquareRoot => {
				T::SquareRootCurve::pool_balance_reverse(total_supply, tea_amount, true)
			}
		}
	}
}
