#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::traits::AtLeast32BitUnsigned;

pub trait BuyBoundingCurve<Balance>
where
	Balance: AtLeast32BitUnsigned,
{
	fn buy(amount: Balance, total_supply: Balance) -> Balance;
}

pub trait SellBoundingCurve<Balance>
where
	Balance: AtLeast32BitUnsigned,
{
	fn sell(amount: Balance, total_supply: Balance) -> Balance;
}
