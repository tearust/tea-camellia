use crate::square_root::k1000::K1000_STEP100_AREA_LIST;
use crate::square_root::k700::K700_STEP100_AREA_LIST;
use crate::*;

mod k1000;
mod k700;

const AREA_LIST_LENGTH: usize = 1000;

/// Implement equation: `y = k√x`
///
/// The genesis const parameter K represents the 100 times of `k`.
pub struct UnsignedSquareRoot<Balance, const K: u32>
where
	Balance: AtLeast32BitUnsigned + Default + Clone + Debug,
{
	phantom: PhantomData<Balance>,
}

impl<Balance, const K: u32> UnsignedSquareRoot<Balance, K>
where
	Balance: AtLeast32BitUnsigned + Default + Clone + Debug,
{
	fn unit_price() -> Balance {
		Self::u128_to_balance(DOLLARS)
	}

	fn u128_to_balance(amount: u128) -> Balance {
		amount.try_into().map_err(|_| "").unwrap()
	}

	fn recursively_balance_reverse_calculation(
		x_n: Balance,
		area: Balance,
		precision: Balance,
		times: &mut usize,
	) -> Balance {
		*times += 1;

		let result = if x_n.is_zero() {
			Zero::zero()
		} else {
			x_n.clone() - x_n.clone() * 2u32.into() / 3u32.into()
				+ area.clone() * 100u32.into() / K.into() / x_n.integer_sqrt()
		};

		if approximately_equals(x_n, result.clone(), precision.clone()) {
			result
		} else {
			Self::recursively_balance_reverse_calculation(result, area, precision, times)
		}
	}

	fn select_nearest_area_and_x(current_area: Balance) -> (Balance, Balance) {
		let select_fn = |it: &[u32; AREA_LIST_LENGTH]| {
			let mut best_area: u32 = 0;
			let mut best_x: u32 = 0;
			for (i, area) in it.iter().enumerate() {
				if current_area < Balance::from(*area) {
					break;
				}

				best_area = *area;
				if i.is_zero() {
					best_x = 1;
				} else {
					best_x = 100 * i as u32;
				}
			}

			(Balance::from(best_area), Balance::from(best_x))
		};

		match K {
			1000 => select_fn(&K1000_STEP100_AREA_LIST),
			700 => select_fn(&K700_STEP100_AREA_LIST),
			_ => (Self::pool_balance(One::one()), One::one()),
		}
	}
}

impl<Balance, const K: u32> BoundingCurveInterface<Balance> for UnsignedSquareRoot<Balance, K>
where
	Balance: AtLeast32BitUnsigned + Default + Clone + Debug,
{
	fn buy_price(total_supply: Balance) -> Balance {
		total_supply.integer_sqrt() * K.into() / 100u32.into()
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
	}

	fn pool_balance_reverse(area: Balance, precision: Balance) -> Balance {
		if area.is_zero() {
			return Zero::zero();
		}

		let (seed_area, seed_x) = Self::select_nearest_area_and_x(area.clone());
		if area == seed_area {
			return seed_x;
		}

		let mut times = 0;
		let result = Self::recursively_balance_reverse_calculation(
			seed_x,
			area.clone(),
			precision,
			&mut times,
		);
		#[cfg(feature = "std")]
		println!(
			"area {:?} (K: {}) calculated result is {:?}, calculated times: {}",
			area, K, result, times
		);
		result
	}
}

impl<Balance, const K: u32> Default for UnsignedSquareRoot<Balance, K>
where
	Balance: AtLeast32BitUnsigned + Default + Clone + Debug,
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
		type RootSquare_100 = UnsignedSquareRoot<Balance, 100>; // y = √x
		assert_eq!(
			<RootSquare_100 as BoundingCurveInterface<Balance>>::pool_balance(0),
			0
		);
		assert_eq!(
			<RootSquare_100 as BoundingCurveInterface<Balance>>::pool_balance(100),
			666
		);
		assert_eq!(
			<RootSquare_100 as BoundingCurveInterface<Balance>>::pool_balance(DOLLARS),
			666666666666666666
		);
		assert_eq!(
			<RootSquare_100 as BoundingCurveInterface<Balance>>::pool_balance(100 * DOLLARS),
			666666666666666666666
		);

		#[allow(non_camel_case_types)]
		type RootSquare_1000 = UnsignedSquareRoot<Balance, 1000>; // y = 10√x
		assert_eq!(
			<RootSquare_1000 as BoundingCurveInterface<Balance>>::pool_balance(0),
			0
		);
		assert_eq!(
			<RootSquare_1000 as BoundingCurveInterface<Balance>>::pool_balance(1),
			6
		);
		assert_eq!(
			<RootSquare_1000 as BoundingCurveInterface<Balance>>::pool_balance(10),
			200
		);
		assert_eq!(
			<RootSquare_1000 as BoundingCurveInterface<Balance>>::pool_balance(100),
			6666
		);
		assert_eq!(
			<RootSquare_1000 as BoundingCurveInterface<Balance>>::pool_balance(DOLLARS),
			6666666666666666666
		);
		assert_eq!(
			<RootSquare_1000 as BoundingCurveInterface<Balance>>::pool_balance(100 * DOLLARS),
			6666666666666666666666
		);

		#[allow(non_camel_case_types)]
		type RootSquare_700 = UnsignedSquareRoot<Balance, 700>; // y = 7√x
		assert_eq!(
			<RootSquare_700 as BoundingCurveInterface<Balance>>::pool_balance(0),
			0
		);
		assert_eq!(
			<RootSquare_700 as BoundingCurveInterface<Balance>>::pool_balance(1),
			4
		);
		assert_eq!(
			<RootSquare_700 as BoundingCurveInterface<Balance>>::pool_balance(10),
			140
		);
		assert_eq!(
			<RootSquare_700 as BoundingCurveInterface<Balance>>::pool_balance(100),
			4666
		);
		assert_eq!(
			<RootSquare_700 as BoundingCurveInterface<Balance>>::pool_balance(DOLLARS),
			4666666666666666666
		);
		assert_eq!(
			<RootSquare_700 as BoundingCurveInterface<Balance>>::pool_balance(100 * DOLLARS),
			4666666666666666666666
		);
	}

	#[test]
	fn pool_balance_reverse_works() {
		#[allow(non_camel_case_types)]
		type RootSquare_100 = UnsignedSquareRoot<Balance, 100>; // y = √x
		assert_eq!(
			<RootSquare_100 as BoundingCurveInterface<Balance>>::pool_balance_reverse(0, 1),
			0
		);
		assert_eq!(
			<RootSquare_100 as BoundingCurveInterface<Balance>>::pool_balance_reverse(666, 1),
			100
		);
		assert_eq!(
			<RootSquare_100 as BoundingCurveInterface<Balance>>::pool_balance_reverse(
				666666666666666666,
				1
			),
			DOLLARS
		);
		assert_eq!(
			<RootSquare_100 as BoundingCurveInterface<Balance>>::pool_balance_reverse(
				666666666666666666666,
				1
			),
			100 * DOLLARS
		);

		#[allow(non_camel_case_types)]
		type RootSquare_1000 = UnsignedSquareRoot<Balance, 1000>; // y = 10√x
		assert_eq!(
			<RootSquare_1000 as BoundingCurveInterface<Balance>>::pool_balance_reverse(0, 1),
			0
		);
		assert_eq!(
			<RootSquare_1000 as BoundingCurveInterface<Balance>>::pool_balance_reverse(6, 1),
			1
		);
		assert_eq!(
			<RootSquare_1000 as BoundingCurveInterface<Balance>>::pool_balance_reverse(200, 1),
			10
		);
		assert_eq!(
			<RootSquare_1000 as BoundingCurveInterface<Balance>>::pool_balance_reverse(6666, 1),
			100
		);
		assert_eq!(
			<RootSquare_1000 as BoundingCurveInterface<Balance>>::pool_balance_reverse(
				6666666666666666666,
				1
			),
			DOLLARS
		);
		assert_eq!(
			<RootSquare_1000 as BoundingCurveInterface<Balance>>::pool_balance_reverse(
				6666666666666666666666,
				1
			),
			100 * DOLLARS
		);

		#[allow(non_camel_case_types)]
		type RootSquare_700 = UnsignedSquareRoot<Balance, 700>; // y = 7√x
		assert_eq!(
			<RootSquare_700 as BoundingCurveInterface<Balance>>::pool_balance_reverse(0, 1),
			0
		);
		assert_eq!(
			<RootSquare_700 as BoundingCurveInterface<Balance>>::pool_balance_reverse(140, 1),
			10
		);
		assert_eq!(
			<RootSquare_700 as BoundingCurveInterface<Balance>>::pool_balance_reverse(4666, 1),
			100
		);
		assert_eq!(
			<RootSquare_700 as BoundingCurveInterface<Balance>>::pool_balance_reverse(
				4666666666666666666,
				1
			),
			DOLLARS
		);
		assert_eq!(
			<RootSquare_700 as BoundingCurveInterface<Balance>>::pool_balance_reverse(
				4666666666666666666666,
				1
			),
			100 * DOLLARS
		);
	}

	#[test]
	fn buy_and_sell_price_works() {
		#[allow(non_camel_case_types)]
		type RootSquare_100 = UnsignedSquareRoot<Balance, 100>; // y = x
		assert_eq!(
			<RootSquare_100 as BoundingCurveInterface<Balance>>::buy_price(0),
			0
		);
		assert_eq!(
			<RootSquare_100 as BoundingCurveInterface<Balance>>::sell_price(0),
			0
		);
		assert_eq!(
			<RootSquare_100 as BoundingCurveInterface<Balance>>::buy_price(DOLLARS),
			1000000
		);
		assert_eq!(
			<RootSquare_100 as BoundingCurveInterface<Balance>>::sell_price(DOLLARS),
			DOLLARS * 1000000
		);
		assert_eq!(
			<RootSquare_100 as BoundingCurveInterface<Balance>>::buy_price(100 * DOLLARS),
			10000000
		);
		assert_eq!(
			<RootSquare_100 as BoundingCurveInterface<Balance>>::sell_price(100 * DOLLARS),
			DOLLARS * 100000
		);

		#[allow(non_camel_case_types)]
		type RootSquare_1 = UnsignedSquareRoot<Balance, 1>; // y = x/100
		assert_eq!(
			<RootSquare_1 as BoundingCurveInterface<Balance>>::buy_price(0),
			0
		);
		assert_eq!(
			<RootSquare_1 as BoundingCurveInterface<Balance>>::sell_price(0),
			0
		);
		assert_eq!(
			<RootSquare_1 as BoundingCurveInterface<Balance>>::buy_price(DOLLARS),
			10000
		);
		assert_eq!(
			<RootSquare_1 as BoundingCurveInterface<Balance>>::sell_price(DOLLARS),
			DOLLARS * 100000000
		);
		assert_eq!(
			<RootSquare_1 as BoundingCurveInterface<Balance>>::buy_price(100 * DOLLARS),
			100000
		);
		assert_eq!(
			<RootSquare_1 as BoundingCurveInterface<Balance>>::sell_price(100 * DOLLARS),
			DOLLARS * 10000000
		);

		#[allow(non_camel_case_types)]
		type RootSquare_1000 = UnsignedSquareRoot<Balance, 1000>; // y = 10√x
		assert_eq!(
			<RootSquare_1000 as BoundingCurveInterface<Balance>>::buy_price(0),
			0
		);
		assert_eq!(
			<RootSquare_1000 as BoundingCurveInterface<Balance>>::sell_price(0),
			0
		);
		assert_eq!(
			<RootSquare_1000 as BoundingCurveInterface<Balance>>::buy_price(DOLLARS),
			10000000
		);
		assert_eq!(
			<RootSquare_1000 as BoundingCurveInterface<Balance>>::sell_price(DOLLARS),
			DOLLARS * 100000
		);
		assert_eq!(
			<RootSquare_1000 as BoundingCurveInterface<Balance>>::buy_price(100 * DOLLARS),
			100000000
		);
		assert_eq!(
			<RootSquare_1000 as BoundingCurveInterface<Balance>>::sell_price(100 * DOLLARS),
			DOLLARS * 10000
		);

		#[allow(non_camel_case_types)]
		type RootSquare_700 = UnsignedSquareRoot<Balance, 700>; // y = 7√x
		assert_eq!(
			<RootSquare_700 as BoundingCurveInterface<Balance>>::buy_price(0),
			0
		);
		assert_eq!(
			<RootSquare_700 as BoundingCurveInterface<Balance>>::sell_price(0),
			0
		);
		assert_eq!(
			<RootSquare_700 as BoundingCurveInterface<Balance>>::buy_price(DOLLARS),
			7000000
		);
		assert_eq!(
			<RootSquare_700 as BoundingCurveInterface<Balance>>::sell_price(DOLLARS),
			142857142857142857
		);
		assert_eq!(
			<RootSquare_700 as BoundingCurveInterface<Balance>>::buy_price(100 * DOLLARS),
			70000000
		);
		assert_eq!(
			<RootSquare_700 as BoundingCurveInterface<Balance>>::sell_price(100 * DOLLARS),
			14285714285714285
		);
	}
}
