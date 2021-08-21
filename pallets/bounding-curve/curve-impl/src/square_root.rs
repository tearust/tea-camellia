use crate::*;

/// Implement equation: `y = k√x + b`
///
/// The genesis const parameter K represents the 100 times of `k`.
/// The genesis const parameter B represents the 100 times of `b`.
pub struct UnsignedSquareRoot<Balance, const K: u32, const B: u32>
where
	Balance: AtLeast32BitUnsigned + Default + Clone,
{
	phantom: PhantomData<Balance>,
}

impl<Balance, const A: u32, const B: u32> UnsignedSquareRoot<Balance, A, B>
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

impl<Balance, const K: u32, const B: u32> BoundingCurveInterface<Balance>
	for UnsignedSquareRoot<Balance, K, B>
where
	Balance: AtLeast32BitUnsigned + Default + Clone,
{
	fn buy_price(total_supply: Balance) -> Balance {
		total_supply.integer_sqrt() * K.into() / 100u32.into() + Balance::from(B) / 100u32.into()
	}

	fn sell_price(total_supply: Balance) -> Balance {
		let buy_price = Self::buy_price(total_supply);
		if buy_price.is_zero() {
			return Zero::zero();
		}
		Self::unit_price() * Self::unit_price() / buy_price
	}

	fn pool_balance(x: Balance) -> Balance {
		x.integer_sqrt() * x.clone() * K.into() * 2u32.into() / 100u32.into() / 3u32.into()
			+ x * B.into() / 100u32.into()
	}

	fn pool_balance_reverse(_area: Balance) -> Balance {
		todo!()
	}
}

impl<Balance, const K: u32, const B: u32> Default for UnsignedSquareRoot<Balance, K, B>
where
	Balance: AtLeast32BitUnsigned + Default + Clone,
{
	fn default() -> Self {
		UnsignedSquareRoot {
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
		type RootSquare_100_0 = UnsignedSquareRoot<Balance, 100, 0>; // y = √x
		assert_eq!(
			<RootSquare_100_0 as BoundingCurveInterface<Balance>>::pool_balance(0),
			0
		);
		assert_eq!(
			<RootSquare_100_0 as BoundingCurveInterface<Balance>>::pool_balance(100),
			666
		);
		assert_eq!(
			<RootSquare_100_0 as BoundingCurveInterface<Balance>>::pool_balance(DOLLARS),
			666666666666666666
		);
		assert_eq!(
			<RootSquare_100_0 as BoundingCurveInterface<Balance>>::pool_balance(100 * DOLLARS),
			666666666666666666666
		);

		#[allow(non_camel_case_types)]
		type RootSquare_1000_0 = UnsignedSquareRoot<Balance, 1000, 0>; // y = 10√x
		assert_eq!(
			<RootSquare_1000_0 as BoundingCurveInterface<Balance>>::pool_balance(0),
			0
		);
		assert_eq!(
			<RootSquare_1000_0 as BoundingCurveInterface<Balance>>::pool_balance(100),
			6666
		);
		assert_eq!(
			<RootSquare_1000_0 as BoundingCurveInterface<Balance>>::pool_balance(DOLLARS),
			6666666666666666666
		);
		assert_eq!(
			<RootSquare_1000_0 as BoundingCurveInterface<Balance>>::pool_balance(100 * DOLLARS),
			6666666666666666666666
		);

		#[allow(non_camel_case_types)]
		type RootSquare_700_0 = UnsignedSquareRoot<Balance, 700, 0>; // y = 7√x
		assert_eq!(
			<RootSquare_700_0 as BoundingCurveInterface<Balance>>::pool_balance(0),
			0
		);
		assert_eq!(
			<RootSquare_700_0 as BoundingCurveInterface<Balance>>::pool_balance(100),
			4666
		);
		assert_eq!(
			<RootSquare_700_0 as BoundingCurveInterface<Balance>>::pool_balance(DOLLARS),
			4666666666666666666
		);
		assert_eq!(
			<RootSquare_700_0 as BoundingCurveInterface<Balance>>::pool_balance(100 * DOLLARS),
			4666666666666666666666
		);
	}

	#[test]
	fn buy_and_sell_price_works() {
		#[allow(non_camel_case_types)]
		type RootSquare_100_0 = UnsignedSquareRoot<Balance, 100, 0>; // y = x
		assert_eq!(
			<RootSquare_100_0 as BoundingCurveInterface<Balance>>::buy_price(0),
			0
		);
		assert_eq!(
			<RootSquare_100_0 as BoundingCurveInterface<Balance>>::sell_price(0),
			0
		);
		assert_eq!(
			<RootSquare_100_0 as BoundingCurveInterface<Balance>>::buy_price(DOLLARS),
			1000000
		);
		assert_eq!(
			<RootSquare_100_0 as BoundingCurveInterface<Balance>>::sell_price(DOLLARS),
			DOLLARS * 1000000
		);
		assert_eq!(
			<RootSquare_100_0 as BoundingCurveInterface<Balance>>::buy_price(100 * DOLLARS),
			10000000
		);
		assert_eq!(
			<RootSquare_100_0 as BoundingCurveInterface<Balance>>::sell_price(100 * DOLLARS),
			DOLLARS * 100000
		);

		#[allow(non_camel_case_types)]
		type RootSquare_1_0 = UnsignedSquareRoot<Balance, 1, 0>; // y = x/100
		assert_eq!(
			<RootSquare_1_0 as BoundingCurveInterface<Balance>>::buy_price(0),
			0
		);
		assert_eq!(
			<RootSquare_1_0 as BoundingCurveInterface<Balance>>::sell_price(0),
			0
		);
		assert_eq!(
			<RootSquare_1_0 as BoundingCurveInterface<Balance>>::buy_price(DOLLARS),
			10000
		);
		assert_eq!(
			<RootSquare_1_0 as BoundingCurveInterface<Balance>>::sell_price(DOLLARS),
			DOLLARS * 100000000
		);
		assert_eq!(
			<RootSquare_1_0 as BoundingCurveInterface<Balance>>::buy_price(100 * DOLLARS),
			100000
		);
		assert_eq!(
			<RootSquare_1_0 as BoundingCurveInterface<Balance>>::sell_price(100 * DOLLARS),
			DOLLARS * 10000000
		);

		#[allow(non_camel_case_types)]
		type RootSquare_1_100 = UnsignedSquareRoot<Balance, 1, 100>; // y = x/100 + 1
		assert_eq!(
			<RootSquare_1_100 as BoundingCurveInterface<Balance>>::buy_price(0),
			1
		);
		assert_eq!(
			<RootSquare_1_100 as BoundingCurveInterface<Balance>>::sell_price(0),
			DOLLARS * DOLLARS
		);
		assert_eq!(
			<RootSquare_1_100 as BoundingCurveInterface<Balance>>::buy_price(DOLLARS),
			10001
		);
		assert_eq!(
			<RootSquare_1_100 as BoundingCurveInterface<Balance>>::sell_price(DOLLARS),
			99990000999900009999
		);
		assert_eq!(
			<RootSquare_1_100 as BoundingCurveInterface<Balance>>::buy_price(100 * DOLLARS),
			100001
		);
		assert_eq!(
			<RootSquare_1_100 as BoundingCurveInterface<Balance>>::sell_price(100 * DOLLARS),
			9999900000999990000
		);

		#[allow(non_camel_case_types)]
		type RootSquare_1000_0 = UnsignedSquareRoot<Balance, 1000, 0>; // y = 10√x
		assert_eq!(
			<RootSquare_1000_0 as BoundingCurveInterface<Balance>>::buy_price(0),
			0
		);
		assert_eq!(
			<RootSquare_1000_0 as BoundingCurveInterface<Balance>>::sell_price(0),
			0
		);
		assert_eq!(
			<RootSquare_1000_0 as BoundingCurveInterface<Balance>>::buy_price(DOLLARS),
			10000000
		);
		assert_eq!(
			<RootSquare_1000_0 as BoundingCurveInterface<Balance>>::sell_price(DOLLARS),
			DOLLARS * 100000
		);
		assert_eq!(
			<RootSquare_1000_0 as BoundingCurveInterface<Balance>>::buy_price(100 * DOLLARS),
			100000000
		);
		assert_eq!(
			<RootSquare_1000_0 as BoundingCurveInterface<Balance>>::sell_price(100 * DOLLARS),
			DOLLARS * 10000
		);

		#[allow(non_camel_case_types)]
		type RootSquare_700_0 = UnsignedSquareRoot<Balance, 700, 0>; // y = 7√x
		assert_eq!(
			<RootSquare_700_0 as BoundingCurveInterface<Balance>>::buy_price(0),
			0
		);
		assert_eq!(
			<RootSquare_700_0 as BoundingCurveInterface<Balance>>::sell_price(0),
			0
		);
		assert_eq!(
			<RootSquare_700_0 as BoundingCurveInterface<Balance>>::buy_price(DOLLARS),
			7000000
		);
		assert_eq!(
			<RootSquare_700_0 as BoundingCurveInterface<Balance>>::sell_price(DOLLARS),
			142857142857142857
		);
		assert_eq!(
			<RootSquare_700_0 as BoundingCurveInterface<Balance>>::buy_price(100 * DOLLARS),
			70000000
		);
		assert_eq!(
			<RootSquare_700_0 as BoundingCurveInterface<Balance>>::sell_price(100 * DOLLARS),
			14285714285714285
		);
	}
}
