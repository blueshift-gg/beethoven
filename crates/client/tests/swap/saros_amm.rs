use {
    beethoven_client::{
        get_associated_token_address, resolve_swap, swap::saros_amm::SAROS_AMM_PROGRAM_ID,
        SwapProtocol, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const USDT_MINT: Address = address!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const SWAP_INFO: Address = address!("DtLM35DLrZTCPsh3WYRDi36528aZiNsTwXk71EJTjRFG");
const AUTHORITY_INFO: Address = address!("DSyCtcQDxy6y2iebzVdxpRXiiN6fumQozS3eVPnQgAmT");
const USDT_SWAP_SOURCE_INFO: Address = address!("gziKuBRMtdcHSzAQRJLFbkLQHB7DQKx1Hz83APnEQYT");
const USDC_SWAP_SOURCE_INFO: Address = address!("2PNC93VZsyd38QD23NApjhesUCMZSNcArsj4xzB74rQ3");
const POOL_MINT_INFO: Address = address!("9LHtzoDpKgqS7jMr4RHruTvxHDZKPcKnQvcbm4LUfpwN");
const POOL_FEE_ACCOUNT_INFO: Address = address!("CRRfsi4W5ZgyC2M79yWhma7CBZ9qgg8GHFiqU7poyy2f");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_saros_amm_resolve_with_known_pool() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::SarosAmm {
            swap_info: Some(SWAP_INFO),
        },
        &USDT_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 11, "saros amm requires 11 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, SAROS_AMM_PROGRAM_ID,
        "saros amm program"
    );

    // Swap info
    assert_eq!(accounts[1].pubkey, SWAP_INFO, "swap info");

    // Authority info
    assert_eq!(accounts[2].pubkey, AUTHORITY_INFO, "authority info");

    // User transfer authority info
    assert_eq!(accounts[3].pubkey, user, "user transfer authority info");
    assert!(accounts[3].is_signer);
    assert!(accounts[3].is_writable);

    // Source info
    let expected_source_info = get_associated_token_address(&user, &USDT_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[4].pubkey, expected_source_info, "source info");
    assert!(accounts[4].is_writable);

    // Swap source info
    assert_eq!(
        accounts[5].pubkey, USDT_SWAP_SOURCE_INFO,
        "swap source info"
    );
    assert!(accounts[5].is_writable);

    // Swap destination info
    assert_eq!(
        accounts[6].pubkey, USDC_SWAP_SOURCE_INFO,
        "swap destination info"
    );
    assert!(accounts[6].is_writable);

    // Destination info
    let expected_destination_info =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[7].pubkey, expected_destination_info,
        "destination info"
    );
    assert!(accounts[7].is_writable);

    // Pool mint info
    assert_eq!(accounts[8].pubkey, POOL_MINT_INFO, "pool mint info");
    assert!(accounts[8].is_writable);

    // Pool fee account info
    assert_eq!(
        accounts[9].pubkey, POOL_FEE_ACCOUNT_INFO,
        "pool fee account info"
    );
    assert!(accounts[9].is_writable);

    // Token program info
    assert_eq!(accounts[10].pubkey, TOKEN_PROGRAM_ID, "token program info");

    // Saros AMM has no extra data
    assert!(data.is_empty());
}

#[tokio::test]
async fn test_saros_amm_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    // Selling USDC for USDT — vaults and mints should be flipped
    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::SarosAmm {
            swap_info: Some(SWAP_INFO),
        },
        &USDC_MINT,
        &USDT_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 11, "saros amm requires 11 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, SAROS_AMM_PROGRAM_ID,
        "saros amm program"
    );

    // Swap info
    assert_eq!(accounts[1].pubkey, SWAP_INFO, "swap info");

    // Authority info
    assert_eq!(accounts[2].pubkey, AUTHORITY_INFO, "authority info");

    // User transfer authority info
    assert_eq!(accounts[3].pubkey, user, "user transfer authority info");
    assert!(accounts[3].is_signer);
    assert!(accounts[3].is_writable);

    // Source info
    let expected_source_info = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[4].pubkey, expected_source_info, "source info");
    assert!(accounts[4].is_writable);

    // Swap source info
    assert_eq!(
        accounts[5].pubkey, USDC_SWAP_SOURCE_INFO,
        "swap source info"
    );
    assert!(accounts[5].is_writable);

    // Swap destination info
    assert_eq!(
        accounts[6].pubkey, USDT_SWAP_SOURCE_INFO,
        "swap destination info"
    );
    assert!(accounts[6].is_writable);

    // Destination info
    let expected_destination_info =
        get_associated_token_address(&user, &USDT_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[7].pubkey, expected_destination_info,
        "destination info"
    );
    assert!(accounts[7].is_writable);

    // Pool mint info
    assert_eq!(accounts[8].pubkey, POOL_MINT_INFO, "pool mint info");
    assert!(accounts[8].is_writable);

    // Pool fee account info
    assert_eq!(
        accounts[9].pubkey, POOL_FEE_ACCOUNT_INFO,
        "pool fee account info"
    );
    assert!(accounts[9].is_writable);

    // Token program info
    assert_eq!(accounts[10].pubkey, TOKEN_PROGRAM_ID, "token program info");

    // Saros AMM has no extra data
    assert!(data.is_empty());
}
