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

		fn estimate_required_tea_when_buy(tapp_id: Option<u64>, token_amount: Balance, buy_curve_k: Option<u32>) -> Balance;

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
		/// - 1. User holding tokens (inverstor side only, not including mining reserved balance)
		///   2. User reserved tokens (mining reserved balance only)
		/// - Token sell price
		/// - Owner
		/// - Detail
		/// - Link
		fn list_user_assets(who: AccountId) -> Vec<(
			Vec<u8>,
			u64,
			Vec<u8>,
			(Balance, Balance),
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

		fn tapp_staking_details(
			tapp_id: u64,
			only_investing: bool,
		) -> Vec<(AccountId, Balance)>;

		fn list_cml_hosting_tapps(cml_id: u64) -> Vec<(
			u64,
			Option<u32>,
			u64,
			Vec<u8>,
			Vec<u8>,
			Vec<u8>,
			Vec<u8>,
			u32,
			Balance)>;

		fn cml_performance(cml_id: u64) -> (Option<u32>, Option<u32>, u32);

		fn approved_links(allowed: bool) -> Vec<(Vec<u8>, Option<u64>, Vec<u8>, Option<AccountId>)>;

		fn user_notification_count(account: AccountId) -> u32;
	}
}
