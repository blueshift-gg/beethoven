use {
    beethoven_client::{
        get_associated_token_address, resolve_swap, swap::zerofi::ZEROFI_PROGRAM_ID, SwapProtocol,
        SYSVAR_INSTRUCTIONS_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const MARKET: Address = address!("AWguet57BQuPftMuiV6vY89TQCQXxyTvQQ4QmoS7K2Mt");
const CFG_SOL: Address = address!("7RHJ2WfexqUxy7SXfbNZRZDgZi3D9jtMAQp9VhfzpU8T");
const SOL_VAULT: Address = address!("ERP5RTV6cWmoGrv7r9W2V5pbgDFSepc4j97qNnx1Jris");
const CFG_USDC: Address = address!("Ef7zPqj4NuZHwaTczUTY9oRbxXrfZseUcKcqPaidCZ5W");
const USDC_VAULT: Address = address!("7wYJVD8iXmMQjND1fwi1hPr68QwruVVtirbotyJZXaVH");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_zerofi_resolve_with_known_market() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let cfg_in = CFG_SOL;
    let cfg_out = CFG_USDC;

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Zerofi {
            market: Some(MARKET),
            cfg_in,
            cfg_out,
        },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 10, "zerofi requires 10 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, ZEROFI_PROGRAM_ID);

    // Market
    assert_eq!(accounts[1].pubkey, MARKET);
    assert!(accounts[1].is_writable);

    // Cfg in
    assert_eq!(accounts[2].pubkey, cfg_in);
    assert!(accounts[2].is_writable);

    // Ta in
    assert_eq!(accounts[3].pubkey, SOL_VAULT);
    assert!(accounts[3].is_writable);

    // Cfg out
    assert_eq!(accounts[4].pubkey, cfg_out);
    assert!(accounts[4].is_writable);

    // Ta out
    assert_eq!(accounts[5].pubkey, USDC_VAULT);
    assert!(accounts[5].is_writable);

    // Usr ta in
    let expected_usr_ta_in = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[6].pubkey, expected_usr_ta_in);
    assert!(accounts[6].is_writable);

    // Usr ta out
    let expected_usr_ta_out = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[7].pubkey, expected_usr_ta_out);
    assert!(accounts[7].is_writable);

    // Token program
    assert_eq!(accounts[8].pubkey, TOKEN_PROGRAM_ID);

    // Sysvar instructions
    assert_eq!(accounts[9].pubkey, SYSVAR_INSTRUCTIONS_ID);

    // swap has no extra data
    assert!(data.is_empty());
}

#[tokio::test]
async fn test_zerofi_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let cfg_in = CFG_USDC;
    let cfg_out = CFG_SOL;

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Zerofi {
            market: Some(MARKET),
            cfg_in,
            cfg_out,
        },
        &USDC_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 10, "zerofi requires 10 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, ZEROFI_PROGRAM_ID);

    // Market
    assert_eq!(accounts[1].pubkey, MARKET);
    assert!(accounts[1].is_writable);

    // Cfg in
    assert_eq!(accounts[2].pubkey, cfg_in);
    assert!(accounts[2].is_writable);

    // Ta in
    assert_eq!(accounts[3].pubkey, USDC_VAULT);
    assert!(accounts[3].is_writable);

    // Cfg out
    assert_eq!(accounts[4].pubkey, cfg_out);
    assert!(accounts[4].is_writable);

    // Ta out
    assert_eq!(accounts[5].pubkey, SOL_VAULT);
    assert!(accounts[5].is_writable);

    // Usr ta in
    let expected_usr_ta_in = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[6].pubkey, expected_usr_ta_in);
    assert!(accounts[6].is_writable);

    // Usr ta out
    let expected_usr_ta_out = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[7].pubkey, expected_usr_ta_out);
    assert!(accounts[7].is_writable);

    // Token program
    assert_eq!(accounts[8].pubkey, TOKEN_PROGRAM_ID);

    // Sysvar instructions
    assert_eq!(accounts[9].pubkey, SYSVAR_INSTRUCTIONS_ID);

    // swap has no extra data
    assert!(data.is_empty());
}
