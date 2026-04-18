use {
    beethoven_client::{resolve_swap, SwapProtocol},
    solana_address::Address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = Address::from_str_const("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = Address::from_str_const("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const MARKET: Address = Address::from_str_const("ENhU8LsaR7vDD2G1CsWcsuSGNrih9Cv5WZEk7q9kPapQ");
const BASE_VAULT: Address = Address::from_str_const("AKjfJDv4ywdpCDrj7AURuNkGA3696GTVFgrMwk4TjkKs");
const QUOTE_VAULT: Address =
    Address::from_str_const("FN9K6rTdWtRDUPmLTN2FnGvLZpHVNRN2MeRghKknSGDs");

const MANIFEST_PROGRAM_ID: Address =
    Address::from_str_const("MNFSTqtC93rEfYHB6hF82sKdZpUDFWkViLByLd1k1Ms");
const TOKEN_PROGRAM_ID: Address =
    Address::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const SYSTEM_PROGRAM_ID: Address = Address::from_str_const("11111111111111111111111111111111");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_manifest_resolve_with_known_market() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Manifest {
            market: Some(MARKET),
            is_exact_in: true,
        },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 15, "manifest requires 15 accounts");

    // Protocol program ID (used by on-chain routing)
    assert_eq!(accounts[0].pubkey, MANIFEST_PROGRAM_ID);

    // User accounts
    assert_eq!(accounts[1].pubkey, user, "payer");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);
    assert_eq!(accounts[2].pubkey, user, "owner");
    assert!(accounts[2].is_signer);

    // Market
    assert_eq!(accounts[3].pubkey, MARKET);

    // System program
    assert_eq!(accounts[4].pubkey, SYSTEM_PROGRAM_ID);

    // Trader ATAs — derived from user, verify they match expected derivation
    let expected_trader_base =
        beethoven_client::get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    let expected_trader_quote =
        beethoven_client::get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_trader_base, "trader_base ATA");
    assert_eq!(
        accounts[6].pubkey, expected_trader_quote,
        "trader_quote ATA"
    );

    // Vaults (read from on-chain market state at verified offsets)
    assert_eq!(accounts[7].pubkey, BASE_VAULT, "base_vault");
    assert_eq!(accounts[8].pubkey, QUOTE_VAULT, "quote_vault");

    // Token programs — both SPL Token for WSOL/USDC
    assert_eq!(accounts[9].pubkey, TOKEN_PROGRAM_ID, "base_token_program");
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "quote_token_program");

    // Mints (read from on-chain market state)
    assert_eq!(accounts[10].pubkey, WSOL_MINT, "base_mint");
    assert_eq!(accounts[12].pubkey, USDC_MINT, "quote_mint");

    // Global PDAs (deterministic from program ID)
    let (expected_global, _) = Address::find_program_address(&[b"global"], &MANIFEST_PROGRAM_ID);
    let (expected_global_vault, _) =
        Address::find_program_address(&[b"global-vault"], &MANIFEST_PROGRAM_ID);
    assert_eq!(accounts[13].pubkey, expected_global, "global");
    assert_eq!(accounts[14].pubkey, expected_global_vault, "global_vault");

    // is_base_in should be inferred as true (WSOL == base_mint)
    // is_exact_in was passed as true
    assert_eq!(data, vec![1u8, 1u8]);
}

#[tokio::test]
async fn test_manifest_resolve_quote_in() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    // Selling USDC for WSOL — mint_a=USDC means is_base_in should be inferred as false
    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Manifest {
            market: Some(MARKET),
            is_exact_in: true,
        },
        &USDC_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 15);
    assert_eq!(accounts[0].pubkey, MANIFEST_PROGRAM_ID);

    // is_base_in=false (USDC != base_mint), is_exact_in=true
    assert_eq!(data, vec![0u8, 1u8]);
}
