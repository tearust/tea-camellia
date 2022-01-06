use crate::generator::defrost::make_generate_defrost_time_fn;
use crate::generator::lifespan::make_generate_lifespan_fn;
use crate::generator::performance::make_generate_performance_fn;
use crate::{CmlType, DefrostScheduleType, GenesisSeeds};
use log::info;

mod defrost;
mod lifespan;
mod performance;

pub type WideSeed = [u8; 32];
pub type ShortSeed = [u8; 16];

/// Generate fixed number of seeds with random properties.
pub fn init_genesis(seed: WideSeed) -> GenesisSeeds {
	info!("init_genesis");
	GenesisSeeds::generate(
		make_generate_defrost_time_fn(seed),
		make_generate_lifespan_fn(seed),
		make_generate_performance_fn(seed),
	)
}

/// generate an individual seed for each random generation.
///
/// `seed` is the proto seed for generating individual seed
/// `class` defines seed specific application, such as generating: defrost time, lifespan, performance...
/// `sub_type` defines a sub-type about the application, such as: `DefrostScheduleType` in generate
/// 	defrost time, `CmlType` in generate lifetime
/// `seq_id` defines a sequence number to make a distinguish with each generation.
pub fn generate_individual_seed(seed: WideSeed, class: u8, sub_type: u8, seq_id: u64) -> ShortSeed {
	let mut reverse_index = seed.len() - 1;
	let mut result_seed = seed.clone();

	let seq_bytes = seq_id.to_le_bytes();
	for i in 0..seq_bytes.len() {
		result_seed[reverse_index] = seq_bytes[i];
		reverse_index -= 1;
	}

	result_seed[reverse_index] = sub_type;
	reverse_index -= 1;

	result_seed[reverse_index] = class;

	let mut result = [0u8; 16];
	let step = result.len();
	for i in 0..step {
		let sum = result_seed[i] as u16 + result_seed[i + step] as u16;
		result[i] = (sum / 2) as u8;
	}

	result
}

pub fn defrost_schedule_sub_type_value(defrost_schedule: DefrostScheduleType) -> u8 {
	match defrost_schedule {
		DefrostScheduleType::Investor => 1,
		DefrostScheduleType::Team => 2,
	}
}

pub fn cml_type_sub_type_value(cml_type: CmlType) -> u8 {
	match cml_type {
		CmlType::A => 1,
		CmlType::B => 2,
		CmlType::C => 3,
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn generate_individual_seed_works() {
		let mut raw_seed = [16u8; 32];

		let result = generate_individual_seed(raw_seed, 16, 16, u64::from_le_bytes([16u8; 8]));
		assert_eq!(result, [16u8; 16]);

		let result = generate_individual_seed(raw_seed, 0, 0, u64::from_le_bytes([0u8; 8]));
		assert_eq!(
			result,
			[16, 16, 16, 16, 16, 16, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8]
		);

		for i in 0..raw_seed.len() {
			raw_seed[i] = i as u8;
		}
		let result = generate_individual_seed(
			raw_seed,
			22,
			23,
			u64::from_le_bytes([31, 30, 29, 28, 27, 26, 25, 24]),
		);
		assert_eq!(
			result,
			[8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23]
		);
	}
}
