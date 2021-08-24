#![cfg_attr(not(feature = "std"), no_std)]

use bounding_curve_interface::BoundingCurveInterface;
use sp_runtime::traits::{AtLeast32BitUnsigned, One, Zero};
use sp_std::fmt::Debug;
use sp_std::marker::PhantomData;

pub mod linear;
pub mod square_root;

pub fn approximately_equals<Balance>(a: Balance, b: Balance, precision: Balance) -> bool
where
	Balance: AtLeast32BitUnsigned + Default + Clone,
{
	let abs = match a >= b {
		true => a - b,
		false => b - a,
	};
	abs <= precision
}
