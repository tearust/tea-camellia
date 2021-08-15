#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::traits::AtLeast32BitUnsigned;
use sp_std::vec::Vec;

pub trait GenesisBankOperation {
	type AccountId: PartialEq + Clone;
	type BlockNumber: Default + AtLeast32BitUnsigned + Clone;
	type Balance: Clone;

	fn is_cml_in_loan(cml_id: u64) -> bool;

	fn calculate_loan_amount(cml_id: u64, block_height: Self::BlockNumber) -> Self::Balance;

	fn user_collaterals(who: &Self::AccountId) -> Vec<(u64, Self::BlockNumber)>;
}
