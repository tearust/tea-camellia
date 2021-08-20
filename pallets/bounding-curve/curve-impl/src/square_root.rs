use crate::*;

/// y = k√x + b
pub struct SquareRootCurve {}

impl<Balance> BoundingCurve<Balance> for SquareRootCurve
where
	Balance: AtLeast32BitUnsigned + Default,
{
	fn buy_price(total_supply: Balance) -> Balance {
		todo!()
	}

	fn sell_price(total_supply: Balance) -> Balance {
		todo!()
	}

	fn pool_balance(x: Balance) -> Balance{
		todo!()
	}

	fn pool_balance_reverse(x: Balance) -> Balance {
		todo!()
	}
}

impl Default for SquareRootCurve {
	fn default() -> Self {
		SquareRootCurve {}
	}
}
