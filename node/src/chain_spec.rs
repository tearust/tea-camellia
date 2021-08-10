use camellia_runtime::{
	constants::currency::DOLLARS,
	opaque::SessionKeys,
	pallet_cml::{generator::init_genesis, GenesisCoupons, GenesisSeeds},
	AccountId, AuthorityDiscoveryConfig, BabeConfig, Balance, BalancesConfig, CmlConfig,
	CouncilConfig, DemocracyConfig, ElectionsConfig, GenesisBankConfig, GenesisConfig,
	GenesisExchangeConfig, GrandpaConfig, ImOnlineConfig, SessionConfig, Signature, StakerStatus,
	StakingConfig, SudoConfig, SystemConfig, TeaConfig, TechnicalCommitteeConfig, WASM_BINARY,
};
use hex_literal::hex;
use jsonrpc_core::serde_json;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_service::{ChainType, Properties};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto::AccountId32, sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill,
};
use std::collections::HashSet;
use std::str::FromStr;

const INITIAL_ACCOUNT_BALANCE: Balance = 10000 * DOLLARS;
const COUPON_ACCOUNT_BALANCE: Balance = 1 * DOLLARS;

const INITIAL_EXCHANGE_TEA_BALANCE: Balance = 40_000 * DOLLARS;
const INITIAL_EXCHANGE_USD_BALANCE: Balance = 40_000 * DOLLARS;

const INITIAL_COMPETITION_USER_USD_BALANCE: Balance = 1000 * DOLLARS;

// address derived from [0u8; 32] that the corresponding private key we don't know
const GENESIS_BANK_OPERATION_ADDRESS: &str = "5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM";
// address derived from [1u8; 32] that the corresponding private key we don't know
const GENESIS_EXCHANGE_OPERATION_ADDRESS: &str = "5C62Ck4UrFPiBtoCmeSrgF7x9yv9mn38446dhCpsi2mLHiFT";

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

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
	hex_str: &str,
) -> (
	AccountId,
	AccountId,
	BabeId,
	GrandpaId,
	ImOnlineId,
	AuthorityDiscoveryId,
) {
	(
		get_account_id_from_hex_string::<sr25519::Public>(hex_str),
		get_account_id_from_hex_string::<sr25519::Public>(hex_str),
		public_from_hex_string::<BabeId>(hex_str),
		public_from_hex_string::<GrandpaId>(hex_str),
		public_from_hex_string::<ImOnlineId>(hex_str),
		public_from_hex_string::<AuthorityDiscoveryId>(hex_str),
	)
}

fn get_properties(symbol: &str) -> Properties {
	serde_json::json!({
		"tokenDecimals": 12,
		"ss58Format": 0,
		"tokenSymbol": symbol,
	})
	.as_object()
	.unwrap()
	.clone()
}

pub fn development_config(
	genesis_coupons: GenesisCoupons<AccountId>,
	seed: [u8; 32],
) -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			let genesis_seeds = init_genesis(seed);
			let mut endowed_accounts = vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			];
			let imported_endowed_accounts = get_unique_accounts(&genesis_coupons);
			endowed_accounts.extend(imported_endowed_accounts);

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
				genesis_coupons.clone(),
				genesis_seeds,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		Some(get_properties("TEA")),
		// Extensions
		None,
	))
}

pub fn canary_testnet_config(
	genesis_coupons: GenesisCoupons<AccountId>,
	seed: [u8; 32],
) -> Result<ChainSpec, String> {
	const ROOT_PUB_STR: &str = "d28a175da66df33a0b9573d90691bdb75470b11a1b640d3e359dcd1263306b12";
	const ENDOWED_ACCOUNTS_PUB_STR: [&str; 4] = [
		ROOT_PUB_STR,
		"6a2e15ae634749343f528be99b2c652d562d83b29a767250accb7b8f8a897815",
		"f641ccbee2c683f67bb45ae7108c811dcda078fdb8d1225085200a485dd38433",
		"ae948264f576389d41bc37f7861253363527233fc4be4995fa923439ba3e465e",
	];

	let endowed_accounts = ENDOWED_ACCOUNTS_PUB_STR
		.iter()
		.map(|v| get_account_id_from_hex_string::<sr25519::Public>(v))
		.collect();
	let root_account = get_account_id_from_hex_string::<sr25519::Public>(ROOT_PUB_STR);
	let initial_authorities = ENDOWED_ACCOUNTS_PUB_STR
		.iter()
		.map(|v| authority_keys_from_hex_string(v))
		.collect();

	testnet_config(
		genesis_coupons,
		seed,
		endowed_accounts,
		// initial balance only for root account
		vec![(root_account.clone(), INITIAL_ACCOUNT_BALANCE)],
		initial_authorities,
		root_account,
	)
}

pub fn local_testnet_config(
	genesis_coupons: GenesisCoupons<AccountId>,
	seed: [u8; 32],
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
		genesis_coupons,
		seed,
		endowed_accounts,
		endowed_balances,
		vec![
			authority_keys_from_seed("Alice"),
			authority_keys_from_seed("Bob"),
			authority_keys_from_seed("Charlie"),
			authority_keys_from_seed("Dave"),
			authority_keys_from_seed("Eve"),
			authority_keys_from_seed("Ferdie"),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
	)
}

pub fn testnet_config(
	genesis_coupons: GenesisCoupons<AccountId>,
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
) -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			let mut endowed_balances = endowed_balances.clone();
			let initial_authorities = initial_authorities.clone();
			let endowed_accounts = endowed_accounts.clone();
			let root_key = root_key.clone();

			let imported_endowed_accounts = get_unique_accounts(&genesis_coupons);
			endowed_balances.extend(generate_account_balance_list(
				&imported_endowed_accounts,
				COUPON_ACCOUNT_BALANCE,
			));

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
				genesis_coupons.clone(),
				genesis_seeds,
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
		None,
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
	genesis_coupons: GenesisCoupons<AccountId>,
	genesis_seeds: GenesisSeeds,
) -> GenesisConfig {
	let genesis_bank_operation_account =
		AccountId32::from_str(GENESIS_BANK_OPERATION_ADDRESS).unwrap();
	let genesis_exchange_operation_account =
		AccountId32::from_str(GENESIS_EXCHANGE_OPERATION_ADDRESS).unwrap();

	initial_balances.push((
		genesis_exchange_operation_account.clone(),
		INITIAL_EXCHANGE_TEA_BALANCE,
	));
	let competition_users = genesis_coupons
		.coupons
		.iter()
		.map(|coupon| (coupon.account.clone(), INITIAL_COMPETITION_USER_USD_BALANCE))
		.collect();

	const STASH: Balance = 100 * DOLLARS;
	let num_endowed_accounts = endowed_accounts.len();
	GenesisConfig {
		frame_system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: initial_balances,
		},
		pallet_babe: BabeConfig {
			authorities: vec![],
			epoch_config: Some(camellia_runtime::BABE_GENESIS_EPOCH_CONFIG),
		},
		pallet_grandpa: GrandpaConfig {
			authorities: vec![],
		},
		pallet_sudo: SudoConfig {
			// Assign network admin rights.
			key: root_key,
		},
		pallet_session: SessionConfig {
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
		pallet_staking: StakingConfig {
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator))
				.collect(),
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			..Default::default()
		},
		pallet_im_online: ImOnlineConfig { keys: vec![] },
		pallet_authority_discovery: AuthorityDiscoveryConfig { keys: vec![] },
		pallet_elections_phragmen: ElectionsConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.map(|member| (member, STASH))
				.collect(),
		},
		pallet_collective_Instance1: CouncilConfig::default(),
		pallet_collective_Instance2: TechnicalCommitteeConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.collect(),
			phantom: Default::default(),
		},
		pallet_membership_Instance1: Default::default(),
		pallet_democracy: DemocracyConfig::default(),
		pallet_treasury: Default::default(),

		pallet_tea: TeaConfig {
			builtin_nodes: vec![
				hex!("df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b"),
				hex!("c7e016fad0796bb68594e49a6ef1942cf7e73497e69edb32d19ba2fab3696596"),
				hex!("2754d7e9c73ced5b302e12464594110850980027f8f83c469e8145eef59220b6"),
				hex!("c9380fde1ba795fc656ab08ab4ef4482cf554790fd3abcd4642418ae8fb5fd52"),
				hex!("bd1c0ec25a96172791fe16c28323ceb0c515f17bcd11da4fb183ffd7e6fbb769"),
			],
		},
		pallet_cml: CmlConfig {
			genesis_coupons,
			genesis_seeds,
		},
		pallet_genesis_bank: GenesisBankConfig {
			operation_account: genesis_bank_operation_account,
		},
		pallet_genesis_exchange: GenesisExchangeConfig {
			operation_account: genesis_exchange_operation_account,
			operation_usd_amount: INITIAL_EXCHANGE_USD_BALANCE,
			operation_tea_amount: INITIAL_EXCHANGE_TEA_BALANCE,
			competition_users,
		},
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

fn get_unique_accounts(genesis_coupons: &GenesisCoupons<AccountId>) -> Vec<AccountId> {
	let accounts: HashSet<AccountId> = genesis_coupons
		.coupons
		.iter()
		.map(|item| item.account.clone())
		.collect();
	accounts.iter().cloned().collect()
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

#[cfg(test)]
mod tests {
	use crate::chain_spec::get_unique_accounts;
	use camellia_runtime::pallet_cml::{
		CmlType, CouponConfig, DefrostScheduleType, GenesisCoupons,
	};
	use sp_runtime::AccountId32;

	#[test]
	fn get_unique_accounts_works() {
		let mut accounts = Vec::new();
		for i in 0..=9u8 {
			accounts.push([i; 32])
		}
		accounts.push(accounts[accounts.len() - 1]); // duplicate the last one

		let result = get_unique_accounts(&GenesisCoupons {
			coupons: accounts
				.iter()
				.map(|account| CouponConfig {
					account: AccountId32::new(account.clone()),
					cml_type: CmlType::A,
					schedule_type: DefrostScheduleType::Team,
					amount: 10,
				})
				.collect(),
		});

		assert_eq!(result.len(), 10);
		for i in 0..=9u8 {
			assert!(result.contains(&AccountId32::new([i; 32])));
		}
	}
}
