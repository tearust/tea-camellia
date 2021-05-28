use super::*;

impl<T: auction::Config> auction::Pallet<T> {
  
  pub fn get_next_auction_id() -> T::AuctionId {
		let cid = LastAuctionId::<T>::get();
		let _id = cid.clone();
		LastAuctionId::<T>::mutate(|_id| *_id += One::one());

		cid
	}

  pub(super) fn new_auction_item(
    cml_id: T::AssetId,
    cml_owner: T::AccountId,
    starting_price: BalanceOf<T>,
    buy_now_price: Option<BalanceOf<T>>,
  ) -> AuctionItem<T::AuctionId, T::AccountId, T::AssetId, BalanceOf<T>, T::BlockNumber> {

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
    auction_item: &AuctionItem<T::AuctionId, T::AccountId, T::AssetId, BalanceOf<T>, T::BlockNumber>
  ) -> BalanceOf<T> {
    let min_price = &auction_item.starting_price;
    if let Some(bid_user) = &auction_item.bid_user {
      let bid_item = BidStore::<T>::get(&bid_user, auction_item.id).unwrap();

      return bid_item.price;
    }

    *min_price
  }
}