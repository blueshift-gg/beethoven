use {
    beethoven_client::{
        get_associated_token_address, resolve_swap, swap::fusion_amm::FUSION_AMM_PROGRAM_ID,
        SwapProtocol, MEMO_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const FUSION_POOL: Address = address!("7VuKeevbvbQQcxz6N4SNLmuq6PYy4AcGQRDssoqo4t65");
const WSOL_VAULT: Address = address!("CYuiCBEhHLAYcDbFVtJ1KfgeQaQuN2sV18pNmzcDsbM7");
const USDC_VAULT: Address = address!("CjQWTPK84zwBq1PjVXmhmtKqhD9BfnMP7dcUFuN8Ljyd");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_fusion_amm_resolve_with_known_pool() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let sqrt_price_limit: u128 = 0;
    let amount_specified_is_input = true;
    let a_to_b = true;
    let remaining_accounts_info = None;

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::FusionAmm {
            fusion_pool: Some(FUSION_POOL),
            sqrt_price_limit,
            amount_specified_is_input,
            a_to_b,
            remaining_accounts_info,
        },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 15, "fusion amm requires 15 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, FUSION_AMM_PROGRAM_ID,
        "fusion amm program"
    );

    // Token program A
    assert_eq!(accounts[1].pubkey, TOKEN_PROGRAM_ID, "token program a");

    // Token program B
    assert_eq!(accounts[2].pubkey, TOKEN_PROGRAM_ID, "token program b");

    // Memo program
    assert_eq!(accounts[3].pubkey, MEMO_PROGRAM_ID, "memo program");

    // Token authority
    assert_eq!(accounts[4].pubkey, user, "token authority");
    assert!(accounts[4].is_signer);
    assert!(accounts[4].is_writable);

    // Fusion pool
    assert_eq!(accounts[5].pubkey, FUSION_POOL, "fusion pool");
    assert!(accounts[5].is_writable);

    // Token mint A
    assert_eq!(accounts[6].pubkey, WSOL_MINT, "token mint a");

    // Token mint B
    assert_eq!(accounts[7].pubkey, USDC_MINT, "token mint b");

    // Token owner account A
    let expected_token_owner_account_a =
        get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[8].pubkey, expected_token_owner_account_a,
        "token owner account a"
    );
    assert!(accounts[8].is_writable);

    // Token owner account B
    let expected_token_owner_account_b =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[9].pubkey, expected_token_owner_account_b,
        "token owner account b"
    );
    assert!(accounts[9].is_writable);

    // Token vault A
    assert_eq!(accounts[10].pubkey, WSOL_VAULT, "token vault a");
    assert!(accounts[10].is_writable);

    // Token vault B
    assert_eq!(accounts[11].pubkey, USDC_VAULT, "token vault b");
    assert!(accounts[11].is_writable);

    // Tick array 0
    assert!(accounts[12].is_writable);

    // Tick array 1
    assert!(accounts[13].is_writable);

    // Tick array 2
    assert!(accounts[14].is_writable);

    // swap extra data
    assert_eq!(data.len(), 19);

    // sqrt_price_limit
    assert_eq!(
        data[0..16],
        sqrt_price_limit.to_le_bytes(),
        "sqrt price limit"
    );

    // amount_specified_is_input
    assert_eq!(
        data[16], amount_specified_is_input as u8,
        "amount specified is input"
    );

    // a_to_b
    assert_eq!(data[17], a_to_b as u8, "a to b");

    // remaining_accounts_info - None
    assert_eq!(data[18], 0, "remaining accounts info - none");
}

#[tokio::test]
async fn test_fusion_amm_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let sqrt_price_limit: u128 = 0;
    let amount_specified_is_input = true;
    let a_to_b = false;
    let remaining_accounts_info = None;

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::FusionAmm {
            fusion_pool: Some(FUSION_POOL),
            sqrt_price_limit,
            amount_specified_is_input,
            a_to_b,
            remaining_accounts_info,
        },
        &USDC_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 15, "fusion amm requires 15 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, FUSION_AMM_PROGRAM_ID,
        "fusion amm program"
    );

    // Token program A
    assert_eq!(accounts[1].pubkey, TOKEN_PROGRAM_ID, "token program a");

    // Token program B
    assert_eq!(accounts[2].pubkey, TOKEN_PROGRAM_ID, "token program b");

    // Memo program
    assert_eq!(accounts[3].pubkey, MEMO_PROGRAM_ID, "memo program");

    // Token authority
    assert_eq!(accounts[4].pubkey, user, "token authority");
    assert!(accounts[4].is_signer);
    assert!(accounts[4].is_writable);

    // Fusion pool
    assert_eq!(accounts[5].pubkey, FUSION_POOL, "fusion pool");
    assert!(accounts[5].is_writable);

    // Token mint A
    assert_eq!(accounts[6].pubkey, WSOL_MINT, "token mint a");

    // Token mint B
    assert_eq!(accounts[7].pubkey, USDC_MINT, "token mint b");

    // Token owner account A
    let expected_token_owner_account_a =
        get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[8].pubkey, expected_token_owner_account_a,
        "token owner account a"
    );
    assert!(accounts[8].is_writable);

    // Token owner account B
    let expected_token_owner_account_b =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[9].pubkey, expected_token_owner_account_b,
        "token owner account b"
    );
    assert!(accounts[9].is_writable);

    // Token vault A
    assert_eq!(accounts[10].pubkey, WSOL_VAULT, "token vault a");
    assert!(accounts[10].is_writable);

    // Token vault B
    assert_eq!(accounts[11].pubkey, USDC_VAULT, "token vault b");
    assert!(accounts[11].is_writable);

    // Tick array 0
    assert!(accounts[12].is_writable);

    // Tick array 1
    assert!(accounts[13].is_writable);

    // Tick array 2
    assert!(accounts[14].is_writable);

    // swap extra data
    assert_eq!(data.len(), 19);

    // sqrt_price_limit
    assert_eq!(
        data[0..16],
        sqrt_price_limit.to_le_bytes(),
        "sqrt price limit"
    );

    // amount_specified_is_input
    assert_eq!(
        data[16], amount_specified_is_input as u8,
        "amount specified is input"
    );

    // a_to_b
    assert_eq!(data[17], a_to_b as u8, "a to b");

    // remaining_accounts_info - None
    assert_eq!(data[18], 0, "remaining accounts info - none");
}
