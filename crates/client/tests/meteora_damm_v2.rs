use {
    beethoven_client::{
        resolve_swap,
        swap::meteora_damm_v2::{CP_AMM_PROGRAM_ID, POOL_AUTHORITY},
        SwapProtocol,
    },
    solana_address::Address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = Address::from_str_const("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = Address::from_str_const("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

const TOKEN_PROGRAM_ID: Address =
    Address::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

fn get_token_vault_pda(mint: &Address, pool: &Address) -> (Address, u8) {
    Address::find_program_address(
        &[b"token_vault", mint.as_ref(), pool.as_ref()],
        &CP_AMM_PROGRAM_ID,
    )
}

#[tokio::test]
async fn test_meteora_damm_v2_resolve_with_known_pool() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, _data) = resolve_swap(
        &rpc,
        &SwapProtocol::MeteoraDammV2 { pool: None },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 15, "meteora damm v2 requires 15 accounts");

    // Protocol program
    assert_eq!(accounts[0].pubkey, CP_AMM_PROGRAM_ID);
    assert!(!accounts[0].is_signer);
    assert!(!accounts[0].is_writable);

    // Pool authority
    assert_eq!(accounts[1].pubkey, POOL_AUTHORITY);
    assert!(!accounts[1].is_signer);
    assert!(!accounts[1].is_writable);

    // Pool
    assert!(accounts[2].is_writable);
    assert!(!accounts[2].is_signer);

    // Input token account
    let expected_wsol_ata =
        beethoven_client::get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[3].pubkey, expected_wsol_ata);
    assert!(accounts[3].is_writable);

    // Output token account
    let expected_usdc_ata =
        beethoven_client::get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[4].pubkey, expected_usdc_ata);
    assert!(accounts[4].is_writable);

    let pool = accounts[2].pubkey;

    // Token a vault
    let (expected_token_a_vault, _) = get_token_vault_pda(&WSOL_MINT, &pool);
    assert_eq!(accounts[5].pubkey, expected_token_a_vault);
    assert!(accounts[5].is_writable);

    // Token b vault
    let (expected_token_b_vault, _) = get_token_vault_pda(&USDC_MINT, &pool);
    assert_eq!(accounts[6].pubkey, expected_token_b_vault);
    assert!(accounts[6].is_writable);

    // Token a mint
    assert_eq!(accounts[7].pubkey, WSOL_MINT);
    assert!(!accounts[7].is_writable);

    // Token b mint
    assert_eq!(accounts[8].pubkey, USDC_MINT);
    assert!(!accounts[8].is_writable);

    // Payer
    assert_eq!(accounts[9].pubkey, user);
    assert!(accounts[9].is_signer);
    assert!(!accounts[9].is_writable);

    // Token a program
    assert_eq!(accounts[10].pubkey, TOKEN_PROGRAM_ID);
    assert!(!accounts[10].is_writable);

    // Token b program
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID);
    assert!(!accounts[11].is_writable);

    // Referral token account, by default the program itself
    assert_eq!(accounts[12].pubkey, CP_AMM_PROGRAM_ID);
    assert!(!accounts[12].is_writable);

    // Event authority
    let (expected_event_authority, _) =
        Address::find_program_address(&[b"__event_authority"], &CP_AMM_PROGRAM_ID);
    assert_eq!(accounts[13].pubkey, expected_event_authority);
    assert!(!accounts[13].is_writable);
    assert!(!accounts[13].is_signer);

    // Meteora DAMM v2 Program itself
    assert_eq!(accounts[14].pubkey, CP_AMM_PROGRAM_ID);
    assert!(!accounts[14].is_writable);
    assert!(!accounts[14].is_signer);
}

#[tokio::test]
async fn test_meteora_damm_v2_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, _data) = resolve_swap(
        &rpc,
        &SwapProtocol::MeteoraDammV2 { pool: None },
        &USDC_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 15, "meteora damm v2 requires 15 accounts");

    // Protocol program
    assert_eq!(accounts[0].pubkey, CP_AMM_PROGRAM_ID);
    assert!(!accounts[0].is_signer);
    assert!(!accounts[0].is_writable);

    // Pool authority
    assert_eq!(accounts[1].pubkey, POOL_AUTHORITY);
    assert!(!accounts[1].is_signer);
    assert!(!accounts[1].is_writable);

    // Pool
    assert!(accounts[2].is_writable);
    assert!(!accounts[2].is_signer);

    // Input token account (USDC in)
    let expected_usdc_ata =
        beethoven_client::get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[3].pubkey, expected_usdc_ata);
    assert!(accounts[3].is_writable);

    // Output token account (WSOL out)
    let expected_wsol_ata =
        beethoven_client::get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[4].pubkey, expected_wsol_ata);
    assert!(accounts[4].is_writable);

    let pool = accounts[2].pubkey;

    // Token a vault
    let (expected_token_a_vault, _) = get_token_vault_pda(&USDC_MINT, &pool);
    assert_eq!(accounts[5].pubkey, expected_token_a_vault);
    assert!(accounts[5].is_writable);

    // Token b vault
    let (expected_token_b_vault, _) = get_token_vault_pda(&WSOL_MINT, &pool);
    assert_eq!(accounts[6].pubkey, expected_token_b_vault);
    assert!(accounts[6].is_writable);

    // Token a mint
    assert_eq!(accounts[7].pubkey, USDC_MINT);
    assert!(!accounts[7].is_writable);

    // Token b mint
    assert_eq!(accounts[8].pubkey, WSOL_MINT);
    assert!(!accounts[8].is_writable);

    // Payer
    assert_eq!(accounts[9].pubkey, user);
    assert!(accounts[9].is_signer);
    assert!(!accounts[9].is_writable);

    // Token a program
    assert_eq!(accounts[10].pubkey, TOKEN_PROGRAM_ID);
    assert!(!accounts[10].is_writable);

    // Token b program
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID);
    assert!(!accounts[11].is_writable);

    // Referral token account, by default the program itself
    assert_eq!(accounts[12].pubkey, CP_AMM_PROGRAM_ID);
    assert!(!accounts[12].is_writable);

    // Event authority
    let (expected_event_authority, _) =
        Address::find_program_address(&[b"__event_authority"], &CP_AMM_PROGRAM_ID);
    assert_eq!(accounts[13].pubkey, expected_event_authority);
    assert!(!accounts[13].is_writable);
    assert!(!accounts[13].is_signer);

    // Meteora DAMM v2 Program itself
    assert_eq!(accounts[14].pubkey, CP_AMM_PROGRAM_ID);
    assert!(!accounts[14].is_writable);
    assert!(!accounts[14].is_signer);
}
