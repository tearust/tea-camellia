use codec::Codec;
use genesis_bank_runtime_api::GenesisBankApi as GenesisBankRuntimeApi;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use node_primitives::{Balance, BlockNumber};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

mod types;

pub use types::*;

const RUNTIME_ERROR: i64 = 1;

#[rpc]
pub trait GenesisBankApi<BlockHash, AccountId> {
	#[rpc(name = "cml_cmlLienRedeemAmount")]
	fn cml_lien_redeem_amount(
		&self,
		cml_id: u64,
		block_height: BlockNumber,
		at: Option<BlockHash>,
	) -> Result<Price>;

	#[rpc(name = "cml_userCmlLienList")]
	fn user_cml_lien_list(&self, who: AccountId, at: Option<BlockHash>) -> Result<Vec<u64>>;

	#[rpc(name = "cml_bankOwnedCmls")]
	fn bank_owned_cmls(&self, at: Option<BlockHash>) -> Result<Vec<u64>>;
}

pub struct GenesisBankApiImpl<C, M> {
	// If you have more generics, no need to SumStorage<C, M, N, P, ...>
	// just use a tuple like SumStorage<C, (M, N, P, ...)>
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> GenesisBankApiImpl<C, M> {
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

impl<C, Block, AccountId> GenesisBankApi<<Block as BlockT>::Hash, AccountId>
	for GenesisBankApiImpl<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: genesis_bank_runtime_api::GenesisBankApi<Block, AccountId>,
	AccountId: Codec,
{
	fn cml_lien_redeem_amount(
		&self,
		cml_id: u64,
		block_height: BlockNumber,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Price> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result: Balance = api
			.cml_lien_redeem_amount(&at, cml_id, block_height)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(Price(result))
	}

	fn user_cml_lien_list(
		&self,
		who: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<u64>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api
			.user_cml_lien_list(&at, &who)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}

	fn bank_owned_cmls(&self, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<u64>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api
			.bank_owned_cmls(&at)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}
}
