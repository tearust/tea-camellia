use super::*;

impl<T: auction::Config> auction::Pallet<T> {
  
  pub fn get_next_auction_id() -> T::AuctionId {
		let cid = LastAuctionId::<T>::get();
		let _id = cid.clone();
		LastAuctionId::<T>::mutate(|_id| *_id += One::one());

		cid
	}

  pub(super) fn new_auction_item(
    cml_id: T::CmlId,
    cml_owner: T::AccountId,
    starting_price: BalanceOf<T>,
    buy_now_price: Option<BalanceOf<T>>,
  ) -> AuctionItem<T::AuctionId, T::AccountId, T::CmlId, BalanceOf<T>, T::BlockNumber> {

    let current_block = frame_system::Pallet::<T>::block_number();

    let period: u64 = 1_000_000;
    let end_block = period.saturated_into::<T::BlockNumber>() + current_block;

    AuctionItem {
      id: Self::get_next_auction_id(),
      cml_id,
      cml_owner,
      starting_price,
      buy_now_price,
      start_at: current_block,
      end_at: end_block,

      status: b"normal".to_vec(),
      bid_user: None,
    }
  }

  pub(super) fn update_bid_price_for_auction_item(
    auction_id: &T::AuctionId,
    bid_user: T::AccountId,
  ){
    AuctionStore::<T>::mutate(&auction_id, |maybe_item| {
      if let Some(ref mut item) = maybe_item {
        item.bid_user = Some(bid_user);
      }
    });
  }

  pub(super) fn new_bid_item(
    auction_id: T::AuctionId,
    who: T::AccountId,
    price: BalanceOf<T>,
  ) -> BidItem<T::AuctionId, T::AccountId, BalanceOf<T>, T::BlockNumber> {
    let current_block = frame_system::Pallet::<T>::block_number();

    BidItem {
      auction_id,
      user: who,
      price,
      created_at: current_block,
      updated_at: current_block,
    }
  }

  pub(super) fn get_min_bid_price(
    auction_item: &AuctionItem<T::AuctionId, T::AccountId, T::CmlId, BalanceOf<T>, T::BlockNumber>,
    who: &T::AccountId,
  ) -> BalanceOf<T> {
    let min_price = &auction_item.starting_price;
    
    // TODO calcaute min price

    // if let Some(bid_user) = &auction_item.bid_user {
    //   let bid_item = BidStore::<T>::get(&bid_user, auction_item.id).unwrap();

    //   return bid_item.price - auction_item.starting_price;
    // }
    if let Some(_bid_item) = BidStore::<T>::get(&who, auction_item.id) {
      return <T as auction::Config>::Currency::minimum_balance();
    }
    
    *min_price
  }

  pub fn delete_auction(
    auction_id: &T::AuctionId,
  ) -> Result<(), Error<T>> {

    // remove from AuctionStore
    let auction_item = AuctionStore::<T>::take(&auction_id).unwrap();
    let who = auction_item.cml_owner;

    // remove from UserAuctionStore
    UserAuctionStore::<T>::mutate(&who, |maybe_list| {
      if let Some(ref mut list) = maybe_list {
        if let Some(index) = list.iter().position(|x| *x == *auction_id) {
          list.remove(index);
        }
      }
    });

    // remove from AuctionBidStore
    if let Some(bid_user_list) = AuctionBidStore::<T>::take(&auction_id){
      for user in bid_user_list.iter() {

        // remove from BidStore
        let _bid_item = BidStore::<T>::take(&user, &auction_id).unwrap();
  
        // TODO return bid price
  
        // remove from UserBidStore
        UserBidStore::<T>::mutate(&user, |maybe_list| {
          if let Some(ref mut list) = maybe_list {
            if let Some(index) = list.iter().position(|x| *x == *auction_id) {
              list.remove(index);
            }
          }
        });
      }
    }
    
    Ok(())
  
  }

  pub fn delete_bid(
    who: &T::AccountId,
    auction_id: &T::AuctionId,
  ) -> Result<(), Error<T>> {
    // remove from BidStore
    let _bid_item = BidStore::<T>::take(&who, &auction_id).unwrap();
    // TODO return bid price.

    // remove from UserBidStore
    UserBidStore::<T>::mutate(&who, |maybe_list| {
      if let Some(ref mut list) = maybe_list {
        if let Some(index) = list.iter().position(|x| *x == *auction_id) {
          list.remove(index);
        }
      }
    });

    // remove from AuctionBidStore
    AuctionBidStore::<T>::mutate(&auction_id, |maybe_list| {
      if let Some(ref mut list) = maybe_list {
        if let Some(index) = list.iter().position(|x| &x.cmp(&who) == &Ordering::Equal) {
          list.remove(index);
        }
      }
    });


    Ok(())
  }
}