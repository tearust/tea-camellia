#![cfg(feature = "std")]

use crate::generator::defrost::make_generate_defrost_time_fn;
use crate::generator::lifespan::make_generate_lifespan_fn;
use crate::generator::performance::make_generate_performance_fn;
use crate::{DefrostScheduleType, GenesisSeeds};
use log::info;

mod defrost;
mod lifespan;
mod performance;

/// Generate fixed number of seeds with random properties.
pub fn init_genesis(seed: [u8; 32]) -> GenesisSeeds {
	info!("init_genesis");
	GenesisSeeds::generate(
		make_generate_defrost_time_fn(seed, DefrostScheduleType::Team),
		make_generate_defrost_time_fn(seed, DefrostScheduleType::Investor),
		make_generate_lifespan_fn(seed),
		make_generate_performance_fn(seed),
	)
}
