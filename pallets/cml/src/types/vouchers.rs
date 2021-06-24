use crate::{CmlType, DefrostScheduleType};
use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug)]
pub struct Voucher {
	pub amount: u32,
	pub cml_type: CmlType,
}

#[derive(Encode, Decode, PartialEq, Clone, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct VoucherConfig<AccountId> {
	pub account: AccountId,
	pub cml_type: CmlType,
	pub schedule_type: DefrostScheduleType,
	pub amount: u32,
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
			cml_type: self.cml_type,
		}
	}
}

impl<AccountId> VoucherConfig<AccountId> {
	pub fn new(
		account: AccountId,
		cml_type: CmlType,
		schedule_type: DefrostScheduleType,
		amount: u32,
	) -> Self {
		VoucherConfig {
			account,
			cml_type,
			schedule_type,
			amount,
		}
	}
}

impl<AccountId> Default for GenesisVouchers<AccountId> {
	fn default() -> Self {
		GenesisVouchers { vouchers: vec![] }
	}
}
