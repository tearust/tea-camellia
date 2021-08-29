use crate::square_root::k10::K10_STEP100_AREA_LIST;
use crate::square_root::k7::K7_STEP100_AREA_LIST;
use crate::*;

mod k10;
mod k7;

const AREA_LIST_LENGTH: usize = 1000;
const K_BASE_POINT: u32 = 100000000u32;

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
	fn recursively_balance_reverse_calculation(
		x_n: Balance,
		area: Balance,
		precision: Balance,
		times: &mut usize,
	) -> Balance {
		*times += 1;
		// println!("xn {:?}", &x_n);
		// println!(
		// 	"parta {:?}",
		// 	area.clone() / K.into() * 10u32.into() * 1_000_000u32.into() / x_n.integer_sqrt()
		// );
		// println!("partb {:?}", x_n.clone() * 2u32.into() / 3u32.into());
		let result = if x_n.is_zero() {
			Zero::zero()
		} else {
			x_n.clone()
				+ area.clone() / K.into() * 10u32.into() / 1_000_000u32.into() / x_n.integer_sqrt()
				- x_n.clone() * 2u32.into() / 3u32.into() / 1_000_000u32.into() / 1_000_000u32.into()
		};

		// println!("result {:?}", &result);

		if approximately_equals(x_n, result.clone(), precision.clone()) {
			result
		} else {
			Self::recursively_balance_reverse_calculation(result, area, precision, times)
		}
	}

	// fn select_nearest_area_and_x(current_area: Balance) -> (Balance, Balance) {
	// 	let select_fn = |it: &[u32; AREA_LIST_LENGTH]| {
	// 		let mut best_area: u32 = 0;
	// 		let mut best_x: u32 = 0;
	// 		for (i, area) in it.iter().enumerate() {
	// 			if current_area < Balance::from(*area) {
	// 				break;
	// 			}

	// 			best_area = *area;
	// 			if i.is_zero() {
	// 				best_x = 1;
	// 			} else {
	// 				best_x = 100 * i as u32;
	// 			}
	// 		}

	// 		(Balance::from(best_area), Balance::from(best_x))
	// 	};

	// 	match K {
	// 		1000 => select_fn(&K10_STEP100_AREA_LIST),
	// 		700 => select_fn(&K7_STEP100_AREA_LIST),
	// 		_ => (Self::pool_balance(One::one()), One::one()),
	// 	}
	// }
}

impl<Balance, const K: u32> BondingCurveInterface<Balance> for UnsignedSquareRoot<Balance, K>
where
	Balance: AtLeast32BitUnsigned + Default + Clone + Debug,
{
	/// Input unit 1E12 is one token. Output is unit 1E12 is one TEA
	fn buy_price(total_supply: Balance) -> Balance {
		total_supply.integer_sqrt() * K.into() / 10u32.into() * 1_000_000u32.into()
	}

	/// Input unit 1E12 is one token. Output is unit 1E12 is one TEA
	fn pool_balance(x: Balance) -> Balance {
		x.integer_sqrt() * x.clone() * K.into() * 2u32.into() / 1_000_000u32.into() / 30u32.into()
	}
	/// Input unit 1E12 is one token. Output is unit 1E12 is one TEA
	fn pool_balance_reverse(area: Balance, precision: Balance) -> Balance {
		if area.is_zero() {
			return Zero::zero();
		}

		
		let mut times = 0;
		let mut last_diff = Balance::zero();
		let mut x_n: Balance = Balance::from(1_100_000u32) * Balance::from(1_000_000u32);
		let diff = |a: Balance, b: Balance| {
			if a > b {
				a - b
			} else {
				b - a
			}
		};
		loop {
			let x_n_plus_1: Balance = {
				if x_n.is_zero() {
					Zero::zero()
				} else {
					x_n.clone()
						+ area.clone() / K.into() * 10u32.into() * 1_000_000u32.into() / x_n.integer_sqrt()
						- x_n.clone() * 2u32.into() / 3u32.into()
				}
			};
			// println!(
			// 	"precision is {:?}, xn is {:?}, diff is {:?}, time: {:?}",
			// 	precision.clone(),
			// 	x_n.clone(),
			// 	x_n_plus_1.clone(),
			// 	&times
			// );
			if approximately_equals(x_n.clone(), x_n_plus_1.clone(), precision.clone()) {
				#[cfg(feature = "std")]
				println!(
					"Exit now   precision is {:?}, xn is {:?}, diff is {:?}, time: {:?}",
					precision.clone(),
					x_n.clone(),
					diff(x_n.clone(), x_n_plus_1.clone()),
					&times
				);
				#[cfg(feature = "std")]
				println!("exiting with {} loops", times);
				return x_n_plus_1;
			} else {
				let new_diff = diff(x_n.clone(), x_n_plus_1.clone());
				if (last_diff > Balance::zero()) && (new_diff.clone() > last_diff.clone()) {
					#[cfg(feature = "std")]
					println!(
						"Exit now because the diff increased  precision is {:?}, xn is {:?}, diff is {:?}, time: {:?}",
						precision.clone(),
						x_n.clone(),
						&last_diff,
						&times
					);
					#[cfg(feature = "std")]
					println!("exiting with {} loops", times);
					return x_n;
				}
				x_n = x_n_plus_1;
				last_diff = new_diff;
				times += 1;
			}
		}
		// result
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

	const CENTS: node_primitives::Balance = 10_000_000_000;
	const DOLLARS: node_primitives::Balance = 100 * CENTS;

	#[test]
	fn pool_balance_works() {
		#[allow(non_camel_case_types)]
		type RootSquare_10 = UnsignedSquareRoot<Balance, 10>; // y = 10√x
		assert_eq!(
			<RootSquare_10 as BondingCurveInterface<Balance>>::pool_balance(0),
			0
		);
		assert_eq!(
			<RootSquare_10 as BondingCurveInterface<Balance>>::pool_balance(100000),
			21
		);
		assert_eq!(
			<RootSquare_10 as BondingCurveInterface<Balance>>::pool_balance(1000000),
			666
		);
		assert_eq!(
			<RootSquare_10 as BondingCurveInterface<Balance>>::pool_balance(10000000),
			21080
		);
		assert_eq!(
			<RootSquare_10 as BondingCurveInterface<Balance>>::pool_balance(100000000),
			666666
		);
		assert_eq!(
			<RootSquare_10 as BondingCurveInterface<Balance>>::pool_balance(DOLLARS),
			666666666666
		);
		assert_eq!(
			<RootSquare_10 as BondingCurveInterface<Balance>>::pool_balance(100 * DOLLARS),
			666666666666666
		);

		#[allow(non_camel_case_types)]
		type RootSquare_7 = UnsignedSquareRoot<Balance, 7>; // y = 7√x
		assert_eq!(
			<RootSquare_7 as BondingCurveInterface<Balance>>::pool_balance(0),
			0
		);
		assert_eq!(
			<RootSquare_7 as BondingCurveInterface<Balance>>::pool_balance(1000000),
			466
		);
		assert_eq!(
			<RootSquare_7 as BondingCurveInterface<Balance>>::pool_balance(10000000),
			14756
		);
		assert_eq!(
			<RootSquare_7 as BondingCurveInterface<Balance>>::pool_balance(100000000),
			466666
		);
		assert_eq!(
			<RootSquare_7 as BondingCurveInterface<Balance>>::pool_balance(DOLLARS),
			466666666666
		);
		assert_eq!(
			<RootSquare_7 as BondingCurveInterface<Balance>>::pool_balance(100 * DOLLARS),
			466666666666666
		);
	}
	#[test]
	fn kevin() {
		#[allow(non_camel_case_types)]
		type RootSquare_10 = UnsignedSquareRoot<Balance, 10>; // y = 10√x
		type RootSquare_7 = UnsignedSquareRoot<Balance, 7>; // y = 7√x
		println!("Kevin say hi");
		let x = <RootSquare_10 as BondingCurveInterface<Balance>>::buy_price(DOLLARS);
		println!("K10 buy one token use T:{:?}/10000", &x / 100000000);
		let x = <RootSquare_10 as BondingCurveInterface<Balance>>::buy_price(100 * DOLLARS);
		println!("K10 buy 100 token use T:{:?}/10000", &x / 100000000);
		let x = <RootSquare_7 as BondingCurveInterface<Balance>>::buy_price(DOLLARS);
		println!("K7 buy one token use T:{:?}/10000", &x / 100000000);
		let x = <RootSquare_7 as BondingCurveInterface<Balance>>::buy_price(100 * DOLLARS);
		println!("K7 buy 100 token use T:{:?}/10000", &x / 100000000);
		let x = <RootSquare_10 as BondingCurveInterface<Balance>>::pool_balance(DOLLARS);
		println!(
			"K10 when supply is 1 token, pool balance T is {:?}/10000",
			x / 100000000 // <RootSquare_10 as BondingCurveInterface<Balance>>::pool_balance(DOLLARS)
		);
		println!(
			"K10 now let us find how much token can receive when spending {:?}/10000 TEA. answer is {:?}/10000",
			x / 100000000,
			<RootSquare_10 as BondingCurveInterface<Balance>>::pool_balance_reverse(x, 10) / 100000000
		);
		let x = <RootSquare_10 as BondingCurveInterface<Balance>>::pool_balance(100 * DOLLARS);
		println!(
			"K10 when supply is 100 token, pool balance T is {:?}/10000",
			x / 100000000
		);
		println!(
			"K10  now let us find how much token can receive when spending {:?}/10000 TEA. answer is {:?}/10000",
			x / 100000000,
			<RootSquare_10 as BondingCurveInterface<Balance>>::pool_balance_reverse(x, 10)
				/ 100000000
		);

		let x = <RootSquare_7 as BondingCurveInterface<Balance>>::pool_balance(DOLLARS);
		println!(
			"K7 when supply is 1 token, pool balance T is {:?}/10000",
			x / 100000000 // <RootSquare_10 as BondingCurveInterface<Balance>>::pool_balance(DOLLARS)
		);
		println!(
			"K7 now let us find how much token can receive when spending {:?}/10000 TEA. answer is {:?}/10000",
			x / 100000000,
			<RootSquare_7 as BondingCurveInterface<Balance>>::pool_balance_reverse(x, 10)
				/ 100000000
		);
		let x = <RootSquare_7 as BondingCurveInterface<Balance>>::pool_balance(100 * DOLLARS);
		println!(
			"K7 when supply is 100 token, pool balance T is {:?}/10000",
			x / 100000000
		);
		println!(
			"K7  now let us find how much token can receive when spending {:?}/10000 TEA. answer is {:?}/10000",
			x / 100000000,
			<RootSquare_7 as BondingCurveInterface<Balance>>::pool_balance_reverse(x, 10) / 100000000
		);
	}

	// 666666666666
	// #[test]
	// fn pool_balance_reverse_works() {
	// 	#[allow(non_camel_case_types)]
	// 	type RootSquare_10 = UnsignedSquareRoot<Balance, 10>; // y = 10√x
	// 	assert_eq!(
	// 		<RootSquare_10 as BondingCurveInterface<Balance>>::pool_balance_reverse(0, 1),
	// 		0
	// 	);
	// 	assert!(approximately_equals(
	// 		<RootSquare_10 as BondingCurveInterface<Balance>>::pool_balance_reverse(66, 1),
	// 		1000000,
	// 		7000
	// 	));
	// 	assert_eq!(
	// 		<RootSquare_10 as BondingCurveInterface<Balance>>::pool_balance_reverse(2108, 1),
	// 		10000000
	// 	);
	// 	assert!(approximately_equals(
	// 		<RootSquare_10 as BondingCurveInterface<Balance>>::pool_balance_reverse(66666, 100000),
	// 		100000000,
	// 		6000
	// 	));
	// 	assert!(approximately_equals(
	// 		<RootSquare_10 as BondingCurveInterface<Balance>>::pool_balance_reverse(66666666666, 100000),
	// 		DOLLARS,
	// 		20000
	// 	));
	// 	assert!(approximately_equals(
	// 		<RootSquare_10 as BondingCurveInterface<Balance>>::pool_balance_reverse(
	// 			66666666666666,
	// 			100000
	// 		),
	// 		100 * DOLLARS,
	// 		35000
	// 	));

	// 	#[allow(non_camel_case_types)]
	// 	type RootSquare_7 = UnsignedSquareRoot<Balance, 7>; // y = 7√x
	// 	assert_eq!(
	// 		<RootSquare_7 as BondingCurveInterface<Balance>>::pool_balance_reverse(0, 1),
	// 		0
	// 	);
	// 	assert!(approximately_equals(
	// 		<RootSquare_7 as BondingCurveInterface<Balance>>::pool_balance_reverse(46, 1),
	// 		1000000,
	// 		10000,
	// 	));
	// 	assert!(approximately_equals(
	// 		<RootSquare_7 as BondingCurveInterface<Balance>>::pool_balance_reverse(1475, 100000),
	// 		10000000,
	// 		3000
	// 	));
	// 	assert!(approximately_equals(
	// 		<RootSquare_7 as BondingCurveInterface<Balance>>::pool_balance_reverse(46666, 100000),
	// 		100000000,
	// 		5500
	// 	));
	// 	assert!(approximately_equals(
	// 		<RootSquare_7 as BondingCurveInterface<Balance>>::pool_balance_reverse(46666666666, 100000),
	// 		DOLLARS,
	// 		20000
	// 	));
	// 	assert!(approximately_equals(
	// 		<RootSquare_7 as BondingCurveInterface<Balance>>::pool_balance_reverse(
	// 			46666666666666,
	// 			100000
	// 		),
	// 		100 * DOLLARS,
	// 		35000
	// 	));
	// }

	#[test]
	fn buy_and_sell_price_works() {
		#[allow(non_camel_case_types)]
		type RootSquare_10 = UnsignedSquareRoot<Balance, 10>; // y = 10√x
		assert_eq!(
			<RootSquare_10 as BondingCurveInterface<Balance>>::buy_price(0),
			0
		);
		assert_eq!(
			<RootSquare_10 as BondingCurveInterface<Balance>>::buy_price(DOLLARS),
			1_000_000_000_000
		);
		assert_eq!(
			<RootSquare_10 as BondingCurveInterface<Balance>>::buy_price(100 * DOLLARS),
			10_000_000_000_000
		);
		assert_eq!(
			<RootSquare_10 as BondingCurveInterface<Balance>>::buy_price(10000 * DOLLARS),
			100_000_000_000_000
		);

		#[allow(non_camel_case_types)]
		type RootSquare_7 = UnsignedSquareRoot<Balance, 7>; // y = 7√x
		assert_eq!(
			<RootSquare_7 as BondingCurveInterface<Balance>>::buy_price(0),
			0
		);
		assert_eq!(
			<RootSquare_7 as BondingCurveInterface<Balance>>::buy_price(DOLLARS),
			700_000_000_000
		);
		assert_eq!(
			<RootSquare_7 as BondingCurveInterface<Balance>>::buy_price(100 * DOLLARS),
			7_000_000_000_000
		);
		assert_eq!(
			<RootSquare_7 as BondingCurveInterface<Balance>>::buy_price(10000 * DOLLARS),
			70_000_000_000_000
		);
	}
}
