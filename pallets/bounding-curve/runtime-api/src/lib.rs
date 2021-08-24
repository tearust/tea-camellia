#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use node_primitives::Balance;
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
	pub trait BoundingCurveApi<AccountId>
	where
		AccountId: Codec,
	{
		fn query_price(tapp_id: u64) -> (Balance, Balance);

		fn estimate_required_tea_when_buy(tapp_id: u64, token_amount: Balance) -> Balance;

		fn estimate_receive_tea_when_sell(tapp_id: u64, token_amount: Balance) -> Balance;

		fn estimate_receive_token_when_buy(tapp_id: u64, tea_amount: Balance) -> Balance;

		fn estimate_required_token_when_sell(tapp_id: u64, tea_amount: Balance) -> Balance;

		/// Returned item fields:
		/// - TApp Name
		/// - TApp Id
		/// - TApp Ticker
		/// - Total supply
		/// - Token buy price
		/// - Token sell price
		/// - Owner
		/// - Detail
		/// - Link
		fn list_tapps() -> Vec<(
			Vec<u8>,
			u64,
			Vec<u8>,
			Balance,
			Balance,
			Balance,
			AccountId,
			Vec<u8>,
			Vec<u8>,
		)>;

		/// Returned item fields:
		/// - TApp Name
		/// - TApp Id
		/// - TApp Ticker
		/// - User holding tokens
		/// - Token sell price
		/// - Owner
		/// - Detail
		/// - Link
		fn list_user_assets(who: AccountId) -> Vec<(
			Vec<u8>,
			u64,
			Vec<u8>,
			Balance,
			Balance,
			AccountId,
			Vec<u8>,
			Vec<u8>,
		)>;
	}
}
