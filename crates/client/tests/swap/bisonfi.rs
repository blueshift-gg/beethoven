use {
    beethoven_client::{
        get_associated_token_address, resolve_swap, swap::bisonfi::BISONFI_PROGRAM_ID,
        SwapProtocol, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const MARKET: Address = address!("8FnX3xo2yYw3EUE6w3nQA4GfXGS9wpK6oj3veJpbFzLo");
const MARKET_TA_A: Address = address!("ATRsNGv2nDw7hSMfkUTBoVUDsFDwN7po7KbecyiGWNB4");
const MARKET_TA_B: Address = address!("2Y7HATmn9aJBcxCskE5V2U2epmjvkZmB51zTJBbhj4cU");
const DFLOW_LOGGER: Address = address!("8xeaWCsJYxRoudEZGJWURdfrtFhLYZz9b4iHJnW5tb3d");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_bisonfi_resolve_with_known_market_a_to_b() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let exact_out = false;
    let logger = DFLOW_LOGGER;

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Bisonfi {
            market: Some(MARKET),
            exact_out,
            logger,
        },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 10, "bisonfi requires 10 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, BISONFI_PROGRAM_ID, "bisonfi program");

    // User
    assert_eq!(accounts[1].pubkey, user, "user");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Market
    assert_eq!(accounts[2].pubkey, MARKET, "market");
    assert!(accounts[2].is_writable);

    // Market TA A
    assert_eq!(accounts[3].pubkey, MARKET_TA_A, "market_ta_a");
    assert!(accounts[3].is_writable);

    // Market TA B
    assert_eq!(accounts[4].pubkey, MARKET_TA_B, "market_ta_b");
    assert!(accounts[4].is_writable);

    // User ATA A
    let expected_user_ata_a = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_user_ata_a, "user_ata_a");
    assert!(accounts[5].is_writable);

    // User ATA B
    let expected_user_ata_b = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[6].pubkey, expected_user_ata_b, "user_ata_b");
    assert!(accounts[6].is_writable);

    // Token Program A
    assert_eq!(accounts[7].pubkey, TOKEN_PROGRAM_ID, "token_prog_a");

    // Token Program B
    assert_eq!(accounts[8].pubkey, TOKEN_PROGRAM_ID, "token_prog_b");

    // Logger
    assert_eq!(accounts[9].pubkey, logger, "logger");

    // b_to_a
    assert_eq!(data[0], 0u8, "b_to_a");

    // exact_out
    assert_eq!(data[1], exact_out as u8, "exact_out");
}

#[tokio::test]
async fn test_bisonfi_resolve_with_known_market_b_to_a() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let exact_out = false;
    let logger = DFLOW_LOGGER;

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Bisonfi {
            market: Some(MARKET),
            exact_out,
            logger,
        },
        &USDC_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 10, "bisonfi requires 10 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, BISONFI_PROGRAM_ID, "bisonfi program");

    // User
    assert_eq!(accounts[1].pubkey, user, "user");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Market
    assert_eq!(accounts[2].pubkey, MARKET, "market");
    assert!(accounts[2].is_writable);

    // Market TA A
    assert_eq!(accounts[3].pubkey, MARKET_TA_A, "market_ta_a");
    assert!(accounts[3].is_writable);

    // Market TA B
    assert_eq!(accounts[4].pubkey, MARKET_TA_B, "market_ta_b");
    assert!(accounts[4].is_writable);

    // User ATA A
    let expected_user_ata_a = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_user_ata_a, "user_ata_a");
    assert!(accounts[5].is_writable);

    // User ATA B
    let expected_user_ata_b = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[6].pubkey, expected_user_ata_b, "user_ata_b");
    assert!(accounts[6].is_writable);

    // Token Program A
    assert_eq!(accounts[7].pubkey, TOKEN_PROGRAM_ID, "token_prog_a");

    // Token Program B
    assert_eq!(accounts[8].pubkey, TOKEN_PROGRAM_ID, "token_prog_b");

    // Logger
    assert_eq!(accounts[9].pubkey, logger, "logger");

    // b_to_a
    assert_eq!(data[0], 1u8, "b_to_a");

    // exact_out
    assert_eq!(data[1], exact_out as u8, "exact_out");
}
