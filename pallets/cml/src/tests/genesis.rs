use crate::param::{GENESIS_SEED_A_COUNT, GENESIS_SEED_B_COUNT, GENESIS_SEED_C_COUNT};
use crate::{
	mock::*, types::*, CmlStore, InvestorCouponStore, LastCmlId, LuckyDrawBox, TeamCouponStore,
};

#[test]
fn genesis_build_related_logic_works() {
	let coupon_config1 = CouponConfig {
		account: 1,
		cml_type: CmlType::A,
		schedule_type: DefrostScheduleType::Team,
		amount: 100,
	};
	let coupon_config2 = CouponConfig {
		account: 2,
		cml_type: CmlType::B,
		schedule_type: DefrostScheduleType::Investor,
		amount: 200,
	};

	ExtBuilder::default()
		.init_seeds()
		.coupons(vec![coupon_config1.clone(), coupon_config2.clone()])
		.build()
		.execute_with(|| {
			let coupon1 = TeamCouponStore::<Test>::get(1, CmlType::A);
			assert!(coupon1.is_some());
			let coupon1 = coupon1.unwrap();
			assert_eq!(coupon1.amount, coupon_config1.amount);

			let coupon2 = InvestorCouponStore::<Test>::get(2, CmlType::B);
			assert!(coupon2.is_some());
			let coupon2 = coupon2.unwrap();
			assert_eq!(coupon2.amount, coupon_config2.amount);

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
				assert!(CmlStore::<Test>::contains_key(i));
				let cml = CmlStore::<Test>::get(i);
				assert_eq!(cml.id(), i);

				if cml.check_seed_validity(&0).is_ok() {
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
