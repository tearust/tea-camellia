use crate::mock::*;
use log::info;

pub fn start_simulation(height: u64) {
	const USER_COUNT: u32 = 100;
	let mut builder = ExtBuilder::default();
	builder.set_account_number(USER_COUNT);

	builder.build().execute_with(|| {
		let mut current_height = frame_system::Pallet::<Test>::block_number();
		while current_height <= height {
			info!("start block {}", current_height);

			// do something here

			current_height += 1;
			frame_system::Pallet::<Test>::set_block_number(current_height);
		}

		// dump results, account balances etc.
		for i in 1..=USER_COUNT {
			info!(
				"user {} details: balance is {}",
				i,
				Balances::free_balance(i as u64)
			);
		}
	})
}
