use auction_runtime_api::AuctionApi as AuctionRuntimeApi;
use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use node_primitives::Balance;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;
use types::*;

const RUNTIME_ERROR: i64 = 1;

mod types;

#[rpc]
pub trait AuctionApi<BlockHash, AccountId> {
	#[rpc(name = "auction_userAuctionList")]
	fn user_auction_list(&self, who: AccountId, at: Option<BlockHash>) -> Result<Vec<u64>>;

	#[rpc(name = "auction_userBidList")]
	fn user_bid_list(&self, who: AccountId, at: Option<BlockHash>) -> Result<Vec<u64>>;

	#[rpc(name = "auction_currentAuctionList")]
	fn current_auction_list(&self, at: Option<BlockHash>) -> Result<Vec<u64>>;

	/// first return value is the minimum bid price, the second return value indicates if the cml
	/// is mining
	#[rpc(name = "auction_calculateMinimumBidPrice")]
	fn estimate_minimum_bid_price(
		&self,
		auction_id: u64,
		who: AccountId,
		at: Option<BlockHash>,
	) -> Result<(Price, Option<Price>, bool)>;

	#[rpc(name = "auction_penaltyAmount")]
	fn penalty_amount(
		&self,
		auction_id: u64,
		who: AccountId,
		at: Option<BlockHash>,
	) -> Result<Price>;
}

pub struct AuctionApiImpl<C, M> {
	// If you have more generics, no need to SumStorage<C, M, N, P, ...>
	// just use a tuple like SumStorage<C, (M, N, P, ...)>
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> AuctionApiImpl<C, M> {
	/// Create new `SumStorage` instance with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> RpcError {
	RpcError {
		code: ErrorCode::ServerError(RUNTIME_ERROR),
		message: "Auction runtime error".into(),
		data: Some(format!("{:?}", err).into()),
	}
}

impl<C, Block, AccountId> AuctionApi<<Block as BlockT>::Hash, AccountId>
	for AuctionApiImpl<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: auction_runtime_api::AuctionApi<Block, AccountId>,
	AccountId: Codec,
{
	fn user_auction_list(
		&self,
		who: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<u64>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api
			.user_auction_list(&at, &who)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}

	fn user_bid_list(
		&self,
		who: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<u64>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api
			.user_bid_list(&at, &who)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}

	fn current_auction_list(&self, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<u64>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api
			.current_auction_list(&at)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}

	fn estimate_minimum_bid_price(
		&self,
		auction_id: u64,
		who: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<(Price, Option<Price>, bool)> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let (amount, origin, is_mining): (Balance, Option<Balance>, bool) = api
			.estimate_minimum_bid_price(&at, auction_id, &who)
			.map_err(runtime_error_into_rpc_err)?;
		Ok((
			Price(amount),
			origin.map(|balance| Price(balance)),
			is_mining,
		))
	}

	fn penalty_amount(
		&self,
		auction_id: u64,
		who: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Price> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let amount: Balance = api
			.penalty_amount(&at, auction_id, &who)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(Price(amount))
	}
}
