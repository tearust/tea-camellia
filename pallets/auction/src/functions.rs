use super::*;

impl<T: auction::Config> AuctionOperation for auction::Pallet<T> {
	type AccountId = T::AccountId;
	type Balance = BalanceOf<T>;
	type BlockNumber = T::BlockNumber;

	fn is_cml_in_auction(cml_id: u64) -> bool {
		for (_, item) in AuctionStore::<T>::iter() {
			if item.cml_id == cml_id {
				return true;
			}
		}
		false
	}

	fn create_new_bid(sender: &Self::AccountId, auction_id: &AuctionId, price: Self::Balance) {
		let auction_item = AuctionStore::<T>::get(auction_id);

		let (essential_balance, _) = Self::essential_bid_balance(price, &auction_item.cml_id);
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
	}

	fn update_current_winner(auction_id: &AuctionId, bid_user: &Self::AccountId) {
		AuctionStore::<T>::mutate(&auction_id, |item| {
			item.bid_user = Some(bid_user.clone());
		});
	}

	// return current block window number and next.
	fn get_window_block() -> (Self::BlockNumber, Self::BlockNumber) {
		let current_block = frame_system::Pallet::<T>::block_number();
		let current_index = current_block / T::AuctionDealWindowBLock::get();
		let next_index = current_index + <T::BlockNumber>::saturated_from(1_u64);

		(
			current_index * T::AuctionDealWindowBLock::get(),
			next_index * T::AuctionDealWindowBLock::get(),
		)
	}
}

impl<T: auction::Config> auction::Pallet<T> {
	pub(crate) fn add_auction_to_storage(
		auction_item: AuctionItem<T::AccountId, BalanceOf<T>, T::BlockNumber>,
	) {
		let (_, next_window) = Self::get_window_block();
		Self::insert_into_end_block_store(next_window, auction_item.id);

		AuctionStore::<T>::insert(auction_item.id, auction_item);
	}

	pub fn next_auction_id() -> AuctionId {
		LastAuctionId::<T>::mutate(|id| {
			if *id < u64::MAX {
				*id += 1;
			} else {
				*id = 1;
			}

			*id
		})
	}

	pub(super) fn new_auction_item(
		cml_id: CmlId,
		cml_owner: T::AccountId,
		starting_price: BalanceOf<T>,
		buy_now_price: Option<BalanceOf<T>>,
		auto_renew: bool,
	) -> AuctionItem<T::AccountId, BalanceOf<T>, T::BlockNumber> {
		AuctionItem {
			id: Self::next_auction_id(),
			cml_id,
			cml_owner,
			starting_price,
			buy_now_price,
			auto_renew,
			start_at: frame_system::Pallet::<T>::block_number(),
			status: AuctionStatus::Normal,
			bid_user: None,
		}
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

	pub fn clear_auction_pledge(sender: &T::AccountId, auction_id: &AuctionId) -> bool {
		let pledge_amount = T::AuctionPledgeAmount::get();
		let unreserved_amount =
			pledge_amount - T::CurrencyOperations::unreserve(sender, pledge_amount);

		if let Some(list) = AuctionBidStore::<T>::get(&auction_id) {
			match (unreserved_amount / T::AuctionOwnerPenaltyForEachBid::get()).try_into() {
				Ok(penalty_user_count) => {
					list.iter().take(penalty_user_count).for_each(|acc| {
						if let Err(e) = T::CurrencyOperations::transfer(
							sender,
							&acc,
							T::AuctionOwnerPenaltyForEachBid::get(),
							AllowDeath,
						) {
							// should never happen, record here just in case
							warn!("transfer from {:?} to {:?} failed: {:?}", sender, &acc, e);
						}
					});
					if !list.is_empty() {
						return true;
					}
				}
				Err(_) => warn!("calculate auction penalty count failed"),
			}
		}
		false
	}

	pub fn delete_auction(auction_id: &AuctionId, success_user: Option<&T::AccountId>) {
		// remove from AuctionStore
		AuctionStore::<T>::remove(&auction_id);

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
				let bid_item = Self::delete_bid(&user, auction_id);

				if success_user.is_some() && *success_user.unwrap() == user {
					continue;
				}
				Self::return_for_bid(&user, &bid_item);
			}
		}
	}

	pub fn check_delete_bid(who: &T::AccountId, auction_id: &AuctionId) -> DispatchResult {
		ensure!(
			AuctionStore::<T>::contains_key(&auction_id),
			Error::<T>::AuctionNotExist
		);

		ensure!(
			BidStore::<T>::contains_key(who, auction_id),
			Error::<T>::NotFoundBid
		);

		let bid_item = BidStore::<T>::get(&who, &auction_id);
		let total = Self::bid_total_price(&bid_item);
		ensure!(
			T::CurrencyOperations::reserved_balance(who) >= total,
			Error::<T>::NotEnoughReserveBalance
		);
		Ok(())
	}

	pub fn return_for_bid(
		who: &T::AccountId,
		bid_item: &BidItem<T::AccountId, BalanceOf<T>, T::BlockNumber>,
	) {
		let total = Self::bid_total_price(bid_item);
		T::CurrencyOperations::unreserve(who, total);
	}

	pub fn return_for_bid_with_penalty(
		who: &T::AccountId,
		bid_item: &BidItem<T::AccountId, BalanceOf<T>, T::BlockNumber>,
	) {
		let total = Self::bid_total_price(bid_item);
		let penalty_amount = Self::calculate_penalty_amount(bid_item);

		if !penalty_amount.is_zero() {
			T::CurrencyOperations::slash_reserved(who, penalty_amount);
		}
		T::CurrencyOperations::unreserve(who, total.saturating_sub(penalty_amount));
	}

	pub(crate) fn calculate_penalty_amount(
		bid_item: &BidItem<T::AccountId, BalanceOf<T>, T::BlockNumber>,
	) -> BalanceOf<T> {
		let (highest_account, highest_price) = Self::highest_bid(&bid_item.auction_id);
		let penalty_amount = match highest_price > bid_item.price {
			true => Zero::zero(),
			false => {
				if !highest_account.eq(&Default::default()) {
					AuctionStore::<T>::mutate(&bid_item.auction_id, |auction_item| {
						auction_item.bid_user = Some(highest_account)
					});
					bid_item.price.saturating_sub(highest_price)
				} else {
					let starting_price =
						AuctionStore::<T>::mutate(&bid_item.auction_id, |auction_item| {
							auction_item.bid_user = None;
							auction_item.starting_price
						});
					bid_item.price.saturating_sub(starting_price)
				}
			}
		};

		penalty_amount
	}

	pub fn delete_bid(
		who: &T::AccountId,
		auction_id: &AuctionId,
	) -> BidItem<T::AccountId, BalanceOf<T>, T::BlockNumber> {
		// remove from AuctionBidStore
		AuctionBidStore::<T>::mutate(&auction_id, |maybe_list| {
			if let Some(ref mut list) = maybe_list {
				if let Some(index) = list.iter().position(|x| &x.cmp(&who) == &Ordering::Equal) {
					list.remove(index);
				}
			}
		});

		BidStore::<T>::take(&who, &auction_id)
	}

	/// this function assume the given user is the highest user in the `AuctionBidStore`,
	/// so it is not safe to call it without judging `auction_id` and `highest_user`
	pub(crate) fn highest_bid(auction_id: &AuctionId) -> (T::AccountId, BalanceOf<T>) {
		let account_ids = AuctionBidStore::<T>::get(auction_id).unwrap();

		let mut result_account: T::AccountId = Default::default();
		let mut bid_price: BalanceOf<T> = Zero::zero();
		account_ids.iter().for_each(|acc| {
			let bid_item = BidStore::<T>::get(acc, auction_id);
			if bid_item.price > bid_price {
				bid_price = bid_item.price;
				result_account = acc.clone();
			}
		});

		(result_account, bid_price)
	}

	pub(crate) fn bid_total_price(
		bid_item: &BidItem<T::AccountId, BalanceOf<T>, T::BlockNumber>,
	) -> BalanceOf<T> {
		let mut total = bid_item.price;
		if let Some(deposit) = bid_item.deposit {
			total = total.saturating_add(deposit);
		}
		total
	}

	pub fn complete_auction(
		auction_item: &AuctionItem<T::AccountId, BalanceOf<T>, T::BlockNumber>,
		target: &T::AccountId,
	) {
		let bid_item = BidStore::<T>::get(&target, &auction_item.id);
		let (essential_balance, _) =
			Self::essential_bid_balance(bid_item.price, &auction_item.cml_id);
		T::CurrencyOperations::unreserve(target, essential_balance);

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

		T::CurrencyOperations::unreserve(&auction_item.cml_owner, T::AuctionPledgeAmount::get());
		Self::delete_auction(&auction_item.id, Some(target));
		Self::deposit_event(Event::AuctionSuccess(
			auction_item.id,
			target.clone(),
			bid_item.price,
		));
	}

	pub fn is_end_of_auction_window(current_height: &T::BlockNumber) -> bool {
		// offset with 3 to void overlapping with staking period
		*current_height % T::AuctionDealWindowBLock::get() == 3u32.into()
	}

	pub fn try_complete_auctions() {
		let (current_window, next_window) = Self::get_window_block();

		if let Some(auction_list) = EndBlockAuctionStore::<T>::take(current_window) {
			for auction_id in auction_list.iter() {
				let auction_item = AuctionStore::<T>::get(&auction_id);
				if let Some(bid_user) = auction_item.bid_user.as_ref() {
					Self::complete_auction(&auction_item, bid_user);
				} else {
					Self::deal_with_not_complete_auction(next_window, &auction_item);
				}
			}
		}
	}

	pub(crate) fn deal_with_not_complete_auction(
		next_window: T::BlockNumber,
		auction_item: &AuctionItem<T::AccountId, BalanceOf<T>, T::BlockNumber>,
	) {
		let current_height = frame_system::Pallet::<T>::block_number();
		let cml = T::CmlOperation::cml_by_id(&auction_item.cml_id);

		if (cml.is_ok() && !cml.unwrap().should_dead(&current_height))
			&& auction_item.auto_renew
			&& T::CurrencyOperations::free_balance(&auction_item.cml_owner)
				>= T::AuctionFeePerWindow::get()
		{
			let _ = T::CurrencyOperations::slash(
				&auction_item.cml_owner,
				T::AuctionFeePerWindow::get(),
			);
			Self::insert_into_end_block_store(next_window, auction_item.id);
		} else {
			T::CurrencyOperations::unreserve(
				&auction_item.cml_owner,
				T::AuctionPledgeAmount::get(),
			);
			Self::delete_auction(&auction_item.id, None);
		}
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
		let (essential_balance, deposit_staking) =
			Self::essential_bid_balance(price, &auction_item.cml_id);
		ensure!(
			T::CurrencyOperations::free_balance(sender) >= essential_balance,
			match deposit_staking {
				true => Error::<T>::NotEnoughBalanceForBidAndFirstStakingSlot,
				false => Error::<T>::NotEnoughBalanceForBid,
			}
		);

		let min_price = Self::min_bid_price(&auction_item, sender)?;
		ensure!(min_price <= price, Error::<T>::InvalidBidPrice);

		ensure!(
			&auction_item.cml_owner.cmp(&sender) != &Ordering::Equal,
			Error::<T>::BidSelfBelongs
		);
		ensure!(
			AuctionBidStore::<T>::get(auction_id)
				.unwrap_or_default()
				.len() < T::MaxUsersPerAuction::get() as usize,
			Error::<T>::OverTheMaxUsersPerAuctionLimit
		);

		Ok(())
	}

	pub(crate) fn essential_bid_balance(
		price: BalanceOf<T>,
		cml_id: &CmlId,
	) -> (BalanceOf<T>, bool) {
		let deposit_price = T::CmlOperation::cml_deposit_price(cml_id);
		match deposit_price {
			Some(p) => (price.saturating_add(p), true),
			None => (price, false),
		}
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

	pub(crate) fn try_complete_auction(sender: &T::AccountId, auction_id: &AuctionId) -> bool {
		let auction_item = AuctionStore::<T>::get(auction_id);
		let bid_item = BidStore::<T>::get(&sender, &auction_id);

		if Self::can_by_now(&auction_item, bid_item.price) {
			Self::complete_auction(&auction_item, &sender);
			return true;
		}
		false
	}

	pub(crate) fn auction_store_contains(cml_id: CmlId) -> bool {
		for (_, bid_item) in AuctionStore::<T>::iter() {
			if bid_item.cml_id == cml_id {
				return true;
			}
		}
		false
	}
}

#[cfg(test)]
mod tests {
	use crate::tests::seed_from_lifespan;
	use crate::{
		mock::*, AuctionBidStore, AuctionId, AuctionItem, AuctionOperation, BidItem, BidStore,
		Config, EndBlockAuctionStore, LastAuctionId,
	};
	use frame_support::{
		assert_ok,
		traits::{Currency, ReservableCurrency},
	};
	use pallet_cml::{CmlId, CmlStore, UserCmlStore, CML};

	#[test]
	fn next_auction_id_works() {
		new_test_ext().execute_with(|| {
			// default auction id started at 0
			assert_eq!(LastAuctionId::<Test>::get(), 0);

			assert_eq!(Auction::next_auction_id(), 1);

			LastAuctionId::<Test>::set(u64::MAX - 1);
			assert_eq!(Auction::next_auction_id(), u64::MAX);

			// auction id back to 0
			assert_eq!(Auction::next_auction_id(), 1);
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

	#[test]
	fn clear_auction_pledge_works() {
		new_test_ext().execute_with(|| {
			let owner = 33;
			<Test as Config>::Currency::make_free_balance_be(&owner, AUCTION_PLEDGE_AMOUNT);
			<Test as Config>::Currency::reserve(&owner, AUCTION_PLEDGE_AMOUNT).unwrap();

			let auction_id = 11;
			let user_count = 5;
			let mut user_ids = Vec::new();
			for i in 0..user_count {
				user_ids.push(i);
			}
			AuctionBidStore::<Test>::insert(auction_id, user_ids);

			Auction::clear_auction_pledge(&owner, &auction_id);

			assert_eq!(
				<Test as Config>::Currency::free_balance(&owner),
				AUCTION_PLEDGE_AMOUNT - AUCTION_OWNER_PENALTY_FOR_EACH_BID * (user_count as u128)
			);
			for i in 0..user_count {
				assert_eq!(
					<Test as Config>::Currency::free_balance(&i),
					AUCTION_OWNER_PENALTY_FOR_EACH_BID
				);
			}
		})
	}

	#[test]
	fn clear_auction_pledge_works_if_auction_user_count_is_large() {
		new_test_ext().execute_with(|| {
			let owner = 33333;
			<Test as Config>::Currency::make_free_balance_be(&owner, AUCTION_PLEDGE_AMOUNT);
			<Test as Config>::Currency::reserve(&owner, AUCTION_PLEDGE_AMOUNT).unwrap();

			let auction_id = 11;
			let user_count = 1000; // user count is too large to get penalty for each of them
			let mut user_ids = Vec::new();
			for i in 0..user_count {
				user_ids.push(i);
			}
			AuctionBidStore::<Test>::insert(auction_id, user_ids);

			Auction::clear_auction_pledge(&owner, &auction_id);

			assert_eq!(<Test as Config>::Currency::free_balance(&owner), 0);
			let penalty_count = (AUCTION_PLEDGE_AMOUNT / AUCTION_OWNER_PENALTY_FOR_EACH_BID) as u64;
			for i in 0..penalty_count {
				assert_eq!(
					<Test as Config>::Currency::free_balance(&i),
					AUCTION_OWNER_PENALTY_FOR_EACH_BID
				);
			}
			for i in penalty_count..1000 {
				assert_eq!(<Test as Config>::Currency::free_balance(&i), 0);
			}
		})
	}

	#[test]
	fn clear_auction_pledge_works_if_owner_account_be_slashed() {
		new_test_ext().execute_with(|| {
			let owner = 33333;
			let rest_amount = 50;
			<Test as Config>::Currency::make_free_balance_be(
				&owner,
				AUCTION_PLEDGE_AMOUNT + rest_amount,
			);
			<Test as Config>::Currency::reserve(&owner, AUCTION_PLEDGE_AMOUNT).unwrap();

			let auction_id = 11;
			let user_count = 100;
			let mut user_ids = Vec::new();
			for i in 0..user_count {
				user_ids.push(i);
			}
			AuctionBidStore::<Test>::insert(auction_id, user_ids);

			let slash_amount = 30;
			let _ = <Test as Config>::Currency::slash_reserved(&owner, slash_amount); // owner is slashed for some reason
			assert_eq!(
				<Test as Config>::Currency::free_balance(&owner),
				rest_amount
			);
			assert_eq!(
				<Test as Config>::Currency::reserved_balance(&owner),
				AUCTION_PLEDGE_AMOUNT - slash_amount
			);

			Auction::clear_auction_pledge(&owner, &auction_id);

			// penalty should not effect free balance
			assert_eq!(
				<Test as Config>::Currency::free_balance(&owner),
				rest_amount
			);

			for i in 0..(AUCTION_PLEDGE_AMOUNT - slash_amount) {
				assert_eq!(
					<Test as Config>::Currency::free_balance(&(i as u64)),
					AUCTION_OWNER_PENALTY_FOR_EACH_BID
				);
			}
			for i in (AUCTION_PLEDGE_AMOUNT - slash_amount)..100 {
				assert_eq!(<Test as Config>::Currency::free_balance(&(i as u64)), 0);
			}
		})
	}

	#[test]
	fn try_complete_auctions_works() {
		new_test_ext().execute_with(|| {
			let owner1 = 11;
			let owner2 = 12;
			let owner3 = 13;
			let owner4 = 14;
			let owner5 = 15;
			let user1 = 21;
			let user2 = 22;
			let user3 = 23;
			let cml_id1 = 31;
			let cml_id2 = 32;
			let cml_id3 = 33;
			let cml_id4 = 34;
			let cml_id5 = 35;
			let origin_amount = 10000;
			<Test as Config>::Currency::make_free_balance_be(&owner1, origin_amount);
			<Test as Config>::Currency::make_free_balance_be(&owner2, origin_amount);
			<Test as Config>::Currency::make_free_balance_be(&owner3, origin_amount);
			<Test as Config>::Currency::make_free_balance_be(
				&owner4,
				AUCTION_PLEDGE_AMOUNT + AUCTION_FEE_PER_WINDOW,
			);
			<Test as Config>::Currency::make_free_balance_be(&owner5, origin_amount);
			<Test as Config>::Currency::make_free_balance_be(&user1, origin_amount);
			<Test as Config>::Currency::make_free_balance_be(&user2, origin_amount);
			<Test as Config>::Currency::make_free_balance_be(&user3, origin_amount);

			// - auction_id1 bid with two users
			// - auction_id2 bid with buy_now_price
			// - auction_id3 have no bids that should move to the next window,
			// - user4 have not enough free balance so auction_id4 will be removed
			// - user5 set auto_renew with false so auction_id5 will be removed
			let auction_id1 = new_bid_item(owner1, cml_id1, 100, 1000, true);
			let auction_id2 = new_bid_item(owner2, cml_id2, 100, 1000, true);
			let auction_id3 = new_bid_item(owner3, cml_id3, 100, 1000, true);
			let _auction_id4 = new_bid_item(owner4, cml_id4, 100, 1000, true);
			let _auction_id5 = new_bid_item(owner5, cml_id5, 100, 1000, false);
			let (current, next) = Auction::get_window_block();
			assert_eq!(EndBlockAuctionStore::<Test>::get(current), None);
			assert_eq!(EndBlockAuctionStore::<Test>::get(next).unwrap().len(), 5);

			let user1_bid_price = 150;
			assert_ok!(Auction::bid_for_auction(
				Origin::signed(user1),
				auction_id1,
				user1_bid_price,
			));
			let user2_bid_price = 200;
			assert_ok!(Auction::bid_for_auction(
				Origin::signed(user2),
				auction_id1,
				user2_bid_price,
			));
			let user3_bid_price = 1000;
			assert_ok!(Auction::bid_for_auction(
				Origin::signed(user3),
				auction_id2,
				user3_bid_price,
			));

			// check balances of all users
			assert_eq!(
				<Test as Config>::Currency::free_balance(&owner1),
				origin_amount - AUCTION_PLEDGE_AMOUNT - AUCTION_FEE_PER_WINDOW
			);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&owner2),
				origin_amount + user3_bid_price - AUCTION_FEE_PER_WINDOW
			);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&owner3),
				origin_amount - AUCTION_PLEDGE_AMOUNT - AUCTION_FEE_PER_WINDOW
			);
			assert_eq!(<Test as Config>::Currency::free_balance(&owner4), 0);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&owner5),
				origin_amount - AUCTION_PLEDGE_AMOUNT - AUCTION_FEE_PER_WINDOW
			);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&user1),
				origin_amount - user1_bid_price
			);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&user2),
				origin_amount - user2_bid_price
			);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&user3),
				origin_amount - user3_bid_price
			);

			frame_system::Pallet::<Test>::set_block_number(AUCTION_DEAL_WINDOW_BLOCK as u64);
			let (current, next) = Auction::get_window_block();
			assert_eq!(EndBlockAuctionStore::<Test>::get(current).unwrap().len(), 4);
			assert_eq!(EndBlockAuctionStore::<Test>::get(next), None);
			Auction::try_complete_auctions();

			assert_eq!(EndBlockAuctionStore::<Test>::get(current), None);
			assert_eq!(EndBlockAuctionStore::<Test>::get(next).unwrap().len(), 1);
			assert_eq!(
				EndBlockAuctionStore::<Test>::get(next).unwrap()[0],
				auction_id3
			);

			// check balances of all users
			assert_eq!(
				<Test as Config>::Currency::free_balance(&owner1),
				origin_amount + user2_bid_price - AUCTION_FEE_PER_WINDOW
			);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&owner2),
				origin_amount + user3_bid_price - AUCTION_FEE_PER_WINDOW
			);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&owner3),
				origin_amount - AUCTION_PLEDGE_AMOUNT - AUCTION_FEE_PER_WINDOW * 2
			);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&owner4),
				AUCTION_PLEDGE_AMOUNT
			);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&owner5),
				origin_amount - AUCTION_FEE_PER_WINDOW
			);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&user1),
				origin_amount
			);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&user2),
				origin_amount - user2_bid_price
			);
			assert_eq!(
				<Test as Config>::Currency::free_balance(&user3),
				origin_amount - user3_bid_price
			);
		})
	}

	fn new_bid_item(
		user_id: u64,
		cml_id: CmlId,
		starting_price: u128,
		buy_now_price: u128,
		auto_renew: bool,
	) -> AuctionId {
		UserCmlStore::<Test>::insert(user_id, cml_id, ());
		let cml = CML::from_genesis_seed(seed_from_lifespan(cml_id, 100));
		CmlStore::<Test>::insert(cml_id, cml);

		assert_ok!(Auction::put_to_store(
			Origin::signed(user_id),
			cml_id,
			starting_price,
			Some(buy_now_price),
			auto_renew,
		));

		Auction::last_auction_id()
	}
}
