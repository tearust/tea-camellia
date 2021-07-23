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
