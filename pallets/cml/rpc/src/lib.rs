use cml_runtime_api::CmlApi as CmlRuntimeApi;
use codec::{Codec};
use jsonrpsee::{
	core::{Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
	types::error::{CallError, ErrorCode, ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

mod types;

pub use types::*;

#[rpc(server)]
pub trait CmlApi<BlockHash, AccountId>
where
	AccountId: Codec,
{
	#[method(name = "cml_userCmlList")]
	fn user_cml_list(&self, who: AccountId, at: Option<BlockHash>) -> RpcResult<Vec<u64>>;
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
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> JsonRpseeError {
	JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
		ErrorCode::InternalError.code(),
		"Runtime error",
		Some(format!("{:?}", err)),
	)))
}

impl<C, Block, AccountId> CmlApiServer<<Block as BlockT>::Hash, AccountId> for CmlApiImpl<C, Block>
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
	) -> RpcResult<Vec<u64>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api
			.user_cml_list(&at, who)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}
}
