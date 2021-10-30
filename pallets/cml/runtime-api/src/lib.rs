#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use node_primitives::Balance;
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
	pub trait CmlApi<AccountId>
	where
		AccountId: Codec,
	{
		fn user_cml_list(who: &AccountId) -> Vec<u64>;

		/// return type: the first field is ID of the cml, the second field is slot index within
		/// the cml.
		fn user_staking_list(who: &AccountId) -> Vec<(u64, u64)>;

		fn current_mining_cml_list() -> Vec<(u64, Vec<u8>, Vec<u8>, Vec<u8>)>;

		fn staking_price_table() -> Vec<Balance>;

		fn estimate_stop_mining_penalty(cml_id: u64) -> Balance;
	}
}
