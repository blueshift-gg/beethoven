use {
    beethoven_client::{
        get_associated_token_address, resolve_swap, swap::byreal_clmm::BYREAL_CLMM_PROGRAM_ID,
        SwapProtocol, MEMO_PROGRAM_ID, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const AMM_CONFIG: Address = address!("4E6xP73xzTs4aCvY92hbXRwWkYptNwvViPZmLcZEBUk4");
const POOL_STATE: Address = address!("9GTj99g9tbz9U6UYDsX6YeRTgUnkYG6GTnHv3qLa5aXq");
const SOL_VAULT: Address = address!("5BzogZvHNEuwstR4iwTWdd7jknFBZqJQWVjxPsDfEUD6");
const USDC_VAULT: Address = address!("HL8turx8hJEEPVH4ivxzxwfdxVA1PH3LeYbSmh3hYfzz");
const OBSERVATION_STATE: Address = address!("3T6qNbQqWYDfSTew1ifsNedtoeDP8LRuCegmUH27ykEZ");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_byreal_clmm_resolve_with_known_pool() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let sqrt_price_limit_x64: u128 = 0;
    let is_base_input = true;

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::ByrealClmm {
            pool: Some(POOL_STATE),
            sqrt_price_limit_x64,
            is_base_input,
        },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 14, "byreal clmm requires 14 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, BYREAL_CLMM_PROGRAM_ID,
        "byreal clmm program"
    );

    // Payer
    assert_eq!(accounts[1].pubkey, user, "payer");
    assert!(accounts[1].is_signer);

    // Amm config
    assert_eq!(accounts[2].pubkey, AMM_CONFIG, "amm config");

    // Pool state
    assert_eq!(accounts[3].pubkey, POOL_STATE, "pool state");
    assert!(accounts[3].is_writable);

    // Input token account
    let expected_wsol_ata = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[4].pubkey, expected_wsol_ata, "input token account");
    assert!(accounts[4].is_writable);

    // Output token account
    let expected_usdc_ata = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[5].pubkey, expected_usdc_ata,
        "output token account"
    );
    assert!(accounts[5].is_writable);

    // Input vault
    assert_eq!(accounts[6].pubkey, SOL_VAULT, "input vault");
    assert!(accounts[6].is_writable);

    // Output vault
    assert_eq!(accounts[7].pubkey, USDC_VAULT, "output vault");
    assert!(accounts[7].is_writable);

    // Observation state
    assert_eq!(accounts[8].pubkey, OBSERVATION_STATE, "observation state");
    assert!(accounts[8].is_writable);

    // Token program
    assert_eq!(accounts[9].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Token 2022 program
    assert_eq!(
        accounts[10].pubkey, TOKEN_2022_PROGRAM_ID,
        "token 2022 program"
    );

    // Memo program
    assert_eq!(accounts[11].pubkey, MEMO_PROGRAM_ID, "memo program");

    // Input token mint
    assert_eq!(accounts[12].pubkey, WSOL_MINT, "input token mint");

    // Output token mint
    assert_eq!(accounts[13].pubkey, USDC_MINT, "output token mint");

    // sqrt_price_limit_x64
    assert_eq!(
        u128::from_le_bytes(data[0..16].try_into().unwrap()),
        sqrt_price_limit_x64,
        "sqrt_price_limit_x64"
    );

    // is_base_input
    assert_eq!(data[16] != 0, is_base_input, "is_base_input");
}

#[tokio::test]
async fn test_byreal_clmm_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let sqrt_price_limit_x64: u128 = 0;
    let is_base_input = true;

    // Selling USDC for WSOL — mints, ATAs, and vaults should be flipped
    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::ByrealClmm {
            pool: Some(POOL_STATE),
            sqrt_price_limit_x64,
            is_base_input,
        },
        &USDC_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 14, "byreal clmm requires 14 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, BYREAL_CLMM_PROGRAM_ID,
        "byreal clmm program"
    );

    // Payer
    assert_eq!(accounts[1].pubkey, user, "payer");
    assert!(accounts[1].is_signer);

    // Amm config
    assert_eq!(accounts[2].pubkey, AMM_CONFIG, "amm config");

    // Pool state
    assert_eq!(accounts[3].pubkey, POOL_STATE, "pool state");
    assert!(accounts[3].is_writable);

    // Input token account
    let expected_usdc_ata = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[4].pubkey, expected_usdc_ata, "input token account");
    assert!(accounts[4].is_writable);

    // Output token account
    let expected_wsol_ata = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[5].pubkey, expected_wsol_ata,
        "output token account"
    );
    assert!(accounts[5].is_writable);

    // Input vault
    assert_eq!(accounts[6].pubkey, USDC_VAULT, "input vault");
    assert!(accounts[6].is_writable);

    // Output vault
    assert_eq!(accounts[7].pubkey, SOL_VAULT, "output vault");
    assert!(accounts[7].is_writable);

    // Observation state
    assert_eq!(accounts[8].pubkey, OBSERVATION_STATE, "observation state");
    assert!(accounts[8].is_writable);

    // Token program
    assert_eq!(accounts[9].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Token 2022 program
    assert_eq!(
        accounts[10].pubkey, TOKEN_2022_PROGRAM_ID,
        "token 2022 program"
    );

    // Memo program
    assert_eq!(accounts[11].pubkey, MEMO_PROGRAM_ID, "memo program");

    // Input token mint
    assert_eq!(accounts[12].pubkey, USDC_MINT, "input token mint");

    // Output token mint
    assert_eq!(accounts[13].pubkey, WSOL_MINT, "output token mint");

    // sqrt_price_limit_x64
    assert_eq!(
        u128::from_le_bytes(data[0..16].try_into().unwrap()),
        sqrt_price_limit_x64,
        "sqrt_price_limit_x64"
    );

    // is_base_input
    assert_eq!(data[16] != 0, is_base_input, "is_base_input");
}
