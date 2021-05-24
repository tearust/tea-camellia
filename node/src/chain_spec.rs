use camellia_runtime::{
    constants::currency::DOLLARS, pallet_cml::Dai, AccountId, BabeConfig, Balance, BalancesConfig,
    CmlConfig, GenesisConfig, GrandpaConfig, Signature, SudoConfig, SystemConfig, TeaConfig,
    WASM_BINARY,
};
use hex_literal::hex;
use jsonrpc_core::serde_json;
use sc_service::{ChainType, Properties};
use sp_consensus_babe::AuthorityId as BabeId;
use sp_core::{crypto, sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use std::str::FromStr;
// use sp_core::crypto::Ss58Codec;

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

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(
    seed: &str,
) -> (
    AccountId,
    AccountId,
    BabeId,
    GrandpaId,
    // todo: uncomment ImOnlineId and AuthorityDiscoveryId later
    // ImOnlineId,
    // AuthorityDiscoveryId,
) {
    (
        get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<BabeId>(seed),
        get_from_seed::<GrandpaId>(seed),
        // get_from_seed::<ImOnlineId>(seed),
        // get_from_seed::<AuthorityDiscoveryId>(seed),
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

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
    let jacky_account =
        crypto::AccountId32::from_str("5EtQMJ6mYtuzgtXiWCW8AjjxdHe4K3CUAWVkgU3agb2oKMGs").unwrap();

    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![authority_keys_from_seed("Alice")],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                ],
                10000 * DOLLARS,
                vec![
                    (get_account_id_from_seed::<sr25519::Public>("Alice"), 1389),
                    (jacky_account.clone(), 1389),
                ],
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

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
                ],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
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
                ],
                1 << 60,
                vec![(get_account_id_from_seed::<sr25519::Public>("Alice"), 1389)],
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
    initial_authorities: Vec<(AccountId, AccountId, BabeId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    endowed_balance: Balance,
    dai_list: Vec<(AccountId, Dai)>,
) -> GenesisConfig {
    GenesisConfig {
        frame_system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        pallet_balances: BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, endowed_balance))
                .collect(),
        },
        pallet_babe: BabeConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.2.clone(), 1))
                .collect(),
            epoch_config: Some(camellia_runtime::BABE_GENESIS_EPOCH_CONFIG),
        },
        pallet_grandpa: GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.3.clone(), 1))
                .collect(),
        },
        pallet_sudo: SudoConfig {
            // Assign network admin rights.
            key: root_key,
        },

        pallet_tea: TeaConfig {
            builtin_nodes: vec![
                hex!("df38cb4f12479041c8e8d238109ef2a150b017f382206e24fee932e637c2db7b"),
                hex!("c7e016fad0796bb68594e49a6ef1942cf7e73497e69edb32d19ba2fab3696596"),
                hex!("2754d7e9c73ced5b302e12464594110850980027f8f83c469e8145eef59220b6"),
                hex!("c9380fde1ba795fc656ab08ab4ef4482cf554790fd3abcd4642418ae8fb5fd52"),
                hex!("bd1c0ec25a96172791fe16c28323ceb0c515f17bcd11da4fb183ffd7e6fbb769"),
            ],
        },
        pallet_cml: CmlConfig { dai_list },
    }
}
