use codec::Codec;
use genesis_exchange_runtime_api::GenesisExchangeApi as GenesisExchangeRuntimeApi;
use jsonrpsee::{
	core::{Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorCode, ErrorObject},
};
use node_primitives::Balance;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

mod types;

pub use types::*;

#[rpc(server)]
pub trait GenesisExchangeApi<BlockHash, AccountId> {
	/// Returns
	/// 1. current 1TEA equals how many USD amount
	/// 2. current 1USD equals how many TEA amount
	/// 3. exchange remains USD
	/// 4. exchange remains TEA
	/// 5. product of  exchange remains USD and exchange remains TEA
	#[method(name = "cml_currentExchangeRate")]
	fn current_exchange_rate(
		&self,
		at: Option<BlockHash>,
	) -> RpcResult<(Price, Price, Price, Price, Price)>;

	#[method(name = "cml_estimateAmount")]
	fn estimate_amount(
		&self,
		withdraw_amount: Balance,
		buy_tea: bool,
		at: Option<BlockHash>,
	) -> RpcResult<Price>;
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
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> JsonRpseeError {
	JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
		ErrorCode::InternalError.code(),
		"Runtime error",
		Some(format!("{:?}", err)),
	)))
}

impl<C, Block, AccountId> GenesisExchangeApiServer<<Block as BlockT>::Hash, AccountId>
	for GenesisExchangeApiImpl<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: genesis_exchange_runtime_api::GenesisExchangeApi<Block, AccountId>,
	AccountId: Codec + Clone,
{
	fn current_exchange_rate(
		&self,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<(Price, Price, Price, Price, Price)> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result: (Balance, Balance, Balance, Balance, Balance) = api
			.current_exchange_rate(&at)
			.map_err(runtime_error_into_rpc_err)?;
		Ok((
			Price(result.0),
			Price(result.1),
			Price(result.2),
			Price(result.3),
			Price(result.4),
		))
	}

	fn estimate_amount(
		&self,
		withdraw_amount: Balance,
		buy_tea: bool,
		at: Option<<Block as BlockT>::Hash>,
	) -> RpcResult<Price> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result: Balance = api
			.estimate_amount(&at, withdraw_amount, buy_tea)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(Price(result))
	}
}
