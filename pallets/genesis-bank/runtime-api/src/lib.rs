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
		fn cml_calculate_loan_amount(cml_id: u64, block_height: BlockNumber) -> Balance;

		fn user_collateral_list(who: &AccountId) -> Vec<u64>;
	}
}
