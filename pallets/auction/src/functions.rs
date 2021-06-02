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

    // TODO remove end_block
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

  pub fn get_window_block() 
  -> (T::BlockNumber, T::BlockNumber) {
    let current_block = frame_system::Pallet::<T>::block_number();
    let current_window = current_block / T::AuctionDealWindowBLock::get();
    let next_window = (current_window + <T::BlockNumber>::saturated_from(1_u64)) * T::AuctionDealWindowBLock::get();

    info!("11111 => {:?}", current_block % T::AuctionDealWindowBLock::get());

    (current_window, next_window)
  }

  fn get_next_window(current_block: T::BlockNumber) -> T::BlockNumber {
    current_block + T::AuctionDealWindowBLock::get()
  }

  pub fn add_auction_to_storage(
    auction_item: AuctionItem<T::AuctionId, T::AccountId, T::CmlId, BalanceOf<T>, T::BlockNumber>,
    who: &T::AccountId,
  ){

    UserAuctionStore::<T>::mutate(&who, |maybe_list| {	
      if let Some(ref mut list) = maybe_list {
        list.push(auction_item.id);
      }
      else {
        *maybe_list = Some(vec![auction_item.id]);
      }
    });

    let (_, next_window) = Self::get_window_block();

    EndblockAuctionStore::<T>::mutate(next_window, |maybe_list| {
      if let Some(ref mut list) = maybe_list {
        list.push(auction_item.id);
      }
      else {
        *maybe_list = Some(vec![auction_item.id]);
      }
    });

    AuctionStore::<T>::insert(auction_item.id, auction_item);
  }

  // pub push_auction_to_next_window(){
  //   let (current_window, next_window) = Self::get_window_block();
  //   if let (auction_list) = EndblockAuctionStore::<T>::take(current_window) {
  //     EndblockAuctionStore::<T>::insert(next_window, auction_list);
  //   }
  // }

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

    // withdraw owner lock fee
    // Self::lock_tea(&who, T::AuctionDeposit::get());
    // <T as auction::Config>::Currency::unreserve(&who, T::AuctionDeposit::get())?;

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

  pub fn complete_auction(
    auction_item: &AuctionItem<T::AuctionId, T::AccountId, T::CmlId, BalanceOf<T>, T::BlockNumber>,
    target: &T::AccountId,
  ) -> Result<(), Error<T>> {

    let rs = cml::Pallet::<T>::transfer_cml_other(
      &auction_item.cml_owner, 
      &auction_item.cml_id, 
      &target,
    );

    match rs {
      Ok(_) => {
        Self::delete_auction(&auction_item.id)?;
      },
      Err(_) => {}
    }

    Ok(())
  }

  // when in window block, check each auction could complet or not.
  pub fn check_auction_in_block_window(

  ) -> Result<(), Error<T>> {
    let (current_window, next_window) = Self::get_window_block();
    let current_block = frame_system::Pallet::<T>::block_number();

    if (current_block % T::AuctionDealWindowBLock::get()) > <T::BlockNumber>::saturated_from(3_u64) {
      return Err(Error::<T>::NotInWindowBlock);
    }

    if let Some(auction_list) = EndblockAuctionStore::<T>::take(current_window) {
      for auction_id in auction_list.iter() {
        if let Some(auction_item) = AuctionStore::<T>::get(&auction_id) {
          Self::check_each_auction_in_block_window(auction_item, next_window)?;
        }
        
      }
    }

    Ok(())
  }

  fn check_each_auction_in_block_window(
    auction_item: AuctionItem<T::AuctionId, T::AccountId, T::CmlId, BalanceOf<T>, T::BlockNumber>,
    next_window: T::BlockNumber,
  ) -> Result<(), Error<T>> {
    if let Some(ref bid_user) = auction_item.bid_user {

      Self::complete_auction(&auction_item, &bid_user)?;
    }
    else {
      // put to next block
      EndblockAuctionStore::<T>::mutate(next_window, |maybe_list| {
        if let Some(ref mut list) = maybe_list {
          list.push(auction_item.id);
        }
        else {
          *maybe_list = Some(vec![auction_item.id]);
        }
      });
    }

    Ok(())
  }

  pub fn reserve(
    who: &T::AccountId,
    amount: BalanceOf<T>,
  ) -> DispatchResult {

    <T as auction::Config>::Currency::reserve(&who, amount)?;
    Ok(())
  }
  pub fn unreserve(
    who: &T::AccountId,
    amount: BalanceOf<T>,
  ) -> DispatchResult {

    <T as auction::Config>::Currency::unreserve(&who, amount);
    Ok(())
  }
}