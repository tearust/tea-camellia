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

	#[rpc(name = "cml_userStakingList")]
	fn user_staking_list(&self, who: AccountId, at: Option<BlockHash>) -> Result<Vec<(u64, u64)>>;

	#[rpc(name = "cml_currentMiningCmlList")]
	fn current_mining_cml_list(
		&self,
		at: Option<BlockHash>,
	) -> Result<Vec<(u64, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, u32)>>;

	#[rpc(name = "cml_stakingPriceTable")]
	fn staking_price_table(&self, at: Option<BlockHash>) -> Result<Vec<Price>>;

	#[rpc(name = "cml_estimateStopMiningPenalty")]
	fn estimate_stop_mining_penalty(&self, cml_id: u64, at: Option<BlockHash>) -> Result<Price>;

	#[rpc(name = "cml_listCmlInfo")]
	fn list_cmls_info(
		&self,
		exclude_account: Option<AccountId>,
		at: Option<BlockHash>,
	) -> Result<Vec<(AccountId, Vec<(u64, String, String)>)>>;
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

	fn current_mining_cml_list(
		&self,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<(u64, Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>, u32)>> {
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

	fn estimate_stop_mining_penalty(
		&self,
		cml_id: u64,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Price> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result: Balance = api
			.estimate_stop_mining_penalty(&at, cml_id)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(Price(result))
	}

	fn list_cmls_info(
		&self,
		exclude_account: Option<AccountId>,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<(AccountId, Vec<(u64, String, String)>)>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result: Vec<(AccountId, Vec<(u64, Vec<u8>, Vec<u8>)>)> = api
			.list_cmls_info(&at, exclude_account)
			.map_err(runtime_error_into_rpc_err)?;

		let mut rtn = Vec::new();
		for (acc, values) in result {
			let mut array = Vec::new();

			for (id, cml_type, status) in values {
				array.push((
					id,
					String::from_utf8(cml_type).map_err(runtime_error_into_rpc_err)?,
					String::from_utf8(status).map_err(runtime_error_into_rpc_err)?,
				));
			}
			rtn.push((acc, array));
		}
		Ok(rtn)
	}
}
