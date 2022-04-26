//! A collection of node-specific RPC methods.
//! Substrate provides the `sc-rpc` crate, which defines the core RPC layer
//! used by Substrate nodes. This file extends those RPC definitions with
//! capabilities that are specific to this project's runtime configuration.

#![warn(missing_docs)]

use camellia_runtime::{opaque::Block, AccountId, Balance, Index};
use node_rpc::{BabeDeps, FullDeps, GrandpaDeps};
use sc_client_api::AuxStore;
use sc_consensus_babe_rpc::BabeRpcHandler;
use sc_finality_grandpa_rpc::GrandpaRpcHandler;
pub use sc_rpc_api::DenyUnsafe;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_consensus::SelectChain;
use sp_consensus_babe::BabeApi;

/// Instantiate all full RPC extensions.
pub fn create_full<C, P, SC, B>(
	deps: FullDeps<C, P, SC, B>,
) -> Result<jsonrpc_core::IoHandler<sc_rpc_api::Metadata>, Box<dyn std::error::Error + Send + Sync>>
where
	C: ProvideRuntimeApi<Block>
		+ HeaderBackend<Block>
		+ AuxStore
		+ HeaderMetadata<Block, Error = BlockChainError>
		+ Sync
		+ Send
		+ 'static,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
	C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: pallet_mmr_rpc::MmrRuntimeApi<Block, <Block as sp_runtime::traits::Block>::Hash>,
	C::Api: BabeApi<Block>,
	C::Api: BlockBuilder<Block>,
	P: TransactionPool + 'static,
	SC: SelectChain<Block> + 'static,
	B: sc_client_api::Backend<Block> + Send + Sync + 'static,
	B::State: sc_client_api::backend::StateBackend<sp_runtime::traits::HashFor<Block>>,

	C::Api: machine_runtime_api::MachineApi<Block, AccountId>,
	C::Api: cml_runtime_api::CmlApi<Block, AccountId>,
{
	use pallet_mmr_rpc::{Mmr, MmrApi};
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
	use substrate_frame_rpc_system::{FullSystem, SystemApi};

	let mut io = jsonrpc_core::IoHandler::default();
	let FullDeps {
		client,
		pool,
		select_chain,
		chain_spec,
		deny_unsafe,
		babe,
		grandpa,
	} = deps;

	let BabeDeps {
		keystore,
		babe_config,
		shared_epoch_changes,
	} = babe;
	let GrandpaDeps {
		shared_voter_state,
		shared_authority_set,
		justification_stream,
		subscription_executor,
		finality_provider,
	} = grandpa;

	io.extend_with(SystemApi::to_delegate(FullSystem::new(
		client.clone(),
		pool,
		deny_unsafe,
	)));

	io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
		client.clone(),
	)));

	io.extend_with(MmrApi::to_delegate(Mmr::new(client.clone())));
	io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
		client.clone(),
	)));
	io.extend_with(sc_consensus_babe_rpc::BabeApi::to_delegate(
		BabeRpcHandler::new(
			client.clone(),
			shared_epoch_changes.clone(),
			keystore,
			babe_config,
			select_chain,
			deny_unsafe,
		),
	));
	io.extend_with(sc_finality_grandpa_rpc::GrandpaApi::to_delegate(
		GrandpaRpcHandler::new(
			shared_authority_set.clone(),
			shared_voter_state,
			justification_stream,
			subscription_executor,
			finality_provider,
		),
	));

	io.extend_with(sc_sync_state_rpc::SyncStateRpcApi::to_delegate(
		sc_sync_state_rpc::SyncStateRpcHandler::new(
			chain_spec,
			client.clone(),
			shared_authority_set,
			shared_epoch_changes,
			deny_unsafe,
		)?,
	));

	// Extend this RPC with a custom API by using the following syntax.
	// `YourRpcStruct` should have a reference to a client, which is needed
	// to call into the runtime.
	// `io.extend_with(YourRpcTrait::to_delegate(YourRpcStruct::new(ReferenceToClient, ...)));`
	io.extend_with(machine_rpc::MachineApi::to_delegate(
		machine_rpc::MachineApiImpl::new(client.clone()),
	));
	io.extend_with(cml_rpc::CmlApi::to_delegate(cml_rpc::CmlApiImpl::new(
		client.clone(),
	)));

	Ok(io)
}
