#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use node_primitives::Balance;
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
	pub trait AuctionApi<AccountId>
	where
		AccountId: Codec,
	{
		fn user_auction_list(who: &AccountId) -> Vec<u64>;

		fn user_bid_list(who: &AccountId) -> Vec<u64>;

		fn current_auction_list() -> Vec<u64>;

		fn estimate_minimum_bid_price(auction_id: u64, who: &AccountId) -> (Balance, bool);

		fn penalty_amount(auction_id: u64, who: &AccountId) -> Balance;
	}
}
