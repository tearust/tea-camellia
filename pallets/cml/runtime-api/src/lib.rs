#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
	pub trait CmlApi<AccountId>
	where
		AccountId: Codec,
	{
		fn get_user_cml_list(who: &AccountId) -> Vec<u64>;

		/// return type: the first field is ID of the cml, the second field is slot index within
		/// the cml.
		fn get_user_staking_list(who: &AccountId) -> Vec<(u64, u64)>;
	}
}
