//! Benchmarking setup for pallet-auction

use super::*;
#[allow(unused)]
use crate::Pallet as Template;
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite, whitelisted_caller};
use frame_system::RawOrigin;
use pallet_cml::{CmlId, CmlOperation, CmlType, DefrostScheduleType, Seed, CML};

const MAX_USERS_PER_AUCTION: u64 = 10000;
const AVERAGE_END_BLOCK_AUCTION_COUNT: u64 = 100;

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

	bid_for_auction_normal {
		let caller: T::AccountId = whitelisted_caller();
		let auction_id = 22;
		T::Currency::make_free_balance_be(&caller, 10000000u32.into());

		let starting_price: BalanceOf<T> = 100u32.into();
		init_auction::<T>(4, &T::AccountId::default(), auction_id, starting_price, None);
	}: bid_for_auction(RawOrigin::Signed(caller), auction_id, starting_price)

	bid_for_auction_with_buy_now_price {
		let caller: T::AccountId = whitelisted_caller();
		let auction_id = 22;
		T::Currency::make_free_balance_be(&caller, 10000000u32.into());

		let buy_now_price: BalanceOf<T> = 1000u32.into();
		init_auction::<T>(4, &T::AccountId::default(), auction_id, 100u32.into(), Some(buy_now_price));
	}: bid_for_auction(RawOrigin::Signed(caller), auction_id, buy_now_price)

	remove_bid_for_auction {
		let caller: T::AccountId = whitelisted_caller();
		let auction_id = 22;
		let total_price: BalanceOf<T> = 10000000u32.into();
		let starting_price: BalanceOf<T> = 100u32.into();
		T::Currency::make_free_balance_be(&caller, total_price);
		T::Currency::reserve(&caller, starting_price).unwrap();

		init_auction::<T>(4, &caller, auction_id, starting_price, None);

		T::AuctionOperation::create_new_bid(&caller, &auction_id, starting_price);
		AuctionBidStore::<T>::mutate(&auction_id, |list| {
			if let Some(list) = list {
				for i in 0 .. MAX_USERS_PER_AUCTION-1 {
					list.insert(0, T::AccountId::default());
				}
			}
		});
	}: _(RawOrigin::Signed(caller.clone()), auction_id)
	verify {
		assert_eq!(BidStore::<T>::iter().count(), 0);
		assert_eq!(AuctionBidStore::<T>::get(&auction_id).unwrap().len() as u64, MAX_USERS_PER_AUCTION - 1);
		// todo should pass
		// assert_eq!(T::Currency::reserved_balance(&caller), 0u32.into());
		// assert_eq!(T::Currency::free_balance(&caller), total_price);
	}

	remove_from_store_with_no_bid {
		let caller: T::AccountId = whitelisted_caller();
		let auction_id = 22;
		T::Currency::make_free_balance_be(&caller, 10000000u32.into());

		let buy_now_price: BalanceOf<T> = 1000u32.into();
		init_auction::<T>(4, &caller, auction_id, 100u32.into(), Some(buy_now_price));
	}: remove_from_store(RawOrigin::Signed(caller), auction_id)
	verify {
		assert_eq!(BidStore::<T>::iter().count(), 0);
	}

	remove_from_store_with_bids {
		let caller: T::AccountId = whitelisted_caller();
		let auction_id = AVERAGE_END_BLOCK_AUCTION_COUNT + 1;
		T::Currency::make_free_balance_be(&caller, 10000000u32.into());
		T::Currency::reserve(&caller, 10000000u32.into()).unwrap();

		let buy_now_price: BalanceOf<T> = 1000u32.into();
		init_auction::<T>(4, &caller, auction_id, 100u32.into(), Some(buy_now_price));

		AuctionBidStore::<T>::mutate(&auction_id, |list| {
			if let Some(list) = list {
				for i in 0 .. MAX_USERS_PER_AUCTION {
					list.insert(0, T::AccountId::default());
				}
			}
		});

		let (current, next) = T::AuctionOperation::get_window_block();
		let mut auction_list = vec![auction_id];
		for i in 0..AVERAGE_END_BLOCK_AUCTION_COUNT {
			auction_list.push(i);
		}
		EndBlockAuctionStore::<T>::insert(current, auction_list.clone());
		EndBlockAuctionStore::<T>::insert(next, auction_list);
	}: remove_from_store(RawOrigin::Signed(caller.clone()), auction_id)
	verify {
		assert_eq!(EndBlockAuctionStore::<T>::get(current).unwrap().len(), AVERAGE_END_BLOCK_AUCTION_COUNT as usize);
		assert_eq!(EndBlockAuctionStore::<T>::get(next).unwrap().len(), AVERAGE_END_BLOCK_AUCTION_COUNT as usize);
		assert_eq!(AuctionBidStore::<T>::get(&auction_id), None);
		assert_eq!(T::Currency::reserved_balance(&caller), 9999900u32.into());
	}
}

impl_benchmark_test_suite!(Template, crate::mock::new_test_ext(), crate::mock::Test);

fn init_auction<T: Config>(
	cml_id: CmlId,
	owner: &T::AccountId,
	auction_id: AuctionId,
	starting_price: BalanceOf<T>,
	buy_now_price: Option<BalanceOf<T>>,
) {
	let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
	T::CmlOperation::add_cml(&T::AccountId::default(), cml);

	let mut auction_item = AuctionItem::default();
	auction_item.id = auction_id;
	auction_item.cml_owner = owner.clone();
	auction_item.cml_id = cml_id;
	auction_item.starting_price = starting_price;
	auction_item.buy_now_price = buy_now_price;
	T::AuctionOperation::add_auction_to_storage(auction_item);
}

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
