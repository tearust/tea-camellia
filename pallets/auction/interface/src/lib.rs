#![cfg_attr(not(feature = "std"), no_std)]

pub trait AuctionOperation {
	type AccountId: Default;
	type Balance: Default;
	type BlockNumber: Default;

	fn is_cml_in_auction(cml_id: u64) -> bool;

	fn create_new_bid(sender: &Self::AccountId, auction_id: &u64, price: Self::Balance);

	fn update_current_winner(auction_id: &u64, bid_user: &Self::AccountId);

	// return current block window number and next.
	fn get_window_block() -> (Self::BlockNumber, Self::BlockNumber);
}
