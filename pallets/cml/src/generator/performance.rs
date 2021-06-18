use crate::param::{
	Performance, BASE_PERFORMANCE_A, BASE_PERFORMANCE_B, BASE_PERFORMANCE_C, PERFORMANCE_DEVIATION,
};
use crate::CmlType;
use rand::{thread_rng, Rng};

pub fn make_generate_performance_fn() -> impl Fn(CmlType) -> Performance {
	move |cml_type: CmlType| {
		let mut rng = thread_rng();
		let r: u8 = rng.gen();
		let base_performance = {
			match cml_type {
				CmlType::A => BASE_PERFORMANCE_A,
				CmlType::B => BASE_PERFORMANCE_B,
				CmlType::C => BASE_PERFORMANCE_C,
			}
		};
		let random_offset = (r as f64 / u8::MAX as f64 - 0.5) * PERFORMANCE_DEVIATION as f64
			/ 100.0 * base_performance as f64;
		(base_performance as f64 + random_offset as f64) as Performance
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn check_seeds_performance_distribution() {
		println!("List differet types of seeds and their perfornmance");
		let test_sample_count = 1000;

		let mut total_a: u64 = 0;
		let mut total_b: u64 = 0;
		let mut total_c: u64 = 0;
		for _ in 0..test_sample_count {
			let closure = make_generate_performance_fn();
			total_a += closure(CmlType::A) as u64;
		}
		for _ in 0..test_sample_count {
			let closure = make_generate_performance_fn();
			total_b += closure(CmlType::B) as u64;
		}
		for _ in 0..test_sample_count {
			let closure = make_generate_performance_fn();
			total_c += closure(CmlType::C) as u64;
		}
		println!(
			"avg performance of seeds a is {} points",
			total_a / test_sample_count
		);
		println!(
			"avg performance of seeds b is {} points",
			total_b / test_sample_count
		);
		println!(
			"avg performance of seeds c is {} points",
			total_c / test_sample_count
		);
		for _ in 0..20 {
			let closure = make_generate_performance_fn();
			println!("performance seeds b is {}", closure(CmlType::B));
		}
	}
}
