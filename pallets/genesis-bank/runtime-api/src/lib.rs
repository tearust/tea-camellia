#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use node_primitives::{Balance, BlockNumber};
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
	pub trait GenesisBankApi<AccountId>
	where
		AccountId: Codec,
	{
		fn cml_lien_redeem_amount(cml_id: u64, block_height: BlockNumber) -> Balance;

		fn user_cml_lien_list(who: &AccountId) -> Vec<u64>;

		fn bank_owned_cmls() -> Vec<u64>;
	}
}
