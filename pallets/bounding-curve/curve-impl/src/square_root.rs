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

	fn pool_balance(x: Balance, delta_x: Balance, negative: bool) -> Balance {
		todo!()
	}

	fn pool_balance_reverse(x: Balance, tea_amount: Balance, negative: bool) -> Balance {
		todo!()
	}
}

impl Default for SquareRootCurve {
	fn default() -> Self {
		SquareRootCurve {}
	}
}
