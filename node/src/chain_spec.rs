use camellia_runtime::{
	constants::currency::DOLLARS,
	opaque::SessionKeys,
	pallet_cml::{generator::init_genesis, GenesisSeeds},
	AccountId, AuthorityDiscoveryConfig, BabeConfig, Balance, BalancesConfig, Block, CmlConfig,
	CouncilConfig, DemocracyConfig, ElectionsConfig, GenesisConfig, GenesisExchangeConfig,
	GrandpaConfig, ImOnlineConfig, MachineConfig, SessionConfig, Signature, StakerStatus,
	StakingConfig, SudoConfig, SystemConfig, TechnicalCommitteeConfig, WASM_BINARY,
};
use grandpa_primitives::AuthorityId as GrandpaId;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::ByteArray;
use sp_core::{crypto::AccountId32, sr25519, Pair, Public};
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill,
};
use std::str::FromStr;

const INITIAL_ACCOUNT_BALANCE: Balance = 2_000_000 * DOLLARS;
const INITIAL_VALIDATOR_BALANCE: Balance = 100 * DOLLARS;

const INITIAL_EXCHANGE_TEA_BALANCE: Balance = 100_000_000 * DOLLARS;
const INITIAL_EXCHANGE_USD_BALANCE: Balance = 100_000 * DOLLARS;
const BONDING_CURVE_NPC_INITIAL_USD_BALANCE: Balance = 1_000_000 * DOLLARS;

// address derived from [1u8; 32] that the corresponding private key we don't know
const GENESIS_EXCHANGE_OPERATION_ADDRESS: &str = "5C62Ck4UrFPiBtoCmeSrgF7x9yv9mn38446dhCpsi2mLHiFT";
// NPC is predefined "sudo" user in competition csv file, the following is address and initial amounts settings
const NPC_ADDRESS: &str = "5D2od84fg3GScGR139Li56raDWNQQhzgYbV7QsEJKS4KfTGv";
const DAO_RESERVED_ACCOUNT: &str = "5Hq3maxnpUx566bEDcLARMVAnGEqtoV7ytzXbtqieen7dXhs";

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<Block>,
	/// The light sync state extension used by the sync-state rpc.
	pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

pub fn public_from_hex_string<TPublic: Public>(hex_str: &str) -> <TPublic::Pair as Pair>::Public {
	<TPublic::Pair as Pair>::Public::from_slice(
		hex::decode(hex_str)
			.expect(format!("{} failed to decode to hex", hex_str).as_str())
			.as_slice(),
	)
	.unwrap()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn get_account_id_from_hex_string<TPublic: Public>(hex_str: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(public_from_hex_string::<TPublic>(hex_str)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(
	seed: &str,
) -> (
	AccountId,
	AccountId,
	BabeId,
	GrandpaId,
	ImOnlineId,
	AuthorityDiscoveryId,
) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

pub fn authority_keys_from_hex_string(
	sr25519_str: &str,
	ed25519_str: &str,
) -> (
	AccountId,
	AccountId,
	BabeId,
	GrandpaId,
	ImOnlineId,
	AuthorityDiscoveryId,
) {
	(
		get_account_id_from_hex_string::<sr25519::Public>(sr25519_str),
		get_account_id_from_hex_string::<sr25519::Public>(sr25519_str),
		public_from_hex_string::<BabeId>(sr25519_str),
		public_from_hex_string::<GrandpaId>(ed25519_str),
		public_from_hex_string::<ImOnlineId>(sr25519_str),
		public_from_hex_string::<AuthorityDiscoveryId>(sr25519_str),
	)
}

pub fn development_config(
	seed: [u8; 32],
	tapp_startup: Vec<([u8; 32], u64, Vec<u8>)>,
) -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name (spec_name)
		"node",
		"node",
		ChainType::Development,
		move || {
			let genesis_seeds = init_genesis(seed);
			let endowed_accounts = vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			];

			let endowed_balances =
				generate_account_balance_list(&endowed_accounts, INITIAL_ACCOUNT_BALANCE);

			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				endowed_accounts,
				endowed_balances,
				genesis_seeds,
				tapp_startup.clone(),
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		Some("TEA"),
		// Extensions
		Default::default(),
		Default::default(),
	))
}

pub fn canary_testnet_config(
	initial_validator_count: u32,
	seed: [u8; 32],
	tapp_startup: Vec<([u8; 32], u64, Vec<u8>)>,
) -> Result<ChainSpec, String> {
	// Canary Alice
	const ROOT_PUB_STR: (&str, &str) = (
		"d28a175da66df33a0b9573d90691bdb75470b11a1b640d3e359dcd1263306b12",
		"a19ab5f7e9e57b51e346a462f6178c40bd08810133562b340d7759098786f856",
	);
	// initial 7+2 validator accounts
	const ENDOWED_ACCOUNTS_PUB_STR: [(&str, &str); 7] = [
		ROOT_PUB_STR,
		// Canary Bob
		(
			"6a2e15ae634749343f528be99b2c652d562d83b29a767250accb7b8f8a897815",
			"d8be5951a4ffa51c0c39c5869835dd999435edc7a9afd19784b71a63be77d382",
		),
		// Canary Charlie
		(
			"f641ccbee2c683f67bb45ae7108c811dcda078fdb8d1225085200a485dd38433",
			"8d103f39de4ae64178f5458f09b63967d8c5632cd966cdf28c5da788c78570fd",
		),
		// Canary Dave
		(
			"ae948264f576389d41bc37f7861253363527233fc4be4995fa923439ba3e465e",
			"185b7d09bf57d149e9b5ddee0e0ab37109c165ce75f34471725652043fc28569",
		),
		// Canary Eve
		(
			"8aa95b05807541333b1e813aac09324a2da8b3944f9ca0ec0d1ed1a3ce62156d",
			"f67da55264b0fca59cf5e85e836a697e153c3cdc0a4cbbfad83854c8484b6bec",
		),
		// Canary Ferdie
		(
			"0680b9f25482187be19be68c55330e3e4c346bcfa74027efc3a34fea9eecb944",
			"3acb32af2212577621266bd7983b09bff74f4adf33c6f5d7e96e81c43876c7f2",
		),
		// Canary George
		(
			"4269ae995ed87351689b3397d122c06c7f77bdf593074e31113b13300e360626",
			"fd485a9e576d25999f05d2534fc8f52824bb180cad604cdfd734f92a4d846112",
		),
	];
	if initial_validator_count > 7 {
		return Err("initial validator count should less than 7".into());
	}

	let endowed_accounts: Vec<AccountId> = ENDOWED_ACCOUNTS_PUB_STR
		[0..initial_validator_count as usize]
		.iter()
		.map(|v| get_account_id_from_hex_string::<sr25519::Public>(v.0))
		.collect();
	let root_account = get_account_id_from_hex_string::<sr25519::Public>(ROOT_PUB_STR.0);
	let initial_authorities = ENDOWED_ACCOUNTS_PUB_STR[0..initial_validator_count as usize]
		.iter()
		.map(|v| authority_keys_from_hex_string(v.0, v.1))
		.collect();
	let endowed_balances = endowed_accounts
		.iter()
		.map(|account| {
			if account.eq(&root_account) {
				(account.clone(), INITIAL_ACCOUNT_BALANCE)
			} else {
				(account.clone(), INITIAL_VALIDATOR_BALANCE)
			}
		})
		.collect();

	testnet_config(
		seed,
		endowed_accounts,
		// initial balance only for root account
		endowed_balances,
		initial_authorities,
		root_account,
		tapp_startup,
	)
}

pub fn local_testnet_config(
	seed: [u8; 32],
	tapp_startup: Vec<([u8; 32], u64, Vec<u8>)>,
) -> Result<ChainSpec, String> {
	let endowed_accounts = vec![
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		get_account_id_from_seed::<sr25519::Public>("Bob"),
		get_account_id_from_seed::<sr25519::Public>("Charlie"),
		get_account_id_from_seed::<sr25519::Public>("Dave"),
		get_account_id_from_seed::<sr25519::Public>("Eve"),
		get_account_id_from_seed::<sr25519::Public>("Ferdie"),
		get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
		get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
		get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
		get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
		get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
		get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
	];
	let endowed_balances =
		generate_account_balance_list(&endowed_accounts, INITIAL_ACCOUNT_BALANCE);

	testnet_config(
		seed,
		endowed_accounts,
		endowed_balances,
		vec![
			authority_keys_from_seed("Alice"),
			authority_keys_from_seed("Bob"),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		tapp_startup,
	)
}

pub fn testnet_config(
	seed: [u8; 32],
	endowed_accounts: Vec<AccountId>,
	endowed_balances: Vec<(AccountId, Balance)>,
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		BabeId,
		GrandpaId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	root_key: AccountId,
	tapp_startup: Vec<([u8; 32], u64, Vec<u8>)>,
) -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"node",
		"node",
		ChainType::Local,
		move || {
			let endowed_balances = endowed_balances.clone();
			let initial_authorities = initial_authorities.clone();
			let endowed_accounts = endowed_accounts.clone();
			let root_key = root_key.clone();

			let genesis_seeds = init_genesis(seed);

			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				initial_authorities,
				// Sudo account
				root_key,
				// Pre-funded accounts
				endowed_accounts,
				endowed_balances,
				genesis_seeds,
				tapp_startup.clone(),
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		// Extensions
		Default::default(),
		Default::default(),
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		BabeId,
		GrandpaId,
		ImOnlineId,
		AuthorityDiscoveryId,
	)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	mut initial_balances: Vec<(AccountId, Balance)>,
	genesis_seeds: GenesisSeeds,
	tapp_startup: Vec<([u8; 32], u64, Vec<u8>)>,
) -> GenesisConfig {
	let genesis_exchange_operation_account =
		AccountId32::from_str(GENESIS_EXCHANGE_OPERATION_ADDRESS).unwrap();
	let npc_account = AccountId32::from_str(NPC_ADDRESS).unwrap();
	let dao_reserved = AccountId32::from_str(DAO_RESERVED_ACCOUNT).unwrap();

	initial_balances.push((
		genesis_exchange_operation_account.clone(),
		INITIAL_EXCHANGE_TEA_BALANCE,
	));
	if let Some(index) = initial_balances
		.iter()
		.position(|(acc, _)| acc.eq(&npc_account))
	{
		initial_balances.remove(index);
	}
	initial_balances.push((npc_account.into(), INITIAL_VALIDATOR_BALANCE));

	let startup_cmls: Vec<u64> = tapp_startup.iter().map(|(_, cml_id, _)| *cml_id).collect();
	let num_endowed_accounts = endowed_accounts.len();
	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: initial_balances,
		},
		babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(camellia_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		grandpa: GrandpaConfig {
			authorities: vec![],
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(x.2.clone(), x.3.clone(), x.4.clone(), x.5.clone()),
					)
				})
				.collect::<Vec<_>>(),
		},
		staking: StakingConfig {
			stakers: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.1.clone(),
						INITIAL_VALIDATOR_BALANCE,
						StakerStatus::Validator,
					)
				})
				.collect(),
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: initial_authorities.len() as u32,
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			..Default::default()
		},
		im_online: ImOnlineConfig { keys: vec![] },
		authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
		elections: ElectionsConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.map(|member| (member, INITIAL_VALIDATOR_BALANCE))
				.collect(),
		},
		council: CouncilConfig::default(),
		technical_committee: TechnicalCommitteeConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.collect(),
			phantom: Default::default(),
		},
		technical_membership: Default::default(),
		democracy: DemocracyConfig::default(),

		machine: MachineConfig {
			startup_tapp_bindings: tapp_startup,
			startup_owner: Some(dao_reserved.clone()),
		},
		cml: CmlConfig {
			npc_account: Some(npc_account.clone()),
			startup_account: Some(dao_reserved.clone()),
			genesis_seeds,
			startup_cmls,
		},
		genesis_exchange: GenesisExchangeConfig {
			operation_account: Some(genesis_exchange_operation_account),
			npc_account: Some(npc_account.clone()),
			operation_usd_amount: INITIAL_EXCHANGE_USD_BALANCE,
			operation_tea_amount: INITIAL_EXCHANGE_TEA_BALANCE,
			bonding_curve_npc: Some((npc_account.clone(), BONDING_CURVE_NPC_INITIAL_USD_BALANCE)),
			initial_usd_interest_rate: 0, // let initial usd interest rate be 0.02%
			borrow_debt_ratio_cap: 0,     // initial borrow debt ratio cap is 0.
		},
		transaction_payment: Default::default(),
	}
}

fn session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
) -> SessionKeys {
	SessionKeys {
		babe,
		grandpa,
		im_online,
		authority_discovery,
	}
}

fn generate_account_balance_list(
	endowed_accounts: &Vec<AccountId>,
	balance: Balance,
) -> Vec<(AccountId, Balance)> {
	endowed_accounts
		.iter()
		.cloned()
		.map(|k| (k, balance))
		.collect()
}
