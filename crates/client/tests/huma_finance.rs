use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::huma_finance::{
            HUMA_CONFIG, HUMA_FINANCE_PROGRAM_ID, MODE_CONFIG, POOL_AUTHORITY, POOL_CONFIG,
            POOL_STATE, POOL_UNDERLYING_TOKEN, PST_MINT,
        },
        SwapProtocol, TOKEN_PROGRAM_ID,
    },
    solana_address::Address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const USDC_MINT: Address = Address::from_str_const("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_huma_finance_resolve_deposit() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::HumaFinance,
        &USDC_MINT,
        &PST_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(
        accounts.len(),
        14,
        "huma finance deposit requires 14 accounts"
    );

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, HUMA_FINANCE_PROGRAM_ID,
        "huma finance program"
    );

    // Depositor
    assert_eq!(accounts[1].pubkey, user, "depositor");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Huma config
    assert_eq!(accounts[2].pubkey, HUMA_CONFIG, "huma config");

    // Pool config
    assert_eq!(accounts[3].pubkey, POOL_CONFIG, "pool config");

    // Pool state
    assert_eq!(accounts[4].pubkey, POOL_STATE, "pool state");
    assert!(accounts[4].is_writable);

    // Mode config
    assert_eq!(accounts[5].pubkey, MODE_CONFIG, "mode config");

    // Mode mint (PST)
    assert_eq!(accounts[6].pubkey, PST_MINT, "mode mint");
    assert!(accounts[6].is_writable);

    // Pool authority
    assert_eq!(accounts[7].pubkey, POOL_AUTHORITY, "pool authority");

    // Underlying mint
    assert_eq!(accounts[8].pubkey, USDC_MINT, "underlying mint");

    // Pool underlying token
    assert_eq!(
        accounts[9].pubkey, POOL_UNDERLYING_TOKEN,
        "pool underlying token"
    );
    assert!(accounts[9].is_writable);

    // Depositor underlying token
    let expected_depositor_underlying_token =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[10].pubkey, expected_depositor_underlying_token,
        "depositor underlying token"
    );
    assert!(accounts[10].is_writable);

    // Depositor mode token
    let expected_depositor_mode_token =
        get_associated_token_address(&user, &PST_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[11].pubkey, expected_depositor_mode_token,
        "depositor mode token"
    );
    assert!(accounts[11].is_writable);

    // Underlying token program
    assert_eq!(
        accounts[12].pubkey, TOKEN_PROGRAM_ID,
        "underlying token program"
    );

    // Mode token program
    assert_eq!(accounts[13].pubkey, TOKEN_PROGRAM_ID, "mode token program");

    // no extra data
    assert!(data.is_empty(), "deposit has no extra data");
}
