#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
	pub trait AuctionApi<AccountId>
	where
		AccountId: Codec,
	{
		fn user_auction_list(who: &AccountId) -> Vec<u64>;

		fn user_bid_list(who: &AccountId) -> Vec<u64>;
	}
}
