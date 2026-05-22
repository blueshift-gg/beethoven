use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::scale_amm::{Side, SCALE_AMM_PROGRAM_ID},
        SwapProtocol, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const MINT_B: Address = address!("7j5Zo8vzDTN8qJhWSFY9RWPE76rVRXMkqvGLeaWqcyz9");
const POOL: Address = address!("AZDqVz1TiKYGcMhaYKBMoCnRH6bXXqoxuZNR4dLL8B8K");
const PLATFORM_CONFIG: Address = address!("232KbYciAe6ma2VCB6gQyofix8qQwyZd2WYVhpNx8SyR");
const OWNER: Address = address!("BXfXDZh5HfyyPPHT5xYUVXWve5oJ2cY2P2Y6VyKwoqGg");
const VAULT_A: Address = address!("5Lsuh97Dnzsj9wp2DspyodwmUTX2ABiFYqCRtH7Ym65o");
const VAULT_B: Address = address!("ENpu9WqhnEzUSQzhEqVx6LtpXoexmRqjWYAGYYfhDGnt");
const FEE_BENEFICIARY_ATA: Address = address!("7XRb5qdYdCh1QUp6WZtHGtyGgwVnuu8BPS3fr2FvXboD");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_scale_amm_resolve_with_known_pool() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, _data) = resolve_swap(
        &rpc,
        &SwapProtocol::ScaleAmm {
            pool: Some(POOL),
            side: Side::Buy,
        },
        &WSOL_MINT,
        &MINT_B,
        &user,
    )
    .await
    .unwrap();

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, SCALE_AMM_PROGRAM_ID,
        "scale_amm program"
    );

    // Pool
    assert_eq!(accounts[1].pubkey, POOL, "pool");
    assert!(accounts[1].is_writable);

    // User
    assert_eq!(accounts[2].pubkey, user, "user");
    assert!(accounts[2].is_signer);
    assert!(accounts[2].is_writable);

    // Owner
    assert_eq!(accounts[3].pubkey, OWNER, "owner");

    // Mint a
    assert_eq!(accounts[4].pubkey, WSOL_MINT, "mint_a");

    // Mint b
    assert_eq!(accounts[5].pubkey, MINT_B, "mint_b");

    // User ta a
    let expected_user_ta_a = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[6].pubkey, expected_user_ta_a, "user_ta_a ATA");
    assert!(accounts[6].is_writable);

    // User ta b
    let expected_user_ta_b = get_associated_token_address(&user, &MINT_B, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[7].pubkey, expected_user_ta_b, "user_ta_b ATA");
    assert!(accounts[7].is_writable);

    // Vault a
    assert_eq!(accounts[8].pubkey, VAULT_A, "vault_a ATA");
    assert!(accounts[8].is_writable);

    // Vault b
    assert_eq!(accounts[9].pubkey, VAULT_B, "vault_b ATA");
    assert!(accounts[9].is_writable);

    // Platform fee ta a
    // address dependent on config fee beneficiary
    assert!(accounts[10].is_writable);

    // Token program a
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "token_program_a");

    // Token program b
    assert_eq!(accounts[12].pubkey, TOKEN_PROGRAM_ID, "token_program_b");

    // System program
    assert_eq!(accounts[13].pubkey, SYSTEM_PROGRAM_ID, "system program");

    // Config
    assert_eq!(accounts[14].pubkey, PLATFORM_CONFIG, "config");

    // Fee beneficiary accounts
    assert_eq!(
        accounts[15].pubkey, FEE_BENEFICIARY_ATA,
        "fee_beneficiary_ata"
    );
    assert!(accounts[15].is_writable);
}

#[tokio::test]
async fn test_scale_amm_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::ScaleAmm {
            pool: Some(POOL),
            side: Side::Sell,
        },
        &WSOL_MINT,
        &MINT_B,
        &user,
    )
    .await
    .unwrap();

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, SCALE_AMM_PROGRAM_ID,
        "scale_amm program"
    );

    // Pool
    assert_eq!(accounts[1].pubkey, POOL, "pool");
    assert!(accounts[1].is_writable);

    // User
    assert_eq!(accounts[2].pubkey, user, "user");
    assert!(accounts[2].is_signer);
    assert!(accounts[2].is_writable);

    // Owner
    assert_eq!(accounts[3].pubkey, OWNER, "owner");

    // Mint a
    assert_eq!(accounts[4].pubkey, WSOL_MINT, "mint_a");

    // Mint b
    assert_eq!(accounts[5].pubkey, MINT_B, "mint_b");

    // User ta a
    let expected_user_ta_a = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[6].pubkey, expected_user_ta_a, "user_ta_a ATA");
    assert!(accounts[6].is_writable);

    // User ta b
    let expected_user_ta_b = get_associated_token_address(&user, &MINT_B, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[7].pubkey, expected_user_ta_b, "user_ta_b ATA");
    assert!(accounts[7].is_writable);

    // Vault a
    assert_eq!(accounts[8].pubkey, VAULT_A, "vault_a ATA");
    assert!(accounts[8].is_writable);

    // Vault b
    assert_eq!(accounts[9].pubkey, VAULT_B, "vault_b ATA");
    assert!(accounts[9].is_writable);

    // Platform fee ta a
    // address dependent on config fee beneficiary
    assert!(accounts[10].is_writable);

    // Token program a
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "token_program_a");

    // Token program b
    assert_eq!(accounts[12].pubkey, TOKEN_PROGRAM_ID, "token_program_b");

    // System program
    assert_eq!(accounts[13].pubkey, SYSTEM_PROGRAM_ID, "system program");

    // Config
    assert_eq!(accounts[14].pubkey, PLATFORM_CONFIG, "config");

    // Fee beneficiary accounts
    assert_eq!(
        accounts[15].pubkey, FEE_BENEFICIARY_ATA,
        "fee_beneficiary_ata"
    );
    assert!(accounts[15].is_writable);

    // side
    assert_eq!(data, vec![Side::Sell as u8]);
}
