use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::meteora_damm_v2::{CP_AMM_PROGRAM_ID, POOL_AUTHORITY},
        SwapProtocol, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const POOL: Address = address!("CGPxT5d1uf9a8cKVJuZaJAU76t2EfLGbTmRbfvLLZp5j");

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
    let user = address!("11111111111111111111111111111112");

    let (accounts, _data) = resolve_swap(
        &rpc,
        &SwapProtocol::MeteoraDammV2 { pool: Some(POOL) },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 15, "meteora damm v2 requires 15 accounts");

    // Protocol program
    assert_eq!(
        accounts[0].pubkey, CP_AMM_PROGRAM_ID,
        "meteora damm v2 program"
    );

    // Pool authority
    assert_eq!(accounts[1].pubkey, POOL_AUTHORITY, "pool authority");

    // Pool
    assert_eq!(accounts[2].pubkey, POOL, "pool");
    assert!(accounts[2].is_writable);

    // Input token account
    let expected_wsol_ata = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[3].pubkey, expected_wsol_ata, "input token account");
    assert!(accounts[3].is_writable);

    // Output token account
    let expected_usdc_ata = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[4].pubkey, expected_usdc_ata,
        "output token account"
    );
    assert!(accounts[4].is_writable);

    // Token a vault
    let (expected_token_a_vault, _) = get_token_vault_pda(&WSOL_MINT, &POOL);
    assert_eq!(accounts[5].pubkey, expected_token_a_vault, "token a vault");
    assert!(accounts[5].is_writable);

    // Token b vault
    let (expected_token_b_vault, _) = get_token_vault_pda(&USDC_MINT, &POOL);
    assert_eq!(accounts[6].pubkey, expected_token_b_vault, "token b vault");
    assert!(accounts[6].is_writable);

    // Token a mint
    assert_eq!(accounts[7].pubkey, WSOL_MINT, "token a mint");

    // Token b mint
    assert_eq!(accounts[8].pubkey, USDC_MINT, "token b mint");

    // Payer
    assert_eq!(accounts[9].pubkey, user, "payer");
    assert!(accounts[9].is_signer);

    // Token a program
    assert_eq!(accounts[10].pubkey, TOKEN_PROGRAM_ID, "token a program");

    // Token b program
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "token b program");

    // Referral token account, by default the program itself
    assert_eq!(
        accounts[12].pubkey, CP_AMM_PROGRAM_ID,
        "referral token account"
    );

    // Event authority
    let (expected_event_authority, _) =
        Address::find_program_address(&[b"__event_authority"], &CP_AMM_PROGRAM_ID);
    assert_eq!(
        accounts[13].pubkey, expected_event_authority,
        "event authority"
    );

    // Meteora DAMM v2 Program itself
    assert_eq!(
        accounts[14].pubkey, CP_AMM_PROGRAM_ID,
        "meteora DAMM V2 program"
    );
}

#[tokio::test]
async fn test_meteora_damm_v2_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, _data) = resolve_swap(
        &rpc,
        &SwapProtocol::MeteoraDammV2 { pool: Some(POOL) },
        &USDC_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 15, "meteora damm v2 requires 15 accounts");

    // Protocol program
    assert_eq!(
        accounts[0].pubkey, CP_AMM_PROGRAM_ID,
        "meteora damm v2 program"
    );

    // Pool authority
    assert_eq!(accounts[1].pubkey, POOL_AUTHORITY, "pool authority");

    // Pool
    assert_eq!(accounts[2].pubkey, POOL, "pool");
    assert!(accounts[2].is_writable);

    // Input token account (USDC in)
    let expected_usdc_ata = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[3].pubkey, expected_usdc_ata, "input token account");
    assert!(accounts[3].is_writable);

    // Output token account (WSOL out)
    let expected_wsol_ata = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[4].pubkey, expected_wsol_ata,
        "output token account"
    );
    assert!(accounts[4].is_writable);

    // Token a vault
    let (expected_token_a_vault, _) = get_token_vault_pda(&WSOL_MINT, &POOL);
    assert_eq!(accounts[5].pubkey, expected_token_a_vault, "token a vault");
    assert!(accounts[5].is_writable);

    // Token b vault
    let (expected_token_b_vault, _) = get_token_vault_pda(&USDC_MINT, &POOL);
    assert_eq!(accounts[6].pubkey, expected_token_b_vault, "token b vault");
    assert!(accounts[6].is_writable);

    // Token a mint
    assert_eq!(accounts[7].pubkey, WSOL_MINT, "token a mint");

    // Token b mint
    assert_eq!(accounts[8].pubkey, USDC_MINT, "token b mint");

    // Payer
    assert_eq!(accounts[9].pubkey, user, "payer");
    assert!(accounts[9].is_signer);

    // Token a program
    assert_eq!(accounts[10].pubkey, TOKEN_PROGRAM_ID, "token a program");

    // Token b program
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "token b program");

    // Referral token account, by default the program itself
    assert_eq!(
        accounts[12].pubkey, CP_AMM_PROGRAM_ID,
        "referral token account"
    );

    // Event authority
    let (expected_event_authority, _) =
        Address::find_program_address(&[b"__event_authority"], &CP_AMM_PROGRAM_ID);
    assert_eq!(
        accounts[13].pubkey, expected_event_authority,
        "event authority"
    );

    // Meteora DAMM v2 Program itself
    assert_eq!(
        accounts[14].pubkey, CP_AMM_PROGRAM_ID,
        "meteora DAMM V2 program"
    );
}
