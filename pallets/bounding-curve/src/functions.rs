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

	pub fn buy_token_inner(who: &T::AccountId, tapp_id: TAppId, amount: BalanceOf<T>) {
		if let Err(e) = T::CurrencyOperations::transfer(
			who,
			&OperationAccount::<T>::get(),
			amount,
			ExistenceRequirement::AllowDeath,
		) {
			// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
			log::error!("transfer free balance failed: {:?}", e);
			return;
		}
		let deposit_tapp_amount = Self::calculate_buy_amount(tapp_id, amount);

		AccountTable::<T>::mutate(who, tapp_id, |amount| {
			*amount = amount.saturating_add(deposit_tapp_amount);
		});
		TotalSupplyTable::<T>::mutate(tapp_id, |amount| {
			*amount = amount.saturating_add(deposit_tapp_amount);
		});
	}

	pub fn sell_token_inner(who: &T::AccountId, tapp_id: TAppId, tapp_amount: BalanceOf<T>) {
		if let Err(e) = AccountTable::<T>::mutate(who, tapp_id, |amount| {
			match amount.checked_sub(&tapp_amount) {
				Some(a) => {
					*amount == a;
					Ok(())
				}
				None => Err("account tapp token is not enough"),
			}
		}) {
			// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
			log::error!("{}", e);
			return;
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
			return;
		}

		TotalSupplyTable::<T>::mutate(tapp_id, |amount| {
			*amount = amount.saturating_sub(tapp_amount);
		});
	}

	pub(crate) fn calculate_buy_amount(tapp_id: TAppId, amount: BalanceOf<T>) -> BalanceOf<T> {
		let tapp_item = TAppBoundingCurve::<T>::get(tapp_id);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);
		match tapp_item.buy_curve {
			BuyCurveType::Linear => T::LinearBuyCurve::buy(amount, total_supply),
		}
	}

	pub(crate) fn calculate_sell_amount(tapp_id: TAppId, amount: BalanceOf<T>) -> BalanceOf<T> {
		let tapp_item = TAppBoundingCurve::<T>::get(tapp_id);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);
		match tapp_item.sell_curve {
			SellCurveType::Linear => T::LinearSellCurve::sell(amount, total_supply),
		}
	}
}
