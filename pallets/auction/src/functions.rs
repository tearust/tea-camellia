use super::*;

impl<T: auction::Config> auction::Pallet<T> {
	pub fn get_next_auction_id() -> AuctionId {
		LastAuctionId::<T>::mutate(|id| {
			if *id < u64::MAX {
				*id += 1;
			} else {
				*id = 0;
			}

			*id
		})
	}

	pub(super) fn new_auction_item(
		cml_id: CmlId,
		cml_owner: T::AccountId,
		starting_price: BalanceOf<T>,
		buy_now_price: Option<BalanceOf<T>>,
	) -> AuctionItem<T::AccountId, BalanceOf<T>, T::BlockNumber> {
		AuctionItem {
			id: Self::get_next_auction_id(),
			cml_id,
			cml_owner,
			starting_price,
			buy_now_price,
			start_at: frame_system::Pallet::<T>::block_number(),
			status: AuctionStatus::Normal,
			bid_user: None,
		}
	}

	pub(super) fn update_current_winner(auction_id: &AuctionId, bid_user: &T::AccountId) {
		AuctionStore::<T>::mutate(&auction_id, |item| {
			item.bid_user = Some(bid_user.clone());
		});
	}

	pub(super) fn new_bid_item(
		auction_id: AuctionId,
		who: T::AccountId,
		price: BalanceOf<T>,
		deposit: Option<BalanceOf<T>>,
	) -> BidItem<T::AccountId, BalanceOf<T>, T::BlockNumber> {
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
	pub fn get_window_block() -> (T::BlockNumber, T::BlockNumber) {
		let current_block = frame_system::Pallet::<T>::block_number();
		let current_index = current_block / T::AuctionDealWindowBLock::get();
		let next_index = current_index + <T::BlockNumber>::saturated_from(1_u64);

		(
			current_index * T::AuctionDealWindowBLock::get(),
			next_index * T::AuctionDealWindowBLock::get(),
		)
	}

	pub fn add_auction_to_storage(
		auction_item: AuctionItem<T::AccountId, BalanceOf<T>, T::BlockNumber>,
		who: &T::AccountId,
	) {
		if UserAuctionStore::<T>::contains_key(who) {
			UserAuctionStore::<T>::mutate(&who, |list| {
				list.push(auction_item.id);
			});
		} else {
			UserAuctionStore::<T>::insert(who.clone(), vec![auction_item.id]);
		}

		let (_, next_window) = Self::get_window_block();
		Self::insert_into_end_block_store(next_window, auction_item.id);

		AuctionStore::<T>::insert(auction_item.id, auction_item);
	}

	fn insert_into_end_block_store(window_height: T::BlockNumber, auction_id: AuctionId) {
		EndBlockAuctionStore::<T>::mutate(window_height, |maybe_list| {
			if let Some(ref mut list) = maybe_list {
				list.push(auction_id);
			} else {
				*maybe_list = Some(vec![auction_id]);
			}
		});
	}

	pub(super) fn min_bid_price(
		auction_item: &AuctionItem<T::AccountId, BalanceOf<T>, T::BlockNumber>,
		who: &T::AccountId,
	) -> Result<BalanceOf<T>, Error<T>> {
		if auction_item.bid_user.is_none() {
			return Ok(auction_item.starting_price);
		}

		let origin_bid_price = match BidStore::<T>::contains_key(who, auction_item.id) {
			true => BidStore::<T>::get(who, auction_item.id).price,
			false => <BalanceOf<T>>::saturated_from(0_u128),
		};

		let starting_price = {
			if let Some(bid_user) = &auction_item.bid_user {
				ensure!(
					BidStore::<T>::contains_key(&bid_user, auction_item.id),
					Error::<T>::NotFoundBid
				);
				let bid_item = BidStore::<T>::get(&bid_user, auction_item.id);
				bid_item.price
			} else {
				auction_item.starting_price
			}
		};

		let rs = starting_price
			.saturating_sub(origin_bid_price)
			.saturating_add(T::MinPriceForBid::get());

		Ok(rs)
	}

	pub fn delete_auction(auction_id: &AuctionId) {
		// remove from AuctionStore
		let auction_item = AuctionStore::<T>::take(&auction_id);
		let who = auction_item.cml_owner;

		// withdraw owner lock fee
		// Self::lock_tea(&who, T::AuctionDeposit::get());
		// <T as auction::Config>::Currency::unreserve(&who, T::AuctionDeposit::get())?;

		// remove from UserAuctionStore
		UserAuctionStore::<T>::mutate(&who, |list| {
			if let Some(index) = list.iter().position(|x| *x == *auction_id) {
				list.remove(index);
			}
		});

		// remove from EndBlockAuctionStore
		let (current_window, next_window) = Self::get_window_block();
		EndBlockAuctionStore::<T>::mutate(current_window, |maybe_list| {
			if let Some(ref mut list) = maybe_list {
				if let Some(index) = list.iter().position(|x| *x == *auction_id) {
					list.remove(index);
				}
			}
		});
		EndBlockAuctionStore::<T>::mutate(next_window, |maybe_list| {
			if let Some(ref mut list) = maybe_list {
				if let Some(index) = list.iter().position(|x| *x == *auction_id) {
					list.remove(index);
				}
			}
		});

		// remove from AuctionBidStore
		if let Some(bid_user_list) = AuctionBidStore::<T>::get(&auction_id) {
			for user in bid_user_list.into_iter() {
				Self::delete_bid(&user, &auction_id);
			}
		}
	}

	pub fn check_delete_bid(who: &T::AccountId, auction_id: &AuctionId) -> DispatchResult {
		ensure!(
			BidStore::<T>::contains_key(who, auction_id),
			Error::<T>::NotFoundBid
		);

		let bid_item = BidStore::<T>::take(&who, &auction_id);
		let total = Self::bid_total_price(&bid_item);
		ensure!(
			T::CurrencyOperations::reserved_balance(who) >= total,
			Error::<T>::NotEnoughReserveBalance
		);
		// todo check user bid store
		// todo check auction bid store
		Ok(())
	}

	pub fn delete_bid(who: &T::AccountId, auction_id: &AuctionId) {
		// remove from BidStore
		let bid_item = BidStore::<T>::take(&who, &auction_id);
		// return lock balance
		let total = Self::bid_total_price(&bid_item);
		// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
		let res = T::CurrencyOperations::unreserve(&who, total);
		if res.is_err() {
			return;
		}

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
	}

	fn bid_total_price(
		bid_item: &BidItem<T::AccountId, BalanceOf<T>, T::BlockNumber>,
	) -> BalanceOf<T> {
		let mut total = bid_item.price;
		if let Some(deposit) = bid_item.deposit {
			total = total.saturating_add(deposit);
		}
		total
	}

	pub fn check_complete_auction(
		auction_item: &AuctionItem<T::AccountId, BalanceOf<T>, T::BlockNumber>,
		target: &T::AccountId,
	) -> DispatchResult {
		T::CmlOperation::check_transfer_cml_to_other(
			&auction_item.cml_owner,
			&auction_item.cml_id,
			&target,
		)?;
		Ok(())
	}

	pub fn complete_auction(
		auction_item: &AuctionItem<T::AccountId, BalanceOf<T>, T::BlockNumber>,
		target: &T::AccountId,
	) {
		let bid_item = BidStore::<T>::get(&target, &auction_item.id);
		let essential_balance = Self::essential_bid_balance(bid_item.price, &auction_item.cml_id);
		if T::CurrencyOperations::unreserve(target, essential_balance).is_err() {
			return;
		}

		if T::CmlOperation::check_transfer_cml_to_other(
			&auction_item.cml_owner,
			&auction_item.cml_id,
			&target,
		)
		.is_err()
		{
			return;
		}

		// transfer price from bid_user to seller.
		if T::CurrencyOperations::transfer(
			&target,
			&auction_item.cml_owner,
			bid_item.price,
			AllowDeath,
		)
		.is_err()
		{
			return;
		}

		T::CmlOperation::transfer_cml_to_other(
			&auction_item.cml_owner,
			&auction_item.cml_id,
			&target,
		);

		Self::delete_auction(&auction_item.id);
		Self::deposit_event(Event::AuctionSuccess(
			auction_item.id,
			target.clone(),
			bid_item.price,
		));
	}

	// when in window block, check each auction could complet or not.
	pub fn check_auction_in_block_window() -> DispatchResult {
		let (current_window, next_window) = Self::get_window_block();
		let current_block = frame_system::Pallet::<T>::block_number();

		ensure!(
			(current_block % T::AuctionDealWindowBLock::get())
				> <T::BlockNumber>::saturated_from(3_u64),
			Error::<T>::NotInWindowBlock
		);

		if let Some(auction_list) = EndBlockAuctionStore::<T>::take(current_window) {
			info!("auction_list => {:?}", auction_list);
			for auction_id in auction_list.iter() {
				let auction_item = AuctionStore::<T>::get(&auction_id);
				Self::check_each_auction_in_block_window(auction_item, next_window)?;
			}
		}

		Ok(())
	}

	fn check_each_auction_in_block_window(
		auction_item: AuctionItem<T::AccountId, BalanceOf<T>, T::BlockNumber>,
		next_window: T::BlockNumber,
	) -> DispatchResult {
		if let Some(ref bid_user) = auction_item.bid_user {
			Self::complete_auction(&auction_item, &bid_user);
		} else {
			// put to next block
			Self::insert_into_end_block_store(next_window, auction_item.id);
		}

		Ok(())
	}

	pub(crate) fn check_bid_for_auction(
		sender: &T::AccountId,
		auction_id: &AuctionId,
		price: BalanceOf<T>,
	) -> DispatchResult {
		ensure!(
			AuctionStore::<T>::contains_key(auction_id),
			Error::<T>::AuctionNotExist
		);
		let auction_item = AuctionStore::<T>::get(auction_id);

		// validate balance
		let essential_balance = Self::essential_bid_balance(price, &auction_item.cml_id);
		ensure!(
			T::CurrencyOperations::free_balance(sender) >= essential_balance,
			Error::<T>::NotEnoughBalance
		);

		let min_price = Self::min_bid_price(&auction_item, sender)?;
		ensure!(min_price <= price, Error::<T>::InvalidBidPrice);

		ensure!(
			&auction_item.cml_owner.cmp(&sender) != &Ordering::Equal,
			Error::<T>::BidSelfBelongs
		);

		Ok(())
	}

	fn essential_bid_balance(price: BalanceOf<T>, cml_id: &CmlId) -> BalanceOf<T> {
		let deposit_price = T::CmlOperation::cml_deposit_price(cml_id);
		match deposit_price {
			Some(p) => price.saturating_add(p),
			None => price,
		}
	}

	pub(crate) fn create_new_bid(
		sender: &T::AccountId,
		auction_id: &AuctionId,
		price: BalanceOf<T>,
	) {
		let auction_item = AuctionStore::<T>::get(auction_id);

		let essential_balance = Self::essential_bid_balance(price, &auction_item.cml_id);
		// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
		if T::CurrencyOperations::reserve(&sender, essential_balance).is_err() {
			return;
		}

		let item = Self::new_bid_item(
			auction_item.id,
			sender.clone(),
			price,
			T::CmlOperation::cml_deposit_price(&auction_item.cml_id),
		);
		BidStore::<T>::insert(sender.clone(), auction_item.id, item);
		AuctionBidStore::<T>::mutate(auction_item.id, |maybe_list| {
			if let Some(ref mut list) = maybe_list {
				list.push(sender.clone());
			} else {
				*maybe_list = Some(vec![sender.clone()]);
			}
		});

		UserBidStore::<T>::mutate(&sender, |maybe_list| {
			if let Some(ref mut list) = maybe_list {
				list.push(auction_item.id);
			} else {
				*maybe_list = Some(vec![auction_item.id]);
			}
		});
	}

	pub(crate) fn increase_bid_price(
		sender: &T::AccountId,
		auction_id: &AuctionId,
		price: BalanceOf<T>,
	) {
		// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
		if T::CurrencyOperations::reserve(&sender, price).is_err() {
			return;
		}

		let current_block = frame_system::Pallet::<T>::block_number();
		BidStore::<T>::mutate(&sender, &auction_id, |item| {
			item.price = item.price.saturating_add(price);
			item.updated_at = current_block;
		});
	}

	pub(crate) fn can_by_now(
		auction_item: &AuctionItem<T::AccountId, BalanceOf<T>, T::BlockNumber>,
		price: BalanceOf<T>,
	) -> bool {
		if let Some(buy_now_price) = auction_item.buy_now_price {
			if price >= buy_now_price {
				return true;
			}
		}
		false
	}

	pub(crate) fn try_complete_auction(sender: &T::AccountId, auction_id: &AuctionId) {
		let auction_item = AuctionStore::<T>::get(auction_id);
		let bid_item = BidStore::<T>::get(&sender, &auction_id);

		if Self::can_by_now(&auction_item, bid_item.price) {
			Self::complete_auction(&auction_item, &sender);
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::{mock::*, AuctionItem, BidItem, BidStore, LastAuctionId};

	#[test]
	fn get_next_auction_id_works() {
		new_test_ext().execute_with(|| {
			// default auction id started at 0
			assert_eq!(LastAuctionId::<Test>::get(), 0);

			assert_eq!(Auction::get_next_auction_id(), 1);

			LastAuctionId::<Test>::set(u64::MAX - 1);
			assert_eq!(Auction::get_next_auction_id(), u64::MAX);

			// auction id back to 0
			assert_eq!(Auction::get_next_auction_id(), 0);
		})
	}

	#[test]
	fn get_window_block_works() {
		new_test_ext().execute_with(|| {
			assert_eq!(frame_system::Pallet::<Test>::block_number(), 0);
			let (current, next) = Auction::get_window_block();
			assert_eq!(current, 0);
			assert_eq!(next, AUCTION_DEAL_WINDOW_BLOCK as u64);

			for i in 1..AUCTION_DEAL_WINDOW_BLOCK {
				frame_system::Pallet::<Test>::set_block_number(i as u64);

				let (current, next) = Auction::get_window_block();
				assert_eq!(current, 0);
				assert_eq!(next, AUCTION_DEAL_WINDOW_BLOCK as u64);
			}

			frame_system::Pallet::<Test>::set_block_number(AUCTION_DEAL_WINDOW_BLOCK as u64);
			let (current, next) = Auction::get_window_block();
			assert_eq!(current, AUCTION_DEAL_WINDOW_BLOCK as u64);
			assert_eq!(next, (AUCTION_DEAL_WINDOW_BLOCK as u64) * 2);
		})
	}

	#[test]
	fn min_bid_price_works() {
		new_test_ext().execute_with(|| {
			let user1 = 1;
			let user2 = 2;
			let starting_price = 100;
			let mut auction_item = AuctionItem::default();
			auction_item.starting_price = starting_price;

			// first bid
			assert!(!BidStore::<Test>::contains_key(user1, auction_item.id)); // user1 not bid before
			assert!(auction_item.bid_user.is_none()); // auction not bid yet
			assert_eq!(
				Auction::min_bid_price(&auction_item, &user1).unwrap(),
				auction_item.starting_price
			);

			let mut bid_item1 = BidItem::default();
			bid_item1.price = auction_item.starting_price;
			BidStore::<Test>::insert(user1, auction_item.id, bid_item1);
			auction_item.bid_user = Some(user1);

			// user2 bid after user1
			assert!(!BidStore::<Test>::contains_key(user2, auction_item.id)); // user2 not bid before
			assert_eq!(
				Auction::min_bid_price(&auction_item, &user2).unwrap(),
				auction_item.starting_price + MIN_PRICE_FOR_BID
			);

			let mut bid_item2 = BidItem::default();
			bid_item2.price = auction_item.starting_price + MIN_PRICE_FOR_BID * 2; // double min price
			BidStore::<Test>::insert(user2, auction_item.id, bid_item2);
			auction_item.bid_user = Some(user2);

			// user1 bid the second time
			assert_eq!(
				Auction::min_bid_price(&auction_item, &user1).unwrap(),
				MIN_PRICE_FOR_BID * 3
			);
			BidStore::<Test>::mutate(user1, auction_item.id, |item| {
				item.price += MIN_PRICE_FOR_BID * 3;
			});
			auction_item.bid_user = Some(user1);

			// user2 bid the second time
			assert_eq!(
				Auction::min_bid_price(&auction_item, &user2).unwrap(),
				MIN_PRICE_FOR_BID * 2
			);
		})
	}
}
