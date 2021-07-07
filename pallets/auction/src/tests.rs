use crate::{
	mock::*, AuctionBidStore, AuctionItem, AuctionStore, BidStore, EndBlockAuctionStore, Error,
	LastAuctionId, UserAuctionStore, UserBidStore,
};
use frame_support::{assert_noop, assert_ok, traits::Currency};
use pallet_cml::{
	CmlId, CmlStore, CmlType, Config, DefrostScheduleType, Error as CmlError, Seed, SeedProperties,
	UserCmlStore, CML,
};
use pallet_utils::CurrencyOperations;

#[test]
fn put_to_store_works() {
	new_test_ext().execute_with(|| {
		<Test as Config>::Currency::make_free_balance_be(&1, AUCTION_PLEDGE_AMOUNT);
		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		cml.defrost(&0);
		cml.convert_to_tree(&0);
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Auction::put_to_store(Origin::signed(1), cml_id, 1000, None));

		let auction_id = 1; // this is the first auction so ID is 1
		let store_list = UserAuctionStore::<Test>::get(1);
		assert_eq!(store_list.len(), 1);
		assert_eq!(store_list.get(0).unwrap(), &auction_id);

		let (_, next_window) = Auction::get_window_block();
		let auction_list = EndBlockAuctionStore::<Test>::get(next_window).unwrap();
		assert_eq!(auction_list.len(), 1);
		assert_eq!(auction_list.get(0).unwrap(), &auction_id);

		let auction = AuctionStore::<Test>::get(auction_id);
		assert_eq!(auction.cml_owner, 1);
		assert_eq!(auction.cml_id, cml_id);
	})
}

#[test]
fn put_not_exist_cml_to_store_should_fail() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Auction::put_to_store(Origin::signed(1), 11, 1000, None),
			CmlError::<Test>::NotFoundCML
		);
	})
}

#[test]
fn put_not_my_cml_to_store_should_fail() {
	new_test_ext().execute_with(|| {
		let cml_id = 11;
		CmlStore::<Test>::insert(
			cml_id,
			CML::from_genesis_seed(seed_from_lifespan(cml_id, 100)),
		);

		let rs = Auction::put_to_store(Origin::signed(1), cml_id, 1000, None);
		assert_noop!(rs, CmlError::<Test>::CMLOwnerInvalid);
	})
}

#[test]
fn put_to_store_should_fail_if_free_balance_lower_than_auction_pledge_amount() {
	new_test_ext().execute_with(|| {
		<Test as Config>::Currency::make_free_balance_be(&1, AUCTION_PLEDGE_AMOUNT - 1);
		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		assert_noop!(
			Auction::put_to_store(Origin::signed(1), cml_id, 1000, None),
			Error::<Test>::InsufficientAuctionPledge
		);
	})
}

#[test]
fn put_to_store_should_fail_if_cml_is_dead() {
	new_test_ext().execute_with(|| {
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, AUCTION_PLEDGE_AMOUNT);
		let cml_id = 11;
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		cml.defrost(&0);
		cml.convert_to_tree(&0);

		CmlStore::<Test>::insert(cml_id, cml);
		UserCmlStore::<Test>::insert(user, cml_id, ());

		frame_system::Pallet::<Test>::set_block_number(100);
		let rs = Auction::put_to_store(Origin::signed(user), cml_id, 1000, None);
		assert_noop!(rs, Error::<Test>::NotAllowToAuction);
	});
}

#[test]
fn put_to_store_works_for_frozen_seed() {
	new_test_ext().execute_with(|| {
		<Test as Config>::Currency::make_free_balance_be(&1, AUCTION_PLEDGE_AMOUNT);
		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		let rs = Auction::put_to_store(Origin::signed(1), cml_id, 1000, None);
		assert_ok!(rs);
	});
}

#[test]
fn put_to_store_works_for_locked_frozen_seed() {
	new_test_ext().execute_with(|| {
		<Test as Config>::Currency::make_free_balance_be(&1, AUCTION_PLEDGE_AMOUNT);
		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let mut seed = seed_from_lifespan(cml_id, 100);
		seed.defrost_time = Some(1000);
		let cml = CML::from_genesis_seed(seed);
		assert!(cml.is_frozen_seed());
		assert!(!cml.seed_valid(&0));
		CmlStore::<Test>::insert(cml_id, cml);

		let rs = Auction::put_to_store(Origin::signed(1), cml_id, 1000, None);
		assert_ok!(rs);
	});
}

#[test]
fn put_to_store_works_for_fresh_seed() {
	new_test_ext().execute_with(|| {
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, AUCTION_PLEDGE_AMOUNT);
		let cml_id = 11;
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		cml.defrost(&0);
		CmlStore::<Test>::insert(cml_id, cml);
		UserCmlStore::<Test>::insert(user, cml_id, ());

		let rs = Auction::put_to_store(Origin::signed(user), cml_id, 1000, None);
		assert_ok!(rs);
	});
}

#[test]
fn put_to_store_should_fail_if_seed_has_overed_fresh_duration() {
	new_test_ext().execute_with(|| {
		let user = 1;
		<Test as Config>::Currency::make_free_balance_be(&user, AUCTION_PLEDGE_AMOUNT);
		let cml_id = 11;
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		cml.defrost(&0);
		let fresh_duration = cml.get_fresh_duration();
		CmlStore::<Test>::insert(cml_id, cml);
		UserCmlStore::<Test>::insert(user, cml_id, ());

		frame_system::Pallet::<Test>::set_block_number(fresh_duration);
		let rs = Auction::put_to_store(Origin::signed(user), cml_id, 1000, None);
		assert_noop!(rs, Error::<Test>::NotAllowToAuction);
	});
}

#[test]
fn bid_for_auction_works() {
	new_test_ext().execute_with(|| {
		let user_id = 11;
		let auction_id = 22;
		let user1_origin_balance = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&user_id, user1_origin_balance);

		let starting_price = 100;
		let mut auction_item = default_auction_item(auction_id, 2, 1);
		auction_item.starting_price = starting_price;
		Auction::add_auction_to_storage(auction_item, &2);

		assert_ok!(Auction::bid_for_auction(
			Origin::signed(user_id),
			auction_id,
			starting_price
		));

		let bid_item = BidStore::<Test>::get(user_id, auction_id);
		assert_eq!(bid_item.auction_id, auction_id);
		assert_eq!(bid_item.price, starting_price);
		assert_eq!(bid_item.user, user_id);

		let auction_bid_list = AuctionBidStore::<Test>::get(auction_id).unwrap();
		assert_eq!(auction_bid_list.len(), 1);
		assert_eq!(auction_bid_list.get(0).unwrap(), &user_id);

		let user_bid_list = UserBidStore::<Test>::get(user_id).unwrap();
		assert_eq!(user_bid_list.len(), 1);
		assert_eq!(user_bid_list.get(0).unwrap(), &auction_id);

		assert_eq!(
			<Test as Config>::Currency::free_balance(&user_id),
			user1_origin_balance - starting_price
		);
	})
}

#[test]
fn bid_for_diff_auction_to_check_user_balance() {
	// cml was not CmlLive, no need deposit.
	new_test_ext().execute_with(|| {
		let owner = 2;
		<Test as Config>::Currency::make_free_balance_be(&owner, AUCTION_PLEDGE_AMOUNT);
		let bid_user = 10;
		<Test as Config>::Currency::make_free_balance_be(&bid_user, 1000);

		let cml_id = 1;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		assert!(cml.is_frozen_seed());
		Cml::add_cml(&owner, cml);

		assert_ok!(Auction::put_to_store(
			Origin::signed(owner),
			cml_id,
			100,
			None
		));

		let auction_id = UserAuctionStore::<Test>::get(owner)[0];

		// user bid cml with 150
		assert_ok!(Auction::bid_for_auction(
			Origin::signed(bid_user),
			auction_id,
			150,
		));

		let balance = <Test as Config>::Currency::free_balance(bid_user);
		// user balance was 850
		assert_eq!(balance, 850);
	});

	// cml was CmlLive, need deposit.
	new_test_ext().execute_with(|| {
		let owner = 2;
		let bid_user = 10;
		let user_origin_balance = 10000;
		<Test as Config>::Currency::make_free_balance_be(&bid_user, user_origin_balance);
		<Test as Config>::Currency::make_free_balance_be(&owner, user_origin_balance);

		let cml_id = 1;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		UserCmlStore::<Test>::insert(owner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(owner),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));
		let starting_price = 100;
		assert_ok!(Auction::put_to_store(
			Origin::signed(owner),
			cml_id,
			starting_price,
			None
		));

		let auction_id = UserAuctionStore::<Test>::get(owner)[0];

		let bid_price = 150;
		assert_ok!(Auction::bid_for_auction(
			Origin::signed(bid_user),
			auction_id,
			bid_price,
		));

		assert_eq!(
			Utils::free_balance(&bid_user),
			user_origin_balance - bid_price - STAKING_PRICE
		);
		assert_eq!(
			Utils::reserved_balance(&bid_user),
			bid_price + STAKING_PRICE
		);
	});
}

#[test]
fn two_user_bid_for_auction_works() {
	new_test_ext().execute_with(|| {
		let user1_id = 1;
		let user2_id = 2;
		let auction_id = 22;
		<Test as Config>::Currency::make_free_balance_be(&user1_id, 100 * 1000);
		<Test as Config>::Currency::make_free_balance_be(&user2_id, 100 * 1000);
		let mut auction_item = default_auction_item(auction_id, 5, 1);
		auction_item.starting_price = 100;
		Auction::add_auction_to_storage(auction_item, &5);

		let user1_bid_price = 150;
		assert_ok!(Auction::bid_for_auction(
			Origin::signed(user1_id),
			auction_id,
			user1_bid_price,
		));
		let bid_item = BidStore::<Test>::get(user1_id, auction_id);
		assert_eq!(bid_item.user, user1_id);
		assert_eq!(bid_item.price, user1_bid_price);

		let user2_bid_price = 200;
		assert_ok!(Auction::bid_for_auction(
			Origin::signed(user2_id),
			auction_id,
			user2_bid_price
		));
		let bid_item2 = BidStore::<Test>::get(user2_id, auction_id);
		assert_eq!(bid_item2.user, user2_id);
		assert_eq!(bid_item2.price, user2_bid_price);

		// bid_item1 stay the same
		let bid_item1 = BidStore::<Test>::get(user1_id, auction_id);
		assert_eq!(bid_item1.user, user1_id);
		assert_eq!(bid_item1.price, user1_bid_price);

		let bid_list = AuctionBidStore::<Test>::get(auction_id).unwrap();
		assert_eq!(bid_list.len(), 2);
		assert_eq!(bid_list[0], user1_id);
		assert_eq!(bid_list[1], user2_id);
	})
}

#[test]
fn bid_for_auction_add_price_works() {
	new_test_ext().execute_with(|| {
		let user1_id = 1;
		let user2_id = 2;
		let auction_id = 22;
		<Test as Config>::Currency::make_free_balance_be(&user1_id, 100 * 1000);
		<Test as Config>::Currency::make_free_balance_be(&user2_id, 100 * 1000);
		let mut auction_item = default_auction_item(auction_id, 5, 1);
		auction_item.starting_price = 100;
		Auction::add_auction_to_storage(auction_item, &5);

		let user1_bid_price = 150;
		assert_ok!(Auction::bid_for_auction(
			Origin::signed(user1_id),
			auction_id,
			user1_bid_price,
		));
		let bid_item = BidStore::<Test>::get(user1_id, auction_id);
		assert_eq!(bid_item.price, user1_bid_price);

		// add user2 bid for auction
		assert_ok!(Auction::bid_for_auction(
			Origin::signed(user2_id),
			auction_id,
			200
		));

		let user1_add_price = 100;
		assert_ok!(Auction::bid_for_auction(
			Origin::signed(user1_id),
			auction_id,
			user1_add_price,
		));
		let bid_item = BidStore::<Test>::get(user1_id, auction_id);
		assert_eq!(bid_item.price, user1_bid_price + user1_add_price);
	})
}

#[test]
fn bid_for_seed_with_buy_now_price_should_work() {
	new_test_ext().execute_with(|| {
		let user_id = 1;
		let owner = 2;
		let user_origin_balance = 100 * 1000;
		let owner_origin_balance = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&user_id, user_origin_balance);
		<Test as Config>::Currency::make_free_balance_be(&owner, owner_origin_balance);

		let auction_id = 22;
		let mut auction_item = default_auction_item(auction_id, owner, 1);
		let buy_now_price = 1000;
		auction_item.buy_now_price = Some(buy_now_price);
		Auction::add_auction_to_storage(auction_item, &owner);

		assert_ok!(Auction::bid_for_auction(
			Origin::signed(user_id),
			auction_id,
			buy_now_price
		));
		assert_eq!(AuctionStore::<Test>::get(auction_id).bid_user, None);
		assert!(!BidStore::<Test>::contains_key(user_id, auction_id));

		assert_eq!(
			Utils::free_balance(&user_id),
			user_origin_balance - buy_now_price
		);
		assert_eq!(Utils::reserved_balance(&user_id), 0);

		assert_eq!(
			Utils::free_balance(&owner),
			owner_origin_balance + buy_now_price
		);
	})
}

#[test]
fn bid_for_mining_tree_with_buy_now_price_should_work() {
	new_test_ext().execute_with(|| {
		let user_id = 1;
		let owner = 2;
		let user_origin_balance = 100 * 1000;
		let owner_origin_balance = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&user_id, user_origin_balance);
		<Test as Config>::Currency::make_free_balance_be(&owner, owner_origin_balance);

		let cml_id = 1;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		UserCmlStore::<Test>::insert(owner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(owner),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));
		let starting_price = 100;
		let buy_now_price = 500;
		assert_ok!(Auction::put_to_store(
			Origin::signed(owner),
			cml_id,
			starting_price,
			Some(buy_now_price)
		));
		assert_eq!(
			Utils::free_balance(&owner),
			owner_origin_balance - STAKING_PRICE
		);
		let auction_id = UserAuctionStore::<Test>::get(owner)[0];

		assert_ok!(Auction::bid_for_auction(
			Origin::signed(user_id),
			auction_id,
			buy_now_price
		));
		assert_eq!(AuctionStore::<Test>::get(auction_id).bid_user, None);
		assert!(!BidStore::<Test>::contains_key(user_id, auction_id));

		assert_eq!(
			Utils::free_balance(&user_id),
			user_origin_balance - buy_now_price - STAKING_PRICE
		);
		assert_eq!(Utils::reserved_balance(&user_id), STAKING_PRICE);

		assert_eq!(
			Utils::free_balance(&owner),
			owner_origin_balance + buy_now_price
		);
		assert_eq!(Utils::reserved_balance(&owner), 0);
	})
}

#[test]
fn bid_for_auction_with_insufficient_balance_should_fail() {
	new_test_ext().execute_with(|| {
		let auction_id = 22;
		AuctionStore::<Test>::insert(auction_id, default_auction_item(auction_id, 2, 1));

		assert_noop!(
			Auction::bid_for_auction(Origin::signed(1), auction_id, 10),
			Error::<Test>::NotEnoughBalance
		);
	})
}

#[test]
fn bid_mining_cml_should_have_sufficient_free_balance_for_staking() {
	new_test_ext().execute_with(|| {
		let user_id = 1;
		let owner = 2;
		let starting_price = 100;
		let user_origin_balance = starting_price + STAKING_PRICE - 1; // user first bid price is insufficient
		let owner_origin_balance = STAKING_PRICE + AUCTION_PLEDGE_AMOUNT;
		<Test as Config>::Currency::make_free_balance_be(&user_id, user_origin_balance);
		<Test as Config>::Currency::make_free_balance_be(&owner, owner_origin_balance);

		let cml_id = 1;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		UserCmlStore::<Test>::insert(owner, cml_id, ());
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Cml::start_mining(
			Origin::signed(owner),
			cml_id,
			[1u8; 32],
			b"miner_ip".to_vec()
		));
		assert_ok!(Auction::put_to_store(
			Origin::signed(owner),
			cml_id,
			starting_price,
			None
		));
		let auction_id = UserAuctionStore::<Test>::get(owner)[0];

		assert_noop!(
			Auction::bid_for_auction(Origin::signed(user_id), auction_id, starting_price),
			Error::<Test>::NotEnoughBalance
		);
	})
}

#[test]
fn bid_for_not_exist_auction_should_fail() {
	new_test_ext().execute_with(|| {
		<Test as Config>::Currency::make_free_balance_be(&1, 100 * 1000);

		let auction_id = 22;
		assert_noop!(
			Auction::bid_for_auction(Origin::signed(1), auction_id, 10),
			Error::<Test>::AuctionNotExist
		);
	})
}

#[test]
fn bid_for_auction_belongs_to_myself_should_fail() {
	new_test_ext().execute_with(|| {
		let auction_id = 22;
		let owner_id = 1;
		<Test as Config>::Currency::make_free_balance_be(&owner_id, 100 * 1000);
		AuctionStore::<Test>::insert(auction_id, default_auction_item(auction_id, owner_id, 1));

		assert_noop!(
			Auction::bid_for_auction(Origin::signed(owner_id), auction_id, 10),
			Error::<Test>::BidSelfBelongs
		);
	})
}

#[test]
fn bid_for_auction_with_invalid_price_should_fail() {
	// lower than start price
	new_test_ext().execute_with(|| {
		let owner_id = 1;
		let auction_id = 22;
		<Test as Config>::Currency::make_free_balance_be(&owner_id, 100 * 1000);
		let mut auction_item = default_auction_item(auction_id, 2, 1);
		auction_item.starting_price = 100;
		AuctionStore::<Test>::insert(auction_id, auction_item);

		assert_noop!(
			Auction::bid_for_auction(Origin::signed(owner_id), auction_id, 10), // 10 is lower than starting price
			Error::<Test>::InvalidBidPrice
		);
	});

	// second bid price should larger than first bid price
	new_test_ext().execute_with(|| {
		let user1_id = 1;
		let user2_id = 2;
		let auction_id = 22;
		<Test as Config>::Currency::make_free_balance_be(&user1_id, 100 * 1000);
		<Test as Config>::Currency::make_free_balance_be(&user2_id, 100 * 1000);
		let mut auction_item = default_auction_item(auction_id, 5, 1);
		auction_item.starting_price = 100;
		AuctionStore::<Test>::insert(auction_id, auction_item);

		let user1_bid_price = 150;
		assert_ok!(Auction::bid_for_auction(
			Origin::signed(user1_id),
			auction_id,
			user1_bid_price,
		));

		let user2_bid_price = 130;
		assert_noop!(
			Auction::bid_for_auction(Origin::signed(user2_id), auction_id, user2_bid_price),
			Error::<Test>::InvalidBidPrice
		);
	});

	// user add price should larger than the former price
	new_test_ext().execute_with(|| {
		let user1_id = 1;
		let user2_id = 2;
		let auction_id = 22;
		<Test as Config>::Currency::make_free_balance_be(&user1_id, 100 * 1000);
		<Test as Config>::Currency::make_free_balance_be(&user2_id, 100 * 1000);
		let mut auction_item = default_auction_item(auction_id, 5, 1);
		auction_item.starting_price = 100;
		AuctionStore::<Test>::insert(auction_id, auction_item);

		let user1_bid_price = 150;
		assert_ok!(Auction::bid_for_auction(
			Origin::signed(user1_id),
			auction_id,
			user1_bid_price,
		));

		// add user2 bid for auction
		assert_ok!(Auction::bid_for_auction(
			Origin::signed(user2_id),
			auction_id,
			200
		));

		let user1_add_price = 30; // user1_bid_price + user1_add_price < 200 (the second user bid price)
		assert_noop!(
			Auction::bid_for_auction(Origin::signed(user1_id), auction_id, user1_add_price),
			Error::<Test>::InvalidBidPrice
		);
	})
}

#[test]
fn remove_bid_for_auction_works() {
	new_test_ext().execute_with(|| {
		let user1_id = 1;
		let user2_id = 2;
		let initial_balance = 100 * 1000;
		let auction_id = 22;

		<Test as Config>::Currency::make_free_balance_be(&user1_id, initial_balance);
		<Test as Config>::Currency::make_free_balance_be(&user2_id, initial_balance);
		let auction_item = default_auction_item(auction_id, 5, 1);
		Auction::add_auction_to_storage(auction_item, &5);

		let user1_bid_price = 150;
		assert_ok!(Auction::bid_for_auction(
			Origin::signed(user1_id),
			auction_id,
			user1_bid_price
		));
		assert_ok!(Auction::bid_for_auction(
			Origin::signed(user2_id),
			auction_id,
			200
		));
		assert_eq!(AuctionBidStore::<Test>::get(auction_id).unwrap().len(), 2);
		assert_eq!(UserBidStore::<Test>::get(user1_id).unwrap().len(), 1);
		assert!(BidStore::<Test>::contains_key(user1_id, auction_id));
		assert_eq!(Utils::reserved_balance(&user1_id), user1_bid_price);
		let bid_item = BidStore::<Test>::get(&user1_id, &auction_id);
		assert_eq!(bid_item.price, user1_bid_price);
		assert_eq!(Auction::bid_total_price(&bid_item), user1_bid_price);

		assert_ok!(Auction::remove_bid_for_auction(
			Origin::signed(user1_id),
			auction_id
		));
		assert_eq!(AuctionBidStore::<Test>::get(auction_id).unwrap().len(), 1);
		assert_eq!(UserBidStore::<Test>::get(user1_id).unwrap().len(), 0);
		assert!(!BidStore::<Test>::contains_key(user1_id, auction_id));
		// todo should pass
		// assert_eq!(Utils::free_balance(&user1_id), initial_balance);
		// assert_eq!(Utils::reserved_balance(&user1_id), 0);
	})
}

#[test]
fn remove_not_exist_bid_should_fail() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Auction::remove_bid_for_auction(Origin::signed(1), 11),
			Error::<Test>::AuctionNotExist
		);
	})
}

#[test]
fn remove_not_my_bid_should_fail() {
	new_test_ext().execute_with(|| {
		let user_id = 1;
		let auction_id = 22;
		<Test as Config>::Currency::make_free_balance_be(&user_id, 100 * 1000);
		AuctionStore::<Test>::insert(auction_id, default_auction_item(auction_id, 5, 1));

		assert_ok!(Auction::bid_for_auction(
			Origin::signed(user_id),
			auction_id,
			150
		));

		assert_noop!(
			Auction::remove_bid_for_auction(Origin::signed(2), auction_id),
			Error::<Test>::NotFoundBid
		);
	})
}

#[test]
fn after_remove_we_can_bid_again() {
	new_test_ext().execute_with(|| {
		let user1_id = 1;
		let user2_id = 2;
		let auction_id = 22;
		<Test as Config>::Currency::make_free_balance_be(&user1_id, 100 * 1000);
		<Test as Config>::Currency::make_free_balance_be(&user2_id, 100 * 1000);
		AuctionStore::<Test>::insert(auction_id, default_auction_item(auction_id, 5, 1));

		assert_ok!(Auction::bid_for_auction(
			Origin::signed(user1_id),
			auction_id,
			150
		));
		assert_ok!(Auction::bid_for_auction(
			Origin::signed(user2_id),
			auction_id,
			200
		));
		assert_eq!(AuctionBidStore::<Test>::get(auction_id).unwrap().len(), 2);
		assert_eq!(UserBidStore::<Test>::get(user1_id).unwrap().len(), 1);
		assert!(BidStore::<Test>::contains_key(user1_id, auction_id));

		assert_ok!(Auction::remove_bid_for_auction(
			Origin::signed(user1_id),
			auction_id
		));
		// // todo check user1 balance
		// assert_eq!(
		// 	100 * 1000,
		// 	<Test as Config>::Currency::free_balance(&user1_id)
		// );

		assert_eq!(AuctionBidStore::<Test>::get(auction_id).unwrap().len(), 1);
		assert_eq!(UserBidStore::<Test>::get(user1_id).unwrap().len(), 0);
		assert!(!BidStore::<Test>::contains_key(user1_id, auction_id));

		// user1 bid again
		assert_ok!(Auction::bid_for_auction(
			Origin::signed(user1_id),
			auction_id,
			250
		));
		assert_eq!(AuctionBidStore::<Test>::get(auction_id).unwrap().len(), 2);
		assert_eq!(UserBidStore::<Test>::get(user1_id).unwrap().len(), 1);
		assert!(BidStore::<Test>::contains_key(user1_id, auction_id));
	})
}

#[test]
fn remove_the_winners_bid_should_fail() {
	new_test_ext().execute_with(|| {
		let user_id = 1;
		let auction_id = 22;
		<Test as Config>::Currency::make_free_balance_be(&user_id, 100 * 1000);
		AuctionStore::<Test>::insert(auction_id, default_auction_item(auction_id, 5, 1));

		assert_ok!(Auction::bid_for_auction(
			Origin::signed(user_id),
			auction_id,
			150
		));

		let auction_item = AuctionStore::<Test>::get(auction_id);
		assert_eq!(auction_item.bid_user, Some(user_id));
		assert_noop!(
			Auction::remove_bid_for_auction(Origin::signed(user_id), auction_id),
			Error::<Test>::NotAllowQuitBid
		);
	})
}

#[test]
fn remove_from_store_with_no_bid_works() {
	new_test_ext().execute_with(|| {
		let owner = 1;
		<Test as Config>::Currency::make_free_balance_be(&owner, AUCTION_PLEDGE_AMOUNT);
		let cml_id = 11;
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		cml.defrost(&0);
		CmlStore::<Test>::insert(cml_id, cml);
		UserCmlStore::<Test>::insert(owner, cml_id, ());

		assert_ok!(Auction::put_to_store(
			Origin::signed(owner),
			cml_id,
			1000,
			None
		));

		let auction_id = 1; // this is the first auction so ID is 1
		let (_, next_window) = Auction::get_window_block();

		assert_eq!(UserAuctionStore::<Test>::get(&owner).len(), 1);
		assert_eq!(
			EndBlockAuctionStore::<Test>::get(next_window)
				.unwrap()
				.len(),
			1
		);
		assert!(AuctionStore::<Test>::contains_key(auction_id));

		assert_ok!(Auction::remove_from_store(
			Origin::signed(owner),
			auction_id
		));
		assert!(UserAuctionStore::<Test>::get(owner).is_empty());

		assert!(EndBlockAuctionStore::<Test>::get(next_window)
			.unwrap()
			.is_empty());
		assert!(!AuctionStore::<Test>::contains_key(auction_id));
		// todo check balance of owner
	})
}

#[test]
fn remove_from_store_with_bid_works() {
	new_test_ext().execute_with(|| {
		let owner_id = 1;
		let user_id = 2;
		<Test as Config>::Currency::make_free_balance_be(&user_id, 100 * 1000);
		<Test as Config>::Currency::make_free_balance_be(&owner_id, 100 * 1000);
		let cml_id = 11;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);
		UserCmlStore::<Test>::insert(owner_id, cml_id, ());

		assert_ok!(Auction::put_to_store(
			Origin::signed(owner_id),
			cml_id,
			100,
			None
		));

		let auction_id = 1; // this is the first auction so ID is 1
		assert_ok!(Auction::bid_for_auction(
			Origin::signed(user_id),
			auction_id,
			150
		));

		assert_ok!(Auction::remove_from_store(
			Origin::signed(owner_id),
			auction_id
		));

		assert_eq!(AuctionBidStore::<Test>::get(auction_id).unwrap().len(), 0);
		assert_eq!(UserBidStore::<Test>::get(user_id).unwrap().len(), 0);
		assert!(!BidStore::<Test>::contains_key(user_id, auction_id));
		// todo check balance of user and owner
	})
}

#[test]
fn remove_not_my_auction_from_store_should_fail() {
	new_test_ext().execute_with(|| {
		let auction_id = 22;
		let auction_item = default_auction_item(auction_id, 2, 1);
		AuctionStore::<Test>::insert(auction_id, auction_item);

		assert_noop!(
			Auction::remove_from_store(Origin::signed(1), auction_id),
			Error::<Test>::AuctionOwnerInvalid
		);
	})
}

#[test]
fn remove_not_exist_auction_from_store_should_fail() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Auction::remove_from_store(Origin::signed(1), 11),
			Error::<Test>::AuctionNotExist
		);
	})
}

#[test]
fn after_remove_we_can_start_auction_again() {
	new_test_ext().execute_with(|| {
		let owner_id = 1;
		let amount = 100 * 1000;
		<Test as Config>::Currency::make_free_balance_be(&owner_id, amount);

		let cml_id = 11;
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);
		UserCmlStore::<Test>::insert(owner_id, cml_id, ());

		assert_ok!(Auction::put_to_store(
			Origin::signed(owner_id),
			cml_id,
			100,
			None
		));

		let auction_id = 1; // this is the first auction so ID is 1
		let (_, next_window) = Auction::get_window_block();

		assert_eq!(UserAuctionStore::<Test>::get(owner_id).len(), 1);
		assert_eq!(
			EndBlockAuctionStore::<Test>::get(next_window)
				.unwrap()
				.len(),
			1
		);
		assert!(AuctionStore::<Test>::contains_key(auction_id));

		assert_ok!(Auction::remove_from_store(
			Origin::signed(owner_id),
			auction_id
		));
		assert!(UserAuctionStore::<Test>::get(owner_id).is_empty());

		assert!(EndBlockAuctionStore::<Test>::get(next_window)
			.unwrap()
			.is_empty());
		assert!(!AuctionStore::<Test>::contains_key(auction_id));

		// put to store and
		assert_ok!(Auction::put_to_store(
			Origin::signed(owner_id),
			cml_id,
			1500,
			None
		));
		assert_eq!(UserAuctionStore::<Test>::get(owner_id).len(), 1);

		assert_eq!(
			EndBlockAuctionStore::<Test>::get(next_window)
				.unwrap()
				.len(),
			1
		);

		assert!(AuctionStore::<Test>::contains_key(
			LastAuctionId::<Test>::get()
		));
	})
}

fn default_auction_item(id: u64, owner_id: u64, cml_id: CmlId) -> AuctionItem<u64, u128, u64> {
	let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
	cml.set_owner(&owner_id);
	Cml::add_cml(&owner_id, cml);

	let mut auction_item = AuctionItem::default();
	auction_item.id = id;
	auction_item.cml_owner = owner_id;
	auction_item.cml_id = cml_id;
	auction_item
}

pub fn new_genesis_seed(id: CmlId) -> Seed {
	Seed {
		id,
		cml_type: CmlType::A,
		defrost_schedule: Some(DefrostScheduleType::Team),
		defrost_time: Some(0),
		lifespan: 0,
		performance: 0,
	}
}

pub fn seed_from_lifespan(id: CmlId, lifespan: u32) -> Seed {
	let mut seed = new_genesis_seed(id);
	seed.lifespan = lifespan;
	seed
}
