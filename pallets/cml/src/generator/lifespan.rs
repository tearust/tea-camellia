use crate::param::{BASE_LIFESPAN_A, BASE_LIFESPAN_B, BASE_LIFESPAN_C, DEVIATION};
use crate::CmlType;
use node_primitives::BlockNumber;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

pub fn make_generate_lifespan_fn(seed: [u8; 32]) -> impl Fn(CmlType) -> BlockNumber {
	move |cml_type: CmlType| {
		let mut rng: StdRng = SeedableRng::from_seed(seed);
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
		for _ in 0..test_sample_count {
			let closure = make_generate_lifespan_fn();
			total_a += closure(CmlType::A) as u64;
		}
		for _ in 0..test_sample_count {
			let closure = make_generate_lifespan_fn();
			total_b += closure(CmlType::B) as u64;
		}
		for _ in 0..test_sample_count {
			let closure = make_generate_lifespan_fn();
			total_c += closure(CmlType::C) as u64;
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
		for _ in 0..20 {
			let closure = make_generate_lifespan_fn();
			println!("lifespan seeds b is {}", closure(CmlType::B));
		}
	}
}
