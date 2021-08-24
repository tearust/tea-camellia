use crate::*;

/// Implement equation: `y = a√x + b`
///
/// The genesis const parameter A represents the 100 times of `a`.
/// The genesis const parameter B represents the 100 times of `b`.
pub struct UnsignedLinearCurve<Balance, const A: u32>
where
	Balance: AtLeast32BitUnsigned + Default + Clone,
{
	phantom: PhantomData<Balance>,
}

impl<Balance, const A: u32> BoundingCurveInterface<Balance> for UnsignedLinearCurve<Balance, A>
where
	Balance: AtLeast32BitUnsigned + Default + Clone,
{
	fn buy_price(total_supply: Balance) -> Balance {
		total_supply * A.into() / 100u32.into()
	}

	fn pool_balance(x: Balance) -> Balance {
		x.clone() * x.clone() * A.into() / 100u32.into() / 2u32.into()
	}

	fn pool_balance_reverse(_area: Balance, _precision: Balance) -> Balance {
		todo!()
	}
}

impl<Balance, const A: u32> Default for UnsignedLinearCurve<Balance, A>
where
	Balance: AtLeast32BitUnsigned + Default + Clone,
{
	fn default() -> Self {
		UnsignedLinearCurve {
			phantom: PhantomData,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use node_primitives::Balance;

	const CENTS: node_primitives::Balance = 10_000_000_000;
	const DOLLARS: node_primitives::Balance = 100 * CENTS;

	#[test]
	fn pool_balance_works() {
		#[allow(non_camel_case_types)]
		type Linear_100 = UnsignedLinearCurve<Balance, 100>; // y = x
		assert_eq!(
			<Linear_100 as BoundingCurveInterface<Balance>>::pool_balance(0),
			0
		);
		assert_eq!(
			<Linear_100 as BoundingCurveInterface<Balance>>::pool_balance(100),
			100 * 100 / 2
		);
		assert_eq!(
			<Linear_100 as BoundingCurveInterface<Balance>>::pool_balance(DOLLARS),
			DOLLARS * DOLLARS / 2
		);
		assert_eq!(
			<Linear_100 as BoundingCurveInterface<Balance>>::pool_balance(100 * DOLLARS),
			DOLLARS * DOLLARS * 100 * 100 / 2
		);
	}

	#[test]
	fn buy_and_sell_price_works() {
		#[allow(non_camel_case_types)]
		type Linear_100 = UnsignedLinearCurve<Balance, 100>; // y = x
		assert_eq!(
			<Linear_100 as BoundingCurveInterface<Balance>>::buy_price(0),
			0
		);
		assert_eq!(
			<Linear_100 as BoundingCurveInterface<Balance>>::buy_price(DOLLARS),
			DOLLARS
		);
		assert_eq!(
			<Linear_100 as BoundingCurveInterface<Balance>>::buy_price(100 * DOLLARS),
			DOLLARS * 100
		);

		#[allow(non_camel_case_types)]
		type Linear_1 = UnsignedLinearCurve<Balance, 1>; // y = x/100
		assert_eq!(
			<Linear_1 as BoundingCurveInterface<Balance>>::buy_price(0),
			0
		);
		assert_eq!(
			<Linear_1 as BoundingCurveInterface<Balance>>::buy_price(DOLLARS),
			DOLLARS / 100
		);
		assert_eq!(
			<Linear_1 as BoundingCurveInterface<Balance>>::buy_price(100 * DOLLARS),
			DOLLARS
		);
	}
}
