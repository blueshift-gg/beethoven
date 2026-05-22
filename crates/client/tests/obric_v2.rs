use {
    beethoven_client::{
        get_associated_token_address, resolve_swap, swap::obric_v2::OBRIC_V2_PROGRAM_ID,
        SwapProtocol, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDT_MINT: Address = address!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
const MARKET: Address = address!("BWBHrYqfcjAh5dSiRwzPnY4656cApXVXmkeDmAfwBKQG");
const SECOND_REF_ORACLE: Address = address!("GZsNmWKbqhMYtdSkkvMdEyQF9k5mLmP7tTKYWZjcHVPE");
const THIRD_REF_ORACLE: Address = address!("6YawcNeZ74tRyCv4UfGydYMr7eho7vbUR6ScVffxKAb3");
const RESERVE_X: Address = address!("C3tPQ8TRcHybnPpR8KMASUVD3PukQRRHEsLwxorJMhgm");
const RESERVE_Y: Address = address!("AAamGhyPfpQJWfZHTq944NM1cFvoVLDrQxt7HGjeRQUS");
const REF_ORACLE: Address = address!("J4HJYz4p7TRP96WVFky3vh7XryxoFehHjoRySUTeSeXw");
const X_PRICE_FEED: Address = address!("J4HJYz4p7TRP96WVFky3vh7XryxoFehHjoRySUTeSeXw");
const Y_PRICE_FEED: Address = address!("J4HJYz4p7TRP96WVFky3vh7XryxoFehHjoRySUTeSeXw");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_obric_v2_resolve_with_known_market_x_to_y() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::ObricV2 {
            market: Some(MARKET),
        },
        &USDC_MINT,
        &USDT_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 13, "obric v2 requires 13 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, OBRIC_V2_PROGRAM_ID, "obric v2 program");

    // Market
    assert_eq!(accounts[1].pubkey, MARKET, "market");
    assert!(accounts[1].is_writable);

    // Second ref oracle
    assert_eq!(accounts[2].pubkey, SECOND_REF_ORACLE, "second ref oracle");

    // Third ref oracle
    assert_eq!(accounts[3].pubkey, THIRD_REF_ORACLE, "third ref oracle");

    // Reserve x
    assert_eq!(accounts[4].pubkey, RESERVE_X, "reserve x");
    assert!(accounts[4].is_writable);

    // Reserve y
    assert_eq!(accounts[5].pubkey, RESERVE_Y, "reserve y");
    assert!(accounts[5].is_writable);

    // User ta x
    let expected_ta_x = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[6].pubkey, expected_ta_x, "user ta x");
    assert!(accounts[6].is_writable);

    // User ta y
    let expected_ta_y = get_associated_token_address(&user, &USDT_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[7].pubkey, expected_ta_y, "user ta y");
    assert!(accounts[7].is_writable);

    // Ref oracle
    assert_eq!(accounts[8].pubkey, REF_ORACLE, "ref oracle");

    // X price feed
    assert_eq!(accounts[9].pubkey, X_PRICE_FEED, "x price feed");

    // Y price feed
    assert_eq!(accounts[10].pubkey, Y_PRICE_FEED, "y price feed");

    // User
    assert_eq!(accounts[11].pubkey, user, "user");
    assert!(accounts[11].is_signer);
    assert!(accounts[11].is_writable);

    // Token program
    assert_eq!(accounts[12].pubkey, TOKEN_PROGRAM_ID, "token program");

    // x_to_y
    assert_eq!(data, vec![1_u8], "x_to_y");
}

#[tokio::test]
async fn test_obric_v2_resolve_with_known_market_y_to_x() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::ObricV2 {
            market: Some(MARKET),
        },
        &USDT_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 13, "obric v2 requires 13 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, OBRIC_V2_PROGRAM_ID, "obric v2 program");

    // Market
    assert_eq!(accounts[1].pubkey, MARKET, "market");
    assert!(accounts[1].is_writable);

    // Second ref oracle
    assert_eq!(accounts[2].pubkey, SECOND_REF_ORACLE, "second ref oracle");

    // Third ref oracle
    assert_eq!(accounts[3].pubkey, THIRD_REF_ORACLE, "third ref oracle");

    // Reserve x
    assert_eq!(accounts[4].pubkey, RESERVE_X, "reserve x");
    assert!(accounts[4].is_writable);

    // Reserve y
    assert_eq!(accounts[5].pubkey, RESERVE_Y, "reserve y");
    assert!(accounts[5].is_writable);

    // User ta x
    let expected_ta_x = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[6].pubkey, expected_ta_x, "user ta x");
    assert!(accounts[6].is_writable);

    // User ta y
    let expected_ta_y = get_associated_token_address(&user, &USDT_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[7].pubkey, expected_ta_y, "user ta y");
    assert!(accounts[7].is_writable);

    // Ref oracle
    assert_eq!(accounts[8].pubkey, REF_ORACLE, "ref oracle");

    // X price feed
    assert_eq!(accounts[9].pubkey, X_PRICE_FEED, "x price feed");

    // Y price feed
    assert_eq!(accounts[10].pubkey, Y_PRICE_FEED, "y price feed");

    // User
    assert_eq!(accounts[11].pubkey, user, "user");
    assert!(accounts[11].is_signer);
    assert!(accounts[11].is_writable);

    // Token program
    assert_eq!(accounts[12].pubkey, TOKEN_PROGRAM_ID, "token program");

    // x_to_y
    assert_eq!(data, vec![0_u8], "x_to_y");
}
