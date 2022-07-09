use cml_runtime_api::CmlApi as CmlRuntimeApi;
use codec::{Codec};
use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

mod types;

pub use types::*;

#[rpc]
pub trait CmlApi<BlockHash, AccountId>
where
	AccountId: Codec,
{
	#[rpc(name = "cml_userCmlList")]
	fn user_cml_list(&self, who: AccountId, at: Option<BlockHash>) -> Result<Vec<u64>>;
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
fn runtime_error_into_rpc_err(err: impl std::fmt::Display) -> Error {
	Error {
		code: ErrorCode::InternalError,
		message: "Error while checking migration state".into(),
		data: Some(err.to_string().into()),
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
			.user_cml_list(&at, who)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}
}
