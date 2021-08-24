#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::DispatchResult;

pub trait MiningOperation {
	type AccountId: PartialEq + Clone;

	fn check_buying_mining_machine(who: &Self::AccountId, cml_id: u64) -> DispatchResult;

	fn buy_mining_machine(who: &Self::AccountId, cml_id: u64);
}
