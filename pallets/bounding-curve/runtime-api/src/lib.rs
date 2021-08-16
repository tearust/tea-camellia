#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use node_primitives::{Balance, BlockNumber};
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
	pub trait BoundingCurveApi<AccountId>
	where
		AccountId: Codec,
	{
		fn query_price(tapp_id: u64) -> (Balance, Balance);

		fn estimate_buy(tapp_id: u64, amount: Balance) -> Balance;

		fn estimate_sell(tapp_id: u64, amount: Balance) -> Balance;
	}
}
