// use codec::FullCodec;
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

use pallet_cml::CmlId;
use sp_std::cmp::{Eq, PartialEq};

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub enum AuctionStatus {
	Normal,
	/// If auction fee is not payed for the next window, auction status will be set to suspend,
	/// the auction seller should add sufficient free balance or else the auction will be deleted
	/// in the next window.
	Suspended,
}

pub type AuctionId = u64;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct AuctionItem<AccountId, Balance, BlockNumber>
where
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
	pub status: AuctionStatus,
	pub bid_user: Option<AccountId>,
	pub auto_renew: bool,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
pub struct BidItem<AccountId, Balance, BlockNumber> {
	pub auction_id: AuctionId,
	pub user: AccountId,
	pub price: Balance,
	pub deposit: Option<Balance>,

	pub created_at: BlockNumber,
	pub updated_at: BlockNumber,
}

impl<AccountId, Balance, BlockNumber> Default for BidItem<AccountId, Balance, BlockNumber>
where
	AccountId: Default,
	Balance: Default,
	BlockNumber: Default,
{
	fn default() -> Self {
		BidItem {
			auction_id: 0,
			user: AccountId::default(),
			price: Balance::default(),
			deposit: None,
			created_at: BlockNumber::default(),
			updated_at: BlockNumber::default(),
		}
	}
}

impl<AccountId, Balance, BlockNumber> Default for AuctionItem<AccountId, Balance, BlockNumber>
where
	AccountId: Default,
	Balance: Default,
	BlockNumber: Default,
{
	fn default() -> Self {
		AuctionItem {
			id: 0,
			cml_id: 0,
			cml_owner: AccountId::default(),
			starting_price: Balance::default(),
			buy_now_price: None,
			start_at: BlockNumber::default(),
			status: AuctionStatus::Normal,
			bid_user: None,
			auto_renew: false,
		}
	}
}
