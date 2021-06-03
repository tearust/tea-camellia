use crate::{
    mock::*, types::*, AuctionBidStore, AuctionStore, BidStore, Config, EndblockAuctionStore,
    Error, UserAuctionStore, UserBidStore,
};
use frame_support::{assert_noop, assert_ok, traits::Currency};
use pallet_cml::{
    CmlStatus, CmlStore, DaiStore, Error as CmlError, StakingItem, UserCmlStore, CML,
};

#[test]
fn put_to_store_works() {
    new_test_ext().execute_with(|| {
        let amount = 100 * 1000; // Unit * StakingPrice
        DaiStore::<Test>::insert(1, 100);
        <Test as Config>::Currency::make_free_balance_be(&1, amount);

        Cml::convert_cml_from_dai(Origin::signed(1)).unwrap();
        let cml_list = UserCmlStore::<Test>::get(1).unwrap();
        let cml = CmlStore::<Test>::get(cml_list[0]).unwrap();

        let miner_id = b"miner_id".to_vec();
        let miner_ip = b"miner_ip".to_vec();
        assert_ok!(Cml::active_cml_for_nitro(
            Origin::signed(1),
            cml.id,
            miner_id.clone(),
            miner_ip.clone()
        ));

        assert_ok!(Auction::put_to_store(Origin::signed(1), cml.id, 1000, None));

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
        CmlStore::<Test>::insert(cml_id, default_cml(cml_id));

        let rs = Auction::put_to_store(Origin::signed(1), cml_id, 1000, None);
        assert_noop!(rs, CmlError::<Test>::CMLOwnerInvalid);
    })
}

#[test]
fn put_inactive_cml_to_store_with_diff_cml_status() {
    let create_cml = |id, status| {
        let mut cml = default_cml(id);
        cml.staking_slot = vec![StakingItem {
            owner: 1,
            category: pallet_cml::StakingCategory::Cml,
            amount: Some(1000),
            cml: None,
        }];
        cml.status = status;

        cml
    };

    // fail if `CmlStatus` is Dead
    new_test_ext().execute_with(|| {
        let user = 1;
        let cml_id = 11;
        let cml = create_cml(cml_id, CmlStatus::Dead);

        CmlStore::<Test>::insert(cml_id, cml);
        UserCmlStore::<Test>::insert(user, vec![11]);

        let rs = Auction::put_to_store(Origin::signed(user), cml_id, 1000, None);
        assert_noop!(rs, Error::<Test>::NotAllowToAuction);
    });

    // success for other CmlStatus
    new_test_ext().execute_with(|| {
        let user = 1;
        let cml_id = 11;
        let cml = create_cml(cml_id, CmlStatus::Staking);

        CmlStore::<Test>::insert(cml_id, cml);
        UserCmlStore::<Test>::insert(user, vec![11]);

        let rs = Auction::put_to_store(Origin::signed(user), cml_id, 1000, None);
        assert_ok!(rs);
    });

}

#[test]
fn bid_for_auction_works() {
    new_test_ext().execute_with(|| {
        let user_id = 1;
        let auction_id = 22;
        <Test as Config>::Currency::make_free_balance_be(&user_id, 100 * 1000);
        let mut auction_item = default_auction_item(auction_id, 2);
        auction_item.starting_price = 100;
        AuctionStore::<Test>::insert(auction_id, auction_item);

        assert_ok!(Auction::bid_for_auction(
            Origin::signed(user_id),
            auction_id,
            101
        ));

        let bid_item = BidStore::<Test>::get(user_id, auction_id).unwrap();
        assert_eq!(bid_item.auction_id, auction_id);
        assert_eq!(bid_item.price, 101);
        assert_eq!(bid_item.user, user_id);

        let auction_bid_list = AuctionBidStore::<Test>::get(auction_id).unwrap();
        assert_eq!(auction_bid_list.len(), 1);
        assert_eq!(auction_bid_list.get(0).unwrap(), &user_id);

        let user_bid_list = UserBidStore::<Test>::get(user_id).unwrap();
        assert_eq!(user_bid_list.len(), 1);
        assert_eq!(user_bid_list.get(0).unwrap(), &auction_id);
    })
}

#[test]
fn two_user_bid_for_auction_works() {
    new_test_ext().execute_with(|| {
        let user1_id = 1;
        let user2_id = 2;
        let auction_id = 22;
        <Test as Config>::Currency::make_free_balance_be(&user1_id, 100 * 1000);
        <Test as Config>::Currency::make_free_balance_be(&user2_id, 100 * 1000);
        let mut auction_item = default_auction_item(auction_id, 5);
        auction_item.starting_price = 100;
        AuctionStore::<Test>::insert(auction_id, auction_item);

        let user1_bid_price = 150;
        assert_ok!(Auction::bid_for_auction(
            Origin::signed(user1_id),
            auction_id,
            user1_bid_price,
        ));
        let bid_item = BidStore::<Test>::get(user1_id, auction_id).unwrap();
        assert_eq!(bid_item.user, user1_id);
        assert_eq!(bid_item.price, user1_bid_price);

        let user2_bid_price = 200;
        assert_ok!(Auction::bid_for_auction(
            Origin::signed(user2_id),
            auction_id,
            user2_bid_price
        ));
        let bid_item2 = BidStore::<Test>::get(user2_id, auction_id).unwrap();
        assert_eq!(bid_item2.user, user2_id);
        assert_eq!(bid_item2.price, user2_bid_price);

        let bid_item1 = BidStore::<Test>::get(user1_id, auction_id).unwrap();
        assert_eq!(bid_item1.user, user1_id);
        assert_eq!(bid_item1.price, user1_bid_price);
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
        let mut auction_item = default_auction_item(auction_id, 5);
        auction_item.starting_price = 100;
        AuctionStore::<Test>::insert(auction_id, auction_item);

        let user1_bid_price = 150;
        assert_ok!(Auction::bid_for_auction(
            Origin::signed(user1_id),
            auction_id,
            user1_bid_price,
        ));
        let bid_item = BidStore::<Test>::get(user1_id, auction_id).unwrap();
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
        let bid_item = BidStore::<Test>::get(user1_id, auction_id).unwrap();
        assert_eq!(bid_item.price, user1_bid_price + user1_add_price);
    })
}

#[test]
fn bid_for_auction_im_win_for_now_should_work() {
    new_test_ext().execute_with(|| {
        let owner_id = 1;
        let auction_id = 22;
        <Test as Config>::Currency::make_free_balance_be(&owner_id, 100 * 1000);
        let mut auction_item = default_auction_item(auction_id, 2);
        auction_item.bid_user = Some(owner_id);
        AuctionStore::<Test>::insert(auction_id, auction_item);

        // todo: should not return error
        assert_noop!(
            Auction::bid_for_auction(Origin::signed(owner_id), auction_id, 10),
            Error::<Test>::NoNeedBid
        );
    })
}

#[test]
fn bid_for_auction_with_insufficient_balance_should_fail() {
    new_test_ext().execute_with(|| {
        let auction_id = 22;
        AuctionStore::<Test>::insert(auction_id, default_auction_item(auction_id, 2));

        assert_noop!(
            Auction::bid_for_auction(Origin::signed(1), auction_id, 10),
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
        AuctionStore::<Test>::insert(auction_id, default_auction_item(auction_id, owner_id));

        assert_noop!(
            Auction::bid_for_auction(Origin::signed(owner_id), auction_id, 10),
            Error::<Test>::BidSelfBelongs
        );
    })
}

#[test]
fn bid_for_auction_with_invalid_price_should_faild() {
    // lower than start price
    new_test_ext().execute_with(|| {
        let owner_id = 1;
        let auction_id = 22;
        <Test as Config>::Currency::make_free_balance_be(&owner_id, 100 * 1000);
        let mut auction_item = default_auction_item(auction_id, 2);
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
        let mut auction_item = default_auction_item(auction_id, 5);
        auction_item.starting_price = 100;
        AuctionStore::<Test>::insert(auction_id, auction_item);

        let user1_bid_price = 150;
        assert_ok!(Auction::bid_for_auction(
            Origin::signed(user1_id),
            auction_id,
            user1_bid_price,
        ));

        let user2_bid_price = 130; // > starting_price && < user1_bid_price
                                   // todo: should return `Error::<Test>::InvalidBidPrice` error
        assert_ok!(Auction::bid_for_auction(
            Origin::signed(user2_id),
            auction_id,
            user2_bid_price
        ),);
    });

    // user add price should larger than the former price
    new_test_ext().execute_with(|| {
        let user1_id = 1;
        let user2_id = 2;
        let auction_id = 22;
        <Test as Config>::Currency::make_free_balance_be(&user1_id, 100 * 1000);
        <Test as Config>::Currency::make_free_balance_be(&user2_id, 100 * 1000);
        let mut auction_item = default_auction_item(auction_id, 5);
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
                                  // todo: should return `Error::<Test>::InvalidBidPrice` error
        assert_ok!(Auction::bid_for_auction(
            Origin::signed(user1_id),
            auction_id,
            user1_add_price,
        ));
    })
}

#[test]
fn remove_bid_for_auction_works() {
    new_test_ext().execute_with(|| {
        let user1_id = 1;
        let user2_id = 2;
        let auction_id = 22;
        <Test as Config>::Currency::make_free_balance_be(&user1_id, 100 * 1000);
        <Test as Config>::Currency::make_free_balance_be(&user2_id, 100 * 1000);
        AuctionStore::<Test>::insert(auction_id, default_auction_item(auction_id, 5));

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
        let bid_item = BidStore::<Test>::get(user1_id, auction_id);
        assert!(bid_item.is_some());

        assert_ok!(Auction::remove_bid_for_auction(
            Origin::signed(user1_id),
            auction_id
        ));
        assert_eq!(AuctionBidStore::<Test>::get(auction_id).unwrap().len(), 1);
        assert_eq!(UserBidStore::<Test>::get(user1_id).unwrap().len(), 0);
        let bid_item = BidStore::<Test>::get(user1_id, auction_id);
        assert!(bid_item.is_none());
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
        AuctionStore::<Test>::insert(auction_id, default_auction_item(auction_id, 5));

        assert_ok!(Auction::bid_for_auction(
            Origin::signed(user_id),
            auction_id,
            150
        ));

        // todo: should work
        // assert_noop!(
        //     Auction::remove_bid_for_auction(Origin::signed(2), auction_id),
        //     Error::<Test>::AuctionOwnerInvalid
        // );
    })
}

fn default_cml(cml_id: u64) -> CML<u64, u64, u64, u128> {
    CML {
        id: cml_id,
        group: pallet_cml::CmlGroup::Tpm,
        status: CmlStatus::SeedFrozen,
        life_time: 0,
        lock_time: 0,
        mining_rate: 0,
        staking_slot: vec![],
        created_at: 0,
        miner_id: vec![],
    }
}

fn default_auction_item(id: u64, owner_id: u64) -> AuctionItem<u64, u64, u64, u128, u64> {
    AuctionItem {
        id,
        cml_id: 0,
        cml_owner: owner_id,
        starting_price: 1,
        buy_now_price: None,
        start_at: 0,
        end_at: 0,
        status: vec![],
        bid_user: None,
    }
}
