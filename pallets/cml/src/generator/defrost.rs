use crate::param::{
	BLOCKS_IN_A_DAY, BLOCKS_IN_A_MONTH, BLOCKS_IN_HALF_MONTH, UNFROZEN_SEEDS_PERCENTAGE_INVESTOR,
};
use crate::DefrostScheduleType;
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
		let r: u32 = rng.gen();
		let random_offset: u32 = r % (6 * BLOCKS_IN_A_DAY);
		// assert_eq!(BLOCKS_IN_A_DAY, 144);
		match defrost_schedule {
			DefrostScheduleType::Investor => {
				let prob = r % 100;
				if prob < UNFROZEN_SEEDS_PERCENTAGE_INVESTOR {
					//this seed fall into the the non-frozen 10%
					0
				} else {
					let r: u32 = rng.gen();
					let fall_in_month: u32 = r % 18;
					fall_in_month * BLOCKS_IN_A_MONTH + BLOCKS_IN_HALF_MONTH + random_offset
						- 3 * BLOCKS_IN_A_DAY // let rand_defrost_time_index = 18 as f32 // total eighteen months
				}
			}
			DefrostScheduleType::Team => {
				let fall_in_month: u32 = r % 20;
				BLOCKS_IN_A_MONTH * (2 + fall_in_month) + BLOCKS_IN_HALF_MONTH + random_offset
					- 3 * BLOCKS_IN_A_DAY
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn check_seeds_defrost_time_distribution() {
		println!("Team defrost ...{}", BLOCKS_IN_A_DAY);
		let closure_defrost_time = make_generate_defrost_time_fn(DefrostScheduleType::Team);

		let test_sample_count = 1000;
		let mut distribute = [0u16; 36 * 30];
		let mut month_distribute = [0u16; 24];
		for _i in 0..test_sample_count {
			let defrost_time = closure_defrost_time();
			let defrost_days = (defrost_time / BLOCKS_IN_A_DAY) as usize;
			// println!(
			// 	"Defrost time {}, defrost_day {}",
			// 	defrost_time, defrost_days
			// );
			distribute[defrost_days] += 1;
			let defrost_month = (defrost_time / BLOCKS_IN_A_MONTH) as usize;
			month_distribute[defrost_month] += 1;
		}

		for i in 0..distribute.len() {
			if distribute[i] > 0 {
				println!("day {} seeds {}", i, distribute[i]);
			}
		}
		for i in 0..month_distribute.len() {
			println!("month {} seeds {}", i, month_distribute[i]);
		}
		println!("Investor defrost ...");
		let mut month_distribute = [0u16; 24];
		let closure_defrost_time = make_generate_defrost_time_fn(DefrostScheduleType::Investor);

		let test_sample_count = 1000;
		let mut distribute = [0u16; 36 * 30];
		for _i in 0..test_sample_count {
			let defrost_time = closure_defrost_time();
			let defrost_days = (defrost_time / BLOCKS_IN_A_DAY) as usize;
			distribute[defrost_days] += 1;
			let defrost_month = (defrost_time / BLOCKS_IN_A_MONTH) as usize;
			month_distribute[defrost_month] += 1;
		}

		for i in 0..distribute.len() {
			if distribute[i] > 0 {
				println!("day {} seeds {}", i, distribute[i]);
			}
		}
		for i in 0..month_distribute.len() {
			println!("month {} seeds {}", i, month_distribute[i]);
		}
	}
}
