use crate::{CmlType, DefrostScheduleType};
use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
pub struct Coupon {
	pub amount: u32,
	pub cml_type: CmlType,
}

#[derive(Encode, Decode, PartialEq, Clone, Debug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CouponConfig<AccountId> {
	pub account: AccountId,
	pub cml_type: CmlType,
	pub schedule_type: DefrostScheduleType,
	pub amount: u32,
}

#[derive(Encode, Decode, Clone, Debug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct GenesisCoupons<AccountId> {
	pub coupons: Vec<CouponConfig<AccountId>>,
}

impl<AccountId> Into<Coupon> for CouponConfig<AccountId> {
	fn into(self) -> Coupon {
		Coupon {
			amount: self.amount,
			cml_type: self.cml_type,
		}
	}
}

impl<AccountId> CouponConfig<AccountId> {
	pub fn new(
		account: AccountId,
		cml_type: CmlType,
		schedule_type: DefrostScheduleType,
		amount: u32,
	) -> Self {
		CouponConfig {
			account,
			cml_type,
			schedule_type,
			amount,
		}
	}
}

impl<AccountId> Default for GenesisCoupons<AccountId> {
	fn default() -> Self {
		GenesisCoupons { coupons: vec![] }
	}
}
