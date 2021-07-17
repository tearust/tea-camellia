//! Benchmarking setup for pallet-cml

use super::*;
use crate::param::{
	GENESIS_SEED_A_COUNT, GENESIS_SEED_B_COUNT, GENESIS_SEED_C_COUNT, TEAM_PERCENTAGE,
};
#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use sp_std::convert::TryInto;

const STAKING_SLOTS_MAX_LENGTH: StakingIndex = 1024;
const CENTS: u128 = 10_000_000_000u128;
const DOLLARS: u128 = 100u128 * CENTS;

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
		T::Currency::make_free_balance_be(&caller, u128_to_balance::<T>(10000 * DOLLARS));

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

	stop_balance_staking {
		let s in 0 .. STAKING_SLOTS_MAX_LENGTH;
		let caller: T::AccountId = whitelisted_caller();

		let cml_id: CmlId = 9999;
		prepare_staked_tree::<T>(cml_id, StakingCategory::Tea, &caller);

		T::Currency::make_free_balance_be(&caller, u128_to_balance::<T>(10000 * DOLLARS));
		T::CurrencyOperations::reserve(&caller, u128_to_balance::<T>(10000 * DOLLARS)).unwrap();
	}: stop_staking(RawOrigin::Signed(caller), cml_id, s)

	stop_cml_staking {
		let s in 0 .. STAKING_SLOTS_MAX_LENGTH;
		let caller: T::AccountId = whitelisted_caller();

		let cml_id: CmlId = 9999;
		prepare_staked_tree::<T>(cml_id, StakingCategory::Cml, &caller);

		T::Currency::make_free_balance_be(&caller, u128_to_balance::<T>(10000 * DOLLARS));
		T::CurrencyOperations::reserve(&caller, u128_to_balance::<T>(10000 * DOLLARS)).unwrap();
	}: stop_staking(RawOrigin::Signed(caller), cml_id, s)

	withdraw_staking_reward {
		let amount: BalanceOf<T> = 10000u32.into();
		let caller: T::AccountId = whitelisted_caller();

		AccountRewards::<T>::insert(&caller, amount);
	}: _(RawOrigin::Signed(caller))

	pay_off_mining_credit {
		let amount: BalanceOf<T> = 10000u32.into();
		let caller: T::AccountId = whitelisted_caller();

		T::Currency::make_free_balance_be(&caller, u128_to_balance::<T>(10000 * DOLLARS));
		GenesisMinerCreditStore::<T>::insert(&caller, amount);
	}: _(RawOrigin::Signed(caller.clone()))
	verify {
		assert!(!GenesisMinerCreditStore::<T>::contains_key(&caller));
	}

	dummy_ra_task {
		let caller: T::AccountId = whitelisted_caller();

		let cml_id: CmlId = 4;
		let machine_id: MachineId = [1u8; 32];
		start_mining_inner::<T>(cml_id, machine_id, &caller);
	}: _(RawOrigin::Signed(caller), machine_id)
}

impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test);

fn prepare_staked_tree<T: Config>(cml_id: CmlId, category: StakingCategory, caller: &T::AccountId) {
	let machine_id: MachineId = [1u8; 32];
	let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
	cml.start_mining(
		machine_id,
		new_staking_item::<T>(cml_id, StakingCategory::Tea, caller),
		&T::BlockNumber::zero(),
	);

	for i in 0..STAKING_SLOTS_MAX_LENGTH {
		let staking_cml_id = i as u64;
		cml.staking_slots_mut()
			.push(new_staking_item::<T>(staking_cml_id, category, caller));
		if category == StakingCategory::Cml {
			CmlStore::<T>::insert(
				staking_cml_id,
				new_staking_cml::<T>(cml_id, staking_cml_id, i, caller),
			);
			UserCmlStore::<T>::insert(caller, staking_cml_id, ());
		}
	}

	UserCmlStore::<T>::insert(caller, cml_id, ());
	CmlStore::<T>::insert(cml_id, cml);
	MinerItemStore::<T>::insert(machine_id, MinerItem::default());
}

fn new_staking_cml<T: Config>(
	cml_id: CmlId,
	staking_cml_id: CmlId,
	index: StakingIndex,
	owner: &T::AccountId,
) -> CML<T::AccountId, T::BlockNumber, BalanceOf<T>, T::SeedFreshDuration> {
	let mut cml = CML::from_genesis_seed(seed_from_lifespan(staking_cml_id, 100));
	cml.defrost(&0u32.into());
	cml.convert_to_tree(&0u32.into());
	cml.convert(CmlStatus::Staking(cml_id, index));
	cml.set_owner(owner);
	cml
}

fn new_staking_item<T: Config>(
	cml_id: CmlId,
	category: StakingCategory,
	caller: &T::AccountId,
) -> StakingItem<T::AccountId, BalanceOf<T>> {
	let mut staking_item = StakingItem {
		owner: caller.clone(),
		category,
		amount: None,
		cml: None,
	};
	match category {
		StakingCategory::Cml => staking_item.cml = Some(cml_id),
		StakingCategory::Tea => staking_item.amount = Some(1u32.into()),
	}
	staking_item
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
	match schedule_type {
		DefrostScheduleType::Investor => {
			InvestorCouponStore::<T>::insert(
				account,
				CmlType::A,
				new_coupon(max_a_count as u32, CmlType::A),
			);
			InvestorCouponStore::<T>::insert(
				account,
				CmlType::B,
				new_coupon(max_b_count as u32, CmlType::B),
			);
			InvestorCouponStore::<T>::insert(
				account,
				CmlType::C,
				new_coupon(max_c_count as u32, CmlType::C),
			);
		}
		DefrostScheduleType::Team => {
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
	}
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
	MinerItemStore::<T>::insert(
		machine_id,
		MinerItem {
			cml_id,
			id: machine_id,
			ip: vec![],
			status: MinerStatus::Active,
		},
	);
}

fn new_coupon(amount: u32, cml_type: CmlType) -> Coupon {
	Coupon { amount, cml_type }
}

pub fn seed_from_lifespan(id: CmlId, lifespan: u32) -> Seed {
	Seed {
		id,
		cml_type: CmlType::A,
		defrost_schedule: Some(DefrostScheduleType::Team),
		defrost_time: Some(0),
		lifespan,
		performance: 0,
	}
}

fn u128_to_balance<T: Config>(amount: u128) -> BalanceOf<T> {
	amount.try_into().map_err(|_| "").unwrap()
}
