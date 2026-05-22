use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::orca_whirlpool::{MEMO_PROGRAM_ID, WHIRLPOOL_PROGRAM_ID},
        SwapProtocol, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const WHIRLPOOL: Address = address!("HJPjoWUrhoZzkNfRpHuieeFk9WcZWjwy6PBjZ81ngndJ");
const SOL_VAULT: Address = address!("3YQm7ujtXWJU2e9jhp2QGHpnn1ShXn12QjvzMvDgabpX");
const USDC_VAULT: Address = address!("2JTw1fE2wz1SymWUQ7UqpVtrTuKjcd6mWwYwUJUCh2rq");
const ORACLE: Address = address!("4GkRbcYg1VKsZropgai4dMf2Nj2PkXNLf43knFpavrSi");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_orca_whirlpool_resolve_with_known_pool() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let sqrt_price_limit: u128 = 0;
    let amount_specified_is_input = true;
    let a_to_b = true;
    let remaining_accounts_info = None;

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::OrcaWhirlpool {
            whirlpool: Some(WHIRLPOOL),
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

    assert_eq!(accounts.len(), 17, "orca whirlpool requires 17 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, WHIRLPOOL_PROGRAM_ID,
        "whirlpool program"
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

    // Whirlpool
    assert_eq!(accounts[5].pubkey, WHIRLPOOL, "whirlpool");
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

    // Token vault A
    assert_eq!(accounts[9].pubkey, SOL_VAULT, "token vault a");
    assert!(accounts[9].is_writable);

    // Token owner account B
    let expected_token_owner_account_b =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[10].pubkey, expected_token_owner_account_b,
        "token owner account b"
    );
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

    // Oracle
    assert_eq!(accounts[15].pubkey, ORACLE, "oracle");
    assert!(accounts[15].is_writable);

    // swap_v2 has extra data
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
async fn test_orca_whirlpool_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let sqrt_price_limit: u128 = 0;
    let amount_specified_is_input = true;
    let a_to_b = false;
    let remaining_accounts_info = None;

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::OrcaWhirlpool {
            whirlpool: Some(WHIRLPOOL),
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

    assert_eq!(accounts.len(), 17, "orca whirlpool requires 17 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, WHIRLPOOL_PROGRAM_ID,
        "whirlpool program"
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

    // Whirlpool
    assert_eq!(accounts[5].pubkey, WHIRLPOOL, "whirlpool");
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

    // Token vault A
    assert_eq!(accounts[9].pubkey, SOL_VAULT, "token vault a");
    assert!(accounts[9].is_writable);

    // Token owner account B
    let expected_token_owner_account_b =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[10].pubkey, expected_token_owner_account_b,
        "token owner account b"
    );
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

    // Oracle
    assert_eq!(accounts[15].pubkey, ORACLE, "oracle");
    assert!(accounts[15].is_writable);

    // swap_v2 has extra data
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
