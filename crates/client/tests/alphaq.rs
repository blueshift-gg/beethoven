use {
    beethoven_client::{
        get_associated_token_address, resolve_swap, swap::alphaq::ALPHAQ_PROGRAM_ID, SwapProtocol,
        SYSVAR_INSTRUCTIONS_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const USDT_MINT: Address = address!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const MARKET: Address = address!("Pi9nzTjPxD8DsRfRBGfKYzmefJoJM8TcXu2jyaQjSHm");
const MARKET_STATE: Address = address!("445fd6ffBZqWYsryCgs6wcE8exaLkRsMrefAQ5UHvt8v");
const VAULT_TA_A: Address = address!("GF8SKKobum6UJnhX2mLHePU38htg5vdr9zcY4jH8Pqs2");
const VAULT_TA_B: Address = address!("F2KCaXcp7AoQtxTDvNEDCyMyWjSCAMWNzcyN9dsPfPs5");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_alphaq_resolve_with_known_market_a_to_b() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::AlphaQ {
            market: Some(MARKET),
            market_state: MARKET_STATE,
        },
        &USDT_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 13, "alphaq requires 13 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, ALPHAQ_PROGRAM_ID, "alphaq program");

    // User
    assert_eq!(accounts[1].pubkey, user, "user");
    assert!(accounts[1].is_writable);
    assert!(accounts[1].is_signer);

    // Market
    assert_eq!(accounts[2].pubkey, MARKET, "market");
    assert!(accounts[2].is_writable);

    // Market state
    assert_eq!(accounts[3].pubkey, MARKET_STATE, "market_state");
    assert!(accounts[3].is_writable);

    // User ata a
    let expected_usdt_ata = get_associated_token_address(&user, &USDT_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[4].pubkey, expected_usdt_ata, "user_ata_a");
    assert!(accounts[4].is_writable);

    // User ata b
    let expected_usdc_ata = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_usdc_ata, "user_ata_b");
    assert!(accounts[5].is_writable);

    // Vault ta a
    assert_eq!(accounts[6].pubkey, VAULT_TA_A, "vault_ta_a");
    assert!(accounts[6].is_writable);

    // Vault ta b
    assert_eq!(accounts[7].pubkey, VAULT_TA_B, "vault_ta_b");
    assert!(accounts[7].is_writable);

    // Token authority a
    assert_eq!(accounts[8].pubkey, VAULT_TA_A, "token_authority_a");
    assert!(accounts[8].is_writable);

    // Token authority b
    assert_eq!(accounts[9].pubkey, VAULT_TA_B, "token_authority_b");
    assert!(accounts[9].is_writable);

    // Vendor key
    assert_eq!(accounts[10].pubkey, VAULT_TA_B, "vendor_key");
    assert!(accounts[10].is_writable);

    // Token program
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "token_program");

    // Instructions sysvar
    assert_eq!(
        accounts[12].pubkey, SYSVAR_INSTRUCTIONS_ID,
        "instructions_sysvar"
    );

    // a_to_b
    assert_eq!(data, vec![1u8], "a_to_b");
}

#[tokio::test]
async fn test_alphaq_resolve_with_known_market_b_to_a() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::AlphaQ {
            market: Some(MARKET),
            market_state: MARKET_STATE,
        },
        &USDC_MINT,
        &USDT_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 13, "alphaq requires 13 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, ALPHAQ_PROGRAM_ID, "alphaq program");

    // User
    assert_eq!(accounts[1].pubkey, user, "user");
    assert!(accounts[1].is_writable);
    assert!(accounts[1].is_signer);

    // Market
    assert_eq!(accounts[2].pubkey, MARKET, "market");
    assert!(accounts[2].is_writable);

    // Market state
    assert_eq!(accounts[3].pubkey, MARKET_STATE, "market_state");
    assert!(accounts[3].is_writable);

    // User ata a
    let expected_usdt_ata = get_associated_token_address(&user, &USDT_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[4].pubkey, expected_usdt_ata, "user_ata_a");
    assert!(accounts[4].is_writable);

    // User ata b
    let expected_usdc_ata = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_usdc_ata, "user_ata_b");
    assert!(accounts[5].is_writable);

    // Vault ta a
    assert_eq!(accounts[6].pubkey, VAULT_TA_A, "vault_ta_a");
    assert!(accounts[6].is_writable);

    // Vault ta b
    assert_eq!(accounts[7].pubkey, VAULT_TA_B, "vault_ta_b");
    assert!(accounts[7].is_writable);

    // Token authority a
    assert_eq!(accounts[8].pubkey, VAULT_TA_A, "token_authority_a");
    assert!(accounts[8].is_writable);

    // Token authority b
    assert_eq!(accounts[9].pubkey, VAULT_TA_B, "token_authority_b");
    assert!(accounts[9].is_writable);

    // Vendor key
    assert_eq!(accounts[10].pubkey, VAULT_TA_B, "vendor_key");
    assert!(accounts[10].is_writable);

    // Token program
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "token_program");

    // Instructions sysvar
    assert_eq!(
        accounts[12].pubkey, SYSVAR_INSTRUCTIONS_ID,
        "instructions_sysvar"
    );

    // a_to_b
    assert_eq!(data, vec![0u8], "a_to_b");
}
