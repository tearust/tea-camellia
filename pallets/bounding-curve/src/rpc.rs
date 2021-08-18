use super::*;

impl<T: bounding_curve::Config> bounding_curve::Pallet<T> {
	pub fn query_price(tapp_id: TAppId) -> (BalanceOf<T>, BalanceOf<T>) {
		let one_tea_dollar = Self::one_tea_dollar();
		let one_tapp_dollar = Self::one_tea_dollar();
		(
			Self::estimate_buy(tapp_id, one_tea_dollar),
			Self::estimate_sell(tapp_id, one_tapp_dollar),
		)
	}

	pub fn estimate_buy(tapp_id: TAppId, amount: BalanceOf<T>) -> BalanceOf<T> {
		Self::calculate_buy_amount(tapp_id, amount)
	}

	pub fn estimate_sell(tapp_id: TAppId, amount: BalanceOf<T>) -> BalanceOf<T> {
		Self::calculate_buy_amount(tapp_id, amount)
	}

	fn one_tea_dollar() -> BalanceOf<T> {
		u128_to_balance::<T>(10_000_000_000 * 100)
	}
}

fn u128_to_balance<T: Config>(amount: u128) -> BalanceOf<T> {
	amount.try_into().map_err(|_| "").unwrap()
}
