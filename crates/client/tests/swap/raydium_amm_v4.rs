use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::raydium_amm_v4::RAYDIUM_AMM_V4_PROGRAM_ID, SwapProtocol, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const AMM_ID: Address = address!("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2");
const AMM_AUTHORITY: Address = address!("5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1");
const AMM_COIN_VAULT: Address = address!("DQyrAcCrDXQ7NeoqGgDCZwBvWDcYmFCjSb9JtteuvPpz");
const AMM_PC_VAULT: Address = address!("HLmqeL62xR1QoZ1HKKbXRrdN1p3phKpxRMb2VVopvBBz");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_raydium_amm_v4_resolve_with_known_amm() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::RaydiumAmmV4 { amm: Some(AMM_ID) },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 9, "raydium amm v4 requires 9 accounts");

    // Protocol program
    assert_eq!(
        accounts[0].pubkey, RAYDIUM_AMM_V4_PROGRAM_ID,
        "raydium amm v4 program"
    );

    // Token program
    assert_eq!(accounts[1].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Amm id
    assert_eq!(accounts[2].pubkey, AMM_ID, "amm id");
    assert!(accounts[2].is_writable);

    // Amm authority
    assert_eq!(accounts[3].pubkey, AMM_AUTHORITY, "amm authority");

    // Amm coin vault
    assert_eq!(accounts[4].pubkey, AMM_COIN_VAULT, "amm coin vault");
    assert!(accounts[4].is_writable);

    // Amm pc vault
    assert_eq!(accounts[5].pubkey, AMM_PC_VAULT, "amm pc vault");
    assert!(accounts[5].is_writable);

    // User source token account
    let expected_user_source_token_account =
        get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[6].pubkey, expected_user_source_token_account,
        "user source token account"
    );
    assert!(accounts[6].is_writable);

    // User destination token account
    let expected_user_destination_token_account =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[7].pubkey, expected_user_destination_token_account,
        "user destination token account"
    );
    assert!(accounts[7].is_writable);

    // User wallet account
    assert_eq!(accounts[8].pubkey, user, "user wallet account");
    assert!(accounts[8].is_signer);
    assert!(accounts[8].is_writable);

    // Raydium AMM v4 has no extra data
    assert!(data.is_empty());
}

#[tokio::test]
async fn test_raydium_amm_v4_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::RaydiumAmmV4 { amm: Some(AMM_ID) },
        &USDC_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 9, "raydium amm v4 requires 9 accounts");

    // Protocol program
    assert_eq!(
        accounts[0].pubkey, RAYDIUM_AMM_V4_PROGRAM_ID,
        "raydium amm v4 program"
    );

    // Token program
    assert_eq!(accounts[1].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Amm id
    assert_eq!(accounts[2].pubkey, AMM_ID, "amm id");
    assert!(accounts[2].is_writable);

    // Amm authority
    assert_eq!(accounts[3].pubkey, AMM_AUTHORITY, "amm authority");

    // Amm coin vault
    assert_eq!(accounts[4].pubkey, AMM_COIN_VAULT, "amm coin vault");
    assert!(accounts[4].is_writable);

    // Amm pc vault
    assert_eq!(accounts[5].pubkey, AMM_PC_VAULT, "amm pc vault");
    assert!(accounts[5].is_writable);

    // User source token account
    let expected_user_source_token_account =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[6].pubkey, expected_user_source_token_account,
        "user source token account"
    );
    assert!(accounts[6].is_writable);

    // User destination token account
    let expected_user_destination_token_account =
        get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[7].pubkey, expected_user_destination_token_account,
        "user destination token account"
    );
    assert!(accounts[7].is_writable);

    // User wallet account
    assert_eq!(accounts[8].pubkey, user, "user wallet account");
    assert!(accounts[8].is_signer);
    assert!(accounts[8].is_writable);

    // Raydium AMM v4 has no extra data
    assert!(data.is_empty());
}
