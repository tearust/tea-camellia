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

		fn estimate_amount(withdraw_amount: Balance, buy_tea: bool) -> Balance;

		fn user_asset_list() -> Vec<(AccountId, Balance)>;
	}
}
