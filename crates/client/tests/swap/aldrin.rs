use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::aldrin::{Side, ALDRIN_PROGRAM_ID},
        SwapProtocol, TOKEN_PROGRAM_ID,
    },
    solana_address::Address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = Address::from_str_const("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = Address::from_str_const("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const POOL: Address = Address::from_str_const("4GUniSDrCAZR3sKtLa1AWC8oyYubZeKJQ8KraQmy3Wt5");
const POOL_SIGNER: Address =
    Address::from_str_const("7Zi96LCCjSEEd5yyFik8XvAhfJsdUGzLPMprjKKrdaCA");
const POOL_MINT: Address = Address::from_str_const("3sbMDzGtyHAzJqzxE7DPdLMhrsxQASYoKLkHMYJPuWkp");
const SOL_VAULT: Address = Address::from_str_const("CLt1DtCioiByTizqLhxLAXweXr2g9D4ZEAStibACBg4L");
const USDC_VAULT: Address = Address::from_str_const("2M1JTZsc71V6FhRNjCDSttcs17HewC4KNNNkkc81L3gB");
const FEE_POOL_TOKEN_ACCOUNT: Address =
    Address::from_str_const("DuoYmMoZBy2MyGP8xa3LiWyURjmpfbZbfwRmoPvYKmr6");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_aldrin_resolve_with_known_pool() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Aldrin {
            pool: Some(POOL),
            side: Side::Ask,
        },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 11, "aldrin requires 11 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, ALDRIN_PROGRAM_ID, "aldrin program");

    // Pool
    assert_eq!(accounts[1].pubkey, POOL, "pool");

    // Pool signer
    assert_eq!(accounts[2].pubkey, POOL_SIGNER, "pool signer");

    // Pool mint
    assert_eq!(accounts[3].pubkey, POOL_MINT, "pool mint");
    assert!(accounts[3].is_writable);

    // Base token vault
    assert_eq!(accounts[4].pubkey, SOL_VAULT, "base token vault");
    assert!(accounts[4].is_writable);

    // Quote token vault
    assert_eq!(accounts[5].pubkey, USDC_VAULT, "quote token vault");
    assert!(accounts[5].is_writable);

    // Fee pool token account
    assert_eq!(
        accounts[6].pubkey, FEE_POOL_TOKEN_ACCOUNT,
        "fee pool token account"
    );
    assert!(accounts[6].is_writable);

    // Wallet authority
    assert_eq!(accounts[7].pubkey, user, "wallet authority");
    assert!(accounts[7].is_signer);
    assert!(accounts[7].is_writable);

    // User base token account
    let expected_user_base_token_account =
        get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[8].pubkey, expected_user_base_token_account,
        "user base token account"
    );
    assert!(accounts[8].is_writable);

    // User quote token account
    let expected_user_quote_token_account =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[9].pubkey, expected_user_quote_token_account,
        "user quote token account"
    );
    assert!(accounts[9].is_writable);

    // Token program
    assert_eq!(accounts[10].pubkey, TOKEN_PROGRAM_ID, "token program");

    // ask = 1
    assert_eq!(data, vec![Side::Ask as u8]);
}

#[tokio::test]
async fn test_aldrin_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    // Selling USDC for WSOL — vaults and mints should be flipped
    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Aldrin {
            pool: Some(POOL),
            side: Side::Bid,
        },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 11, "aldrin requires 11 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, ALDRIN_PROGRAM_ID, "aldrin program");

    // Pool
    assert_eq!(accounts[1].pubkey, POOL, "pool");

    // Pool signer
    assert_eq!(accounts[2].pubkey, POOL_SIGNER, "pool signer");

    // Pool mint
    assert_eq!(accounts[3].pubkey, POOL_MINT, "pool mint");

    // Base token vault
    assert_eq!(accounts[4].pubkey, SOL_VAULT, "base token vault");
    assert!(accounts[4].is_writable);

    // Quote token vault
    assert_eq!(accounts[5].pubkey, USDC_VAULT, "quote token vault");
    assert!(accounts[5].is_writable);

    // Fee pool token account
    assert_eq!(
        accounts[6].pubkey, FEE_POOL_TOKEN_ACCOUNT,
        "fee pool token account"
    );
    assert!(accounts[6].is_writable);

    // Wallet authority
    assert_eq!(accounts[7].pubkey, user, "wallet authority");
    assert!(accounts[7].is_signer);
    assert!(accounts[7].is_writable);

    // User base token account
    let expected_user_base_token_account =
        get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[8].pubkey, expected_user_base_token_account,
        "user base token account"
    );
    assert!(accounts[8].is_writable);

    // User quote token account
    let expected_user_quote_token_account =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[9].pubkey, expected_user_quote_token_account,
        "user quote token account"
    );
    assert!(accounts[9].is_writable);

    // Token program
    assert_eq!(accounts[10].pubkey, TOKEN_PROGRAM_ID, "token program");

    // bid = 0
    assert_eq!(data, vec![Side::Bid as u8]);
}
