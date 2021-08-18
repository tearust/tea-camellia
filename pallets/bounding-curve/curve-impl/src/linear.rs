use crate::*;

pub struct LinearBuyCurve {}

impl<Balance> BuyBoundingCurve<Balance> for LinearBuyCurve
where
	Balance: AtLeast32BitUnsigned,
{
	fn buy(amount: Balance, total_supply: Balance) -> Balance {
		total_supply + amount
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
	Balance: AtLeast32BitUnsigned,
{
	fn sell(amount: Balance, total_supply: Balance) -> Balance {
		total_supply - amount
	}
}

impl Default for LinearSellCurve {
	fn default() -> Self {
		LinearSellCurve {}
	}
}
