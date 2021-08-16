#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_std::vec::Vec;

pub trait BoundingCurveFunctions {
	type Balance: Default;

	fn buy_curve(&self, total_supply: Self::Balance) -> Self::Balance;

	fn sell_curve(&self, total_supply: Self::Balance) -> Self::Balance;
}
