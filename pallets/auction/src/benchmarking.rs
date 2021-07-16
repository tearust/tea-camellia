//! Benchmarking setup for pallet-auction

use super::*;
#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use pallet_cml::{CmlId, CmlOperation, CmlType, DefrostScheduleType, Seed, CML};

benchmarks! {
	put_to_store {
		let caller: T::AccountId = whitelisted_caller();
		T::Currency::make_free_balance_be(
			&caller,
			10000000u32.into(),
		);

		let cml_id: CmlId = 4;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		T::CmlOperation::add_cml(&caller, cml);
	}: _(RawOrigin::Signed(caller), cml_id, 100u32.into(), None, true)

	bid_for_auction {
		let caller: T::AccountId = whitelisted_caller();
		let auction_id = 22;
		T::Currency::make_free_balance_be(&caller, 10000000u32.into());

		let cml_id: CmlId = 4;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		T::CmlOperation::add_cml(&caller, cml);

		let starting_price: BalanceOf<T> = 100u32.into();
		let mut auction_item = AuctionItem::default();
		auction_item.id = auction_id;
		auction_item.cml_owner = T::AccountId::default();
		auction_item.cml_id = cml_id;
		auction_item.starting_price = starting_price;
		T::AuctionOperation::add_auction_to_storage(auction_item);
	}: _(RawOrigin::Signed(caller), auction_id, starting_price)
}

impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test);

fn seed_from_lifespan(id: CmlId, lifespan: u32) -> Seed {
	Seed {
		id,
		cml_type: CmlType::A,
		defrost_schedule: Some(DefrostScheduleType::Team),
		defrost_time: Some(0),
		lifespan,
		performance: 0,
	}
}
