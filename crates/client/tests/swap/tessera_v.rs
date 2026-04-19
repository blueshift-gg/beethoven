use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::tessera_v::{GLOBAL_STATE, TESSERA_V_PROGRAM_ID},
        SwapProtocol, SYSVAR_INSTRUCTIONS_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const MARKET: Address = address!("FLckHLGMJy5gEoXWwcE68Nprde1D4araK4TGLw4pQq2n");
const VAULT_A: Address = address!("5pVN5XZB8cYBjNLFrsBCPWkCQBan5K5Mq2dWGzwPgGJV");
const VAULT_B: Address = address!("9t4P5wMwfFkyn92Z7hf463qYKEZf8ERVZsGBEPNp8uJx");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_tessera_v_resolve_with_known_market_a_to_b() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::TesseraV {
            market: Some(MARKET),
            vault_a: VAULT_A,
            vault_b: VAULT_B,
        },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 13, "tessera v requires 13 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, TESSERA_V_PROGRAM_ID,
        "tessera v program"
    );

    // Global state
    assert_eq!(accounts[1].pubkey, GLOBAL_STATE, "global_state");

    // Market
    assert_eq!(accounts[2].pubkey, MARKET, "market");
    assert!(accounts[2].is_writable);

    // User
    assert_eq!(accounts[3].pubkey, user, "user");
    assert!(accounts[3].is_writable);
    assert!(accounts[3].is_signer);

    // Vault a
    assert_eq!(accounts[4].pubkey, VAULT_A, "vault_a");
    assert!(accounts[4].is_writable);

    // Vault b
    assert_eq!(accounts[5].pubkey, VAULT_B, "vault_b");
    assert!(accounts[5].is_writable);

    // User ata a
    let expected_user_ata_a = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[6].pubkey, expected_user_ata_a, "user_ata_a");
    assert!(accounts[6].is_writable);

    // User ata b
    let expected_user_ata_b = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[7].pubkey, expected_user_ata_b, "user_ata_b");
    assert!(accounts[7].is_writable);

    // Mint a
    assert_eq!(accounts[8].pubkey, WSOL_MINT, "mint_a");

    // Mint b
    assert_eq!(accounts[9].pubkey, USDC_MINT, "mint_b");

    // Token program a
    assert_eq!(accounts[10].pubkey, TOKEN_PROGRAM_ID, "token_program_a");

    // Token program b
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "token_program_b");

    // Sysvar instructions
    assert_eq!(
        accounts[12].pubkey, SYSVAR_INSTRUCTIONS_ID,
        "sysvar_instructions"
    );

    // is_a_to_b
    assert_eq!(data, vec![1_u8], "is_a_to_b");
}

#[tokio::test]
async fn test_tessera_v_resolve_with_known_market_b_to_a() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::TesseraV {
            market: Some(MARKET),
            vault_a: VAULT_A,
            vault_b: VAULT_B,
        },
        &USDC_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 13, "tessera v requires 13 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, TESSERA_V_PROGRAM_ID,
        "tessera v program"
    );

    // Global state
    assert_eq!(accounts[1].pubkey, GLOBAL_STATE, "global_state");

    // Market
    assert_eq!(accounts[2].pubkey, MARKET, "market");
    assert!(accounts[2].is_writable);

    // User
    assert_eq!(accounts[3].pubkey, user, "user");
    assert!(accounts[3].is_writable);
    assert!(accounts[3].is_signer);

    // Vault a
    assert_eq!(accounts[4].pubkey, VAULT_A, "vault_a");
    assert!(accounts[4].is_writable);

    // Vault b
    assert_eq!(accounts[5].pubkey, VAULT_B, "vault_b");
    assert!(accounts[5].is_writable);

    // User ata a
    let expected_user_ata_a = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[6].pubkey, expected_user_ata_a, "user_ata_a");
    assert!(accounts[6].is_writable);

    // User ata b
    let expected_user_ata_b = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[7].pubkey, expected_user_ata_b, "user_ata_b");
    assert!(accounts[7].is_writable);

    // Mint a
    assert_eq!(accounts[8].pubkey, WSOL_MINT, "mint_a");

    // Mint b
    assert_eq!(accounts[9].pubkey, USDC_MINT, "mint_b");

    // Token program a
    assert_eq!(accounts[10].pubkey, TOKEN_PROGRAM_ID, "token_program_a");

    // Token program b
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "token_program_b");

    // Sysvar instructions
    assert_eq!(
        accounts[12].pubkey, SYSVAR_INSTRUCTIONS_ID,
        "sysvar_instructions"
    );

    // is_a_to_b
    assert_eq!(data, vec![0_u8], "is_a_to_b");
}
