use crate::*;

/// y = ax + b
pub struct LinearCurve {}

impl<Balance> BoundingCurve<Balance> for LinearCurve
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

impl Default for LinearCurve {
	fn default() -> Self {
		LinearCurve {}
	}
}
