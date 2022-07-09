use super::param::*;
use crate::{CmlId, CmlType};
use codec::{Decode, Encode, MaxEncodedLen};
use node_primitives::BlockNumber;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

pub type ClassFlag = u64;

#[derive(Encode, Decode, Clone, Debug, TypeInfo, MaxEncodedLen)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Seed {
	pub id: CmlId, //seq id starting from 0, this is also the camellia id.
	pub cml_type: CmlType,
	pub lifespan: BlockNumber,
	pub performance: Performance,
	pub class_flag: ClassFlag,
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
			..Default::default()
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
			class_flag: 0,
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
		gen_lifespan: impl Fn(CmlType, u64) -> BlockNumber,
		gen_performance: impl Fn(CmlType, u64) -> Performance,
	) -> Self {
		let a_seeds = Self::generate_batch_type_seeds(
			a_count,
			CmlType::A,
			&mut seq_id,
			&gen_lifespan,
			&gen_performance,
		);

		let b_seeds = Self::generate_batch_type_seeds(
			b_count,
			CmlType::B,
			&mut seq_id,
			&gen_lifespan,
			&gen_performance,
		);

		let c_seeds = Self::generate_batch_type_seeds(
			c_count,
			CmlType::C,
			&mut seq_id,
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
		gen_lifespan: &impl Fn(CmlType, u64) -> BlockNumber,
		gen_performance: &impl Fn(CmlType, u64) -> Performance,
	) -> Vec<Seed> {
		let mut seeds: Vec<Seed> = Vec::new();

		for _ in 0..count {
			seeds.push(Seed::generate(
				cml_type,
				*seq_id,
				gen_lifespan(cml_type, *seq_id),
				gen_performance(cml_type, *seq_id),
			));
			*seq_id += 1;
		}
		seeds
	}
}
