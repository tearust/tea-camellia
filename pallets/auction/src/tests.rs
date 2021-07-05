use crate::{
	mock::*, types::*, AuctionBidStore, AuctionStore, BidStore, Config, EndblockAuctionStore,
	Error, LastAuctionId, UserAuctionStore, UserBidStore,
};
use frame_support::{assert_noop, assert_ok, traits::Currency};
use pallet_cml::{
	CmlId, CmlStatus, CmlStore, CmlType, DefrostScheduleType, Error as CmlError, Seed,
	SeedProperties, StakingItem, UserCmlStore, CML,
};

#[test]
fn put_to_store_works() {
	new_test_ext().execute_with(|| {
		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		cml.defrost(&0);
		cml.convert_to_tree(&0);
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Auction::put_to_store(Origin::signed(1), cml_id, 1000, None));

		let auction_id = 1; // this is the first auction so ID is 1
		let store_list = UserAuctionStore::<Test>::get(1).unwrap();
		assert_eq!(store_list.len(), 1);
		assert_eq!(store_list.get(0).unwrap(), &auction_id);

		let (_, next_window) = Auction::get_window_block();
		let auction_list = EndblockAuctionStore::<Test>::get(next_window).unwrap();
		assert_eq!(auction_list.len(), 1);
		assert_eq!(auction_list.get(0).unwrap(), &auction_id);

		let auction = AuctionStore::<Test>::get(auction_id).unwrap();
		assert_eq!(auction.cml_owner, 1);
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
fn put_to_store_should_fail_if_cml_is_dead() {
	new_test_ext().execute_with(|| {
		let user = 1;
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
		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let mut cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		let rs = Auction::put_to_store(Origin::signed(1), cml_id, 1000, None);
		assert_ok!(rs);
	});
}

#[test]
fn put_to_store_works_for_locked_frozen_seed() {
	new_test_ext().execute_with(|| {
		let cml_id: CmlId = 4;
		UserCmlStore::<Test>::insert(1, cml_id, ());
		let mut seed = seed_from_lifespan(cml_id, 100);
		seed.defrost_time = Some(1000);
		let mut cml = CML::from_genesis_seed(seed);
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

//
// #[test]
// fn bid_for_auction_works() {
// 	new_test_ext().execute_with(|| {
// 		let user_id = 1;
// 		let auction_id = 22;
// 		<Test as Config>::Currency::make_free_balance_be(&user_id, 100 * 1000);
// 		let mut auction_item = default_auction_item(auction_id, 2);
// 		auction_item.starting_price = 100;
// 		AuctionStore::<Test>::insert(auction_id, auction_item);
// 		// UserAuctionStore::<Test>::insert(&user_id, vec![auction_id]);
//
// 		assert_ok!(Auction::bid_for_auction(
// 			Origin::signed(user_id),
// 			auction_id,
// 			101
// 		));
//
// 		let bid_item = BidStore::<Test>::get(user_id, auction_id).unwrap();
// 		assert_eq!(bid_item.auction_id, auction_id);
// 		assert_eq!(bid_item.price, 101);
// 		assert_eq!(bid_item.user, user_id);
//
// 		let auction_bid_list = AuctionBidStore::<Test>::get(auction_id).unwrap();
// 		assert_eq!(auction_bid_list.len(), 1);
// 		assert_eq!(auction_bid_list.get(0).unwrap(), &user_id);
//
// 		let user_bid_list = UserBidStore::<Test>::get(user_id).unwrap();
// 		assert_eq!(user_bid_list.len(), 1);
// 		assert_eq!(user_bid_list.get(0).unwrap(), &auction_id);
// 	})
// }
//
// #[test]
// fn bid_for_diff_auction_to_check_user_balance() {
// 	// cml was not CmlLive, no need deposit.
// 	new_test_ext().execute_with(|| {
// 		let owner = 2;
//
// 		let bid_user = 10;
// 		<Test as Config>::Currency::make_free_balance_be(&bid_user, 1000);
//
// 		let cml_id = 1;
// 		let mut cml = default_cml(cml_id);
// 		cml.status = CmlStatus::SeedFrozen;
//
// 		Cml::add_cml(&owner, cml);
//
// 		assert_ok!(Auction::put_to_store(
// 			Origin::signed(owner),
// 			cml_id,
// 			100,
// 			None
// 		));
//
// 		let auction_id = {
// 			let tmp = UserAuctionStore::<Test>::get(owner).unwrap();
// 			tmp.get(0).unwrap().clone()
// 		};
//
// 		// user bid cml with 150
// 		assert_ok!(Auction::bid_for_auction(
// 			Origin::signed(bid_user),
// 			auction_id,
// 			150,
// 		));
//
// 		let balance = <Test as Config>::Currency::free_balance(bid_user);
// 		// user balance was 850
// 		assert_eq!(balance, 850);
// 	});
//
// 	// cml was CmlLive, need deposit.
// 	new_test_ext().execute_with(|| {
// 		let owner = 2;
//
// 		let bid_user = 10;
// 		<Test as Config>::Currency::make_free_balance_be(&bid_user, 1000);
//
// 		let cml_id = 1;
// 		let mut cml = default_cml(cml_id);
// 		cml.status = CmlStatus::CmlLive;
//
// 		Cml::add_cml(&owner, cml);
//
// 		assert_ok!(Auction::put_to_store(
// 			Origin::signed(owner),
// 			cml_id,
// 			100,
// 			None
// 		));
//
// 		let auction_id = {
// 			let tmp = UserAuctionStore::<Test>::get(owner).unwrap();
// 			tmp.get(0).unwrap().clone()
// 		};
//
// 		// user bid cml with 150
// 		assert_ok!(Auction::bid_for_auction(
// 			Origin::signed(bid_user),
// 			auction_id,
// 			150,
// 		));
//
// 		let balance = <Test as Config>::Currency::free_balance(bid_user);
// 		// user balance was 1000-150-100
// 		assert_eq!(balance, 750);
// 	});
// }
//
// #[test]
// fn two_user_bid_for_auction_works() {
// 	new_test_ext().execute_with(|| {
// 		let user1_id = 1;
// 		let user2_id = 2;
// 		let auction_id = 22;
// 		<Test as Config>::Currency::make_free_balance_be(&user1_id, 100 * 1000);
// 		<Test as Config>::Currency::make_free_balance_be(&user2_id, 100 * 1000);
// 		let mut auction_item = default_auction_item(auction_id, 5);
// 		auction_item.starting_price = 100;
// 		AuctionStore::<Test>::insert(auction_id, auction_item);
//
// 		let user1_bid_price = 150;
// 		assert_ok!(Auction::bid_for_auction(
// 			Origin::signed(user1_id),
// 			auction_id,
// 			user1_bid_price,
// 		));
// 		let bid_item = BidStore::<Test>::get(user1_id, auction_id).unwrap();
// 		assert_eq!(bid_item.user, user1_id);
// 		assert_eq!(bid_item.price, user1_bid_price);
//
// 		let user2_bid_price = 200;
// 		assert_ok!(Auction::bid_for_auction(
// 			Origin::signed(user2_id),
// 			auction_id,
// 			user2_bid_price
// 		));
// 		let bid_item2 = BidStore::<Test>::get(user2_id, auction_id).unwrap();
// 		assert_eq!(bid_item2.user, user2_id);
// 		assert_eq!(bid_item2.price, user2_bid_price);
//
// 		let bid_item1 = BidStore::<Test>::get(user1_id, auction_id).unwrap();
// 		assert_eq!(bid_item1.user, user1_id);
// 		assert_eq!(bid_item1.price, user1_bid_price);
// 	})
// }
//
// #[test]
// fn bid_for_auction_add_price_works() {
// 	new_test_ext().execute_with(|| {
// 		let user1_id = 1;
// 		let user2_id = 2;
// 		let auction_id = 22;
// 		<Test as Config>::Currency::make_free_balance_be(&user1_id, 100 * 1000);
// 		<Test as Config>::Currency::make_free_balance_be(&user2_id, 100 * 1000);
// 		let mut auction_item = default_auction_item(auction_id, 5);
// 		auction_item.starting_price = 100;
// 		AuctionStore::<Test>::insert(auction_id, auction_item);
//
// 		let user1_bid_price = 150;
// 		assert_ok!(Auction::bid_for_auction(
// 			Origin::signed(user1_id),
// 			auction_id,
// 			user1_bid_price,
// 		));
// 		let bid_item = BidStore::<Test>::get(user1_id, auction_id).unwrap();
// 		assert_eq!(bid_item.price, user1_bid_price);
//
// 		// add user2 bid for auction
// 		assert_ok!(Auction::bid_for_auction(
// 			Origin::signed(user2_id),
// 			auction_id,
// 			200
// 		));
//
// 		let user1_add_price = 100;
// 		assert_ok!(Auction::bid_for_auction(
// 			Origin::signed(user1_id),
// 			auction_id,
// 			user1_add_price,
// 		));
// 		let bid_item = BidStore::<Test>::get(user1_id, auction_id).unwrap();
// 		assert_eq!(bid_item.price, user1_bid_price + user1_add_price);
// 	})
// }
//
// #[test]
// fn bid_for_auction_im_win_for_now_should_work() {
// 	new_test_ext().execute_with(|| {
// 		let owner_id = 1;
// 		let auction_id = 22;
// 		<Test as Config>::Currency::make_free_balance_be(&owner_id, 100 * 1000);
// 		let mut auction_item = default_auction_item(auction_id, 2);
// 		auction_item.bid_user = Some(owner_id);
// 		AuctionStore::<Test>::insert(auction_id, auction_item);
// 		BidStore::<Test>::insert(
// 			owner_id,
// 			auction_id,
// 			BidItem {
// 				auction_id,
// 				user: owner_id,
// 				price: 5,
// 				deposit: None,
// 				created_at: 0,
// 				updated_at: 0,
// 			},
// 		);
//
// 		assert_ok!(
// 			Auction::bid_for_auction(Origin::signed(owner_id), auction_id, 10),
// 			// Error::<Test>::NoNeedBid
// 		);
// 	})
// }
//
// #[test]
// fn bid_for_auction_with_insufficient_balance_should_fail() {
// 	new_test_ext().execute_with(|| {
// 		let auction_id = 22;
// 		AuctionStore::<Test>::insert(auction_id, default_auction_item(auction_id, 2));
//
// 		assert_noop!(
// 			Auction::bid_for_auction(Origin::signed(1), auction_id, 10),
// 			Error::<Test>::NotEnoughBalance
// 		);
// 	})
// }
//
// #[test]
// fn bid_for_not_exist_auction_should_fail() {
// 	new_test_ext().execute_with(|| {
// 		<Test as Config>::Currency::make_free_balance_be(&1, 100 * 1000);
//
// 		let auction_id = 22;
// 		assert_noop!(
// 			Auction::bid_for_auction(Origin::signed(1), auction_id, 10),
// 			Error::<Test>::AuctionNotExist
// 		);
// 	})
// }
//
// #[test]
// fn bid_for_auction_belongs_to_myself_should_fail() {
// 	new_test_ext().execute_with(|| {
// 		let auction_id = 22;
// 		let owner_id = 1;
// 		<Test as Config>::Currency::make_free_balance_be(&owner_id, 100 * 1000);
// 		AuctionStore::<Test>::insert(auction_id, default_auction_item(auction_id, owner_id));
//
// 		assert_noop!(
// 			Auction::bid_for_auction(Origin::signed(owner_id), auction_id, 10),
// 			Error::<Test>::BidSelfBelongs
// 		);
// 	})
// }
//
// #[test]
// fn bid_for_auction_with_invalid_price_should_faild() {
// 	// lower than start price
// 	new_test_ext().execute_with(|| {
// 		let owner_id = 1;
// 		let auction_id = 22;
// 		<Test as Config>::Currency::make_free_balance_be(&owner_id, 100 * 1000);
// 		let mut auction_item = default_auction_item(auction_id, 2);
// 		auction_item.starting_price = 100;
// 		AuctionStore::<Test>::insert(auction_id, auction_item);
//
// 		assert_noop!(
// 			Auction::bid_for_auction(Origin::signed(owner_id), auction_id, 10), // 10 is lower than starting price
// 			Error::<Test>::InvalidBidPrice
// 		);
// 	});
//
// 	// second bid price should larger than first bid price
// 	new_test_ext().execute_with(|| {
// 		let user1_id = 1;
// 		let user2_id = 2;
// 		let auction_id = 22;
// 		<Test as Config>::Currency::make_free_balance_be(&user1_id, 100 * 1000);
// 		<Test as Config>::Currency::make_free_balance_be(&user2_id, 100 * 1000);
// 		let mut auction_item = default_auction_item(auction_id, 5);
// 		auction_item.starting_price = 100;
// 		AuctionStore::<Test>::insert(auction_id, auction_item);
//
// 		let user1_bid_price = 150;
// 		assert_ok!(Auction::bid_for_auction(
// 			Origin::signed(user1_id),
// 			auction_id,
// 			user1_bid_price,
// 		));
//
// 		let user2_bid_price = 130;
// 		assert_noop!(
// 			Auction::bid_for_auction(Origin::signed(user2_id), auction_id, user2_bid_price),
// 			Error::<Test>::InvalidBidPrice
// 		);
// 	});
//
// 	// user add price should larger than the former price
// 	new_test_ext().execute_with(|| {
// 		let user1_id = 1;
// 		let user2_id = 2;
// 		let auction_id = 22;
// 		<Test as Config>::Currency::make_free_balance_be(&user1_id, 100 * 1000);
// 		<Test as Config>::Currency::make_free_balance_be(&user2_id, 100 * 1000);
// 		let mut auction_item = default_auction_item(auction_id, 5);
// 		auction_item.starting_price = 100;
// 		AuctionStore::<Test>::insert(auction_id, auction_item);
//
// 		let user1_bid_price = 150;
// 		assert_ok!(Auction::bid_for_auction(
// 			Origin::signed(user1_id),
// 			auction_id,
// 			user1_bid_price,
// 		));
//
// 		// add user2 bid for auction
// 		assert_ok!(Auction::bid_for_auction(
// 			Origin::signed(user2_id),
// 			auction_id,
// 			200
// 		));
//
// 		let user1_add_price = 30; // user1_bid_price + user1_add_price < 200 (the second user bid price)
// 		assert_noop!(
// 			Auction::bid_for_auction(Origin::signed(user1_id), auction_id, user1_add_price,),
// 			Error::<Test>::InvalidBidPrice
// 		);
// 	})
// }
//
// #[test]
// fn remove_bid_for_auction_works() {
// 	new_test_ext().execute_with(|| {
// 		let user1_id = 1;
// 		let user2_id = 2;
// 		let auction_id = 22;
// 		<Test as Config>::Currency::make_free_balance_be(&user1_id, 100 * 1000);
// 		<Test as Config>::Currency::make_free_balance_be(&user2_id, 100 * 1000);
// 		AuctionStore::<Test>::insert(auction_id, default_auction_item(auction_id, 5));
//
// 		assert_ok!(Auction::bid_for_auction(
// 			Origin::signed(user1_id),
// 			auction_id,
// 			150
// 		));
// 		assert_ok!(Auction::bid_for_auction(
// 			Origin::signed(user2_id),
// 			auction_id,
// 			200
// 		));
// 		assert_eq!(AuctionBidStore::<Test>::get(auction_id).unwrap().len(), 2);
// 		assert_eq!(UserBidStore::<Test>::get(user1_id).unwrap().len(), 1);
// 		let bid_item = BidStore::<Test>::get(user1_id, auction_id);
// 		assert!(bid_item.is_some());
//
// 		assert_ok!(Auction::remove_bid_for_auction(
// 			Origin::signed(user1_id),
// 			auction_id
// 		));
// 		assert_eq!(AuctionBidStore::<Test>::get(auction_id).unwrap().len(), 1);
// 		assert_eq!(UserBidStore::<Test>::get(user1_id).unwrap().len(), 0);
// 		let bid_item = BidStore::<Test>::get(user1_id, auction_id);
// 		assert!(bid_item.is_none());
// 	})
// }
//
// #[test]
// fn remove_not_exist_bid_should_fail() {
// 	new_test_ext().execute_with(|| {
// 		assert_noop!(
// 			Auction::remove_bid_for_auction(Origin::signed(1), 11),
// 			Error::<Test>::AuctionNotExist
// 		);
// 	})
// }
//
// #[test]
// fn remove_not_my_bid_should_fail() {
// 	new_test_ext().execute_with(|| {
// 		let user_id = 1;
// 		let auction_id = 22;
// 		<Test as Config>::Currency::make_free_balance_be(&user_id, 100 * 1000);
// 		AuctionStore::<Test>::insert(auction_id, default_auction_item(auction_id, 5));
//
// 		assert_ok!(Auction::bid_for_auction(
// 			Origin::signed(user_id),
// 			auction_id,
// 			150
// 		));
//
// 		assert_noop!(
// 			Auction::remove_bid_for_auction(Origin::signed(2), auction_id),
// 			Error::<Test>::NotFoundBid
// 		);
// 	})
// }
//
// #[test]
// fn after_remove_we_can_bid_again() {
// 	new_test_ext().execute_with(|| {
// 		let user1_id = 1;
// 		let user2_id = 2;
// 		let auction_id = 22;
// 		<Test as Config>::Currency::make_free_balance_be(&user1_id, 100 * 1000);
// 		<Test as Config>::Currency::make_free_balance_be(&user2_id, 100 * 1000);
// 		AuctionStore::<Test>::insert(auction_id, default_auction_item(auction_id, 5));
//
// 		assert_ok!(Auction::bid_for_auction(
// 			Origin::signed(user1_id),
// 			auction_id,
// 			150
// 		));
// 		assert_ok!(Auction::bid_for_auction(
// 			Origin::signed(user2_id),
// 			auction_id,
// 			200
// 		));
// 		assert_eq!(AuctionBidStore::<Test>::get(auction_id).unwrap().len(), 2);
// 		assert_eq!(UserBidStore::<Test>::get(user1_id).unwrap().len(), 1);
// 		let bid_item = BidStore::<Test>::get(user1_id, auction_id);
// 		assert!(bid_item.is_some());
//
// 		assert_ok!(Auction::remove_bid_for_auction(
// 			Origin::signed(user1_id),
// 			auction_id
// 		));
// 		// check user1 balance
// 		assert_eq!(
// 			100 * 1000,
// 			<Test as Config>::Currency::free_balance(&user1_id)
// 		);
//
// 		assert_eq!(AuctionBidStore::<Test>::get(auction_id).unwrap().len(), 1);
// 		assert_eq!(UserBidStore::<Test>::get(user1_id).unwrap().len(), 0);
// 		let bid_item = BidStore::<Test>::get(user1_id, auction_id);
// 		assert!(bid_item.is_none());
//
// 		// user1 bid again
// 		assert_ok!(Auction::bid_for_auction(
// 			Origin::signed(user1_id),
// 			auction_id,
// 			250
// 		));
// 		assert_eq!(AuctionBidStore::<Test>::get(auction_id).unwrap().len(), 2);
// 		assert_eq!(UserBidStore::<Test>::get(user1_id).unwrap().len(), 1);
// 		let bid_item = BidStore::<Test>::get(user1_id, auction_id);
// 		assert!(bid_item.is_some());
// 	})
// }
//
// #[test]
// fn remove_the_winners_bid_should_fail() {
// 	new_test_ext().execute_with(|| {
// 		let user_id = 1;
// 		let auction_id = 22;
// 		<Test as Config>::Currency::make_free_balance_be(&user_id, 100 * 1000);
// 		AuctionStore::<Test>::insert(auction_id, default_auction_item(auction_id, 5));
//
// 		assert_ok!(Auction::bid_for_auction(
// 			Origin::signed(user_id),
// 			auction_id,
// 			150
// 		));
//
// 		let auction_item = AuctionStore::<Test>::get(auction_id).unwrap();
// 		assert_eq!(auction_item.bid_user, Some(user_id));
// 		assert_noop!(
// 			Auction::remove_bid_for_auction(Origin::signed(user_id), auction_id),
// 			Error::<Test>::NotAllowQuitBid
// 		);
// 	})
// }
//
// #[test]
// fn remove_from_store_with_no_bid_works() {
// 	new_test_ext().execute_with(|| {
// 		let owner_id = 1;
// 		let amount = 100 * 1000;
// 		DaiStore::<Test>::insert(owner_id, 100);
// 		<Test as Config>::Currency::make_free_balance_be(&owner_id, amount);
//
// 		Cml::convert_cml_from_dai(Origin::signed(owner_id)).unwrap();
// 		let cml_list = UserCmlStore::<Test>::get(owner_id).unwrap();
// 		let cml = CmlStore::<Test>::get(cml_list[0]).unwrap();
//
// 		assert_ok!(Cml::active_cml_for_nitro(
// 			Origin::signed(owner_id),
// 			cml.id,
// 			b"miner_id".to_vec(),
// 			b"miner_ip".to_vec()
// 		));
//
// 		assert_ok!(Auction::put_to_store(
// 			Origin::signed(owner_id),
// 			cml.id,
// 			1000,
// 			None
// 		));
//
// 		let auction_id = 1; // this is the first auction so ID is 1
// 		let (_, next_window) = Auction::get_window_block();
//
// 		assert_eq!(UserAuctionStore::<Test>::get(owner_id).unwrap().len(), 1);
// 		assert_eq!(
// 			EndblockAuctionStore::<Test>::get(next_window)
// 				.unwrap()
// 				.len(),
// 			1
// 		);
// 		assert!(AuctionStore::<Test>::get(auction_id).is_some());
//
// 		assert_ok!(Auction::remove_from_store(
// 			Origin::signed(owner_id),
// 			auction_id
// 		));
// 		assert!(UserAuctionStore::<Test>::get(owner_id).unwrap().is_empty());
//
// 		assert!(EndblockAuctionStore::<Test>::get(next_window)
// 			.unwrap()
// 			.is_empty());
//
// 		assert!(AuctionStore::<Test>::get(auction_id).is_none());
// 		// todo check balance of owner
// 	})
// }
//
// #[test]
// fn remove_from_store_with_bid_works() {
// 	new_test_ext().execute_with(|| {
// 		let owner_id = 1;
// 		let user_id = 2;
// 		let amount = 100 * 1000;
// 		DaiStore::<Test>::insert(owner_id, 100);
// 		<Test as Config>::Currency::make_free_balance_be(&owner_id, amount);
// 		<Test as Config>::Currency::make_free_balance_be(&user_id, amount);
//
// 		Cml::convert_cml_from_dai(Origin::signed(owner_id)).unwrap();
// 		let cml_list = UserCmlStore::<Test>::get(1).unwrap();
// 		let cml = CmlStore::<Test>::get(cml_list[0]).unwrap();
//
// 		assert_ok!(Cml::active_cml_for_nitro(
// 			Origin::signed(owner_id),
// 			cml.id,
// 			b"miner_id".to_vec(),
// 			b"miner_ip".to_vec()
// 		));
// 		assert_ok!(Auction::put_to_store(
// 			Origin::signed(owner_id),
// 			cml.id,
// 			100,
// 			None
// 		));
//
// 		let auction_id = 1; // this is the first auction so ID is 1
// 		assert_ok!(Auction::bid_for_auction(
// 			Origin::signed(user_id),
// 			auction_id,
// 			150
// 		));
//
// 		assert_ok!(Auction::remove_from_store(
// 			Origin::signed(owner_id),
// 			auction_id
// 		));
//
// 		assert_eq!(AuctionBidStore::<Test>::get(auction_id).unwrap().len(), 0);
// 		assert_eq!(UserBidStore::<Test>::get(user_id).unwrap().len(), 0);
// 		assert!(BidStore::<Test>::get(user_id, auction_id).is_none());
// 		// todo check balance of user and owner
// 	})
// }
//
// #[test]
// fn remove_not_my_auction_from_store_should_fail() {
// 	new_test_ext().execute_with(|| {
// 		let auction_id = 22;
// 		let auction_item = default_auction_item(auction_id, 2);
// 		AuctionStore::<Test>::insert(auction_id, auction_item);
//
// 		assert_noop!(
// 			Auction::remove_from_store(Origin::signed(1), auction_id),
// 			Error::<Test>::AuctionOwnerInvalid
// 		);
// 	})
// }
//
// #[test]
// fn remove_not_exist_auction_from_store_should_fail() {
// 	new_test_ext().execute_with(|| {
// 		assert_noop!(
// 			Auction::remove_from_store(Origin::signed(1), 11),
// 			Error::<Test>::AuctionNotExist
// 		);
// 	})
// }
//
// #[test]
// fn after_remove_we_can_start_auction_again() {
// 	new_test_ext().execute_with(|| {
// 		let owner_id = 1;
// 		let amount = 100 * 1000;
// 		DaiStore::<Test>::insert(owner_id, 100);
// 		<Test as Config>::Currency::make_free_balance_be(&owner_id, amount);
//
// 		Cml::convert_cml_from_dai(Origin::signed(owner_id)).unwrap();
// 		let cml_list = UserCmlStore::<Test>::get(owner_id).unwrap();
// 		let cml = CmlStore::<Test>::get(cml_list[0]).unwrap();
//
// 		assert_ok!(Cml::active_cml_for_nitro(
// 			Origin::signed(owner_id),
// 			cml.id,
// 			b"miner_id".to_vec(),
// 			b"miner_ip".to_vec()
// 		));
//
// 		assert_ok!(Auction::put_to_store(
// 			Origin::signed(owner_id),
// 			cml.id,
// 			1000,
// 			None
// 		));
//
// 		let auction_id = 1; // this is the first auction so ID is 1
// 		let (_, next_window) = Auction::get_window_block();
//
// 		assert_eq!(UserAuctionStore::<Test>::get(owner_id).unwrap().len(), 1);
// 		assert_eq!(
// 			EndblockAuctionStore::<Test>::get(next_window)
// 				.unwrap()
// 				.len(),
// 			1
// 		);
// 		assert!(AuctionStore::<Test>::get(auction_id).is_some());
//
// 		assert_ok!(Auction::remove_from_store(
// 			Origin::signed(owner_id),
// 			auction_id
// 		));
// 		assert!(UserAuctionStore::<Test>::get(owner_id).unwrap().is_empty());
//
// 		assert!(EndblockAuctionStore::<Test>::get(next_window)
// 			.unwrap()
// 			.is_empty());
// 		assert!(AuctionStore::<Test>::get(auction_id).is_none());
//
// 		// put to store and
// 		assert_ok!(Auction::put_to_store(
// 			Origin::signed(owner_id),
// 			cml.id,
// 			1500,
// 			None
// 		));
// 		assert_eq!(UserAuctionStore::<Test>::get(owner_id).unwrap().len(), 1);
//
// 		assert_eq!(
// 			EndblockAuctionStore::<Test>::get(next_window)
// 				.unwrap()
// 				.len(),
// 			1
// 		);
//
// 		let last_auction_index = LastAuctionId::<Test>::get();
// 		assert!(AuctionStore::<Test>::get(last_auction_index - 1).is_some());
// 	})
// }
//
// #[test]
// fn whole_process_for_auction_and_bid() {
// 	new_test_ext().execute_with(|| {
// 		let owner = 1;
// 		let ua = 2;
// 		<Test as Config>::Currency::make_free_balance_be(&owner, 1000);
// 		<Test as Config>::Currency::make_free_balance_be(&ua, 1000);
//
// 		let cml_id = 1;
// 		let cml_item = default_cml(cml_id);
// 		Cml::add_cml(&owner, cml_item);
//
// 		assert_ok!(Auction::put_to_store(
// 			Origin::signed(owner),
// 			cml_id,
// 			100,
// 			Some(200)
// 		));
//
// 		let auction_id = {
// 			let tmp = UserAuctionStore::<Test>::get(owner).unwrap();
// 			tmp.get(0).unwrap().clone()
// 		};
//
// 		assert_ok!(Auction::bid_for_auction(
// 			Origin::signed(ua),
// 			auction_id,
// 			// 250,
// 			110,
// 		));
//
// 		// assert_eq!(1000+250, <Test as Config>::Currency::free_balance(&owner));
// 		// assert_eq!(1000-250, <Test as Config>::Currency::free_balance(&ua));
//
// 		assert_ok!(Auction::remove_from_store(
// 			Origin::signed(owner),
// 			auction_id,
// 		));
//
// 		// println!("11 => {:?}", <Test as Config>::Currency::free_balance(&owner));
// 		// println!("11 => {:?}", <Test as Config>::Currency::free_balance(&ua));
// 		assert_eq!(1000 - 1, <Test as Config>::Currency::free_balance(&owner));
// 		assert_eq!(1000 + 1, <Test as Config>::Currency::free_balance(&ua));
// 	});
// }
//
// fn default_cml(cml_id: u64) -> CML<u64, u64, u64, u128> {
// 	CML {
// 		id: cml_id,
// 		group: pallet_cml::CmlGroup::Tpm,
// 		status: CmlStatus::SeedFrozen,
// 		life_time: 0,
// 		lock_time: 0,
// 		mining_rate: 0,
// 		staking_slot: vec![],
// 		created_at: 0,
// 		miner_id: vec![],
// 	}
// }
//
// fn default_auction_item(id: u64, owner_id: u64) -> AuctionItem<u64, u64, u64, u128, u64> {
// 	let cml_item = default_cml(0);
// 	Cml::add_cml(&owner_id, cml_item);
// 	AuctionItem {
// 		id,
// 		cml_id: 0,
// 		cml_owner: owner_id,
// 		starting_price: 1,
// 		buy_now_price: None,
// 		start_at: 0,
// 		end_at: 0,
// 		status: vec![],
// 		bid_user: None,
// 	}
// }

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
