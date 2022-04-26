use super::cli::Cli;
use camellia_runtime::{
	pallet_cml::{CmlType, DefrostScheduleType},
	AccountId,
};
use sp_core::crypto::AccountId32;
use std::cmp::min;
use std::collections::HashSet;
use std::str::FromStr;

const DEFROST_SCHEDULE_TYPE_INDEX: usize = 2;
const ACCOUNT_ADDRESS_INDEX: usize = 3;
const A_CML_AMOUNT_INDEX: usize = 5;
const B_CML_AMOUNT_INDEX: usize = 6;
const C_CML_AMOUNT_INDEX: usize = 7;

const COMPETITION_ADDRESS_INDEX: usize = 2;

impl Cli {
	pub fn genesis_seed(&self) -> [u8; 32] {
		if let Some(s) = self.genesis_seed.as_ref() {
			seed_from_string(s)
		} else {
			seed_from_string("tearust")
		}
	}
}

fn seed_from_string(s: &str) -> [u8; 32] {
	let mut seed = [0; 32];
	let str_bytes = s.as_bytes();
	let len = min(seed.len(), str_bytes.len());

	for i in 0..len {
		seed[i] = str_bytes[i];
	}
	seed
}

fn parse_defrost_schedule_type(value: Option<&str>) -> Result<DefrostScheduleType, String> {
	return match value
		.ok_or("can't find defrost schedule type field".to_string())?
		.to_uppercase()
		.as_str()
	{
		"INVESTOR" => Ok(DefrostScheduleType::Investor),
		"TEAM" => Ok(DefrostScheduleType::Team),
		_ => Err("failed to parse defrost schedule type".to_string()),
	};
}

fn parse_account_address(value: Option<&str>) -> Result<AccountId, String> {
	Ok(AccountId32::from_str(
		value.ok_or("can't find account address field".to_string())?,
	)?)
}

fn parse_coupon_amount(value: Option<&str>) -> u32 {
	value.unwrap_or_default().parse().unwrap_or_default()
}

#[cfg(test)]
mod tests {
	use super::parse_coupon_configs;
	use frame_benchmarking::frame_support::sp_runtime::AccountId32;

	#[test]
	fn tests() {
		let acc = AccountId32::new([1; 32]);
		println!("{}", acc);
	}

	#[test]
	fn parse_coupon_configs_works() -> Result<(), String> {
		let mut rdr = csv::Reader::from_reader(&include_bytes!("dev.csv")[..]);
		let _configs = parse_coupon_configs(&mut rdr)?;
		Ok(())
	}
}
