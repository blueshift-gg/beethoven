use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::hadron::{FEE_CONFIG_PDA, HADRON_PROGRAM_ID},
        SwapProtocol, SYSVAR_CLOCK_ID, SYSVAR_INSTRUCTIONS_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::Address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = Address::from_str_const("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = Address::from_str_const("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const POOL_CONFIG: Address =
    Address::from_str_const("3BKXKtD8oRgfbfYU997oHYjqcUcwZng8GUcEBPuqrM52");
const MIDPRICE_ORACLE: Address =
    Address::from_str_const("Es1rSVcMYxARBWBFwwuzaxYbHT9XvDviCyUdetHpJBvd");
const CURVE_META: Address = Address::from_str_const("7QiPDHa2KdgFxtMt8trPKoGL9LKgGf8Amj8s5RSqRgrY");
const CURVE_PREFABS: Address =
    Address::from_str_const("B5mqchgv1dMLGAdZuPPRGrZHH8hKkSpM1Sdq9452chUQ");
const VAULT_MINT_X: Address =
    Address::from_str_const("BAWQ8oUSHYpaA836gPJWanXipKU4tb7bAMRHjbzapa7p");
const VAULT_MINT_Y: Address =
    Address::from_str_const("69TeReLEXBAUKPokrwiU9Xw6CBDnKkqbRFGLmvJdWz6L");
const FEE_RECIPIENT: Address =
    Address::from_str_const("7fwqJJhGazsXcMhcwzAjtc2KsKNpbktsxDFbGN3RzYuQ");
const FEE_RECIPIENT_WSOL_ATA: Address =
    Address::from_str_const("5X6eBNjcfQYEZRQWE954ud3d5cR8ez4Ax3asyCuZgC7r");
const CURVE_UPDATES: Address =
    Address::from_str_const("AXV636YSCBadzqbC12DzHF6gMvEswHXsZQzw8aGM3vwo");
const SPREAD_CONFIG: Address =
    Address::from_str_const("Ee3nrioTwmfiAqkGpZUQQod6xNTfSfRxbGDuFMC7JSWY");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_hadron_resolve_with_known_pool() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");
    let expiration = 2_100_000_000i64;

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Hadron {
            config: POOL_CONFIG,
            fee_recipient: FEE_RECIPIENT,
            expiration,
        },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert!(
        accounts.len() == 16 || accounts.len() == 18,
        "hadron base layout is 16 accounts; +2 when spread_config is initialized (got {})",
        accounts.len()
    );

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, HADRON_PROGRAM_ID, "hadron program");

    // Token program x
    assert_eq!(accounts[1].pubkey, TOKEN_PROGRAM_ID, "token program x");

    // Token program y
    assert_eq!(accounts[2].pubkey, TOKEN_PROGRAM_ID, "token program y");

    // Config
    assert_eq!(accounts[3].pubkey, POOL_CONFIG, "config");

    // Midprice oracle
    assert_eq!(accounts[4].pubkey, MIDPRICE_ORACLE, "midprice oracle");

    // Curve meta
    assert_eq!(accounts[5].pubkey, CURVE_META, "curve meta");

    // Curve prefabs
    assert_eq!(accounts[6].pubkey, CURVE_PREFABS, "curve prefabs");
    assert!(accounts[6].is_writable);

    // User
    assert_eq!(accounts[7].pubkey, user, "user");
    assert!(accounts[7].is_signer);
    assert!(!accounts[7].is_writable);

    // User source
    let expected_wsol_ata = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[8].pubkey, expected_wsol_ata, "user source");
    assert!(accounts[8].is_writable);

    // Vault source
    assert_eq!(accounts[9].pubkey, VAULT_MINT_X, "vault source");
    assert!(accounts[9].is_writable);

    // Vault dest
    assert_eq!(accounts[10].pubkey, VAULT_MINT_Y, "vault dest");
    assert!(accounts[10].is_writable);

    // User dest
    let expected_usdc_ata = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[11].pubkey, expected_usdc_ata, "user dest");
    assert!(accounts[11].is_writable);

    // Fee config PDA
    assert_eq!(accounts[12].pubkey, FEE_CONFIG_PDA, "fee config PDA");

    // Fee recipient ATA
    assert_eq!(
        accounts[13].pubkey, FEE_RECIPIENT_WSOL_ATA,
        "fee recipient ATA (input mint = WSOL)"
    );
    assert!(accounts[13].is_writable);

    // Sysvar clock
    assert_eq!(accounts[14].pubkey, SYSVAR_CLOCK_ID, "sysvar clock");

    // Curve updates
    assert_eq!(accounts[15].pubkey, CURVE_UPDATES, "curve updates");
    assert!(accounts[15].is_writable);

    if accounts.len() == 18 {
        // Spread config
        assert_eq!(accounts[16].pubkey, SPREAD_CONFIG, "spread config");

        // Sysvar instructions
        assert_eq!(
            accounts[17].pubkey,
            beethoven_client::SYSVAR_INSTRUCTIONS_ID,
            "sysvar instructions"
        );
    }

    assert_eq!(data.len(), 9);
    assert_eq!(data[0], 1u8, "is_x = selling pool mint X (WSOL)");
    assert_eq!(data[1..9], expiration.to_le_bytes(), "expiration le bytes");
}

#[tokio::test]
async fn test_hadron_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");
    let expiration = 2_100_000_000i64;

    // Selling USDC for WSOL — is_x = 0; vaults and user ATAs swap roles
    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Hadron {
            config: POOL_CONFIG,
            fee_recipient: FEE_RECIPIENT,
            expiration,
        },
        &USDC_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert!(
        accounts.len() == 16 || accounts.len() == 18,
        "hadron base layout is 16 accounts; +2 when spread_config is initialized (got {})",
        accounts.len()
    );

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, HADRON_PROGRAM_ID, "hadron program");

    // Token program x
    assert_eq!(accounts[1].pubkey, TOKEN_PROGRAM_ID, "token program x");

    // Token program y
    assert_eq!(accounts[2].pubkey, TOKEN_PROGRAM_ID, "token program y");

    // Config
    assert_eq!(accounts[3].pubkey, POOL_CONFIG, "config");

    // Midprice oracle (PDAs use canonical pool mint order)
    assert_eq!(accounts[4].pubkey, MIDPRICE_ORACLE, "midprice oracle");

    // Curve meta
    assert_eq!(accounts[5].pubkey, CURVE_META, "curve meta");

    // Curve prefabs
    assert_eq!(accounts[6].pubkey, CURVE_PREFABS, "curve prefabs");
    assert!(accounts[6].is_writable);

    // User
    assert_eq!(accounts[7].pubkey, user, "user");
    assert!(accounts[7].is_signer);
    assert!(!accounts[7].is_writable);

    // User source
    let expected_usdc_ata = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[8].pubkey, expected_usdc_ata, "user source");
    assert!(accounts[8].is_writable);

    // Vault source
    assert_eq!(accounts[9].pubkey, VAULT_MINT_Y, "vault source");
    assert!(accounts[9].is_writable);

    // Vault dest
    assert_eq!(accounts[10].pubkey, VAULT_MINT_X, "vault dest");
    assert!(accounts[10].is_writable);

    // User dest
    let expected_wsol_ata = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[11].pubkey, expected_wsol_ata, "user dest");
    assert!(accounts[11].is_writable);

    // Fee config PDA
    assert_eq!(accounts[12].pubkey, FEE_CONFIG_PDA, "fee config PDA");

    // Fee recipient ATA
    let fee_recipient_usdc_ata =
        get_associated_token_address(&FEE_RECIPIENT, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[13].pubkey, fee_recipient_usdc_ata,
        "fee recipient ATA (input mint = USDC)"
    );
    assert!(accounts[13].is_writable);

    // Sysvar clock
    assert_eq!(accounts[14].pubkey, SYSVAR_CLOCK_ID, "sysvar clock");

    // Curve updates
    assert_eq!(accounts[15].pubkey, CURVE_UPDATES, "curve updates");
    assert!(accounts[15].is_writable);

    if accounts.len() == 18 {
        // Spread config
        assert_eq!(accounts[16].pubkey, SPREAD_CONFIG, "spread config");

        // Sysvar instructions
        assert_eq!(
            accounts[17].pubkey, SYSVAR_INSTRUCTIONS_ID,
            "sysvar instructions"
        );
    }

    assert_eq!(data.len(), 9);
    assert_eq!(data[0], 0u8, "is_x = selling pool mint Y (USDC)");
    assert_eq!(data[1..9], expiration.to_le_bytes(), "expiration le bytes");
}
