use crate::*;

/// y = ax/100 + b/100
pub struct LinearCurve {
	a: u32,
	b: i32,
}

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

	// fn pool_balance(x: Balance, delta_x: Balance, negative: bool) -> Balance {
	fn pool_balance(x: Balance) -> Balance{
		todo!()
	}

	fn pool_balance_reverse(x: Balance) -> Balance {
		todo!()
	}
}

impl Default for LinearCurve {
	fn default() -> Self {
		LinearCurve {
			a: 100u32, b: 0i32
		}
	}
}
