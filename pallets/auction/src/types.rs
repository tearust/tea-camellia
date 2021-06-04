// use codec::FullCodec;
use codec::{Decode, Encode};
use sp_runtime::{
	RuntimeDebug,
};
use sp_std::prelude::*;

use sp_std::{
	cmp::{Eq, PartialEq},
	// fmt::Debug,
	// result,
};

// use super::auction;

// #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
// pub enum Change<Value> {
// 	/// No change.
// 	NoChange,
// 	/// Changed to new value.
// 	NewValue(Value),
// }

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct AuctionItem<AuctionId, AccountId, CmlId, Balance, BlockNumber> {
  pub id: AuctionId,
  pub cml_id: CmlId,
  pub cml_owner: AccountId,
  pub starting_price: Balance,
  pub buy_now_price: Option<Balance>,
  pub start_at: BlockNumber,
  pub end_at: BlockNumber,

	pub status: Vec<u8>,
	
	pub bid_user: Option<AccountId>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct BidItem<AuctionId, AccountId, Balance, BlockNumber> {
  pub auction_id: AuctionId,
	pub user: AccountId,
	pub price: Balance,
	pub deposit: Option<Balance>,
	
	pub created_at: BlockNumber,
	pub updated_at: BlockNumber,
}



