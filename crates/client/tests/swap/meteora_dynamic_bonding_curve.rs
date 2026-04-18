use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::meteora_dynamic_bonding_curve::{
            SwapMode, EVENT_AUTHORITY, METEORA_DYNAMIC_BONDING_CURVE_PROGRAM_ID, POOL_AUTHORITY,
        },
        SwapProtocol, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const BASE_MINT: Address = address!("GSg4JktbLkn3k2rWrPwfKdFyt4v8PBt6L6MRoFLymoon");
const CONFIG: Address = address!("FbKf76ucsQssF7XZBuzScdJfugtsSKwZFYztKsMEhWZM");
const POOL: Address = address!("Buazd488xG6HofYP2T9ZJLerBMghJftymfYYvu1FP3ck");
const BASE_VAULT: Address = address!("P8xdtARQT7GZCxYYtGfaPRxQDVrYejEETNEKcbgpW7U");
const QUOTE_VAULT: Address = address!("GChqf6Ehx9iufcjHJj1kvnEQn2n4QYX5HozV3vioUmb7");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_meteora_dynamic_bonding_curve_resolve_with_known_pool() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let swap_mode = SwapMode::ExactIn;

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::MeteoraDynamicBondingCurve {
            pool: Some(POOL),
            referral_token_account: None,
            swap_mode,
        },
        &BASE_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(
        accounts.len(),
        16,
        "meteora dynamic bonding curve requires 16 accounts"
    );

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, METEORA_DYNAMIC_BONDING_CURVE_PROGRAM_ID,
        "meteora dynamic bonding curve program"
    );

    // Pool authority
    assert_eq!(accounts[1].pubkey, POOL_AUTHORITY, "pool authority");

    // Config
    assert_eq!(accounts[2].pubkey, CONFIG, "config");

    // Pool
    assert_eq!(accounts[3].pubkey, POOL, "pool");
    assert!(accounts[3].is_writable);

    // Input token account
    let expected_input_token_account =
        get_associated_token_address(&user, &BASE_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[4].pubkey, expected_input_token_account,
        "input token account"
    );
    assert!(accounts[4].is_writable);

    // Output token account
    let expected_output_token_account =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[5].pubkey, expected_output_token_account,
        "output token account"
    );
    assert!(accounts[5].is_writable);

    // Base vault
    assert_eq!(accounts[6].pubkey, BASE_VAULT, "base vault");
    assert!(accounts[6].is_writable);

    // Quote vault
    assert_eq!(accounts[7].pubkey, QUOTE_VAULT, "quote vault");
    assert!(accounts[7].is_writable);

    // Base mint
    assert_eq!(accounts[8].pubkey, BASE_MINT, "base mint");

    // Quote mint
    assert_eq!(accounts[9].pubkey, USDC_MINT, "quote mint");

    // Payer
    assert_eq!(accounts[10].pubkey, user, "payer");
    assert!(accounts[10].is_signer);

    // Token base program
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "token base program");

    // Token quote program
    assert_eq!(accounts[12].pubkey, TOKEN_PROGRAM_ID, "token quote program");

    // Referral token account
    assert_eq!(
        accounts[13].pubkey, METEORA_DYNAMIC_BONDING_CURVE_PROGRAM_ID,
        "referral token account"
    );

    // Event authority
    assert_eq!(accounts[14].pubkey, EVENT_AUTHORITY, "event authority");

    // Program
    assert_eq!(
        accounts[15].pubkey, METEORA_DYNAMIC_BONDING_CURVE_PROGRAM_ID,
        "program"
    );

    // swap_mode
    assert_eq!(data[0], swap_mode as u8);
}
