use {
    beethoven_client::{
        get_associated_token_address, resolve_swap, swap::aldrin_v2::ALDRIN_V2_PROGRAM_ID,
        SwapProtocol, TOKEN_PROGRAM_ID,
    },
    solana_address::Address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const USDC_MINT: Address = Address::from_str_const("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const BONK_MINT: Address = Address::from_str_const("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263");
const POOL: Address = Address::from_str_const("BMFQxq1x9GBsSDpHVf8iMVmmoB8rR9sJq3JSSE2kDQwG");
const POOL_SIGNER: Address =
    Address::from_str_const("6yioeAnhko4Vzw9CZsncwaHDz5S12YjTms3eGjhrGJUX");
const POOL_MINT: Address = Address::from_str_const("3SbhGXhZMSoLJKMpGEvb29ZEsXcgmyTizMsjFYJxWPgL");
const USDC_VAULT: Address = Address::from_str_const("J94S454qunF5xH4UdyXbv6j8kTneLajFYzjTpfHJ62cU");
const BONK_VAULT: Address = Address::from_str_const("3hWGKXEtKbAyArLb4y4hpMyhqqXXp4HM1jPrz4mPjxcJ");
const FEE_POOL_TOKEN_ACCOUNT: Address =
    Address::from_str_const("4eRYmUET6EtaVHwYmgxJeHTXVdYJG5pXmpixQ175RAiQ");
const CURVE: Address = Address::from_str_const("2LnTcdBH6zytWoDkTgUakgHmYf9eP51MgkBdTGow4jQL");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_aldrin_v2_resolve_with_known_pool() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::AldrinV2 {
            pool: Some(POOL),
            // ask
            side: 1,
        },
        &USDC_MINT,
        &BONK_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 12, "aldrin v2 requires 12 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, ALDRIN_V2_PROGRAM_ID,
        "aldrin v2 program"
    );

    // Pool
    assert_eq!(accounts[1].pubkey, POOL, "pool");

    // Pool signer
    assert_eq!(accounts[2].pubkey, POOL_SIGNER, "pool signer");

    // Pool mint
    assert_eq!(accounts[3].pubkey, POOL_MINT, "pool mint");
    assert!(accounts[3].is_writable);

    // Base token vault
    assert_eq!(accounts[4].pubkey, USDC_VAULT, "base token vault");
    assert!(accounts[4].is_writable);

    // Quote token vault
    assert_eq!(accounts[5].pubkey, BONK_VAULT, "quote token vault");
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
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[8].pubkey, expected_user_base_token_account,
        "user base token account"
    );
    assert!(accounts[8].is_writable);

    // User quote token account
    let expected_user_quote_token_account =
        get_associated_token_address(&user, &BONK_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[9].pubkey, expected_user_quote_token_account,
        "user quote token account"
    );
    assert!(accounts[9].is_writable);

    // Curve
    assert_eq!(accounts[10].pubkey, CURVE, "curve");
    assert!(!accounts[10].is_writable);

    // Token program
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "token program");

    // ask = 1
    assert_eq!(data, vec![1u8]);
}

#[tokio::test]
async fn test_aldrin_v2_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    // Selling BONK for USDC — vaults and mints should be flipped
    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::AldrinV2 {
            pool: Some(POOL),
            // bid
            side: 0,
        },
        &USDC_MINT,
        &BONK_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 12, "aldrin v2 requires 12 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, ALDRIN_V2_PROGRAM_ID,
        "aldrin v2 program"
    );

    // Pool
    assert_eq!(accounts[1].pubkey, POOL, "pool");

    // Pool signer
    assert_eq!(accounts[2].pubkey, POOL_SIGNER, "pool signer");

    // Pool mint
    assert_eq!(accounts[3].pubkey, POOL_MINT, "pool mint");

    // Base token vault
    assert_eq!(accounts[4].pubkey, USDC_VAULT, "base token vault");
    assert!(accounts[4].is_writable);

    // Quote token vault
    assert_eq!(accounts[5].pubkey, BONK_VAULT, "quote token vault");
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
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[8].pubkey, expected_user_base_token_account,
        "user base token account"
    );
    assert!(accounts[8].is_writable);

    // User quote token account
    let expected_user_quote_token_account =
        get_associated_token_address(&user, &BONK_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[9].pubkey, expected_user_quote_token_account,
        "user quote token account"
    );
    assert!(accounts[9].is_writable);

    // Curve
    assert_eq!(accounts[10].pubkey, CURVE, "curve");
    assert!(!accounts[10].is_writable);

    // Token program
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "token program");

    // bid = 0
    assert_eq!(data, vec![0u8]);
}
