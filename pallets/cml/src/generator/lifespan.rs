use super::WideSeed;
use crate::generator::{cml_type_sub_type_value, generate_individual_seed};
use crate::param::{BASE_LIFESPAN_A, BASE_LIFESPAN_B, BASE_LIFESPAN_C, DEVIATION};
use crate::CmlType;
use node_primitives::BlockNumber;
use rand::{rngs::SmallRng, Rng, SeedableRng};

const LIFESPAN_CLASS_VALUE: u8 = 2;

pub fn make_generate_lifespan_fn(seed: WideSeed) -> impl Fn(CmlType, u64) -> BlockNumber {
	move |cml_type: CmlType, seq_id: u64| {
		let mut rng: SmallRng = SmallRng::from_seed(generate_individual_seed(
			seed,
			LIFESPAN_CLASS_VALUE,
			cml_type_sub_type_value(cml_type),
			seq_id,
		));
		let r: u8 = rng.gen();
		let base_lifespan = {
			match cml_type {
				CmlType::A => BASE_LIFESPAN_A,
				CmlType::B => BASE_LIFESPAN_B,
				CmlType::C => BASE_LIFESPAN_C,
			}
		};
		let random_offset =
			(r as f64 / u8::MAX as f64 - 0.5) * DEVIATION as f64 / 100.0 * base_lifespan as f64;
		(base_lifespan as f64 + random_offset as f64) as BlockNumber
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::param::BLOCKS_IN_A_DAY;

	#[test]
	fn check_seeds_lifespan_distribution() {
		println!("List different types of seeds and their lifespan");
		let test_sample_count = 1000;
		let mut total_a: u64 = 0;
		let mut total_b: u64 = 0;
		let mut total_c: u64 = 0;
		for i in 0..test_sample_count {
			let closure = make_generate_lifespan_fn([1; 32]);
			total_a += closure(CmlType::A, i) as u64;
		}
		for i in 0..test_sample_count {
			let closure = make_generate_lifespan_fn([1; 32]);
			total_b += closure(CmlType::B, i) as u64;
		}
		for i in 0..test_sample_count {
			let closure = make_generate_lifespan_fn([1; 32]);
			total_c += closure(CmlType::C, i) as u64;
		}
		println!(
			"avg lifespan of seeds a is {} days",
			total_a / test_sample_count / BLOCKS_IN_A_DAY as u64
		);
		println!(
			"avg lifespan of seeds b is {} days",
			total_b / test_sample_count / BLOCKS_IN_A_DAY as u64
		);
		println!(
			"avg lifespan of seeds c is {} days",
			total_c / test_sample_count / BLOCKS_IN_A_DAY as u64
		);
		for i in 0..20 {
			let closure = make_generate_lifespan_fn([1; 32]);
			println!("lifespan seeds b is {}", closure(CmlType::B, i));
		}
	}
}
