use codec::FullCodec;
use codec::{Decode, Encode};
use sp_runtime::{
	traits::{AtLeast32Bit, Bounded, MaybeSerializeDeserialize},
	DispatchError, DispatchResult, RuntimeDebug,
};
use sp_std::prelude::*;
use sp_std::{
	cmp::{Eq, PartialEq},
	fmt::Debug,
	result,
};

// use super::auction;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum Change<Value> {
	/// No change.
	NoChange,
	/// Changed to new value.
	NewValue(Value),
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct AuctionItem<AuctionId, AccountId, AssetId, Balance, BlockNumber> {
  id: AuctionId,
  cml_id: AssetId,
  cml_owner: AccountId,
  starting_price: Balance,
  buy_now_price: Balance,
  start_at: BlockNumber,
  end_at: BlockNumber,

  status: Vec<u8>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct BidItem<AuctionId, AccountId, Balance, BlockNumber> {
  auction_id: AuctionId,
	user: AccountId,
	price: Balance,
	
	created_at: BlockNumber,
	updated_at: BlockNumber,
}

