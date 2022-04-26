use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use machine_runtime_api::MachineApi as MachineRuntimeApi;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

const RUNTIME_ERROR: i64 = 1;

#[rpc]
pub trait MachineApi<BlockHash, AccountId> {
	#[rpc(name = "tea_bootNodes")]
	fn boot_nodes(&self, at: Option<BlockHash>) -> Result<Vec<[u8; 32]>>;

	#[rpc(name = "tea_tappStoreStartupNodes")]
	fn tapp_store_startup_nodes(&self, at: Option<BlockHash>) -> Result<Vec<[u8; 32]>>;
}

pub struct MachineApiImpl<C, M> {
	// If you have more generics, no need to SumStorage<C, M, N, P, ...>
	// just use a tuple like SumStorage<C, (M, N, P, ...)>
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> MachineApiImpl<C, M> {
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

impl<C, Block, AccountId> MachineApi<<Block as BlockT>::Hash, AccountId>
	for MachineApiImpl<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: machine_runtime_api::MachineApi<Block, AccountId>,
	AccountId: Codec,
{
	fn boot_nodes(&self, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<[u8; 32]>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api.boot_nodes(&at).map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}

	fn tapp_store_startup_nodes(
		&self,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<[u8; 32]>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api
			.tapp_store_startup_nodes(&at)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}
}
