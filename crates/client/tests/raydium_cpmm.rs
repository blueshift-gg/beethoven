use {
    beethoven_client::{resolve_swap, SwapProtocol, TOKEN_PROGRAM_ID},
    solana_address::Address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = Address::from_str_const("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = Address::from_str_const("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

const RAYDIUM_CPMM_PROGRAM_ID: Address =
    Address::from_str_const("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_raydium_cpmm_resolve_with_known_pair() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::RaydiumCpmm { pool: None },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 14, "raydium cpmm requires 14 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, RAYDIUM_CPMM_PROGRAM_ID);
    assert!(!accounts[0].is_signer);
    assert!(!accounts[0].is_writable);

    // Payer
    assert_eq!(accounts[1].pubkey, user);
    assert!(accounts[1].is_signer);
    assert!(!accounts[1].is_writable);

    // Pool vault and LP mint authority
    let (expected_authority, _) =
        Address::find_program_address(&[b"vault_and_lp_mint_auth_seed"], &RAYDIUM_CPMM_PROGRAM_ID);
    assert_eq!(
        accounts[2].pubkey, expected_authority,
        "vault_and_lp_mint_auth_seed PDA"
    );
    assert!(!accounts[2].is_writable);
    assert!(!accounts[2].is_signer);

    // AMM config
    assert!(!accounts[3].is_signer);
    assert!(!accounts[3].is_writable);

    // Pool state
    assert!(accounts[4].is_writable);
    assert!(!accounts[4].is_signer);

    // Input token account
    let expected_wsol_ata =
        beethoven_client::get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_wsol_ata);
    assert!(accounts[5].is_writable);

    // Output token account
    let expected_usdc_ata =
        beethoven_client::get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[6].pubkey, expected_usdc_ata);
    assert!(accounts[6].is_writable);

    // Input vault
    assert!(accounts[7].is_writable);
    assert!(!accounts[7].is_signer);

    // Output vault
    assert!(accounts[8].is_writable);
    assert!(!accounts[8].is_signer);

    // Input token program
    assert_eq!(accounts[9].pubkey, TOKEN_PROGRAM_ID, "input_token_program");
    assert!(!accounts[9].is_signer);
    assert!(!accounts[9].is_writable);

    // Output token program
    assert_eq!(
        accounts[10].pubkey, TOKEN_PROGRAM_ID,
        "output_token_program"
    );
    assert!(!accounts[10].is_signer);
    assert!(!accounts[10].is_writable);

    // Input token mint
    assert_eq!(accounts[11].pubkey, WSOL_MINT, "token_in_mint");
    assert!(!accounts[11].is_signer);
    assert!(!accounts[11].is_writable);

    // Output token mint
    assert_eq!(accounts[12].pubkey, USDC_MINT, "token_out_mint");
    assert!(!accounts[12].is_signer);
    assert!(!accounts[12].is_writable);

    // Observation state
    assert!(accounts[13].is_writable);
    assert!(!accounts[13].is_signer);

    // Raydium CPMM swap_base_input has no extra data
    assert!(data.is_empty());
}

#[tokio::test]
async fn test_raydium_cpmm_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    // Selling USDC for WSOL — mints and ATAs should be flipped
    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::RaydiumCpmm { pool: None },
        &USDC_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 14);
    assert_eq!(accounts[0].pubkey, RAYDIUM_CPMM_PROGRAM_ID);

    // When mint_a=USDC, mints should be flipped vs canonical order
    assert_eq!(accounts[11].pubkey, USDC_MINT, "token_in_mint");
    assert_eq!(accounts[12].pubkey, WSOL_MINT, "token_out_mint");

    // Vaults swap with direction (still distinct pool reserves)
    assert_ne!(
        accounts[7].pubkey, accounts[8].pubkey,
        "token_in_vault and token_out_vault"
    );

    // User ATAs should also be flipped
    let expected_usdc_ata =
        beethoven_client::get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    let expected_wsol_ata =
        beethoven_client::get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_usdc_ata,);
    assert_eq!(accounts[6].pubkey, expected_wsol_ata,);

    assert!(data.is_empty());
}
