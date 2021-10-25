use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use node_primitives::BlockNumber;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;
use tea_runtime_api::TeaApi as TeaRuntimeApi;

const RUNTIME_ERROR: i64 = 1;

#[rpc]
pub trait TeaApi<BlockHash, AccountId> {
	#[rpc(name = "tea_isRaValidator")]
	fn is_ra_validator(
		&self,
		tea_id: [u8; 32],
		target_tea_id: [u8; 32],
		block_number: BlockNumber,
		at: Option<BlockHash>,
	) -> Result<bool>;
}

pub struct TeaApiImpl<C, M> {
	// If you have more generics, no need to SumStorage<C, M, N, P, ...>
	// just use a tuple like SumStorage<C, (M, N, P, ...)>
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> TeaApiImpl<C, M> {
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

impl<C, Block, AccountId> TeaApi<<Block as BlockT>::Hash, AccountId> for TeaApiImpl<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: tea_runtime_api::TeaApi<Block, AccountId>,
	AccountId: Codec,
{
	fn is_ra_validator(
		&self,
		tea_id: [u8; 32],
		target_tea_id: [u8; 32],
		block_number: BlockNumber,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<bool> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api
			.is_ra_validator(&at, &tea_id, &target_tea_id, block_number)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}
}
