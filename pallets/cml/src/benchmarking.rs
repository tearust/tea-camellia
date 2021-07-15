//! Benchmarking setup for pallet-tea

use super::*;
use crate::param::{
	GENESIS_SEED_A_COUNT, GENESIS_SEED_B_COUNT, GENESIS_SEED_C_COUNT, TEAM_PERCENTAGE,
};
#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;

benchmarks! {
	transfer_coupon {
		let caller: T::AccountId = whitelisted_caller();
		InvestorCouponStore::<T>::insert(
			caller.clone(),
			CmlType::A,
			Coupon {
				amount: 10,
				cml_type: CmlType::A
			}
		);
	}: _(RawOrigin::Signed(caller), T::AccountId::default(), CmlType::A, DefrostScheduleType::Investor, 4)

	draw_investor_cmls_from_coupon {
		let caller: T::AccountId = whitelisted_caller();
		init_lucky_draw_box::<T>(DefrostScheduleType::Investor);
		init_coupon_store::<T>(&caller, DefrostScheduleType::Investor);
	}: draw_cmls_from_coupon(RawOrigin::Signed(caller.clone()), DefrostScheduleType::Investor)
	verify {
		assert_eq!(
			UserCmlStore::<T>::iter()
				.filter(|(k1, _, _)| *k1 == caller)
				.count() as u64,
			get_count_by_schedule_type(GENESIS_SEED_A_COUNT, DefrostScheduleType::Investor) +
			get_count_by_schedule_type(GENESIS_SEED_B_COUNT, DefrostScheduleType::Investor) +
			get_count_by_schedule_type(GENESIS_SEED_C_COUNT, DefrostScheduleType::Investor)
		);
	}

	draw_team_cmls_from_coupon {
		let caller: T::AccountId = whitelisted_caller();
		init_lucky_draw_box::<T>(DefrostScheduleType::Team);
		init_coupon_store::<T>(&caller, DefrostScheduleType::Team);
	}: draw_cmls_from_coupon(RawOrigin::Signed(caller.clone()), DefrostScheduleType::Team)
	verify {
		assert_eq!(
			UserCmlStore::<T>::iter()
				.filter(|(k1, _, _)| *k1 == caller)
				.count() as u64,
			get_count_by_schedule_type(GENESIS_SEED_A_COUNT, DefrostScheduleType::Team) +
			get_count_by_schedule_type(GENESIS_SEED_B_COUNT, DefrostScheduleType::Team) +
			get_count_by_schedule_type(GENESIS_SEED_C_COUNT, DefrostScheduleType::Team)
		);
	}

	active_cml {
		let caller: T::AccountId = whitelisted_caller();
		let cml_id: CmlId = 4;
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		cml.defrost(&T::BlockNumber::zero());
		UserCmlStore::<T>::insert(&caller, cml_id, ());
		CmlStore::<T>::insert(cml_id, cml);
	}: _(RawOrigin::Signed(caller), cml_id)

	start_mining {
		let caller: T::AccountId = whitelisted_caller();
		let cml_id: CmlId = 4;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		UserCmlStore::<T>::insert(&caller, cml_id, ());
		CmlStore::<T>::insert(cml_id, cml);
	}: _(RawOrigin::Signed(caller), cml_id, [1u8; 32], b"miner_ip".to_vec())

	stop_mining {
		let caller: T::AccountId = whitelisted_caller();
		let cml_id: CmlId = 4;
		let machine_id: MachineId = [1u8; 32];
		start_mining_inner::<T>(cml_id, machine_id, &caller);
	}: _(RawOrigin::Signed(caller), cml_id, machine_id)

	start_balance_staking {
		let caller: T::AccountId = whitelisted_caller();
		T::Currency::make_free_balance_be(&caller, 1000000u32.into());

		let cml_id: CmlId = 4;
		let machine_id: MachineId = [1u8; 32];
		start_mining_inner::<T>(cml_id, machine_id, &caller);
	}: start_staking(RawOrigin::Signed(caller), cml_id, None, Some(10))

	start_cml_staking {
		let caller: T::AccountId = whitelisted_caller();

		let cml_id: CmlId = 4;
		let machine_id: MachineId = [1u8; 32];
		start_mining_inner::<T>(cml_id, machine_id, &caller);

		let cml2_id: CmlId = 5;
		let cml2 = CML::from_genesis_seed(seed_from_lifespan(cml2_id, 100));
		UserCmlStore::<T>::insert(&caller, cml2_id, ());
		CmlStore::<T>::insert(cml2_id, cml2);
	}: start_staking(RawOrigin::Signed(caller), cml_id, Some(cml2_id), Some(10))
}

fn init_lucky_draw_box<T: Config>(schedule_type: DefrostScheduleType) {
	let a_count = get_count_by_schedule_type(GENESIS_SEED_A_COUNT, schedule_type);
	let b_count = get_count_by_schedule_type(GENESIS_SEED_B_COUNT, schedule_type);
	let c_count = get_count_by_schedule_type(GENESIS_SEED_C_COUNT, schedule_type);

	let origin_a_box: Vec<u64> = (0..a_count).collect();
	let origin_b_box: Vec<u64> = (a_count..a_count + b_count).collect();
	let origin_c_box: Vec<u64> = (a_count + b_count..a_count + b_count + c_count).collect();
	LuckyDrawBox::<T>::insert(CmlType::A, schedule_type, origin_a_box.clone());
	LuckyDrawBox::<T>::insert(CmlType::B, schedule_type, origin_b_box.clone());
	LuckyDrawBox::<T>::insert(CmlType::C, schedule_type, origin_c_box.clone());
}

fn init_coupon_store<T: Config>(account: &T::AccountId, schedule_type: DefrostScheduleType) {
	let max_a_count = get_count_by_schedule_type(GENESIS_SEED_A_COUNT, schedule_type);
	let max_b_count = get_count_by_schedule_type(GENESIS_SEED_B_COUNT, schedule_type);
	let max_c_count = get_count_by_schedule_type(GENESIS_SEED_C_COUNT, schedule_type);
	TeamCouponStore::<T>::insert(
		account,
		CmlType::A,
		new_coupon(max_a_count as u32, CmlType::A),
	);
	TeamCouponStore::<T>::insert(
		account,
		CmlType::B,
		new_coupon(max_b_count as u32, CmlType::B),
	);
	TeamCouponStore::<T>::insert(
		account,
		CmlType::C,
		new_coupon(max_c_count as u32, CmlType::C),
	);
}

fn get_count_by_schedule_type(count: u64, schedule_type: DefrostScheduleType) -> u64 {
	return match schedule_type {
		DefrostScheduleType::Team => count * TEAM_PERCENTAGE / 100,
		DefrostScheduleType::Investor => count * (100 - TEAM_PERCENTAGE) / 100,
	};
}

fn start_mining_inner<T: Config>(cml_id: CmlId, machine_id: MachineId, caller: &T::AccountId) {
	let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
	cml.start_mining(machine_id, StakingItem::default(), &T::BlockNumber::zero());
	UserCmlStore::<T>::insert(caller, cml_id, ());
	CmlStore::<T>::insert(cml_id, cml);
	MinerItemStore::<T>::insert(machine_id, MinerItem::default());
}

fn new_coupon(amount: u32, cml_type: CmlType) -> Coupon {
	Coupon { amount, cml_type }
}

pub fn seed_from_lifespan(id: CmlId, lifespan: u32) -> Seed {
	let mut seed = Seed::default();
	seed.id = id;
	seed.lifespan = lifespan;
	seed
}
