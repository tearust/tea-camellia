use super::*;

impl<T: bounding_curve::Config> bounding_curve::Pallet<T> {
	pub fn query_price(tapp_id: TAppId) -> (BalanceOf<T>, BalanceOf<T>) {
		let tapp_item = TAppBoundingCurve::<T>::get(tapp_id);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);
		let buy_price = match tapp_item.buy_curve {
			CurveType::Linear => T::LinearCurve::buy_price(total_supply),
			CurveType::SquareRoot => T::SquareRootCurve::buy_price(total_supply),
		};
		let sell_price = match tapp_item.sell_curve {
			CurveType::Linear => T::LinearCurve::sell_price(total_supply),
			CurveType::SquareRoot => T::LinearCurve::sell_price(total_supply),
		};
		(buy_price, sell_price)
	}

	pub fn estimate_required_tea_when_buy(tapp_id: TAppId, tapp_amount: BalanceOf<T>) -> BalanceOf<T> {
		Self::calculate_buy_amount(tapp_id, tapp_amount)
	}

	pub fn estimate_receive_tea_when_sell(tapp_id: TAppId, tapp_amount: BalanceOf<T>) -> BalanceOf<T> {
		Self::calculate_sell_amount(tapp_id, tapp_amount)
	}

	fn one_tea_dollar() -> BalanceOf<T> {
		u128_to_balance::<T>(10_000_000_000 * 100)
	}
}

fn u128_to_balance<T: Config>(amount: u128) -> BalanceOf<T> {
	amount.try_into().map_err(|_| "").unwrap()
}
