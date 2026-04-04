use {
    beethoven_client::{
        resolve_swap, swap::perena::PERENA_PROGRAM_ID, SwapProtocol, TOKEN_2022_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
    },
    solana_address::Address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const USDC_MINT: Address = Address::from_str_const("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDG_MINT: Address = Address::from_str_const("2u1tszSeqZ3qBWF3uNGPFc8TzMk2tdiwknnRMWGWjGWH");
const USDC_USDG_STABLE_POOL: Address =
    Address::from_str_const("5M7McNWX7yBBGrZGB6XhmHYhFwWwwB2ckrA1HEpkf3SA");
const POOL_USDC_VAULT: Address =
    Address::from_str_const("8XTxpDy7BjJkaoZxTiEzCwdwMad6RGBN6oXyfH2yRL7n");
const POOL_USDG_VAULT: Address =
    Address::from_str_const("BcjVG5To1pi3fHMpFoFdurcFwAoYJFzEtKP9ZTfqdjzT");
const NUMERAIRE_CONFIG: Address =
    Address::from_str_const("FS159v4b2jo3fjGBaUFmDzgx7k616XhpKhMwX2Q3HeeD");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_perena_resolve_with_known_pair() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Perena {
            pool: Some(USDC_USDG_STABLE_POOL),
            in_index: 0,
            out_index: 1,
        },
        &USDC_MINT,
        &USDG_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 12, "perena requires 12 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, PERENA_PROGRAM_ID, "perena program");

    // Pool
    assert_eq!(accounts[1].pubkey, USDC_USDG_STABLE_POOL, "pool");
    assert!(accounts[1].is_writable);

    // In mint
    assert_eq!(accounts[2].pubkey, USDC_MINT, "in mint");

    // Out mint
    assert_eq!(accounts[3].pubkey, USDG_MINT, "out mint");

    // In trader
    let expected_usdc_ata =
        beethoven_client::get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[4].pubkey, expected_usdc_ata, "in trader");
    assert!(accounts[4].is_writable);

    // Out trader
    let expected_usdg_ata =
        beethoven_client::get_associated_token_address(&user, &USDG_MINT, &TOKEN_2022_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_usdg_ata, "out trader");
    assert!(accounts[5].is_writable);

    // In vault
    assert_eq!(accounts[6].pubkey, POOL_USDC_VAULT, "in vault");
    assert!(accounts[6].is_writable);

    // Out vault
    assert_eq!(accounts[7].pubkey, POOL_USDG_VAULT, "out vault");
    assert!(accounts[7].is_writable);

    // Numeraire config
    assert_eq!(accounts[8].pubkey, NUMERAIRE_CONFIG, "numeraire config");
    assert!(!accounts[8].is_writable);

    // Payer
    assert_eq!(accounts[9].pubkey, user, "payer");
    assert!(accounts[9].is_signer);
    assert!(accounts[9].is_writable);

    // Token program
    assert_eq!(accounts[10].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Token 2022 program
    assert_eq!(
        accounts[11].pubkey, TOKEN_2022_PROGRAM_ID,
        "token 2022 program"
    );

    // in index = 0 (USDC), out index = 1 (USDG)
    assert_eq!(data, vec![0u8, 1u8]);
}

#[tokio::test]
async fn test_perena_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Perena {
            pool: Some(USDC_USDG_STABLE_POOL),
            in_index: 1,
            out_index: 0,
        },
        &USDG_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 12, "perena requires 12 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, PERENA_PROGRAM_ID, "perena program");

    // Pool
    assert_eq!(accounts[1].pubkey, USDC_USDG_STABLE_POOL, "pool");
    assert!(accounts[1].is_writable);

    // In mint
    assert_eq!(accounts[2].pubkey, USDG_MINT, "in mint");

    // Out mint
    assert_eq!(accounts[3].pubkey, USDC_MINT, "out mint");

    // In trader
    let expected_usdg_ata =
        beethoven_client::get_associated_token_address(&user, &USDG_MINT, &TOKEN_2022_PROGRAM_ID);
    assert_eq!(accounts[4].pubkey, expected_usdg_ata, "in trader");
    assert!(accounts[4].is_writable);

    // Out trader
    let expected_usdc_ata =
        beethoven_client::get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_usdc_ata, "out trader");
    assert!(accounts[5].is_writable);

    // In vault
    assert_eq!(accounts[6].pubkey, POOL_USDG_VAULT, "in vault");
    assert!(accounts[6].is_writable);

    // Out vault
    assert_eq!(accounts[7].pubkey, POOL_USDC_VAULT, "out vault");
    assert!(accounts[7].is_writable);

    // Numeraire config
    assert_eq!(accounts[8].pubkey, NUMERAIRE_CONFIG, "numeraire config");
    assert!(!accounts[8].is_writable);

    // Payer
    assert_eq!(accounts[9].pubkey, user, "payer");
    assert!(accounts[9].is_signer);
    assert!(accounts[9].is_writable);

    // Token program
    assert_eq!(accounts[10].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Token 2022 program
    assert_eq!(
        accounts[11].pubkey, TOKEN_2022_PROGRAM_ID,
        "token 2022 program"
    );

    // in index = 1 (USDG), out index = 0 (USDC)
    assert_eq!(data, vec![1u8, 0u8]);
}
