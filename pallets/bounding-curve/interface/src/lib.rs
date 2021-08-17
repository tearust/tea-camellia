#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_std::vec::Vec;

pub trait BuyBoundingCurve<Balance> {
	fn buy(amount: Balance, total_supply: Balance) -> Balance;
}

pub trait SellBoundingCurve<Balance> {
	fn sell(amount: Balance, total_supply: Balance) -> Balance;
}
