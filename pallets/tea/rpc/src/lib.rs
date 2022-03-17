use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use node_primitives::BlockNumber;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::H256;
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

	#[rpc(name = "tea_bootNodes")]
	fn boot_nodes(&self, at: Option<BlockHash>) -> Result<Vec<[u8; 32]>>;

	#[rpc(name = "tea_tappStoreStartupNodes")]
	fn tapp_store_startup_nodes(&self, at: Option<BlockHash>) -> Result<Vec<[u8; 32]>>;

	#[rpc(name = "tea_allowedPcrs")]
	fn allowed_pcrs(&self, at: Option<BlockHash>) -> Result<Vec<(H256, Vec<Vec<u8>>)>>;

	#[rpc(name = "tea_allowedVersions")]
	fn allowed_versions(
		&self,
		at: Option<BlockHash>,
	) -> Result<Vec<(H256, Vec<(Vec<u8>, Vec<u8>)>, Option<BlockNumber>)>>;

	#[rpc(name = "tea_findTeaIdByPeerId")]
	fn find_tea_id_by_peer_id(
		&self,
		peer_id: Vec<u8>,
		at: Option<BlockHash>,
	) -> Result<Vec<[u8; 32]>>;

	#[rpc(name = "tea_versionExpiredNodes")]
	fn version_expired_nodes(&self, at: Option<BlockHash>) -> Result<Vec<[u8; 32]>>;

	#[rpc(name = "tea_findPeerIdsByConnIds")]
	fn find_peer_ids_by_conn_ids(
		&self,
		conn_ids: Vec<Vec<u8>>,
		at: Option<BlockHash>,
	) -> Result<Vec<(Vec<u8>, [u8; 32])>>;
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

	fn allowed_pcrs(
		&self,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<(H256, Vec<Vec<u8>>)>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api.allowed_pcrs(&at).map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}

	fn allowed_versions(
		&self,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<(H256, Vec<(Vec<u8>, Vec<u8>)>, Option<BlockNumber>)>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api
			.allowed_versions(&at)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}

	fn find_tea_id_by_peer_id(
		&self,
		peer_id: Vec<u8>,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<[u8; 32]>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api
			.find_tea_id_by_peer_id(&at, peer_id)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}

	fn version_expired_nodes(&self, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<[u8; 32]>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api
			.version_expired_nodes(&at)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}

	fn find_peer_ids_by_conn_ids(
		&self,
		conn_ids: Vec<Vec<u8>>,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<(Vec<u8>, [u8; 32])>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api
			.find_peer_ids_by_conn_ids(&at, conn_ids)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}
}
