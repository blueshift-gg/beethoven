use {
    beethoven_client::{
        get_associated_token_address, resolve_swap, swap::solfi_v2::SOLFI_V2_PROGRAM_ID,
        SwapProtocol, SYSVAR_INSTRUCTIONS_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const MARKET: Address = address!("65ZHSArs5XxPseKQbB1B4r16vDxMWnCxHMzogDAqiDUc");
const ORACLE: Address = address!("2ny7eGyZCoeEVTkNLf5HcnJFBKkyA4p4gcrtb3b8y8ou");
const CONFIG: Address = address!("FmxXDSR9WvpJTCh738D1LEDuhMoA8geCtZgHb3isy7Dp");
const BASE_VAULT: Address = address!("CRo8DBwrmd97DJfAnvCv96tZPL5Mktf2NZy2ZnhDer1A");
const QUOTE_VAULT: Address = address!("GhFfLFSprPpfoRaWakPMmJTMJBHuz6C694jYwxy2dAic");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_solfi_v2_resolve_with_known_market() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::SolFiV2 {
            market: Some(MARKET),
            is_quote_to_base: false,
        },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, SOLFI_V2_PROGRAM_ID, "solfi_v2 program");

    // Token transfer authority
    assert_eq!(accounts[1].pubkey, user, "token transfer authority");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Market
    assert_eq!(accounts[2].pubkey, MARKET, "market");
    assert!(accounts[2].is_writable);

    // Oracle
    assert_eq!(accounts[3].pubkey, ORACLE, "oracle");

    // Config
    assert_eq!(accounts[4].pubkey, CONFIG, "config");

    // Base vault
    assert_eq!(accounts[5].pubkey, BASE_VAULT, "base vault");
    assert!(accounts[5].is_writable);

    // Quote vault
    assert_eq!(accounts[6].pubkey, QUOTE_VAULT, "quote vault");
    assert!(accounts[6].is_writable);

    // User base ATA
    let expected_user_base_ata = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[7].pubkey, expected_user_base_ata, "user base ATA");
    assert!(accounts[7].is_writable);

    // User quote ATA
    let expected_user_quote_ata =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[8].pubkey, expected_user_quote_ata,
        "user quote ATA"
    );
    assert!(accounts[8].is_writable);

    // Base mint
    assert_eq!(accounts[9].pubkey, WSOL_MINT, "base mint");

    // Quote mint
    assert_eq!(accounts[10].pubkey, USDC_MINT, "quote mint");

    // Base token program
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "base token program");

    // Quote token program
    assert_eq!(accounts[12].pubkey, TOKEN_PROGRAM_ID, "quote token program");

    // Instructions sysvar
    assert_eq!(
        accounts[13].pubkey, SYSVAR_INSTRUCTIONS_ID,
        "instructions sysvar"
    );

    // is_quote_to_base
    assert_eq!(data, vec![0u8]);
}

#[tokio::test]
async fn test_solfi_v2_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::SolFiV2 {
            market: Some(MARKET),
            is_quote_to_base: true,
        },
        &USDC_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, SOLFI_V2_PROGRAM_ID, "solfi_v2 program");

    // Token transfer authority
    assert_eq!(accounts[1].pubkey, user, "token transfer authority");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Market
    assert_eq!(accounts[2].pubkey, MARKET, "market");
    assert!(accounts[2].is_writable);

    // Oracle
    assert_eq!(accounts[3].pubkey, ORACLE, "oracle");

    // Config
    assert_eq!(accounts[4].pubkey, CONFIG, "config");

    // Base vault
    assert_eq!(accounts[5].pubkey, BASE_VAULT, "base vault");
    assert!(accounts[5].is_writable);

    // Quote vault
    assert_eq!(accounts[6].pubkey, QUOTE_VAULT, "quote vault");
    assert!(accounts[6].is_writable);

    // User base ATA
    let expected_user_base_ata = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[7].pubkey, expected_user_base_ata, "user base ATA");
    assert!(accounts[7].is_writable);

    // User quote ATA
    let expected_user_quote_ata =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[8].pubkey, expected_user_quote_ata,
        "user quote ATA"
    );
    assert!(accounts[8].is_writable);

    // Base mint
    assert_eq!(accounts[9].pubkey, WSOL_MINT, "base mint");

    // Quote mint
    assert_eq!(accounts[10].pubkey, USDC_MINT, "quote mint");

    // Base token program
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "base token program");

    // Quote token program
    assert_eq!(accounts[12].pubkey, TOKEN_PROGRAM_ID, "quote token program");

    // Instructions sysvar
    assert_eq!(
        accounts[13].pubkey, SYSVAR_INSTRUCTIONS_ID,
        "instructions sysvar"
    );

    // is_quote_to_base
    assert_eq!(data, vec![1u8]);
}
