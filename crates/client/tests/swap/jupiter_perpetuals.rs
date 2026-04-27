use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::jupiter_perpetuals::{
            EVENT_AUTHORITY, JLP_MINT, JUPITER_PERPETUALS_PROGRAM_ID, PERPETUALS, POOL,
            SOL_AG_PRICE_FEED, SOL_CUSTODY, SOL_VAULT, TRANSFER_AUTHORITY, USDC_AG_PRICE_FEED,
            USDC_CUSTODY, USDC_MINT, USDC_VAULT, USDT_AG_PRICE_FEED, USDT_CUSTODY,
            WBTC_AG_PRICE_FEED, WBTC_CUSTODY, WETH_AG_PRICE_FEED, WETH_CUSTODY, WSOL_MINT,
        },
        SwapProtocol, TOKEN_PROGRAM_ID,
    },
    solana_address::address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_jupiter_perpetuals_swap_2() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::JupiterPerpetuals,
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(
        accounts.len(),
        18,
        "jupiter perpetuals swap2 requires 18 accounts"
    );

    // Owner
    assert_eq!(accounts[0].pubkey, user);
    assert!(accounts[0].is_signer);
    assert!(accounts[0].is_writable);

    // Funding account
    let expected_funding_account =
        get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[1].pubkey, expected_funding_account);
    assert!(accounts[1].is_writable);

    // Receiving account
    let expected_receiving_account =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[2].pubkey, expected_receiving_account);
    assert!(accounts[2].is_writable);

    // Transfer authority
    assert_eq!(accounts[3].pubkey, TRANSFER_AUTHORITY);

    // Perpetuals
    assert_eq!(accounts[4].pubkey, PERPETUALS);

    // Pool
    assert_eq!(accounts[5].pubkey, POOL);
    assert!(accounts[5].is_writable);

    // Receiving custody
    assert_eq!(accounts[6].pubkey, SOL_CUSTODY);
    assert!(accounts[6].is_writable);

    // Receiving custody doves price account
    assert_eq!(accounts[7].pubkey, SOL_AG_PRICE_FEED);

    // Receiving custody pythnet price account
    assert_eq!(accounts[8].pubkey, SOL_AG_PRICE_FEED);

    // Receiving custody token account
    assert_eq!(accounts[9].pubkey, SOL_VAULT);
    assert!(accounts[9].is_writable);

    // Dispensing custody
    assert_eq!(accounts[10].pubkey, USDC_CUSTODY);
    assert!(accounts[10].is_writable);

    // Dispensing custody doves price account
    assert_eq!(accounts[11].pubkey, USDC_AG_PRICE_FEED);

    // Dispensing custody pythnet price account
    assert_eq!(accounts[12].pubkey, USDC_AG_PRICE_FEED);

    // Dispensing custody token account
    assert_eq!(accounts[13].pubkey, USDC_VAULT);
    assert!(accounts[13].is_writable);

    // Token program
    assert_eq!(accounts[14].pubkey, TOKEN_PROGRAM_ID);

    // Event authority
    assert_eq!(accounts[15].pubkey, EVENT_AUTHORITY);

    // Jupiter Perpetuals program
    assert_eq!(accounts[16].pubkey, JUPITER_PERPETUALS_PROGRAM_ID);

    // Jupiter Perpetuals has no extra data
    assert!(data.is_empty());
}

#[tokio::test]
async fn test_jupiter_perpetuals_add_liquidity_2() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::JupiterPerpetuals,
        &USDC_MINT,
        &JLP_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(
        accounts.len(),
        24,
        "jupiter perpetuals add liquidity2 requires 24 accounts"
    );

    // Owner
    assert_eq!(accounts[0].pubkey, user);
    assert!(accounts[0].is_signer);
    assert!(accounts[0].is_writable);

    // Funding account
    let expected_funding_account =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[1].pubkey, expected_funding_account);
    assert!(accounts[1].is_writable);

    // Lp token account
    let expected_lp_token_account =
        get_associated_token_address(&user, &JLP_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[2].pubkey, expected_lp_token_account);
    assert!(accounts[2].is_writable);

    // Transfer authority
    assert_eq!(accounts[3].pubkey, TRANSFER_AUTHORITY);

    // Perpetuals
    assert_eq!(accounts[4].pubkey, PERPETUALS);

    // Pool
    assert_eq!(accounts[5].pubkey, POOL);
    assert!(accounts[5].is_writable);

    // Custody
    assert_eq!(accounts[6].pubkey, USDC_CUSTODY);
    assert!(!accounts[6].is_writable);

    // Custody doves price account
    assert_eq!(accounts[7].pubkey, USDC_AG_PRICE_FEED);

    // Custody pythnet price account
    assert_eq!(accounts[8].pubkey, USDC_AG_PRICE_FEED);

    // Custody token account
    assert_eq!(accounts[9].pubkey, USDC_VAULT);
    assert!(!accounts[9].is_writable);

    // Lp token mint
    assert_eq!(accounts[10].pubkey, JLP_MINT);
    assert!(accounts[10].is_writable);

    // Token program
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID);

    // Event authority
    assert_eq!(accounts[12].pubkey, EVENT_AUTHORITY);

    // Jupiter Perpetuals program
    assert_eq!(accounts[13].pubkey, JUPITER_PERPETUALS_PROGRAM_ID);

    // SOL custody
    assert_eq!(accounts[14].pubkey, SOL_CUSTODY);

    // WETH custody
    assert_eq!(accounts[15].pubkey, WETH_CUSTODY);

    // WBTC custody
    assert_eq!(accounts[16].pubkey, WBTC_CUSTODY);

    // USDC custody
    assert_eq!(accounts[17].pubkey, USDC_CUSTODY);
    assert!(accounts[17].is_writable);

    // USDT custody
    assert_eq!(accounts[18].pubkey, USDT_CUSTODY);

    // SOL AG price feed
    assert_eq!(accounts[19].pubkey, SOL_AG_PRICE_FEED);

    // WETH AG price feed
    assert_eq!(accounts[20].pubkey, WETH_AG_PRICE_FEED);

    // WBTC AG price feed
    assert_eq!(accounts[21].pubkey, WBTC_AG_PRICE_FEED);

    // USDC AG price feed
    assert_eq!(accounts[22].pubkey, USDC_AG_PRICE_FEED);

    // USDT AG price feed
    assert_eq!(accounts[23].pubkey, USDT_AG_PRICE_FEED);

    // Jupiter Perpetuals has no extra data
    assert!(data.is_empty());
}

#[tokio::test]
async fn test_jupiter_perpetuals_remove_liquidity_2() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::JupiterPerpetuals,
        &JLP_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(
        accounts.len(),
        24,
        "jupiter perpetuals remove liquidity2 requires 24 accounts"
    );

    // Owner
    assert_eq!(accounts[0].pubkey, user);
    assert!(accounts[0].is_signer);
    assert!(accounts[0].is_writable);

    // Receiving account
    let expected_receiving_account =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[1].pubkey, expected_receiving_account);
    assert!(accounts[1].is_writable);

    // Lp token account
    let expected_lp_token_account =
        get_associated_token_address(&user, &JLP_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[2].pubkey, expected_lp_token_account);
    assert!(accounts[2].is_writable);

    // Transfer authority
    assert_eq!(accounts[3].pubkey, TRANSFER_AUTHORITY);

    // Perpetuals
    assert_eq!(accounts[4].pubkey, PERPETUALS);

    // Pool
    assert_eq!(accounts[5].pubkey, POOL);
    assert!(accounts[5].is_writable);

    // Custody
    assert_eq!(accounts[6].pubkey, USDC_CUSTODY);
    assert!(!accounts[6].is_writable);

    // Custody doves price account
    assert_eq!(accounts[7].pubkey, USDC_AG_PRICE_FEED);

    // Custody pythnet price account
    assert_eq!(accounts[8].pubkey, USDC_AG_PRICE_FEED);

    // Custody token account
    assert_eq!(accounts[9].pubkey, USDC_VAULT);
    assert!(!accounts[9].is_writable);

    // Lp token mint
    assert_eq!(accounts[10].pubkey, JLP_MINT);
    assert!(accounts[10].is_writable);

    // Token program
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID);

    // Event authority
    assert_eq!(accounts[12].pubkey, EVENT_AUTHORITY);

    // Jupiter Perpetuals program
    assert_eq!(accounts[13].pubkey, JUPITER_PERPETUALS_PROGRAM_ID);

    // SOL custody
    assert_eq!(accounts[14].pubkey, SOL_CUSTODY);

    // WETH custody
    assert_eq!(accounts[15].pubkey, WETH_CUSTODY);

    // WBTC custody
    assert_eq!(accounts[16].pubkey, WBTC_CUSTODY);

    // USDC custody
    assert_eq!(accounts[17].pubkey, USDC_CUSTODY);
    assert!(accounts[17].is_writable);

    // USDT custody
    assert_eq!(accounts[18].pubkey, USDT_CUSTODY);

    // SOL AG price feed
    assert_eq!(accounts[19].pubkey, SOL_AG_PRICE_FEED);

    // WETH AG price feed
    assert_eq!(accounts[20].pubkey, WETH_AG_PRICE_FEED);

    // WBTC AG price feed
    assert_eq!(accounts[21].pubkey, WBTC_AG_PRICE_FEED);

    // USDC AG price feed
    assert_eq!(accounts[22].pubkey, USDC_AG_PRICE_FEED);

    // USDT AG price feed
    assert_eq!(accounts[23].pubkey, USDT_AG_PRICE_FEED);

    // Jupiter Perpetuals has no extra data
    assert!(data.is_empty());
}
