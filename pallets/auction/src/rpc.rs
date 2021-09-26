use super::*;

impl<T: auction::Config> auction::Pallet<T> {
	pub fn user_auction_list(who: &T::AccountId) -> Vec<u64> {
		AuctionStore::<T>::iter()
			.filter(|(_, item)| item.cml_owner == *who)
			.map(|(id, _)| id)
			.collect()
	}

	pub fn user_bid_list(who: &T::AccountId) -> Vec<u64> {
		BidStore::<T>::iter_prefix(who)
			.map(|(auction_id, _)| auction_id)
			.collect()
	}

	pub fn current_auction_list() -> Vec<u64> {
		AuctionStore::<T>::iter().map(|(id, _)| id).collect()
	}

	/// return values:
	/// 1. minimum bid price
	/// 2. original bid price, if not bid before return `None`
	/// 3. indicates if the cml is mining
	pub fn estimate_minimum_bid_price(
		auction_id: AuctionId,
		who: &T::AccountId,
	) -> (BalanceOf<T>, Option<BalanceOf<T>>, bool) {
		if !AuctionStore::<T>::contains_key(auction_id) {
			return (Default::default(), None, false);
		}

		let current_bid_price = match BidStore::<T>::contains_key(who, auction_id) {
			true => Some(BidStore::<T>::get(who, auction_id).price),
			false => None,
		};

		let min_bid_price = Self::min_bid_price(&AuctionStore::<T>::get(auction_id), &who);
		let (estimate_minimum_bid_price, is_mining) = match min_bid_price {
			Ok(min_bid_price) => {
				let auction_item = AuctionStore::<T>::get(auction_id);
				Self::essential_bid_balance(min_bid_price, &auction_item.cml_id)
			}
			_ => (Default::default(), false),
		};
		(estimate_minimum_bid_price, current_bid_price, is_mining)
	}

	pub fn penalty_amount(auction_id: AuctionId, who: &T::AccountId) -> BalanceOf<T> {
		let bid_item = BidStore::<T>::get(who, auction_id);
		Self::calculate_penalty_amount(&bid_item)
	}
}

#[cfg(test)]
mod tests {
	use crate::mock::*;
	use crate::{AuctionId, AuctionItem, AuctionStore, BidItem, BidStore};

	#[test]
	fn user_auction_list_works() {
		new_test_ext().execute_with(|| {
			let account1 = 1;
			let account2 = 2;

			let account1_auction1 = 11;
			let account1_auction2 = 12;
			let account1_auction3 = 13;

			let account2_auction1 = 21;
			let account2_auction2 = 22;
			let account2_auction3 = 23;
			let account2_auction4 = 24;

			AuctionStore::<Test>::insert(
				account1_auction1,
				new_auction_item(account1_auction1, account1),
			);
			AuctionStore::<Test>::insert(
				account1_auction2,
				new_auction_item(account1_auction2, account1),
			);
			AuctionStore::<Test>::insert(
				account1_auction3,
				new_auction_item(account1_auction3, account1),
			);

			AuctionStore::<Test>::insert(
				account2_auction1,
				new_auction_item(account2_auction1, account2),
			);
			AuctionStore::<Test>::insert(
				account2_auction2,
				new_auction_item(account2_auction2, account2),
			);
			AuctionStore::<Test>::insert(
				account2_auction3,
				new_auction_item(account2_auction3, account2),
			);
			AuctionStore::<Test>::insert(
				account2_auction4,
				new_auction_item(account2_auction4, account2),
			);

			let auction_list = Auction::user_auction_list(&account1);
			assert_eq!(auction_list.len(), 3);
			assert!(auction_list.contains(&account1_auction1));
			assert!(auction_list.contains(&account1_auction2));
			assert!(auction_list.contains(&account1_auction3));

			let auction_list = Auction::user_auction_list(&account2);
			assert_eq!(auction_list.len(), 4);
			assert!(auction_list.contains(&account2_auction1));
			assert!(auction_list.contains(&account2_auction2));
			assert!(auction_list.contains(&account2_auction3));
			assert!(auction_list.contains(&account2_auction4));
		})
	}

	#[test]
	fn user_bid_list_works() {
		new_test_ext().execute_with(|| {
			let account1 = 1;
			let account2 = 2;

			let account1_auction1 = 11;
			let account1_auction2 = 12;
			let account1_auction3 = 13;

			let account2_auction1 = 21;
			let account2_auction2 = 22;
			let account2_auction3 = 23;
			let account2_auction4 = 24;

			BidStore::<Test>::insert(account1, account1_auction1, BidItem::default());
			BidStore::<Test>::insert(account1, account1_auction2, BidItem::default());
			BidStore::<Test>::insert(account1, account1_auction3, BidItem::default());

			BidStore::<Test>::insert(account2, account2_auction1, BidItem::default());
			BidStore::<Test>::insert(account2, account2_auction2, BidItem::default());
			BidStore::<Test>::insert(account2, account2_auction3, BidItem::default());
			BidStore::<Test>::insert(account2, account2_auction4, BidItem::default());

			let auction_list = Auction::user_bid_list(&account1);
			assert_eq!(auction_list.len(), 3);
			assert!(auction_list.contains(&account1_auction1));
			assert!(auction_list.contains(&account1_auction2));
			assert!(auction_list.contains(&account1_auction3));

			let auction_list = Auction::user_bid_list(&account2);
			assert_eq!(auction_list.len(), 4);
			assert!(auction_list.contains(&account2_auction1));
			assert!(auction_list.contains(&account2_auction2));
			assert!(auction_list.contains(&account2_auction3));
			assert!(auction_list.contains(&account2_auction4));
		})
	}

	fn new_auction_item(auction_id: AuctionId, account_id: u64) -> AuctionItem<u64, u128, u64> {
		let mut auction_item = AuctionItem::default();
		auction_item.id = auction_id;
		auction_item.cml_owner = account_id;
		auction_item
	}
}
