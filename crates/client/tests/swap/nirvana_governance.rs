use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::nirvana_governance::{
            ANA_MINT, BACKING_VAULT_MAIN, BACKING_VAULT_NIRV, ESCROW_REV_ANA,
            NIRVANA_GOVERNANCE_PROGRAM_ID, NIRV_MINT, PRICE_CURVE, TENANT, USDC_MINT,
        },
        SwapProtocol, TOKEN_PROGRAM_ID,
    },
    solana_address::Address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_nirvana_governance_resolve_buy() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::NirvanaGovernance { is_buy: true },
        &ANA_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(
        accounts.len(),
        14,
        "nirvana governance requires 14 accounts"
    );

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, NIRVANA_GOVERNANCE_PROGRAM_ID,
        "nirvana governance program"
    );

    // Payer
    assert_eq!(accounts[1].pubkey, user, "payer");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Tenant
    assert_eq!(accounts[2].pubkey, TENANT, "tenant");
    assert!(accounts[2].is_writable);

    // Price curve
    assert_eq!(accounts[3].pubkey, PRICE_CURVE, "price curve");

    // Mint ANA
    assert_eq!(accounts[4].pubkey, ANA_MINT, "mint ANA");
    assert!(accounts[4].is_writable);

    // Mint NIRV
    assert_eq!(accounts[5].pubkey, NIRV_MINT, "mint NIRV");

    // Mint main
    assert_eq!(accounts[6].pubkey, USDC_MINT, "mint main");

    // Backing vault main
    assert_eq!(accounts[7].pubkey, BACKING_VAULT_MAIN, "backing vault main");
    assert!(accounts[7].is_writable);

    // Backing vault NIRV
    assert_eq!(accounts[8].pubkey, BACKING_VAULT_NIRV, "backing vault NIRV");
    assert!(accounts[8].is_writable);

    // Escrow rev ANA
    assert_eq!(accounts[9].pubkey, ESCROW_REV_ANA, "escrow rev ANA");
    assert!(accounts[9].is_writable);

    // Backing src
    let expected_backing_src = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[10].pubkey, expected_backing_src, "backing src");
    assert!(accounts[10].is_writable);

    // Ana dst
    let expected_ana_dst = get_associated_token_address(&user, &ANA_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[11].pubkey, expected_ana_dst, "ana dst");
    assert!(accounts[11].is_writable);

    // Token program
    assert_eq!(accounts[12].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Token program main
    assert_eq!(accounts[13].pubkey, TOKEN_PROGRAM_ID, "token program main");

    // Buy has no extra data
    assert!(data.is_empty(), "buy has no extra data");
}

#[tokio::test]
async fn test_nirvana_governance_resolve_sell() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::NirvanaGovernance { is_buy: false },
        &ANA_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(
        accounts.len(),
        14,
        "nirvana governance requires 14 accounts"
    );

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, NIRVANA_GOVERNANCE_PROGRAM_ID,
        "nirvana governance program"
    );

    // Payer
    assert_eq!(accounts[1].pubkey, user, "payer");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Tenant
    assert_eq!(accounts[2].pubkey, TENANT, "tenant");
    assert!(accounts[2].is_writable);

    // Price curve
    assert_eq!(accounts[3].pubkey, PRICE_CURVE, "price curve");
    assert!(accounts[3].is_writable);

    // Mint ANA
    assert_eq!(accounts[4].pubkey, ANA_MINT, "mint ANA");
    assert!(accounts[4].is_writable);

    // Backing dst
    let expected_backing_dst = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_backing_dst, "backing dst");
    assert!(accounts[5].is_writable);

    // Escrow rev ana
    assert_eq!(accounts[6].pubkey, ESCROW_REV_ANA, "escrow rev ANA");
    assert!(accounts[6].is_writable);

    // Backing vault main
    assert_eq!(accounts[7].pubkey, BACKING_VAULT_MAIN, "backing vault main");
    assert!(accounts[7].is_writable);

    // Backing vault nirv
    assert_eq!(accounts[8].pubkey, BACKING_VAULT_NIRV, "backing vault NIRV");
    assert!(accounts[8].is_writable);

    // Ana src
    let expected_ana_src = get_associated_token_address(&user, &ANA_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[9].pubkey, expected_ana_src, "ana src");
    assert!(accounts[9].is_writable);

    // Mint nirv
    assert_eq!(accounts[10].pubkey, NIRV_MINT, "mint NIRV");

    // Mint main
    assert_eq!(accounts[11].pubkey, USDC_MINT, "mint main");

    // Token program
    assert_eq!(accounts[12].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Token program main
    assert_eq!(accounts[13].pubkey, TOKEN_PROGRAM_ID, "token program main");

    // Buy has no extra data
    assert!(data.is_empty(), "buy has no extra data");
}
