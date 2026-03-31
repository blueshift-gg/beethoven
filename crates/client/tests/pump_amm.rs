use {
    beethoven_client::{
        resolve_swap, SwapProtocol, ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
    },
    solana_address::Address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = Address::from_str_const("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = Address::from_str_const("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

const PUMP_AMM_PROGRAM_ID: Address =
    Address::from_str_const("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");
const FEE_PROGRAM_ID: Address =
    Address::from_str_const("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_pump_amm_resolve_with_known_pool() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::PumpAmm {
            pool: None,
            track_volume: None,
        },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 24, "pump_amm requires 24 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, PUMP_AMM_PROGRAM_ID);

    // Pool
    assert!(accounts[1].is_writable);

    // User
    assert_eq!(accounts[2].pubkey, user);
    assert!(accounts[2].is_signer);
    assert!(accounts[2].is_writable);

    // Global config
    assert!(!accounts[3].is_writable);

    // Base mint
    assert_eq!(accounts[4].pubkey, USDC_MINT);

    // Quote mint
    assert_eq!(accounts[5].pubkey, WSOL_MINT);

    // User base token account
    let expected_usdc_ata =
        beethoven_client::get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[6].pubkey, expected_usdc_ata);
    assert!(accounts[6].is_writable);

    // User quote token account
    let expected_wsol_ata =
        beethoven_client::get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[7].pubkey, expected_wsol_ata);
    assert!(accounts[7].is_writable);

    let pool = accounts[1].pubkey;

    // Pool base token account
    let expected_pool_base_token_account =
        beethoven_client::get_associated_token_address(&pool, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[8].pubkey, expected_pool_base_token_account);
    assert!(accounts[8].is_writable);

    // Pool quote token account
    let expected_pool_quote_token_account =
        beethoven_client::get_associated_token_address(&pool, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[9].pubkey, expected_pool_quote_token_account);
    assert!(accounts[9].is_writable);

    // Protocol fee recipient
    assert!(!accounts[10].is_writable);

    let protocol_fee_recipient = accounts[10].pubkey;

    // Protocol fee recipient token account
    let expected_protocol_fee_recipient_token_account =
        beethoven_client::get_associated_token_address(
            &protocol_fee_recipient,
            &WSOL_MINT,
            &TOKEN_PROGRAM_ID,
        );
    assert_eq!(
        accounts[11].pubkey,
        expected_protocol_fee_recipient_token_account
    );
    assert!(accounts[11].is_writable);

    // Base token program
    assert_eq!(accounts[12].pubkey, TOKEN_PROGRAM_ID);

    // Quote token program
    assert_eq!(accounts[13].pubkey, TOKEN_PROGRAM_ID);

    // System program
    assert_eq!(accounts[14].pubkey, SYSTEM_PROGRAM_ID);

    // Associated token program
    assert_eq!(accounts[15].pubkey, ASSOCIATED_TOKEN_PROGRAM_ID);

    // Event authority
    let (expected_event_authority, _) =
        Address::find_program_address(&[b"__event_authority"], &PUMP_AMM_PROGRAM_ID);
    assert_eq!(accounts[16].pubkey, expected_event_authority);

    // Program
    assert_eq!(accounts[17].pubkey, PUMP_AMM_PROGRAM_ID);

    // Coin creator vault ATA
    assert!(accounts[18].is_writable);

    // Coin creator vault authority
    assert!(!accounts[19].is_writable);

    // Global volume accumulator
    assert!(!accounts[20].is_writable);

    // User volume accumulator
    assert!(accounts[21].is_writable);

    // Fee config
    assert!(!accounts[22].is_writable);

    // Fee program
    assert_eq!(accounts[23].pubkey, FEE_PROGRAM_ID);

    // Pump AMM has extra data
    assert_eq!(data, vec![0, 0]);
}
