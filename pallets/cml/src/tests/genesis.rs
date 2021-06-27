use crate::param::{GENESIS_SEED_A_COUNT, GENESIS_SEED_B_COUNT, GENESIS_SEED_C_COUNT};
use crate::{
	mock::*, types::*, CmlStore, InvestorVoucherStore, LastCmlId, LuckyDrawBox, TeamVoucherStore,
};

#[test]
fn genesis_build_related_logic_works() {
	let voucher_config1 = VoucherConfig {
		account: 1,
		cml_type: CmlType::A,
		schedule_type: DefrostScheduleType::Team,
		amount: 100,
	};
	let voucher_config2 = VoucherConfig {
		account: 2,
		cml_type: CmlType::B,
		schedule_type: DefrostScheduleType::Investor,
		amount: 200,
	};

	ExtBuilder::default()
		.init_seeds()
		.vouchers(vec![voucher_config1.clone(), voucher_config2.clone()])
		.build()
		.execute_with(|| {
			let voucher1 = TeamVoucherStore::<Test>::get(1, CmlType::A);
			assert!(voucher1.is_some());
			let voucher1 = voucher1.unwrap();
			assert_eq!(voucher1.amount, voucher_config1.amount);

			let voucher2 = InvestorVoucherStore::<Test>::get(2, CmlType::B);
			assert!(voucher2.is_some());
			let voucher2 = voucher2.unwrap();
			assert_eq!(voucher2.amount, voucher_config2.amount);

			assert_eq!(
				GENESIS_SEED_A_COUNT as usize,
				LuckyDrawBox::<Test>::get(CmlType::A, DefrostScheduleType::Team).len()
					+ LuckyDrawBox::<Test>::get(CmlType::A, DefrostScheduleType::Investor).len()
			);
			assert_eq!(
				GENESIS_SEED_B_COUNT as usize,
				LuckyDrawBox::<Test>::get(CmlType::B, DefrostScheduleType::Team).len()
					+ LuckyDrawBox::<Test>::get(CmlType::B, DefrostScheduleType::Investor).len()
			);
			assert_eq!(
				GENESIS_SEED_C_COUNT as usize,
				LuckyDrawBox::<Test>::get(CmlType::C, DefrostScheduleType::Team).len()
					+ LuckyDrawBox::<Test>::get(CmlType::C, DefrostScheduleType::Investor).len()
			);

			let mut live_seeds_count: usize = 0;
			for i in 0..(GENESIS_SEED_A_COUNT + GENESIS_SEED_B_COUNT + GENESIS_SEED_C_COUNT) {
				let cml = CmlStore::<Test>::get(i);
				assert!(cml.is_some());
				let cml = cml.unwrap();
				assert_eq!(cml.id(), i);

				if cml.seed_valid(&0).unwrap() {
					live_seeds_count += 1;
				}
			}
			println!("live seeds count: {}", live_seeds_count);

			assert_eq!(
				LastCmlId::<Test>::get(),
				GENESIS_SEED_A_COUNT + GENESIS_SEED_B_COUNT + GENESIS_SEED_C_COUNT
			);
		});
}
