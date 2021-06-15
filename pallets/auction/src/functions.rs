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
    deposit: Option<BalanceOf<T>>
  ) -> BidItem<T::AuctionId, T::AccountId, BalanceOf<T>, T::BlockNumber> {
    let current_block = frame_system::Pallet::<T>::block_number();

    BidItem {
      auction_id,
      user: who,
      price,
      deposit,
      created_at: current_block,
      updated_at: current_block,
    }
  }

  // return current block window number and next.
  pub fn get_window_block() 
  -> (T::BlockNumber, T::BlockNumber) {
    let current_block = frame_system::Pallet::<T>::block_number();
    let current_window = current_block / T::AuctionDealWindowBLock::get();
    let next_window = (current_window + <T::BlockNumber>::saturated_from(1_u64)) * T::AuctionDealWindowBLock::get();

    let current_window = current_window * T::AuctionDealWindowBLock::get();
    (current_window, next_window)
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

  pub(super) fn get_min_bid_price(
    auction_item: &AuctionItem<T::AuctionId, T::AccountId, T::CmlId, BalanceOf<T>, T::BlockNumber>,
    who: &T::AccountId,
  ) -> Result<BalanceOf<T>, Error<T>> {
    let min_price = match BidStore::<T>::get(&who, auction_item.id) {
      Some(bid_item) => bid_item.price,
      None => <BalanceOf<T>>::saturated_from(0_u128)
    };

    let max_price = {
      if let Some(bid_user) = &auction_item.bid_user {
        let bid_item = BidStore::<T>::get(&bid_user, auction_item.id).ok_or(Error::<T>::NotFoundBid)?;
  
        bid_item.price
      }
      else {
        auction_item.starting_price
      }
    };

    let rs = max_price.saturating_sub(min_price).saturating_add(T::MinPriceForBid::get());
    
    Ok(rs)
  }

  pub fn delete_auction(
    auction_id: &T::AuctionId,
  ) -> DispatchResult {

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

    // remove from EndblockAuctionStore
    let (current_window, next_window) = Self::get_window_block();
    EndblockAuctionStore::<T>::mutate(current_window, |maybe_list| {
      if let Some(ref mut list) = maybe_list {
        if let Some(index) = list.iter().position(|x| *x == *auction_id) {
          list.remove(index);
        }
      }
    });
    EndblockAuctionStore::<T>::mutate(next_window, |maybe_list| {
      if let Some(ref mut list) = maybe_list {
        if let Some(index) = list.iter().position(|x| *x == *auction_id) {
          list.remove(index);
        }
      }
    });

    // remove from AuctionBidStore
    if let Some(bid_user_list) = AuctionBidStore::<T>::get(&auction_id){

      for user in bid_user_list.into_iter() {

        Self::delete_bid(&user, &auction_id)?;
       
      }
    }
    
    Ok(())
  
  }

  pub fn delete_bid(
    who: &T::AccountId,
    auction_id: &T::AuctionId,
  ) -> DispatchResult {
    // remove from BidStore
    let bid_item = BidStore::<T>::take(&who, &auction_id).ok_or(Error::<T>::NotFoundBid)?;

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

    // return lock balance
    let mut total = bid_item.price;
    if let Some(deposit) = bid_item.deposit {
      total = total.saturating_add(deposit);
    }

    T::CurrencyOperations::unreserve(&who, total)?;


    Ok(())
  }

  pub fn complete_auction(
    auction_item: &AuctionItem<T::AuctionId, T::AccountId, T::CmlId, BalanceOf<T>, T::BlockNumber>,
    target: &T::AccountId,
  ) -> DispatchResult {

    let bid_item = BidStore::<T>::get(&target, &auction_item.id).ok_or(Error::<T>::NotFoundBid)?;

    let rs = cml::Pallet::<T>::transfer_cml_other(
      &auction_item.cml_owner, 
      &auction_item.cml_id, 
      &target,
    );

    if rs.is_ok() {
      Self::delete_auction(&auction_item.id)?;

      // transfer price from bid_user to seller.
      T::CurrencyOperations::transfer(&target, &auction_item.cml_owner, bid_item.price, AllowDeath)?;

      Self::deposit_event(Event::AuctionSuccess(auction_item.id, target.clone(), bid_item.price));
    }

    Ok(())
  }

  // when in window block, check each auction could complet or not.
  pub fn check_auction_in_block_window(

  ) -> DispatchResult {
    let (current_window, next_window) = Self::get_window_block();
    let current_block = frame_system::Pallet::<T>::block_number();

    ensure!(
      (current_block % T::AuctionDealWindowBLock::get()) > <T::BlockNumber>::saturated_from(3_u64),
      Error::<T>::NotInWindowBlock
    );

    if let Some(auction_list) = EndblockAuctionStore::<T>::take(current_window) {
      info!("auction_list => {:?}", auction_list);
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
  ) -> DispatchResult {
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
}

// #[cfg(test)]
// mod tests {
//   #![warn(unused_imports)]

//   use crate::{
//     mock::*, types::*, Config,
//   };
//   use frame_support::{traits::Currency};
//   // use pallet_cml::{
//   //     CmlStatus, CmlStore, DaiStore, Error as CmlError, StakingItem, UserCmlStore, CML,
//   // };

//   #[test]
//   fn transfer_balance_from_a_to_b_should_work(){
//     new_test_ext().execute_with(||{
//       let a = 1;
//       let b = 2;

//       <Test as Config>::Currency::make_free_balance_be(&a, 1000);
//       <Test as Config>::Currency::make_free_balance_be(&b, 1000);

//       Auction::transfer_balance(&a, &b, 100).unwrap();
//       assert_eq!(<Test as Config>::Currency::free_balance(&a), 1000-100);
//       assert_eq!(<Test as Config>::Currency::free_balance(&b), 1000+100);
//     });
//   }
// }