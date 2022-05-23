use super::cli::Cli;
use std::cmp::min;
use std::convert::TryInto;

impl Cli {
	pub fn genesis_seed(&self) -> [u8; 32] {
		if let Some(s) = self.genesis_seed.as_ref() {
			seed_from_string(s)
		} else {
			seed_from_string("tearust")
		}
	}

	pub fn parse_mining_startup(&self) -> Result<Vec<([u8; 32], u64, Vec<u8>, Vec<u8>)>, String> {
		match self.mining_startup_path.as_ref() {
			Some(path) => {
				let mut rdr = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
				parse_mining_startup_config(&mut rdr)
			}
			None => {
				let mut rdr = csv::Reader::from_reader(&include_bytes!("mining_startup.csv")[..]);
				parse_mining_startup_config(&mut rdr)
			}
		}
	}

	pub fn parse_tapp_startup(&self) -> Result<Vec<([u8; 32], u64, Vec<u8>)>, String> {
		match self.tapp_startup_path.as_ref() {
			Some(path) => {
				let mut rdr = csv::Reader::from_path(path).map_err(|e| e.to_string())?;
				parse_tapp_startup_config(&mut rdr)
			}
			None => {
				let mut rdr = csv::Reader::from_reader(&include_bytes!("tapp_startup.csv")[..]);
				parse_tapp_startup_config(&mut rdr)
			}
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

fn parse_mining_startup_config<R>(
	rdr: &mut csv::Reader<R>,
) -> Result<Vec<([u8; 32], u64, Vec<u8>, Vec<u8>)>, String>
where
	R: std::io::Read,
{
	const MACHINE_ID_INDEX: usize = 0;
	const CML_ID_INDEX: usize = 1;
	const CONN_ID_INDEX: usize = 2;
	const IP_ADDRESS_INDEX: usize = 3;

	let mut startup_list = Vec::new();
	for record in rdr.records() {
		let record = record.map_err(|e| e.to_string())?;
		let machine_id = parse_machine_id(record.get(MACHINE_ID_INDEX))?;
		let cml_id = parse_u64(record.get(CML_ID_INDEX), "cml id")?;
		let conn_id = parse_utf8_encoded(record.get(CONN_ID_INDEX), "conn id")?;
		let ip_address = parse_utf8_encoded(record.get(IP_ADDRESS_INDEX), "ip address")?;

		startup_list.push((machine_id, cml_id, conn_id, ip_address));
	}

	Ok(startup_list)
}

fn parse_tapp_startup_config<R>(
	rdr: &mut csv::Reader<R>,
) -> Result<Vec<([u8; 32], u64, Vec<u8>)>, String>
where
	R: std::io::Read,
{
	const MACHINE_ID_INDEX: usize = 0;
	const CML_ID_INDEX: usize = 1;
	const IP_ADDRESS_INDEX: usize = 2;

	let mut startup_list = Vec::new();
	for record in rdr.records() {
		let record = record.map_err(|e| e.to_string())?;
		let machine_id = parse_machine_id(record.get(MACHINE_ID_INDEX))?;
		let cml_id = parse_u64(record.get(CML_ID_INDEX), "cml id")?;
		let ip_address = parse_utf8_encoded(record.get(IP_ADDRESS_INDEX), "ip address")?;

		startup_list.push((machine_id, cml_id, ip_address));
	}

	Ok(startup_list)
}

fn parse_machine_id(value: Option<&str>) -> Result<[u8; 32], String> {
	hex::decode(value.ok_or("can't find machine id")?)
		.map_err(|e| format!("failed to hex decode machine id: {}", e))?
		.as_slice()
		.try_into()
		.map_err(|e| format!("failed to parse machine id: {}", e))
}

fn parse_utf8_encoded(value: Option<&str>, value_name: &str) -> Result<Vec<u8>, String> {
	Ok(value
		.ok_or(format!("can't find {}", value_name))?
		.as_bytes()
		.to_vec())
}

fn parse_u64(value: Option<&str>, value_name: &str) -> Result<u64, String> {
	value
		.ok_or(format!("can't find {}", value_name))?
		.parse()
		.map_err(|e| format!("failed to parse {}: {}", value_name, e))
}
