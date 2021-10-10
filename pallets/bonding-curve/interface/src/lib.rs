#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_std::prelude::*;

pub trait BondingCurveInterface<Balance>
where
	Balance: AtLeast32BitUnsigned,
{
	/// calculate current token price by given `total_supply`
	fn buy_price(&self, total_supply: Balance) -> Balance;

	/// This is the calculus function of this curve
	/// it calculates the area of a given x
	/// This area is the T in the pool
	fn pool_balance(&self, x: Balance) -> Balance;

	/// given the area (tea token) calculate how much x (tapp token) change
	fn pool_balance_reverse(&self, area: Balance, precision: Balance) -> Balance;
}

pub trait BondingCurveOperation {
	type AccountId: Default;
	type Balance: Default;

	fn list_tapp_ids() -> Vec<u64>;

	fn estimate_hosting_income_statements(
		tapp_id: u64,
	) -> Vec<(Self::AccountId, u64, Self::Balance)>;

	fn current_price(tapp_id: u64) -> (Self::Balance, Self::Balance);

	fn tapp_user_token_asset(who: &Self::AccountId) -> Vec<(u64, Self::Balance)>;

	fn is_cml_hosting(cml_id: u64) -> bool;

	fn transfer_reserved_tokens(from: &Self::AccountId, to: &Self::AccountId, cml_id: u64);

	fn npc_account() -> Self::AccountId;

	fn cml_host_tapps(cml_id: u64) -> Vec<u64>;

	fn try_active_tapp(tapp_id: u64) -> bool;

	fn try_deactive_tapp(tapp_id: u64) -> bool;

	fn pay_hosting_penalty(tapp_id: u64, cml_id: u64);

	fn can_append_pledge(cml_id: u64) -> bool;

	fn append_pledge(cml_id: u64) -> bool;
}
