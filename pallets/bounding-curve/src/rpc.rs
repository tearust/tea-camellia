use super::*;

impl<T: bounding_curve::Config> bounding_curve::Pallet<T> {
	pub fn query_price(tapp_id: TAppId) -> (BalanceOf<T>, BalanceOf<T>) {
		let tapp_item = TAppBoundingCurve::<T>::get(tapp_id);
		let total_supply = TotalSupplyTable::<T>::get(tapp_id);
		let buy_price = match tapp_item.buy_curve {
			CurveType::UnsignedLinear => T::LinearCurve::buy_price(total_supply),
			CurveType::UnsignedSquareRoot => T::SquareRootCurve::buy_price(total_supply),
		};
		let sell_price = match tapp_item.sell_curve {
			CurveType::UnsignedLinear => T::LinearCurve::sell_price(total_supply),
			CurveType::UnsignedSquareRoot => T::LinearCurve::sell_price(total_supply),
		};
		(buy_price, sell_price)
	}

	pub fn estimate_required_tea_when_buy(
		tapp_id: TAppId,
		tapp_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		Self::calculate_buy_amount(tapp_id, tapp_amount)
	}

	pub fn estimate_receive_tea_when_sell(
		tapp_id: TAppId,
		tapp_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		match Self::calculate_sell_amount(tapp_id, tapp_amount) {
			Ok(balance) => balance,
			Err(e) => {
				log::error!("calculate failed: {:?}", e);
				Zero::zero()
			}
		}
	}

	pub fn estimate_receive_token_when_buy(
		tapp_id: TAppId,
		tea_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		Self::calculate_given_increase_tea_how_much_token_mint(tapp_id, tea_amount)
	}

	pub fn estimate_required_token_when_sell(
		tapp_id: TAppId,
		tea_amount: BalanceOf<T>,
	) -> BalanceOf<T> {
		match Self::calculate_given_received_tea_how_much_seller_give_away(tapp_id, tea_amount) {
			Ok(balance) => balance,
			Err(e) => {
				log::error!("calculate failed: {:?}", e);
				Zero::zero()
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::*;
	use frame_support::assert_ok;

	const CENTS: node_primitives::Balance = 10_000_000_000;
	const DOLLARS: node_primitives::Balance = 100 * CENTS;

	#[test]
	fn query_price_works() {
		new_test_ext().execute_with(|| {
			<Test as Config>::Currency::make_free_balance_be(&1, DOLLARS * DOLLARS);

			assert_ok!(BoundingCurve::create_new_tapp(
				Origin::signed(1),
				b"test".to_vec(),
				DOLLARS,
				CurveType::UnsignedLinear,
				CurveType::UnsignedLinear,
			));
			let (buy_price, sell_price) = BoundingCurve::query_price(1);
			assert_eq!(buy_price, DOLLARS);
			assert_eq!(sell_price, DOLLARS);

			assert_ok!(BoundingCurve::create_new_tapp(
				Origin::signed(1),
				b"test2".to_vec(),
				CENTS,
				CurveType::UnsignedLinear,
				CurveType::UnsignedLinear,
			));
			let (buy_price, sell_price) = BoundingCurve::query_price(2);
			assert_eq!(buy_price, CENTS);
			assert_eq!(sell_price, DOLLARS * 100);
		})
	}
}
