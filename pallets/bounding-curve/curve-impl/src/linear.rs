use crate::*;

/// Implement equation: `y = a√x + b`
///
/// The genesis const parameter A represents the 100 times of `a`.
/// The genesis const parameter B represents the 100 times of `b`.
pub struct UnsignedLinearCurve<Balance, const A: u32, const B: u32>
where
	Balance: AtLeast32BitUnsigned + Default + Clone,
{
	phantom: PhantomData<Balance>,
}

impl<Balance, const A: u32, const B: u32> UnsignedLinearCurve<Balance, A, B>
where
	Balance: AtLeast32BitUnsigned + Default + Clone,
{
	fn unit_price() -> Balance {
		Self::u128_to_balance(DOLLARS)
	}

	fn u128_to_balance(amount: u128) -> Balance {
		amount.try_into().map_err(|_| "").unwrap()
	}
}

impl<Balance, const A: u32, const B: u32> BoundingCurveInterface<Balance>
	for UnsignedLinearCurve<Balance, A, B>
where
	Balance: AtLeast32BitUnsigned + Default + Clone,
{
	fn buy_price(total_supply: Balance) -> Balance {
		total_supply * A.into() / 100u32.into() + Balance::from(B) / 100u32.into()
	}

	fn sell_price(total_supply: Balance) -> Balance {
		let buy_price = Self::buy_price(total_supply);
		if buy_price.is_zero() {
			return Zero::zero();
		}
		Self::unit_price() * Self::unit_price() / buy_price
	}

	fn pool_balance(x: Balance) -> Balance {
		x.clone() * x.clone() * A.into() / 100u32.into() / 2u32.into()
			+ x * B.into() / 100u32.into()
	}

	fn pool_balance_reverse(_area: Balance) -> Balance {
		todo!()
	}
}

impl<Balance, const A: u32, const B: u32> Default for UnsignedLinearCurve<Balance, A, B>
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

	#[test]
	fn pool_balance_works() {
		#[allow(non_camel_case_types)]
		type Linear_100_0 = UnsignedLinearCurve<Balance, 100, 0>; // y = x
		assert_eq!(
			<Linear_100_0 as BoundingCurveInterface<Balance>>::pool_balance(0),
			0
		);
		assert_eq!(
			<Linear_100_0 as BoundingCurveInterface<Balance>>::pool_balance(100),
			100 * 100 / 2
		);
		assert_eq!(
			<Linear_100_0 as BoundingCurveInterface<Balance>>::pool_balance(DOLLARS),
			DOLLARS * DOLLARS / 2
		);
		assert_eq!(
			<Linear_100_0 as BoundingCurveInterface<Balance>>::pool_balance(100 * DOLLARS),
			DOLLARS * DOLLARS * 100 * 100 / 2
		);
	}

	#[test]
	fn buy_and_sell_price_works() {
		#[allow(non_camel_case_types)]
		type Linear_100_0 = UnsignedLinearCurve<Balance, 100, 0>; // y = x
		assert_eq!(
			<Linear_100_0 as BoundingCurveInterface<Balance>>::buy_price(0),
			0
		);
		assert_eq!(
			<Linear_100_0 as BoundingCurveInterface<Balance>>::sell_price(0),
			0
		);
		assert_eq!(
			<Linear_100_0 as BoundingCurveInterface<Balance>>::buy_price(DOLLARS),
			DOLLARS
		);
		assert_eq!(
			<Linear_100_0 as BoundingCurveInterface<Balance>>::sell_price(DOLLARS),
			DOLLARS
		);
		assert_eq!(
			<Linear_100_0 as BoundingCurveInterface<Balance>>::buy_price(100 * DOLLARS),
			DOLLARS * 100
		);
		assert_eq!(
			<Linear_100_0 as BoundingCurveInterface<Balance>>::sell_price(100 * DOLLARS),
			DOLLARS / 100
		);

		#[allow(non_camel_case_types)]
		type Linear_1_0 = UnsignedLinearCurve<Balance, 1, 0>; // y = x/100
		assert_eq!(
			<Linear_1_0 as BoundingCurveInterface<Balance>>::buy_price(0),
			0
		);
		assert_eq!(
			<Linear_1_0 as BoundingCurveInterface<Balance>>::sell_price(0),
			0
		);
		assert_eq!(
			<Linear_1_0 as BoundingCurveInterface<Balance>>::buy_price(DOLLARS),
			DOLLARS / 100
		);
		assert_eq!(
			<Linear_1_0 as BoundingCurveInterface<Balance>>::sell_price(DOLLARS),
			DOLLARS * 100
		);
		assert_eq!(
			<Linear_1_0 as BoundingCurveInterface<Balance>>::buy_price(100 * DOLLARS),
			DOLLARS
		);
		assert_eq!(
			<Linear_1_0 as BoundingCurveInterface<Balance>>::sell_price(100 * DOLLARS),
			DOLLARS
		);

		#[allow(non_camel_case_types)]
		type Linear_1_100 = UnsignedLinearCurve<Balance, 1, 100>; // y = x/100 + 1
		assert_eq!(
			<Linear_1_100 as BoundingCurveInterface<Balance>>::buy_price(0),
			1
		);
		assert_eq!(
			<Linear_1_100 as BoundingCurveInterface<Balance>>::sell_price(0),
			DOLLARS * DOLLARS
		);
		assert_eq!(
			<Linear_1_100 as BoundingCurveInterface<Balance>>::buy_price(DOLLARS),
			DOLLARS / 100 + 1
		);
		assert_eq!(
			<Linear_1_100 as BoundingCurveInterface<Balance>>::sell_price(DOLLARS),
			99999999990000
		);
		assert_eq!(
			<Linear_1_100 as BoundingCurveInterface<Balance>>::buy_price(100 * DOLLARS),
			DOLLARS + 1
		);
		assert_eq!(
			<Linear_1_100 as BoundingCurveInterface<Balance>>::sell_price(100 * DOLLARS),
			DOLLARS - 1
		);
	}
}
