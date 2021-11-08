#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use node_primitives::BlockNumber;
use sp_core::H256;
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
	pub trait TeaApi<AccountId>
	where
		AccountId: Codec,
	{
		fn is_ra_validator(
			tea_id: &[u8; 32],
			target_tea_id: &[u8; 32],
			block_number: BlockNumber,
		) -> bool;

		fn boot_nodes() -> Vec<[u8; 32]>;

		fn allowed_pcrs() -> Vec<(H256, Vec<Vec<u8>>)>;
	}
}
