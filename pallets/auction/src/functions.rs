use super::*;

impl<T: auction::Config> auction::Pallet<T> {
  
  pub fn get_next_auction_id() -> T::AuctionId {
		let cid = LastAuctionId::<T>::get();
		let _id = cid.clone();
		LastAuctionId::<T>::mutate(|_id| *_id += One::one());

		cid
	}

  pub fn new_auction_item(
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
    }
  }
}