### Add your own coupon for test

Modify the node/src/chain_spec.rs to add your own account;

```rust
pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
    let jacky_account =
        crypto::AccountId32::from_str("5EtQMJ6mYtuzgtXiWCW8AjjxdHe4K3CUAWVkgU3agb2oKMGs").unwrap();
    // let kevin_account =
    //     crypto::AccountId32::from_str("5DFzp6FGWRkqm8Pm1KX9dWLoVyukeS51cTiQPNqUADEZMFZq").unwrap();

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
                    // kevin_account.clone(),
                    jacky_account.clone(),
                ],
                10000 * DOLLARS,
                vec![
                    (get_account_id_from_seed::<sr25519::Public>("Alice"), 1389),
                    // (kevin_account.clone(), 1389),
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
```

Repalce the jacky_account to your own layer address.