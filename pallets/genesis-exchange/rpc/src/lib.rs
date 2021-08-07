use codec::Codec;
use genesis_exchange_runtime_api::GenesisExchangeApi as GenesisExchangeRuntimeApi;
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
pub trait GenesisExchangeApi<BlockHash, AccountId> {
	#[rpc(name = "cml_currentExchangeRate")]
	fn current_exchange_rate(&self, at: Option<BlockHash>) -> Result<Price>;

	#[rpc(name = "cml_estimateAmount")]
	fn estimate_amount(
		&self,
		withdraw_amount: Balance,
		buy_tea: bool,
		at: Option<BlockHash>,
	) -> Result<Price>;

	#[rpc(name = "cml_userAssetList")]
	fn user_asset_list(&self, at: Option<BlockHash>) -> Result<Vec<(AccountId, Price)>>;
}

pub struct GenesisExchangeApiImpl<C, M> {
	// If you have more generics, no need to SumStorage<C, M, N, P, ...>
	// just use a tuple like SumStorage<C, (M, N, P, ...)>
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> GenesisExchangeApiImpl<C, M> {
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

impl<C, Block, AccountId> GenesisExchangeApi<<Block as BlockT>::Hash, AccountId>
	for GenesisExchangeApiImpl<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: genesis_exchange_runtime_api::GenesisExchangeApi<Block, AccountId>,
	AccountId: Codec + Clone,
{
	fn current_exchange_rate(&self, at: Option<<Block as BlockT>::Hash>) -> Result<Price> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result: Balance = api
			.current_exchange_rate(&at)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(Price(result))
	}

	fn estimate_amount(
		&self,
		withdraw_amount: Balance,
		buy_tea: bool,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Price> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result: Balance = api
			.estimate_amount(&at, withdraw_amount, buy_tea)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(Price(result))
	}

	fn user_asset_list(
		&self,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<(AccountId, Price)>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result: Vec<(AccountId, Balance)> = api
			.user_asset_list(&at)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result
			.iter()
			.map(|(account_id, balance)| (account_id.clone(), Price(*balance)))
			.collect())
	}
}
