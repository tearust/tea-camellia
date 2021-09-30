use bonding_curve_runtime_api::BondingCurveApi as BondingCurveRuntimeApi;
use codec::Codec;
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
pub trait BondingCurveApi<BlockHash, AccountId> {
	#[rpc(name = "bonding_queryPrice")]
	fn query_price(&self, tapp_id: u64, at: Option<BlockHash>) -> Result<(Price, Price)>;

	/// if `tapp_id` is `None` will calculating total supply with zero, and if `buy_curve_k` is `None` will use
	/// default k value (current is 10).
	#[rpc(name = "bonding_estimateTeaRequiredToBuyGivenToken")]
	fn estimate_required_tea_when_buy(
		&self,
		tapp_id: Option<u64>,
		token_amount: Balance,
		buy_curve_k: Option<u32>,
		at: Option<BlockHash>,
	) -> Result<Price>;

	#[rpc(name = "bonding_estimateReceivedTeaBySellGivenToken")]
	fn estimate_receive_tea_when_sell(
		&self,
		tapp_id: u64,
		token_amount: Balance,
		at: Option<BlockHash>,
	) -> Result<Price>;

	#[rpc(name = "bonding_estimateHowMuchTokenBoughtByGivenTea")]
	fn estimate_receive_token_when_buy(
		&self,
		tapp_id: u64,
		tea_amount: Balance,
		at: Option<BlockHash>,
	) -> Result<Price>;

	#[rpc(name = "bonding_estimateHowMuchTokenToSellByGivenTea")]
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
	/// - Host performance requirement (return zero if is none)
	/// - (current hosts (return zero if is none), max hosts (return zero if is none))
	/// - active block number (return none if not active)
	#[rpc(name = "bonding_listTApps")]
	fn list_tapps(
		&self,
		active_only: bool,
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
			u32,
			(u32, u32),
			Option<BlockNumber>,
		)>,
	>;

	/// Returned item fields:
	/// - TApp Name
	/// - TApp Id
	/// - TApp Ticker
	/// - User holding tokens (inverstor side only, not including mining reserved balance)
	/// - Token sell price
	/// - Owner
	/// - Detail
	/// - Link
	/// - Host performance requirement (return zero if is none)
	/// - current hosts (return zero if is none)
	/// - max hosts (return zero if is none)
	/// - Total supply
	#[rpc(name = "bonding_listUserAssets")]
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
			u32,
			u32,
			u32,
			Price,
		)>,
	>;

	/// Returned item fields:
	/// - TApp Name
	/// - TApp Id
	/// - TApp Ticker
	/// - Owner
	/// - Detail
	/// - Link
	/// - Host performance requirement (return zero if is none)
	/// - current hosts (return zero if is none)
	/// - max hosts (return zero if is none)
	#[rpc(name = "bonding_tappDetails")]
	fn tapp_details(
		&self,
		tapp_id: u64,
		at: Option<BlockHash>,
	) -> Result<(
		Vec<u8>,
		u64,
		Vec<u8>,
		AccountId,
		Vec<u8>,
		Vec<u8>,
		u32,
		u32,
		u32,
		Price,
		Price,
		Price,
	)>;

	/// Returned item fields:
	/// - CML Id
	/// - CML current performance
	/// - CML remaining performance
	/// - life remaining
	/// - Hosted tapp list
	#[rpc(name = "bonding_listCandidateMiners")]
	fn list_candidate_miners(
		&self,
		who: AccountId,
		at: Option<BlockHash>,
	) -> Result<Vec<(u64, u32, u32, BlockNumber, Vec<u64>)>>;

	#[rpc(name = "bonding_tappHostedCmls")]
	fn tapp_hosted_cmls(
		&self,
		tapp_id: u64,
		at: Option<BlockHash>,
	) -> Result<
		Vec<(
			u64,
			Option<AccountId>,
			BlockNumber,
			Option<u32>,
			Option<u32>,
			u32,
		)>,
	>;

	/// Returned item fields:
	/// - CML Id
	/// - CML remaining performance
	/// - TApp Id
	/// - TApp Ticker
	/// - TApp Name
	/// - TApp Detail
	/// - TApp Link
	/// - Min performance request
	#[rpc(name = "bonding_listCmlHostingTapps")]
	fn list_cml_hosting_tapps(
		&self,
		cml_id: u64,
		at: Option<BlockHash>,
	) -> Result<
		Vec<(
			u64,
			Option<u32>,
			u64,
			Vec<u8>,
			Vec<u8>,
			Vec<u8>,
			Vec<u8>,
			u32,
		)>,
	>;

	/// returned values:
	/// - current performance calculated by current block height
	/// - remaining performance
	/// - peak performance
	#[rpc(name = "cml_cmlPerformance")]
	fn cml_performance(
		&self,
		cml_id: u64,
		at: Option<BlockHash>,
	) -> Result<(Option<u32>, Option<u32>, u32)>;

	/// Returned item fields:
	/// - Link url
	/// - Tapp id, if not created based on the link value will be none
	/// - Link description
	#[rpc(name = "cml_approvedLinks")]
	fn approved_links(
		&self,
		at: Option<BlockHash>,
	) -> Result<Vec<(Vec<u8>, Option<u64>, Vec<u8>, Option<AccountId>)>>;
}

pub struct BondingCurveApiImpl<C, M> {
	// If you have more generics, no need to SumStorage<C, M, N, P, ...>
	// just use a tuple like SumStorage<C, (M, N, P, ...)>
	client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> BondingCurveApiImpl<C, M> {
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

impl<C, Block, AccountId> BondingCurveApi<<Block as BlockT>::Hash, AccountId>
	for BondingCurveApiImpl<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: bonding_curve_runtime_api::BondingCurveApi<Block, AccountId>,
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
		tapp_id: Option<u64>,
		token_amount: Balance,
		buy_curve_k: Option<u32>,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Price> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result: Balance = api
			.estimate_required_tea_when_buy(&at, tapp_id, token_amount, buy_curve_k)
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
		active_only: bool,
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
			u32,
			(u32, u32),
			Option<BlockNumber>,
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
			u32,
			(u32, u32),
			Option<BlockNumber>,
		)> = api.list_tapps(&at, active_only)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result
			.iter()
			.map(
				|(
					name,
					id,
					ticker,
					total_supply,
					buy_price,
					sell_price,
					owner,
					detail,
					link,
					performance,
					hosts_pair,
					active_height,
				)| {
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
						*performance,
						hosts_pair.clone(),
						active_height.clone(),
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
			u32,
			u32,
			u32,
			Price,
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
			u32,
			u32,
			u32,
			Balance,
		)> = api.list_user_assets(&at, who)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result
			.iter()
			.map(
				|(
					name,
					id,
					ticker,
					amount,
					sell_price,
					owner,
					detail,
					link,
					performance,
					current_hosts,
					max_hosts,
					total_supply,
				)| {
					(
						name.clone(),
						*id,
						ticker.clone(),
						Price(*amount),
						Price(*sell_price),
						owner.clone(),
						detail.clone(),
						link.clone(),
						*performance,
						*current_hosts,
						*max_hosts,
						Price(*total_supply),
					)
				},
			)
			.collect())
	}

	fn tapp_details(
		&self,
		tapp_id: u64,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<(
		Vec<u8>,
		u64,
		Vec<u8>,
		AccountId,
		Vec<u8>,
		Vec<u8>,
		u32,
		u32,
		u32,
		Price,
		Price,
		Price,
	)> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let (
			name,
			tapp_id,
			ticker,
			owner,
			detail,
			link,
			host_performance,
			current_hosts,
			max_hosts,
			total_supply,
			buy_price,
			sell_price,
		) = api.tapp_details(&at, tapp_id)
			.map_err(runtime_error_into_rpc_err)?;
		Ok((
			name,
			tapp_id,
			ticker,
			owner,
			detail,
			link,
			host_performance,
			current_hosts,
			max_hosts,
			Price(total_supply),
			Price(buy_price),
			Price(sell_price),
		))
	}

	fn list_candidate_miners(
		&self,
		who: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<(u64, u32, u32, BlockNumber, Vec<u64>)>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api
			.list_candidate_miners(&at, who)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}

	fn tapp_hosted_cmls(
		&self,
		tapp_id: u64,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<
		Vec<(
			u64,
			Option<AccountId>,
			BlockNumber,
			Option<u32>,
			Option<u32>,
			u32,
		)>,
	> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api
			.tapp_hosted_cmls(&at, tapp_id)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}

	fn list_cml_hosting_tapps(
		&self,
		cml_id: u64,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<
		Vec<(
			u64,
			Option<u32>,
			u64,
			Vec<u8>,
			Vec<u8>,
			Vec<u8>,
			Vec<u8>,
			u32,
		)>,
	> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api
			.list_cml_hosting_tapps(&at, cml_id)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}

	fn cml_performance(
		&self,
		cml_id: u64,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<(Option<u32>, Option<u32>, u32)> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api
			.cml_performance(&at, cml_id)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}

	fn approved_links(
		&self,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<(Vec<u8>, Option<u64>, Vec<u8>, Option<AccountId>)>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let result = api
			.approved_links(&at)
			.map_err(runtime_error_into_rpc_err)?;
		Ok(result)
	}
}
