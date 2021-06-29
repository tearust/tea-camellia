use crate::{CmlId, CmlType, DefrostScheduleType, Seed};

mod cml;
mod draw;
mod genesis;
mod mining;
mod staking;
mod sudo;
mod voucher;

pub fn new_genesis_seed(id: CmlId) -> Seed {
	Seed {
		id,
		cml_type: CmlType::A,
		defrost_schedule: Some(DefrostScheduleType::Team),
		defrost_time: Some(0),
		lifespan: 0,
		performance: 0,
	}
}

pub fn new_genesis_frozen_seed(id: CmlId) -> Seed {
	Seed {
		id,
		cml_type: CmlType::A,
		defrost_schedule: Some(DefrostScheduleType::Team),
		defrost_time: Some(10000),
		lifespan: 0,
		performance: 0,
	}
}
