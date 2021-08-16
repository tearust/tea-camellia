use cml_runtime_api::CmlApi as CmlRuntimeApi;
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
pub trait CmlApi<BlockHash, AccountId> {
	#[rpc(name = "cml_userCmlList")]
	fn user_cml_list(&self, who: AccountId, at: Option<BlockHash>) -> Result<Vec<u64>>;

	#[rpc(name = "cml_userCreditList")]
	fn user_credit_list(&self, who: AccountId, at: Option<BlockHash>) -> Result<Vec<(u64, Price)>>;

	#[rpc(name = "cml_userStakingList")]
	fn user_staking_list(&self, who: AccountId, at: Option<BlockHash>) -> Result<Vec<(u64, u64)>>;

	#[rpc(name = "cml_currentMiningCmlList")]
	fn current_mining_cml_list(&self, at: Option<BlockHash>) -> Result<Vec<u64>>;

	#[rpc(name = "cml_stakingPriceTable")]
	fn staking_price_table(&self, at: Option<BlockHash>) -> Result<Vec<Price>>;

	/// return a pair of values, first is current performance calculated by current block height,
	/// the second is the peak performance.
	#[rpc(name = "cml_cmlPerformance")]
	fn cml_performance(&self, cml_id: u64, at: Option<BlockHash>) -> Result<(u32, u32)>;
}

pub struct CmlApiImpl<C, M> {
	// If you have more generics, no need to SumStorage<C, M, N, P, ...>
	// just use a tuple like SumStorage<C, (M, N, P, ...)>
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> CmlApiImpl<C, M> {
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

impl<C, Block, AccountId> CmlApi<<Block as BlockT>::Hash, AccountId> for CmlApiImpl<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: cml_runtime_api::CmlApi<Block, AccountId>,
	AccountId: Codec,
{
	fn user_cml_list(
		&self,
		who: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<u64>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api
			.user_cml_list(&at, &who)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}

	fn user_credit_list(
		&self,
		who: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<(u64, Price)>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result: Vec<(u64, Balance)> = api
			.user_credit_list(&at, &who)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result.iter().map(|(id, v)| (*id, Price(*v))).collect())
	}

	fn user_staking_list(
		&self,
		who: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<(u64, u64)>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api
			.user_staking_list(&at, &who)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}

	fn current_mining_cml_list(&self, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<u64>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api
			.current_mining_cml_list(&at)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}

	fn staking_price_table(&self, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<Price>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result: Vec<Balance> = api
			.staking_price_table(&at)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result.iter().map(|v| Price(*v)).collect())
	}

	fn cml_performance(
		&self,
		cml_id: u64,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<(u32, u32)> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api
			.cml_performance(&at, cml_id)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}
}
