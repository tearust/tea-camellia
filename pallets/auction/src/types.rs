// use codec::FullCodec;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

use pallet_cml::CmlId;
use sp_std::cmp::{Eq, PartialEq};

// use super::auction;

// #[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
// pub enum Change<Value> {
// 	/// No change.
// 	NoChange,
// 	/// Changed to new value.
// 	NewValue(Value),
// }

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct AuctionItem<AuctionId, AccountId, Balance, BlockNumber>
where
	AuctionId: Default,
	AccountId: Default,
	Balance: Default,
	BlockNumber: Default,
{
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

impl<AuctionId, AccountId, Balance, BlockNumber> Default
	for BidItem<AuctionId, AccountId, Balance, BlockNumber>
where
	AuctionId: Default,
	AccountId: Default,
	Balance: Default,
	BlockNumber: Default,
{
	fn default() -> Self {
		BidItem {
			auction_id: AuctionId::default(),
			user: AccountId::default(),
			price: Balance::default(),
			deposit: None,
			created_at: BlockNumber::default(),
			updated_at: BlockNumber::default(),
		}
	}
}

impl<AuctionId, AccountId, Balance, BlockNumber> Default
	for AuctionItem<AuctionId, AccountId, Balance, BlockNumber>
where
	AuctionId: Default,
	AccountId: Default,
	Balance: Default,
	BlockNumber: Default,
{
	fn default() -> Self {
		AuctionItem {
			id: AuctionId::default(),
			cml_id: 0,
			cml_owner: AccountId::default(),
			starting_price: Balance::default(),
			buy_now_price: None,
			start_at: BlockNumber::default(),
			end_at: BlockNumber::default(),
			status: vec![],
			bid_user: None,
		}
	}
}
