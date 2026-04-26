use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::meteora_dlmm::{EVENT_AUTHORITY, METEORA_DLMM_PROGRAM_ID},
        SwapProtocol, MEMO_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const LB_PAIR: Address = address!("BGm1tav58oGcsQJehL9WXBFXF7D27vZsKefj4xJKD5Y");
const BIN_ARRAY_BITMAP_EXTENSION: Address =
    address!("BzQsUBAbd21nrNDgc7D55EwnABC16uZJ41mgxxqYydHJ");
const RESERVE_X: Address = address!("DwZz4S1Z1LBXomzmncQRVKCYhjCqSAMQ6RPKbUAadr7H");
const RESERVE_Y: Address = address!("4N22J4vW2juHocTntJNmXywSonYjkndCwahjZ2cYLDgb");
const ORACLE: Address = address!("ETc6tqgLrr7wXsH8u2QBK1CyXHX3kvV6WQjBz4cf3sCj");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_meteora_dlmm_resolve_with_known_lb_pair() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::MeteoraDlmm {
            lb_pair: Some(LB_PAIR),
            bin_array_count: Some(16),
            transfer_hook_x_accounts: None,
            transfer_hook_y_accounts: None,
        },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .expect("Meteora DLMM resolve failed");

    assert!(
        accounts.len() >= 17,
        "meteora dlmm requires at least 17 accounts"
    );

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, METEORA_DLMM_PROGRAM_ID,
        "meteora dlmm program"
    );

    // LB pair
    assert_eq!(accounts[1].pubkey, LB_PAIR, "lb pair");
    assert!(accounts[1].is_writable);

    // Bin array bitmap extension
    assert_eq!(
        accounts[2].pubkey, BIN_ARRAY_BITMAP_EXTENSION,
        "bin array bitmap extension"
    );

    // Reserve x
    assert_eq!(accounts[3].pubkey, RESERVE_X, "reserve x");
    assert!(accounts[3].is_writable, "reserve_x");

    // Reserve y
    assert_eq!(accounts[4].pubkey, RESERVE_Y, "reserve y");
    assert!(accounts[4].is_writable);

    // User token in
    let expected_user_token_in = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_user_token_in, "user token in");
    assert!(accounts[5].is_writable);

    // User token out
    let expected_user_token_out =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[6].pubkey, expected_user_token_out,
        "user token out"
    );
    assert!(accounts[6].is_writable);

    // Token x mint
    assert_eq!(accounts[7].pubkey, WSOL_MINT, "token x mint");

    // Token y mint
    assert_eq!(accounts[8].pubkey, USDC_MINT, "token y mint");

    // Oracle
    assert_eq!(accounts[9].pubkey, ORACLE, "oracle");
    assert!(accounts[9].is_writable, "oracle");

    // Host fee in
    assert_eq!(accounts[10].pubkey, METEORA_DLMM_PROGRAM_ID, "host fee in");

    // User
    assert_eq!(accounts[11].pubkey, user, "user");
    assert!(accounts[11].is_signer);
    assert!(accounts[11].is_writable);

    // Token x program
    assert_eq!(accounts[12].pubkey, TOKEN_PROGRAM_ID, "token x program");

    // Token y program
    assert_eq!(accounts[13].pubkey, TOKEN_PROGRAM_ID, "token y program");

    // Memo program
    assert_eq!(accounts[14].pubkey, MEMO_PROGRAM_ID, "memo program");

    // Event authority
    assert_eq!(accounts[15].pubkey, EVENT_AUTHORITY, "event authority");

    // Program itself
    assert_eq!(accounts[16].pubkey, METEORA_DLMM_PROGRAM_ID, "dlmm program");

    // No transfer hook accounts

    // At least one bin array account
    assert!(accounts.len() >= 17, "at least one bin array account");

    // data is vec header of 0 len since there's no transfer hook accounts
    assert_eq!(data, vec![0, 0, 0, 0])
}

#[tokio::test]
async fn test_meteora_dlmm_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::MeteoraDlmm {
            lb_pair: Some(LB_PAIR),
            bin_array_count: Some(16),
            transfer_hook_x_accounts: None,
            transfer_hook_y_accounts: None,
        },
        &USDC_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .expect("Meteora DLMM resolve failed");

    assert!(
        accounts.len() >= 17,
        "meteora dlmm requires at least 17 accounts"
    );

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, METEORA_DLMM_PROGRAM_ID,
        "meteora dlmm program"
    );

    // LB pair
    assert_eq!(accounts[1].pubkey, LB_PAIR, "lb pair");
    assert!(accounts[1].is_writable);

    // Bin array bitmap extension
    assert_eq!(
        accounts[2].pubkey, BIN_ARRAY_BITMAP_EXTENSION,
        "bin array bitmap extension"
    );

    // Reserve x
    assert_eq!(accounts[3].pubkey, RESERVE_X, "reserve x");
    assert!(accounts[3].is_writable, "reserve_x");

    // Reserve y
    assert_eq!(accounts[4].pubkey, RESERVE_Y, "reserve y");
    assert!(accounts[4].is_writable);

    // User token in
    let expected_user_token_in = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_user_token_in, "user token in");
    assert!(accounts[5].is_writable);

    // User token out
    let expected_user_token_out =
        get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[6].pubkey, expected_user_token_out,
        "user token out"
    );
    assert!(accounts[6].is_writable);

    // Token x mint
    assert_eq!(accounts[7].pubkey, USDC_MINT, "token x mint");

    // Token y mint
    assert_eq!(accounts[8].pubkey, WSOL_MINT, "token y mint");

    // Oracle
    assert_eq!(accounts[9].pubkey, ORACLE, "oracle");
    assert!(accounts[9].is_writable, "oracle");

    // Host fee in
    assert_eq!(accounts[10].pubkey, METEORA_DLMM_PROGRAM_ID, "host fee in");

    // User
    assert_eq!(accounts[11].pubkey, user, "user");
    assert!(accounts[11].is_signer);
    assert!(accounts[11].is_writable);

    // Token x program
    assert_eq!(accounts[12].pubkey, TOKEN_PROGRAM_ID, "token x program");

    // Token y program
    assert_eq!(accounts[13].pubkey, TOKEN_PROGRAM_ID, "token y program");

    // Memo program
    assert_eq!(accounts[14].pubkey, MEMO_PROGRAM_ID, "memo program");

    // Event authority
    assert_eq!(accounts[15].pubkey, EVENT_AUTHORITY, "event authority");

    // Program itself
    assert_eq!(accounts[16].pubkey, METEORA_DLMM_PROGRAM_ID, "dlmm program");

    // No transfer hook accounts

    // At least one bin array account
    assert!(accounts.len() >= 17, "at least one bin array account");

    // data is vec header of 0 len since there's no transfer hook accounts
    assert_eq!(data, vec![0, 0, 0, 0])
}
