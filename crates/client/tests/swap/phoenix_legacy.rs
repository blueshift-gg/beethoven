use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::phoenix_legacy::{SelfTradeBehavior, Side, PHOENIX_LEGACY_PROGRAM_ID},
        SwapProtocol, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const LOG_AUTHORITY: Address = address!("7aDTsspkQNGKmrexAN7FLx9oxU3iPczSSvHNggyuqYkR");
const MARKET: Address = address!("4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg");
const BASE_VAULT: Address = address!("8g4Z9d6PqGkgH31tMW6FwxGhwYJrXpxZHQrkikpLJKrG");
const QUOTE_VAULT: Address = address!("3HSYXeGc3LjEPCuzoNDjQN37F1ebsSiR4CqXVqQCdekZ");

fn rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".into())
}

#[tokio::test]
async fn test_phoenix_legacy_resolve_with_known_market_bid() {
    let rpc = RpcClient::new(rpc_url());
    let user = Address::new_unique();

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::PhoenixLegacy {
            market: Some(MARKET),
            // bid
            side: 0,
        },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .expect("resolve phoenix legacy");

    assert_eq!(accounts.len(), 9, "phoenix legacy requires 9 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, PHOENIX_LEGACY_PROGRAM_ID,
        "phoenix v1 program"
    );

    // Log authority
    assert_eq!(accounts[1].pubkey, LOG_AUTHORITY, "log authority");

    // Market
    assert_eq!(accounts[2].pubkey, MARKET, "market");
    assert!(accounts[2].is_writable);

    // Trader
    assert_eq!(accounts[3].pubkey, user, "trader");
    assert!(accounts[3].is_signer);
    assert!(accounts[3].is_writable);

    // Base account
    let expected_base_account = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[4].pubkey, expected_base_account, "base account");
    assert!(accounts[4].is_writable);

    // Quote account
    let expected_quote_account = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_quote_account, "quote account");
    assert!(accounts[5].is_writable);

    // Base vault
    assert_eq!(accounts[6].pubkey, BASE_VAULT, "base vault");
    assert!(accounts[6].is_writable);

    // Quote vault
    assert_eq!(accounts[7].pubkey, QUOTE_VAULT, "quote vault");
    assert!(accounts[7].is_writable);

    // Token program
    assert_eq!(accounts[8].pubkey, TOKEN_PROGRAM_ID, "token program");

    // side
    assert_eq!(data[0], Side::Bid as u8);

    // price_in_ticks
    assert_eq!(&data[1..10], &[0u8; 9]);

    // max_counterpart_lots
    assert_eq!(&data[10..18], &u64::MAX.to_le_bytes());

    // self_trade_behavior
    assert_eq!(data[18], SelfTradeBehavior::CancelProvide as u8);

    // match_limit
    assert_eq!(&data[19..28], &[0u8; 9]);

    // client_order_id
    assert_eq!(&data[28..44], &0u128.to_le_bytes());

    // use_only_deposited_funds
    assert_eq!(data[44], 0);

    // last_valid_slot
    assert_eq!(&data[45..54], &[0u8; 9]);

    // last_valid_unix_timestamp_in_seconds
    assert_eq!(&data[54..63], &[0u8; 9]);
}

#[tokio::test]
async fn test_phoenix_legacy_resolve_with_known_market_ask() {
    let rpc = RpcClient::new(rpc_url());
    let user = Address::new_unique();

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::PhoenixLegacy {
            market: Some(MARKET),
            // ask
            side: 1,
        },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .expect("resolve phoenix legacy");

    assert_eq!(accounts.len(), 9, "phoenix legacy requires 9 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, PHOENIX_LEGACY_PROGRAM_ID,
        "phoenix v1 program"
    );

    // Log authority
    assert_eq!(accounts[1].pubkey, LOG_AUTHORITY, "log authority");

    // Market
    assert_eq!(accounts[2].pubkey, MARKET, "market");
    assert!(accounts[2].is_writable);

    // Trader
    assert_eq!(accounts[3].pubkey, user, "trader");
    assert!(accounts[3].is_signer);
    assert!(accounts[3].is_writable);

    // Base account
    let expected_base_account = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[4].pubkey, expected_base_account, "base account");
    assert!(accounts[4].is_writable);

    // Quote account
    let expected_quote_account = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_quote_account, "quote account");
    assert!(accounts[5].is_writable);

    // Base vault
    assert_eq!(accounts[6].pubkey, BASE_VAULT, "base vault");
    assert!(accounts[6].is_writable);

    // Quote vault
    assert_eq!(accounts[7].pubkey, QUOTE_VAULT, "quote vault");
    assert!(accounts[7].is_writable);

    // Token program
    assert_eq!(accounts[8].pubkey, TOKEN_PROGRAM_ID, "token program");

    // side
    assert_eq!(data[0], Side::Ask as u8);

    // price_in_ticks
    assert_eq!(&data[1..10], &[0u8; 9]);

    // max_counterpart_lots
    assert_eq!(&data[10..18], &0u64.to_le_bytes());

    // self_trade_behavior
    assert_eq!(data[18], SelfTradeBehavior::CancelProvide as u8);

    // match_limit
    assert_eq!(&data[19..28], &[0u8; 9]);

    // client_order_id
    assert_eq!(&data[28..44], &0u128.to_le_bytes());

    // use_only_deposited_funds
    assert_eq!(data[44], 0);

    // last_valid_slot
    assert_eq!(&data[45..54], &[0u8; 9]);

    // last_valid_unix_timestamp_in_seconds
    assert_eq!(&data[54..63], &[0u8; 9]);
}
