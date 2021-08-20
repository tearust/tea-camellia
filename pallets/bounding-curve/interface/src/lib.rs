#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::traits::AtLeast32BitUnsigned;

pub trait BoundingCurve<Balance>
where
	Balance: AtLeast32BitUnsigned,
{
	/// calculate current price by given `total_supply`
	fn buy_price(total_supply: Balance) -> Balance;

	fn sell_price(total_supply: Balance) -> Balance;

	fn pool_balance(x: Balance, delta_x: Balance, negative: bool) -> Balance;

	fn pool_balance_reverse(x: Balance, tea_amount: Balance, negative: bool) -> Balance;
}
