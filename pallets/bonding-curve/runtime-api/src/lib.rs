#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use node_primitives::{Balance, BlockNumber};
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
	pub trait BondingCurveApi<AccountId>
	where
		AccountId: Codec,
	{
		fn query_price(tapp_id: u64) -> (Balance, Balance);

		fn estimate_required_tea_when_buy(tapp_id: Option<u64>, token_amount: Balance) -> Balance;

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
		/// - Host performance requirement (return zero if is none)
		/// - (current hosts (return zero if is none), max hosts (return zero if is none))
		/// - active block number (return none if not active)
		fn list_tapps(active_only: bool) -> Vec<(
			Vec<u8>,
			u64,
			Vec<u8>,
			Balance,
			Balance,
			Balance,
			AccountId,
			Vec<u8>,
			Vec<u8>,
			u32,
			(u32, u32),
			Option<BlockNumber>,
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
			u32,
			u32,
			u32,
			Balance,
		)>;

		fn tapp_details(tapp_id: u64) -> (
			Vec<u8>,
			u64,
			Vec<u8>,
			AccountId,
			Vec<u8>,
			Vec<u8>,
			u32,
			u32,
			u32,
			Balance,
			Balance,
			Balance,
		);

		/// Returned item fields:
		/// - CML Id
		/// - CML current performance
		/// - CML remaining performance
		/// - life remaining
		/// - Hosted tapp list
		fn list_candidate_miners(who: AccountId) -> Vec<(
			u64,
			u32,
			u32,
			BlockNumber,
			Vec<u64>)>;

		fn tapp_hosted_cmls(tapp_id: u64) -> Vec<(
			u64,
			Option<AccountId>,
			BlockNumber,
			Option<u32>,
			Option<u32>,
			u32)>;

		fn list_cml_hosting_tapps(cml_id: u64) -> Vec<(
			u64,
			Option<u32>,
			u64,
			Vec<u8>,
			Vec<u8>,
			Vec<u8>,
			Vec<u8>,
			u32)>;

		fn cml_performance(cml_id: u64) -> (Option<u32>, Option<u32>, u32);

		fn approved_links() -> Vec<(Vec<u8>, Option<u64>, Vec<u8>)>;
	}
}
