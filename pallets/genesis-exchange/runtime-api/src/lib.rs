#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use node_primitives::Balance;
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
	pub trait GenesisExchangeApi<AccountId>
	where
		AccountId: Codec,
	{
		fn current_exchange_rate() -> Balance;

		fn reverse_exchange_rate() -> Balance;

		fn estimate_amount(withdraw_amount: Balance, buy_tea: bool) -> Balance;

		/// each of list items contains the following field:
		/// 1. Account
		/// 2. Projected  7 day mining income (USD)
		/// 3. TEA Account balance (in USD)
		/// 4. USD account balance
		/// 5. Genesis stake debt
		/// 6. genesis loan
		/// 7. Total account value
		fn user_asset_list() -> Vec<(AccountId, Balance, Balance, Balance, Balance, Balance, Balance)>;
	}
}
