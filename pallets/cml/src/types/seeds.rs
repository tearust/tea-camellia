use super::param::*;
use crate::{CmlId, CmlType};
use codec::{Decode, Encode};
use node_primitives::BlockNumber;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

#[derive(Encode, Decode, Clone, Debug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Seed {
	pub id: CmlId, //seq id starting from 0, this is also the camellia id.
	pub cml_type: CmlType,
	pub lifespan: BlockNumber,
	pub performance: Performance,
}

impl Seed {
	pub fn generate(
		cml_type: CmlType,
		cml_id: CmlId,
		lifespan: BlockNumber,
		performance: Performance,
	) -> Self {
		let id = cml_id;
		Seed {
			id,
			cml_type,
			lifespan,
			performance,
		}
	}
}

impl Default for Seed {
	fn default() -> Self {
		Seed {
			id: 0,
			cml_type: CmlType::C,
			lifespan: 0,
			performance: 0,
		}
	}
}

#[derive(Encode, Decode, Clone, Debug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct GenesisSeeds {
	pub a_seeds: Vec<Seed>,
	pub b_seeds: Vec<Seed>,
	pub c_seeds: Vec<Seed>,
}

#[derive(Encode, Decode, PartialEq, Clone, Copy, Debug, TypeInfo)]
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
		mut seq_id: u64,
		a_count: u64,
		b_count: u64,
		c_count: u64,
		team_percentage: u64,
		gen_defrost_time: impl Fn(DefrostScheduleType, u64) -> BlockNumber,
		gen_lifespan: impl Fn(CmlType, u64) -> BlockNumber,
		gen_performance: impl Fn(CmlType, u64) -> Performance,
	) -> Self {
		let a_seeds = Self::generate_batch_type_seeds(
			a_count,
			CmlType::A,
			&mut seq_id,
			team_percentage,
			&gen_defrost_time,
			&gen_lifespan,
			&gen_performance,
		);

		let b_seeds = Self::generate_batch_type_seeds(
			b_count,
			CmlType::B,
			&mut seq_id,
			team_percentage,
			&gen_defrost_time,
			&gen_lifespan,
			&gen_performance,
		);

		let c_seeds = Self::generate_batch_type_seeds(
			c_count,
			CmlType::C,
			&mut seq_id,
			team_percentage,
			&gen_defrost_time,
			&gen_lifespan,
			&gen_performance,
		);

		GenesisSeeds {
			a_seeds,
			b_seeds,
			c_seeds,
		}
	}

	fn generate_batch_type_seeds(
		count: u64,
		cml_type: CmlType,
		seq_id: &mut u64,
		team_percentage: u64,
		gen_defrost_time: &impl Fn(DefrostScheduleType, u64) -> BlockNumber,
		gen_lifespan: &impl Fn(CmlType, u64) -> BlockNumber,
		gen_performance: &impl Fn(CmlType, u64) -> Performance,
	) -> Vec<Seed> {
		let mut seeds: Vec<Seed> = Vec::new();

		for i in 0..count {
			if i < count * team_percentage / 100 {
				seeds.push(Seed::generate(
					cml_type,
					*seq_id,
					gen_lifespan(cml_type, *seq_id),
					gen_performance(cml_type, *seq_id),
				))
			} else {
				seeds.push(Seed::generate(
					cml_type,
					*seq_id,
					gen_lifespan(cml_type, *seq_id),
					gen_performance(cml_type, *seq_id),
				));
			}
			*seq_id += 1;
		}
		seeds
	}
}
