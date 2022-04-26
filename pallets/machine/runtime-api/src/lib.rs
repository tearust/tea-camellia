#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use sp_std::prelude::*;

sp_api::decl_runtime_apis! {
	pub trait MachineApi<AccountId>
	where
		AccountId: Codec,
	{
		fn boot_nodes() -> Vec<[u8; 32]>;

		fn tapp_store_startup_nodes() -> Vec<[u8; 32]>;
	}
}
