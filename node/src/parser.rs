use super::cli::Cli;
use camellia_runtime::{
	pallet_cml::{CmlType, DefrostScheduleType, GenesisVouchers, VoucherConfig},
	AccountId,
};
use sp_core::crypto::AccountId32;
use std::str::FromStr;

const DEFROST_SCHEDULE_TYPE_INDEX: usize = 2;
const ACCOUNT_ADDRESS_INDEX: usize = 3;
const A_CML_AMOUNT_INDEX: usize = 4;
const B_CML_AMOUNT_INDEX: usize = 5;
const C_CML_AMOUNT_INDEX: usize = 6;

impl Cli {
	pub fn parse_genesis_vouchers(&self) -> Result<GenesisVouchers<AccountId>, String> {
		let mut vouchers = Vec::new();
		if let Some(path) = self.genesis_vouchers_path.as_ref() {
			vouchers = parse_voucher_configs(path)?;
		}

		Ok(GenesisVouchers { vouchers })
	}
}

fn parse_voucher_configs(path: &str) -> Result<Vec<VoucherConfig<AccountId>>, String> {
	let mut vouchers = Vec::new();

	let mut rdr = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
	for record in rdr.records() {
		let record = record.map_err(|e| e.to_string())?;
		let schedule_type = parse_defrost_schedule_type(record.get(DEFROST_SCHEDULE_TYPE_INDEX))?;
		let account = parse_account_address(record.get(ACCOUNT_ADDRESS_INDEX))?;

		let a_amount = parse_voucher_amount(record.get(A_CML_AMOUNT_INDEX));
		if a_amount > 0 {
			vouchers.push(VoucherConfig {
				account: account.clone(),
				schedule_type,
				cml_type: CmlType::A,
				amount: a_amount,
			});
		}

		let b_amount = parse_voucher_amount(record.get(B_CML_AMOUNT_INDEX));
		if b_amount > 0 {
			vouchers.push(VoucherConfig {
				account: account.clone(),
				schedule_type,
				cml_type: CmlType::B,
				amount: b_amount,
			});
		}

		let c_amount = parse_voucher_amount(record.get(C_CML_AMOUNT_INDEX));
		if c_amount > 0 {
			vouchers.push(VoucherConfig {
				account: account.clone(),
				schedule_type,
				cml_type: CmlType::C,
				amount: c_amount,
			});
		}
	}

	Ok(vouchers)
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

fn parse_voucher_amount(value: Option<&str>) -> u32 {
	value.unwrap_or_default().parse().unwrap_or_default()
}

#[cfg(test)]
mod tests {
	use super::parse_voucher_configs;
	use camellia_runtime::pallet_cml::{CmlType, DefrostScheduleType};
	use sp_core::crypto::AccountId32;
	use std::str::FromStr;

	#[test]
	fn parse_voucher_configs_works() -> Result<(), String> {
		let account = AccountId32::from_str("5Eo1WB2ieinHgcneq6yUgeJHromqWTzfjKnnhbn43Guq4gVP")
			.map_err(|e| e.to_string())?;
		let configs = parse_voucher_configs("data/dev.csv")?;
		assert_eq!(configs.len(), 6);
		for i in 0..3 {
			assert_eq!(configs[i].schedule_type, DefrostScheduleType::Investor);
			assert_eq!(configs[i].account, account);
		}
		for i in 3..6 {
			assert_eq!(configs[i].schedule_type, DefrostScheduleType::Team);
			assert_eq!(configs[i].account, account);
		}

		assert_eq!(configs[0].cml_type, CmlType::A);
		assert_eq!(configs[0].amount, 4);

		assert_eq!(configs[1].cml_type, CmlType::B);
		assert_eq!(configs[1].amount, 12);

		assert_eq!(configs[2].cml_type, CmlType::C);
		assert_eq!(configs[2].amount, 24);

		assert_eq!(configs[3].cml_type, CmlType::A);
		assert_eq!(configs[3].amount, 6);

		assert_eq!(configs[4].cml_type, CmlType::B);
		assert_eq!(configs[4].amount, 18);

		assert_eq!(configs[5].cml_type, CmlType::C);
		assert_eq!(configs[5].amount, 36);

		Ok(())
	}
}
