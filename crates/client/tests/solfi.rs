use {
    beethoven_client::{
        get_associated_token_address, resolve_swap, swap::solfi::SOLFI_PROGRAM_ID, SwapProtocol,
        SYSVAR_INSTRUCTIONS_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const WETH_MINT: Address = address!("7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs");
const MARKET: Address = address!("7NbPAgjn6W9xH6G1opqYAXVVuJBAtn67DDbD5PTCbA3o");
const BASE_VAULT: Address = address!("9PsHds1eaSLgTBmSefJ2KhjGRcVCbwP9SkaYiWFr31yP");
const QUOTE_VAULT: Address = address!("E5D142djU2atMNuLq8nPr4X2bgskqW32nhYWqPPzU1gS");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_solfi_resolve_with_known_market() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::SolFi {
            market: Some(MARKET),
            is_quote_to_base: true,
        },
        &WETH_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, SOLFI_PROGRAM_ID, "solfi program");

    // Token transfer authority
    assert_eq!(accounts[1].pubkey, user, "token transfer authority");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Market
    assert_eq!(accounts[2].pubkey, MARKET, "market");
    assert!(accounts[2].is_writable);

    // Base vault
    assert_eq!(accounts[3].pubkey, BASE_VAULT, "base vault");
    assert!(accounts[3].is_writable);

    // Quote vault
    assert_eq!(accounts[4].pubkey, QUOTE_VAULT, "quote vault");
    assert!(accounts[4].is_writable);

    // User base ATA
    let expected_user_base_ata = get_associated_token_address(&user, &WETH_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_user_base_ata, "user base ATA");
    assert!(accounts[5].is_writable);

    // User quote ATA
    let expected_user_quote_ata =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[6].pubkey, expected_user_quote_ata,
        "user quote ATA"
    );
    assert!(accounts[6].is_writable);

    // Token program
    assert_eq!(accounts[7].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Instructions sysvar
    assert_eq!(
        accounts[8].pubkey, SYSVAR_INSTRUCTIONS_ID,
        "instructions sysvar"
    );

    // is_quote_to_base
    assert_eq!(data, vec![1u8]);
}

#[tokio::test]
async fn test_solfi_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::SolFi {
            market: Some(MARKET),
            is_quote_to_base: false,
        },
        &USDC_MINT,
        &WETH_MINT,
        &user,
    )
    .await
    .unwrap();

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, SOLFI_PROGRAM_ID, "solfi program");

    // Token transfer authority
    assert_eq!(accounts[1].pubkey, user, "token transfer authority");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Market
    assert_eq!(accounts[2].pubkey, MARKET, "market");
    assert!(accounts[2].is_writable);

    // Base vault
    assert_eq!(accounts[3].pubkey, BASE_VAULT, "base vault");
    assert!(accounts[3].is_writable);

    // Quote vault
    assert_eq!(accounts[4].pubkey, QUOTE_VAULT, "quote vault");
    assert!(accounts[4].is_writable);

    // User base ATA
    let expected_user_base_ata = get_associated_token_address(&user, &WETH_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_user_base_ata, "user base ATA");
    assert!(accounts[5].is_writable);

    // User quote ATA
    let expected_user_quote_ata =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[6].pubkey, expected_user_quote_ata,
        "user quote ATA"
    );
    assert!(accounts[6].is_writable);

    // Token program
    assert_eq!(accounts[7].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Instructions sysvar
    assert_eq!(
        accounts[8].pubkey, SYSVAR_INSTRUCTIONS_ID,
        "instructions sysvar"
    );

    // is_quote_to_base
    assert_eq!(data, vec![0u8]);
}
