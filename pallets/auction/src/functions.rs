use super::*;

impl<T: auction::Config> auction::Pallet<T> {
	pub fn get_next_auction_id() -> T::AuctionId {
		let cid = LastAuctionId::<T>::get();
		let _id = cid.clone();
		LastAuctionId::<T>::mutate(|_id| *_id += One::one());

		cid
	}

	pub(super) fn new_auction_item(
		cml_id: CmlId,
		cml_owner: T::AccountId,
		starting_price: BalanceOf<T>,
		buy_now_price: Option<BalanceOf<T>>,
	) -> AuctionItem<T::AuctionId, T::AccountId, BalanceOf<T>, T::BlockNumber> {
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
		bid_user: &T::AccountId,
	) {
		AuctionStore::<T>::mutate(&auction_id, |item| {
			item.bid_user = Some(bid_user.clone());
		});
	}

	pub(super) fn new_bid_item(
		auction_id: T::AuctionId,
		who: T::AccountId,
		price: BalanceOf<T>,
		deposit: Option<BalanceOf<T>>,
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
	pub fn get_window_block() -> (T::BlockNumber, T::BlockNumber) {
		let current_block = frame_system::Pallet::<T>::block_number();
		let current_window = current_block / T::AuctionDealWindowBLock::get();
		let next_window = (current_window + <T::BlockNumber>::saturated_from(1_u64))
			* T::AuctionDealWindowBLock::get();

		let current_window = current_window * T::AuctionDealWindowBLock::get();
		(current_window, next_window)
	}

	pub fn add_auction_to_storage(
		auction_item: AuctionItem<T::AuctionId, T::AccountId, BalanceOf<T>, T::BlockNumber>,
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

	fn insert_into_end_block_store(window_height: T::BlockNumber, auction_id: T::AuctionId) {
		EndBlockAuctionStore::<T>::mutate(window_height, |maybe_list| {
			if let Some(ref mut list) = maybe_list {
				list.push(auction_id);
			} else {
				*maybe_list = Some(vec![auction_id]);
			}
		});
	}

	pub(super) fn get_min_bid_price(
		auction_item: &AuctionItem<T::AuctionId, T::AccountId, BalanceOf<T>, T::BlockNumber>,
		who: &T::AccountId,
	) -> Result<BalanceOf<T>, Error<T>> {
		let min_price = match BidStore::<T>::contains_key(who, auction_item.id) {
			true => BidStore::<T>::get(who, auction_item.id).price,
			false => <BalanceOf<T>>::saturated_from(0_u128),
		};

		let max_price = {
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

		let rs = max_price
			.saturating_sub(min_price)
			.saturating_add(T::MinPriceForBid::get());

		Ok(rs)
	}

	pub fn delete_auction(auction_id: &T::AuctionId) {
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

	pub fn check_delete_bid(who: &T::AccountId, auction_id: &T::AuctionId) -> DispatchResult {
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

	pub fn delete_bid(who: &T::AccountId, auction_id: &T::AuctionId) {
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
		bid_item: &BidItem<T::AuctionId, T::AccountId, BalanceOf<T>, T::BlockNumber>,
	) -> BalanceOf<T> {
		let mut total = bid_item.price;
		if let Some(deposit) = bid_item.deposit {
			total = total.saturating_add(deposit);
		}
		total
	}

	pub fn complete_auction(
		auction_item: &AuctionItem<T::AuctionId, T::AccountId, BalanceOf<T>, T::BlockNumber>,
		target: &T::AccountId,
	) {
		if !BidStore::<T>::contains_key(&target, &auction_item.id) {
			return;
		}
		if T::CmlOperation::transfer_cml_other(
			&auction_item.cml_owner,
			&auction_item.cml_id,
			&target,
		)
		.is_err()
		{
			return;
		}

		let bid_item = BidStore::<T>::get(&target, &auction_item.id);
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
		auction_item: AuctionItem<T::AuctionId, T::AccountId, BalanceOf<T>, T::BlockNumber>,
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
		auction_id: &T::AuctionId,
		price: BalanceOf<T>,
	) -> DispatchResult {
		ensure!(
			AuctionStore::<T>::contains_key(auction_id),
			Error::<T>::AuctionNotExist
		);
		let auction_item = AuctionStore::<T>::get(auction_id);

		// validate balance
		let essential_balance = Self::get_essential_bid_balance(price, &auction_item.cml_id);
		ensure!(
			T::CurrencyOperations::free_balance(sender) >= essential_balance,
			Error::<T>::NotEnoughBalance
		);

		let min_price = Self::get_min_bid_price(&auction_item, sender)?;
		ensure!(min_price <= price, Error::<T>::InvalidBidPrice);

		ensure!(
			&auction_item.cml_owner.cmp(&sender) != &Ordering::Equal,
			Error::<T>::BidSelfBelongs
		);

		Ok(())
	}

	fn get_essential_bid_balance(price: BalanceOf<T>, cml_id: &CmlId) -> BalanceOf<T> {
		let deposit_price = T::CmlOperation::get_cml_deposit_price(cml_id);
		match deposit_price {
			Some(p) => price.saturating_add(p),
			None => price,
		}
	}

	pub(crate) fn create_new_bid(
		sender: &T::AccountId,
		auction_id: &T::AuctionId,
		price: BalanceOf<T>,
	) {
		let auction_item = AuctionStore::<T>::get(auction_id);

		let essential_balance = Self::get_essential_bid_balance(price, &auction_item.cml_id);
		// SetFn error handling see https://github.com/tearust/tea-camellia/issues/13
		if T::CurrencyOperations::reserve(&sender, essential_balance).is_err() {
			return;
		}

		let item = Self::new_bid_item(
			auction_item.id,
			sender.clone(),
			price,
			T::CmlOperation::get_cml_deposit_price(&auction_item.cml_id),
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
		auction_id: &T::AuctionId,
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

	pub(crate) fn try_complete_auction(sender: &T::AccountId, auction_id: &T::AuctionId) {
		let auction_item = AuctionStore::<T>::get(auction_id);
		if let Some(buy_now_price) = auction_item.buy_now_price {
			let bid_item = BidStore::<T>::get(&sender, &auction_id);
			if bid_item.price >= buy_now_price {
				Self::complete_auction(&auction_item, &sender);
			}
		}
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
