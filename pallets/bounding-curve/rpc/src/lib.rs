use bounding_curve_runtime_api::BoundingCurveApi as BoundingCurveRuntimeApi;
use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use node_primitives::Balance;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

mod types;

pub use types::*;

const RUNTIME_ERROR: i64 = 1;

#[rpc]
pub trait BoundingCurveApi<BlockHash, AccountId> {
	#[rpc(name = "bounding_queryPrice")]
	fn query_price(&self, tapp_id: u64, at: Option<BlockHash>) -> Result<(Price, Price)>;

	#[rpc(name = "bounding_estimateBuy")]
	fn estimate_required_tea_when_buy(&self, tapp_id: u64, amount: Balance, at: Option<BlockHash>) -> Result<Price>;

	#[rpc(name = "bounding_estimateSell")]
	fn estimate_receive_tea_when_sell(&self, tapp_id: u64, amount: Balance, at: Option<BlockHash>) -> Result<Price>;
}

pub struct BoundingCurveApiImpl<C, M> {
	// If you have more generics, no need to SumStorage<C, M, N, P, ...>
	// just use a tuple like SumStorage<C, (M, N, P, ...)>
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> BoundingCurveApiImpl<C, M> {
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
		message: "Runtime error".into(),
		data: Some(format!("{:?}", err).into()),
	}
}

impl<C, Block, AccountId> BoundingCurveApi<<Block as BlockT>::Hash, AccountId>
	for BoundingCurveApiImpl<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: bounding_curve_runtime_api::BoundingCurveApi<Block, AccountId>,
	AccountId: Codec,
{
	fn query_price(
		&self,
		tapp_id: u64,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<(Price, Price)> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let (buy, sell): (Balance, Balance) = api
			.query_price(&at, tapp_id)
			.map_err(runtime_error_into_rpc_err)?;
		Ok((Price(buy), Price(sell)))
	}

	fn estimate_required_tea_when_buy(
		&self,
		tapp_id: u64,
		amount: Balance,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Price> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result: Balance = api
			.estimate_required_tea_when_buy(&at, tapp_id, amount)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(Price(result))
	}

	fn estimate_receive_tea_when_sell(
		&self,
		tapp_id: u64,
		amount: Balance,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Price> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result: Balance = api
			.estimate_receive_tea_when_sell(&at, tapp_id, amount)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(Price(result))
	}
}
