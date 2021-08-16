use super::*;

impl<T: bounding_curve::Config> bounding_curve::Pallet<T> {
	pub fn query_price(tapp_id: TAppId) -> (BalanceOf<T>, BalanceOf<T>) {
		// todo implement me
		(Zero::zero(), Zero::zero())
	}

	pub fn estimate_buy(tapp_id: TAppId, amount: BalanceOf<T>) -> BalanceOf<T> {
		Zero::zero()
	}

	pub fn estimate_sell(tapp_id: TAppId, amount: BalanceOf<T>) -> BalanceOf<T> {
		Zero::zero()
	}
}
