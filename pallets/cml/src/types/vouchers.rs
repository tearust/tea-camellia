use crate::{CmlType, Voucher, VoucherUnlockType};
use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

#[derive(Encode, Decode, PartialEq, Clone, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct VoucherConfig<AccountId> {
	pub account: AccountId,
	pub cml_type: CmlType,
	pub amount: u32,
	pub lock: Option<u32>,
	pub unlock_type: Option<VoucherUnlockType>,
}

#[derive(Encode, Decode, Clone, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct GenesisVouchers<AccountId> {
	pub vouchers: Vec<VoucherConfig<AccountId>>,
}

impl<AccountId> Into<Voucher> for VoucherConfig<AccountId> {
	fn into(self) -> Voucher {
		Voucher {
			amount: self.amount,
			lock: self.lock,
			unlock_type: self.unlock_type,
			cml_type: self.cml_type,
		}
	}
}

impl<AccountId> VoucherConfig<AccountId> {
	pub fn new(
		account: AccountId,
		cml_type: CmlType,
		amount: u32,
		lock: Option<u32>,
		unlock_type: Option<VoucherUnlockType>,
	) -> Self {
		VoucherConfig {
			account,
			cml_type,
			amount,
			lock,
			unlock_type,
		}
	}
}

impl<AccountId> Default for GenesisVouchers<AccountId> {
	fn default() -> Self {
		GenesisVouchers { vouchers: vec![] }
	}
}
