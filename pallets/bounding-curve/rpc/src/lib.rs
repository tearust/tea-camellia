use bounding_curve_runtime_api::BoundingCurveApi as BoundingCurveRuntimeApi;
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
pub trait BoundingCurveApi<BlockHash, AccountId> {
	#[rpc(name = "bounding_queryPrice")]
	fn query_price(&self, tapp_id: u64, at: Option<BlockHash>) -> Result<(Price, Price)>;

	#[rpc(name = "bounding_estimateTeaRequiredToBuyGivenToken")]
	fn estimate_required_tea_when_buy(
		&self,
		tapp_id: u64,
		token_amount: Balance,
		at: Option<BlockHash>,
	) -> Result<Price>;

	#[rpc(name = "bounding_estimateReceivedTeaBySellGivenToken")]
	fn estimate_receive_tea_when_sell(
		&self,
		tapp_id: u64,
		token_amount: Balance,
		at: Option<BlockHash>,
	) -> Result<Price>;

	#[rpc(name = "bounding_estimateHowMuchTokenBoughtByGivenTea")]
	fn estimate_receive_token_when_buy(
		&self,
		tapp_id: u64,
		tea_amount: Balance,
		at: Option<BlockHash>,
	) -> Result<Price>;

	#[rpc(name = "bounding_estimateHowMuchTokenToSellByGivenTea")]
	fn estimate_required_token_when_sell(
		&self,
		tapp_id: u64,
		tea_amount: Balance,
		at: Option<BlockHash>,
	) -> Result<Price>;

	/// Returned item fields:
	/// - TApp Name
	/// - TApp Id
	/// - Total supply
	/// - Token buy price
	/// - Token sell price
	/// - Owner
	/// - Detail
	/// - Link
	#[rpc(name = "bounding_listTApps")]
	fn list_tapps(
		&self,
		at: Option<BlockHash>,
	) -> Result<
		Vec<(
			Vec<u8>,
			u64,
			Vec<u8>,
			Price,
			Price,
			Price,
			AccountId,
			Vec<u8>,
			Vec<u8>,
		)>,
	>;

	/// Returned item fields:
	/// - TApp Name
	/// - TApp Id
	/// - TApp Ticker
	/// - User holding tokens
	/// - Token sell price
	/// - Owner
	/// - Detail
	/// - Link
	#[rpc(name = "bounding_listUserAssets")]
	fn list_user_assets(
		&self,
		who: AccountId,
		at: Option<BlockHash>,
	) -> Result<
		Vec<(
			Vec<u8>,
			u64,
			Vec<u8>,
			Price,
			Price,
			AccountId,
			Vec<u8>,
			Vec<u8>,
		)>,
	>;
}

pub struct BoundingCurveApiImpl<C, M> {
	// If you have more generics, no need to SumStorage<C, M, N, P, ...>
	// just use a tuple like SumStorage<C, (M, N, P, ...)>
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> BoundingCurveApiImpl<C, M> {
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

impl<C, Block, AccountId> BoundingCurveApi<<Block as BlockT>::Hash, AccountId>
	for BoundingCurveApiImpl<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: bounding_curve_runtime_api::BoundingCurveApi<Block, AccountId>,
	AccountId: Codec + Clone,
{
	fn query_price(
		&self,
		tapp_id: u64,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<(Price, Price)> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let (buy, sell): (Balance, Balance) = api
			.query_price(&at, tapp_id)
			.map_err(runtime_error_into_rpc_err)?;
		Ok((Price(buy), Price(sell)))
	}

	fn estimate_required_tea_when_buy(
		&self,
		tapp_id: u64,
		token_amount: Balance,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Price> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result: Balance = api
			.estimate_required_tea_when_buy(&at, tapp_id, token_amount)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(Price(result))
	}

	fn estimate_receive_tea_when_sell(
		&self,
		tapp_id: u64,
		token_amount: Balance,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Price> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result: Balance = api
			.estimate_receive_tea_when_sell(&at, tapp_id, token_amount)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(Price(result))
	}

	fn estimate_receive_token_when_buy(
		&self,
		tapp_id: u64,
		tea_amount: Balance,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Price> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result: Balance = api
			.estimate_receive_token_when_buy(&at, tapp_id, tea_amount)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(Price(result))
	}

	fn estimate_required_token_when_sell(
		&self,
		tapp_id: u64,
		tea_amount: Balance,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Price> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result: Balance = api
			.estimate_required_token_when_sell(&at, tapp_id, tea_amount)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(Price(result))
	}

	fn list_tapps(
		&self,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<
		Vec<(
			Vec<u8>,
			u64,
			Vec<u8>,
			Price,
			Price,
			Price,
			AccountId,
			Vec<u8>,
			Vec<u8>,
		)>,
	> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result: Vec<(
			Vec<u8>,
			u64,
			Vec<u8>,
			Balance,
			Balance,
			Balance,
			AccountId,
			Vec<u8>,
			Vec<u8>,
		)> = api.list_tapps(&at).map_err(runtime_error_into_rpc_err)?;
		Ok(result
			.iter()
			.map(
				|(name, id, ticker, total_supply, buy_price, sell_price, owner, detail, link)| {
					(
						name.clone(),
						*id,
						ticker.clone(),
						Price(*total_supply),
						Price(*buy_price),
						Price(*sell_price),
						owner.clone(),
						detail.clone(),
						link.clone(),
					)
				},
			)
			.collect())
	}

	fn list_user_assets(
		&self,
		who: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<
		Vec<(
			Vec<u8>,
			u64,
			Vec<u8>,
			Price,
			Price,
			AccountId,
			Vec<u8>,
			Vec<u8>,
		)>,
	> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result: Vec<(
			Vec<u8>,
			u64,
			Vec<u8>,
			Balance,
			Balance,
			AccountId,
			Vec<u8>,
			Vec<u8>,
		)> = api.list_user_assets(&at, who)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result
			.iter()
			.map(
				|(name, id, ticker, amount, sell_price, owner, detail, link)| {
					(
						name.clone(),
						*id,
						ticker.clone(),
						Price(*amount),
						Price(*sell_price),
						owner.clone(),
						detail.clone(),
						link.clone(),
					)
				},
			)
			.collect())
	}
}
