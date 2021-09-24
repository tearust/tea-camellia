use crate::square_root::k10::K10_AREAS;
use crate::square_root::k7::K7_AREAS;
use crate::*;

const REFERENCE_POINTS_SIZE: usize = 1000;

mod k10;
mod k7;

/// Implement equation: `y = a√x`
///
/// The genesis const parameter `k` represents the 100 times of `a`.
pub struct UnsignedSquareRoot {
	k: u32,
}

impl<Balance> BondingCurveInterface<Balance> for UnsignedSquareRoot
where
	Balance: AtLeast32BitUnsigned + Default + Clone + Debug,
{
	/// Input unit 1E12 is one token. Output is unit 1E12 is one TEA
	fn buy_price(&self, total_supply: Balance) -> Balance {
		total_supply.integer_sqrt() * self.k.into() / 10u32.into() * 1_000_000u32.into()
	}

	/// Input unit 1E12 is one token. Output is unit 1E12 is one TEA
	fn pool_balance(&self, x: Balance) -> Balance {
		x.integer_sqrt() * x.clone() * self.k.into() * 2u32.into()
			/ 1_000_000u32.into()
			/ 30u32.into()
	}
	/// Input unit 1E12 is one token. Output is unit 1E12 is one TEA
	fn pool_balance_reverse(&self, area: Balance, precision: Balance) -> Balance {
		if area.is_zero() {
			return Zero::zero();
		}

		let mut times = 0;
		let mut last_diff = Balance::zero();
		let mut x_n: Balance = self.select_nearest_reference_point(&area);
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
						+ area.clone() / self.k.into() * 10u32.into() * 1_000_000u32.into()
							/ x_n.integer_sqrt() - x_n.clone() * 2u32.into() / 3u32.into()
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
	}
}

impl UnsignedSquareRoot {
	pub fn new(k: u32) -> Self {
		UnsignedSquareRoot { k: k }
	}

	fn select_nearest_reference_point<Balance>(&self, area: &Balance) -> Balance
	where
		Balance: AtLeast32BitUnsigned + Default + Clone + Debug,
	{
		const CENTS: u128 = 10_000_000_000;
		const DOLLARS: u128 = 100 * CENTS;

		let default_starter: Balance = Balance::from(1_100_000u32) * Balance::from(1_000_000u32);

		let nearest_point = |areas: &[u128; REFERENCE_POINTS_SIZE]| {
			let mut nearest_index = 0;
			for i in 0..REFERENCE_POINTS_SIZE {
				if Balance::try_from(areas[i]).unwrap_or(Zero::zero()) > *area {
					break;
				}
				nearest_index = i;
			}

			match nearest_index {
				0 => default_starter.clone(),
				_ => Balance::try_from((nearest_index as u128) * 100 * DOLLARS)
					.unwrap_or(Zero::zero()),
			}
		};

		match self.k {
			10 => nearest_point(&K10_AREAS),
			7 => nearest_point(&K7_AREAS),
			_ => default_starter,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::panic;

	const CENTS: node_primitives::Balance = 10_000_000_000;
	const DOLLARS: node_primitives::Balance = 100 * CENTS;

	#[test]
	fn tests() {
		let root_square_10 = UnsignedSquareRoot::new(10); // y = 10√x
		for i in 1..=1000 {
			println!("{},", root_square_10.pool_balance(i * 100 * DOLLARS),);
		}
	}

	#[test]
	fn pool_balance_works() {
		let root_square_10 = UnsignedSquareRoot::new(10); // y = 10√x
		assert_eq!(root_square_10.pool_balance(0u128), 0);
		assert_eq!(root_square_10.pool_balance(100000u128), 21);
		assert_eq!(root_square_10.pool_balance(1000000u128), 666);
		assert_eq!(root_square_10.pool_balance(10000000u128), 21080);
		assert_eq!(root_square_10.pool_balance(100000000u128), 666666);
		assert_eq!(root_square_10.pool_balance(DOLLARS), 666666666666);
		assert_eq!(root_square_10.pool_balance(100 * DOLLARS), 666666666666666);

		let root_square_7 = UnsignedSquareRoot::new(7); // y = 7√x
		assert_eq!(root_square_7.pool_balance(0u128), 0);
		assert_eq!(root_square_7.pool_balance(1000000u128), 466);
		assert_eq!(root_square_7.pool_balance(10000000u128), 14756);
		assert_eq!(root_square_7.pool_balance(100000000u128), 466666);
		assert_eq!(root_square_7.pool_balance(DOLLARS), 466666666666);
		assert_eq!(root_square_7.pool_balance(100 * DOLLARS), 466666666666666);
	}
	#[test]
	fn combined_test_buy_sell_tapp_tokevin() {
		let root_square_10 = UnsignedSquareRoot::new(10); // y = 10√x
		let root_square_7 = UnsignedSquareRoot::new(7); // y = 7√x
		let x = root_square_10.buy_price(DOLLARS);
		println!("K10 buy one token use T:{:?}/10000", &x / 100000000);
		let x = root_square_10.buy_price(100 * DOLLARS);
		println!("K10 buy 100 token use T:{:?}/10000", &x / 100000000);
		let x = root_square_7.buy_price(DOLLARS);
		println!("K7 buy one token use T:{:?}/10000", &x / 100000000);
		let x = root_square_7.buy_price(100 * DOLLARS);
		println!("K7 buy 100 token use T:{:?}/10000", &x / 100000000);
		let x = root_square_10.pool_balance(DOLLARS);
		println!(
			"K10 when supply is 1 token, pool balance T is {:?}/10000",
			x / 100000000 // <RootSquare10 as BondingCurveInterface<Balance>>::pool_balance(DOLLARS)
		);
		println!(
			"K10 now let us find how much token can receive when spending {:?}/10000 TEA. answer is {:?}/10000",
			x / 100000000,
			root_square_10.pool_balance_reverse(x, 10) / 100000000
		);
		let x = root_square_10.pool_balance(100 * DOLLARS);
		println!(
			"K10 when supply is 100 token, pool balance T is {:?}/10000",
			x / 100000000
		);
		println!(
			"K10  now let us find how much token can receive when spending {:?}/10000 TEA. answer is {:?}/10000",
			x / 100000000,
			root_square_10.pool_balance_reverse(x, 10)
				/ 100000000
		);

		let x = root_square_7.pool_balance(DOLLARS);
		println!(
			"K7 when supply is 1 token, pool balance T is {:?}/10000",
			x / 100000000 // <RootSquare10 as BondingCurveInterface<Balance>>::pool_balance(DOLLARS)
		);
		println!(
			"K7 now let us find how much token can receive when spending {:?}/10000 TEA. answer is {:?}/10000",
			x / 100000000,
			root_square_7.pool_balance_reverse(x, 10)
				/ 100000000
		);
		let x = root_square_7.pool_balance(100 * DOLLARS);
		println!(
			"K7 when supply is 100 token, pool balance T is {:?}/10000",
			x / 100000000
		);
		println!(
			"K7  now let us find how much token can receive when spending {:?}/10000 TEA. answer is {:?}/10000",
			x / 100000000,
			root_square_7.pool_balance_reverse(x, 10) / 100000000
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
		let root_square_10 = UnsignedSquareRoot::new(10); // y = 10√x
		assert_eq!(root_square_10.buy_price(0u128), 0);
		assert_eq!(root_square_10.buy_price(DOLLARS), 1_000_000_000_000);
		assert_eq!(root_square_10.buy_price(100 * DOLLARS), 10_000_000_000_000);
		assert_eq!(
			root_square_10.buy_price(10000 * DOLLARS),
			100_000_000_000_000
		);

		let root_square_7 = UnsignedSquareRoot::new(7); // y = 7√x
		assert_eq!(root_square_7.buy_price(0u128), 0);
		assert_eq!(root_square_7.buy_price(DOLLARS), 700_000_000_000);
		assert_eq!(root_square_7.buy_price(100 * DOLLARS), 7_000_000_000_000);
		assert_eq!(root_square_7.buy_price(10000 * DOLLARS), 70_000_000_000_000);
	}

	#[test]
	fn check_pool_balance_multiply_overflow() {
		let root_square_10 = UnsignedSquareRoot::new(10); // y = 10√x
												  // 1e24 if safe
		assert_eq!(
			root_square_10.pool_balance(1000000000000000000000000u128),
			666666666666666666666666666666
		);

		let result = panic::catch_unwind(|| {
			root_square_10.pool_balance(1000000000000000000000000u128 * 10);
		});
		// should multiply overflow
		assert!(result.is_err());
	}
}
