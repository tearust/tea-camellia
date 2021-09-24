use crate::*;

/// Implement equation: `y = ax`
///
/// The genesis const parameter `k` represents the 100 times of `a`.
pub struct UnsignedLinearCurve {
	k: u32,
}

impl<Balance> BondingCurveInterface<Balance> for UnsignedLinearCurve
where
	Balance: AtLeast32BitUnsigned + Default + Clone,
{
	fn buy_price(&self, total_supply: Balance) -> Balance {
		total_supply * self.k.into() / 100u32.into()
	}

	fn pool_balance(&self, x: Balance) -> Balance {
		x.clone() * x.clone() * self.k.into() / 100u32.into() / 2u32.into()
	}

	fn pool_balance_reverse(&self, _area: Balance, _precision: Balance) -> Balance {
		todo!()
	}
}

impl UnsignedLinearCurve {
	pub fn new(k: u32) -> Self {
		UnsignedLinearCurve { k }
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const CENTS: node_primitives::Balance = 10_000_000_000;
	const DOLLARS: node_primitives::Balance = 100 * CENTS;

	#[test]
	fn pool_balance_works() {
		let linear_100 = UnsignedLinearCurve::new(100);
		assert_eq!(linear_100.pool_balance(0u128), 0);
		assert_eq!(linear_100.pool_balance(100u128), 100 * 100 / 2);
		assert_eq!(linear_100.pool_balance(DOLLARS), DOLLARS * DOLLARS / 2);
		assert_eq!(
			linear_100.pool_balance(100 * DOLLARS),
			DOLLARS * DOLLARS * 100 * 100 / 2
		);
	}

	#[test]
	fn buy_and_sell_price_works() {
		let linear_100 = UnsignedLinearCurve::new(100);
		assert_eq!(linear_100.buy_price(0u128), 0);
		assert_eq!(linear_100.buy_price(DOLLARS), DOLLARS);
		assert_eq!(linear_100.buy_price(100 * DOLLARS), DOLLARS * 100);

		let linear_1 = UnsignedLinearCurve::new(1);
		assert_eq!(linear_1.buy_price(0u128), 0);
		assert_eq!(linear_1.buy_price(DOLLARS), DOLLARS / 100);
		assert_eq!(linear_1.buy_price(100 * DOLLARS), DOLLARS);
	}
}
