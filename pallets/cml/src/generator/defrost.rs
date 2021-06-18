use crate::param::{BLOCKS_IN_A_DAY, BLOCKS_IN_A_MONTH};
use crate::seeds::DefrostScheduleType;
use node_primitives::BlockNumber;
use rand::{thread_rng, Rng};

///create a closure. this closure is used to generate the random defrost time for different kinds of DefrostSchedule (team or investor).
pub fn make_generate_defrost_time_fn(
	defrost_schedule: DefrostScheduleType,
) -> impl Fn() -> BlockNumber {
	move || {
		let mut defrost_time_point = Vec::new();
		for i in 1..36 {
			defrost_time_point.push(
				i * BLOCKS_IN_A_MONTH - BLOCKS_IN_A_MONTH / 2, //every mid_of_a_month is a defrost time point
			)
		}
		let mut rng = thread_rng();
		let r: u8 = rng.gen();
		let random_offset = (r as f64 / u8::MAX as f64 - 0.5) * 6 as f64 * BLOCKS_IN_A_DAY as f64;
		let team_cliff = 2 * BLOCKS_IN_A_MONTH;

		match defrost_schedule {
			DefrostScheduleType::Investor => {
				let prob: u8 = rng.gen();
				if prob < (u8::MAX as f32 * 0.1) as u8 {
					//this seed fall into the the non-frozen 10%
					0
				} else {
					let rand_defrost_time_index = 18 as f32 // total eighteen months
						* (prob as f32 - (u8::MAX as f32 * 0.1)) // the rest probably of nighty percent
						/ (u8::MAX as f32 - u8::MAX as f32 * 0.1); // the base of the nighty percent

					(defrost_time_point[rand_defrost_time_index as usize] as i32
						+ random_offset as i32) as u32
				}
			}
			DefrostScheduleType::Team => {
				let prob: u8 = rng.gen();
				let rand_defrost_time_index = 20 /*months*/ as f32 * prob as f32 / u8::MAX as f32/*probably between zero to one*/;
				team_cliff
					+ (defrost_time_point[rand_defrost_time_index as usize] as i32
						+ random_offset as i32) as u32
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	#[ignore]
	fn check_seeds_defrost_time_distribution() {
		println!("Team defrost ...");
		let closure_defrost_time = make_generate_defrost_time_fn(DefrostScheduleType::Team);

		let test_sample_count = 1000;
		let mut distribute = [0u16; 36 * 30];
		for _i in 0..test_sample_count {
			let defrost_time = closure_defrost_time();
			let defrost_days = (defrost_time / BLOCKS_IN_A_DAY) as usize;
			distribute[defrost_days] += 1;
		}

		for i in 0..distribute.len() {
			if distribute[i] > 0 {
				println!("day {} seeds {}", i, distribute[i]);
			}
		}
		println!("Investor defrost ...");
		let closure_defrost_time = make_generate_defrost_time_fn(DefrostScheduleType::Investor);

		let test_sample_count = 1000;
		let mut distribute = [0u16; 36 * 30];
		for _i in 0..test_sample_count {
			let defrost_time = closure_defrost_time();
			let defrost_days = (defrost_time / BLOCKS_IN_A_DAY) as usize;
			distribute[defrost_days] += 1;
		}

		for i in 0..distribute.len() {
			if distribute[i] > 0 {
				println!("day {} seeds {}", i, distribute[i]);
			}
		}
	}
}
