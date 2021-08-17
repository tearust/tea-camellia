use crate::*;
use bounding_curve_interface::SellBoundingCurve;

pub struct LinearBuyCurve {}

impl<Balance> BuyBoundingCurve<Balance> for LinearBuyCurve
where
	Balance: Zero,
{
	fn buy(amount: Balance, total_supply: Balance) -> Balance {
		// todo implement me
		Zero::zero()
	}
}

impl Default for LinearBuyCurve {
	fn default() -> Self {
		LinearBuyCurve {}
	}
}

pub struct LinearSellCurve {}

impl<Balance> SellBoundingCurve<Balance> for LinearSellCurve
where
	Balance: Zero,
{
	fn sell(amount: Balance, total_supply: Balance) -> Balance {
		// todo implement me
		Zero::zero()
	}
}

impl Default for LinearSellCurve {
	fn default() -> Self {
		LinearSellCurve {}
	}
}
