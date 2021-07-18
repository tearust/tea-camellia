#![cfg(feature = "std")]

use crate::generator::defrost::make_generate_defrost_time_fn;
use crate::generator::lifespan::make_generate_lifespan_fn;
use crate::generator::performance::make_generate_performance_fn;
use crate::{CmlType, DefrostScheduleType, GenesisSeeds};
use log::info;

mod defrost;
mod lifespan;
mod performance;

/// Generate fixed number of seeds with random properties.
pub fn init_genesis(seed: [u8; 32]) -> GenesisSeeds {
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
pub fn generate_individual_seed(seed: [u8; 32], class: u8, sub_type: u8, seq_id: u64) -> [u8; 32] {
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

	result_seed
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
