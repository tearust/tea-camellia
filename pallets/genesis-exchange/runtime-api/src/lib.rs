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
		/// Returns
		/// 1. current 1TEA equals how many USD amount
		/// 2. current 1USD equals how many TEA amount
		/// 3. exchange remains USD
		/// 4. exchange remains TEA
		/// 5. product of  exchange remains USD and exchange remains TEA
		fn current_exchange_rate() -> (
			Balance,
			Balance,
			Balance,
			Balance,
			Balance,
		);

		fn estimate_amount(withdraw_amount: Balance, buy_tea: bool) -> Balance;

		/// each of list items contains the following field:
		/// 1. Account
		/// 2. Projected  7 day mining income (TEA)
		/// 3. TEA Account balance (in TEA)
		/// 4. USD account balance
		/// 5. TApp token balance
		/// 6. genesis loan
		/// 7. USD debt
		/// 8. Total account value
		fn user_asset_list() -> Vec<(AccountId, Balance, Balance, Balance, Balance, Balance, Balance, Balance, Balance)>;

		fn user_borrowing_usd_margin(who: &AccountId) -> Balance;
	}
}
