use super::param::*;
use crate::{CmlId, CmlType};
use codec::{Decode, Encode};
use node_primitives::BlockNumber;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

#[derive(Encode, Decode, Clone, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Seed {
	pub id: CmlId, //seq id starting from 0, this is also the camellia id.
	pub cml_type: CmlType,
	pub defrost_schedule: Option<DefrostScheduleType>,
	pub defrost_time: Option<BlockNumber>,
	pub lifespan: BlockNumber,
	pub performance: Performance,
}

impl Seed {
	pub fn generate(
		cml_type: CmlType,
		cml_id: CmlId,
		defrost_schedule: DefrostScheduleType,
		generate_defrost_time: impl Fn() -> BlockNumber,
		lifespan: BlockNumber,
		performance: Performance,
	) -> Self {
		let id = cml_id;
		let defrost_time = generate_defrost_time();
		Seed {
			id,
			cml_type,
			defrost_schedule: Some(defrost_schedule),
			defrost_time: Some(defrost_time),
			lifespan,
			performance,
		}
	}
}

#[derive(Encode, Decode, Clone, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct GenesisSeeds {
	pub a_seeds: Vec<Seed>,
	pub b_seeds: Vec<Seed>,
	pub c_seeds: Vec<Seed>,
}

#[derive(Encode, Decode, PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum DefrostScheduleType {
	Investor,
	Team,
}

impl Default for GenesisSeeds {
	fn default() -> Self {
		GenesisSeeds {
			a_seeds: vec![],
			b_seeds: vec![],
			c_seeds: vec![],
		}
	}
}

impl GenesisSeeds {
	pub fn generate(
		gen_defrost_time_for_team: impl Fn() -> BlockNumber,
		gen_defrost_time_for_investor: impl Fn() -> BlockNumber,
		gen_lifespan: impl Fn(CmlType) -> BlockNumber,
		gen_performance: impl Fn(CmlType) -> Performance,
	) -> Self {
		let mut a_seeds: Vec<Seed> = Vec::new();
		let mut seq_id: u64 = 0;
		for i in 0..GENESIS_SEED_A_COUNT {
			if i < GENESIS_SEED_A_COUNT * TEAM_PERCENTAGE / 100 {
				a_seeds.push(Seed::generate(
					CmlType::A,
					seq_id,
					DefrostScheduleType::Team,
					&gen_defrost_time_for_team,
					gen_lifespan(CmlType::A),
					gen_performance(CmlType::A),
				));
			} else {
				a_seeds.push(Seed::generate(
					CmlType::A,
					seq_id,
					DefrostScheduleType::Investor,
					&gen_defrost_time_for_investor,
					gen_lifespan(CmlType::A),
					gen_performance(CmlType::A),
				));
			}
			seq_id += 1;
		}
		let mut b_seeds: Vec<Seed> = Vec::new();
		for i in 0..GENESIS_SEED_B_COUNT {
			if i < GENESIS_SEED_B_COUNT * TEAM_PERCENTAGE / 100 {
				b_seeds.push(Seed::generate(
					CmlType::B,
					seq_id,
					DefrostScheduleType::Team,
					&gen_defrost_time_for_team,
					gen_lifespan(CmlType::B),
					gen_performance(CmlType::B),
				));
			} else {
				b_seeds.push(Seed::generate(
					CmlType::B,
					seq_id,
					DefrostScheduleType::Investor,
					&gen_defrost_time_for_investor,
					gen_lifespan(CmlType::B),
					gen_performance(CmlType::B),
				));
			}
			seq_id += 1;
		}
		let mut c_seeds: Vec<Seed> = Vec::new();

		for i in 0..GENESIS_SEED_C_COUNT {
			if i < GENESIS_SEED_C_COUNT * TEAM_PERCENTAGE / 100 {
				c_seeds.push(Seed::generate(
					CmlType::C,
					seq_id,
					DefrostScheduleType::Team,
					&gen_defrost_time_for_team,
					gen_lifespan(CmlType::C),
					gen_performance(CmlType::C),
				))
			} else {
				c_seeds.push(Seed::generate(
					CmlType::C,
					seq_id,
					DefrostScheduleType::Investor,
					&gen_defrost_time_for_investor,
					gen_lifespan(CmlType::C),
					gen_performance(CmlType::C),
				));
			}
			seq_id += 1;
		}
		GenesisSeeds {
			a_seeds,
			b_seeds,
			c_seeds,
		}
	}
}
