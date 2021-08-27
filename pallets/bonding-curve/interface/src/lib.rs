#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::traits::AtLeast32BitUnsigned;

pub trait BondingCurveInterface<Balance>
where
	Balance: AtLeast32BitUnsigned,
{
	/// calculate current token price by given `total_supply`
	fn buy_price(total_supply: Balance) -> Balance;

	/// This is the calculus function of this curve
	/// it calculates the area of a given x
	/// This area is the T in the pool
	fn pool_balance(x: Balance) -> Balance;

	/// given the area (tea token) calculate how much x (tapp token) change
	fn pool_balance_reverse(area: Balance, precision: Balance) -> Balance;
}
