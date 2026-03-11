use {
    beethoven_client::{resolve_swap, SwapProtocol},
    solana_pubkey::Pubkey,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Pubkey = Pubkey::from_str_const("So11111111111111111111111111111111111111112");
const USDC_MINT: Pubkey = Pubkey::from_str_const("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

const OMNIPAIR_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("omnixgS8fnqHfCcTGKWj6JtKjzpJZ1Y5y9pyFkQDkYE");
const TOKEN_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const TOKEN_2022_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_omnipair_resolve_with_known_pair() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Pubkey::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Omnipair { pair: None },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 15, "omnipair requires 15 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, OMNIPAIR_PROGRAM_ID);
    assert!(!accounts[0].is_signer);
    assert!(!accounts[0].is_writable);

    // Pair
    assert!(accounts[1].is_writable);
    assert!(!accounts[1].is_signer);

    // Rate model, read from pair state
    assert!(accounts[2].is_writable);
    assert!(!accounts[2].is_signer);

    // Futarchy authority
    let (expected_futarchy_authority, _) =
        Pubkey::find_program_address(&[b"futarchy_authority"], &OMNIPAIR_PROGRAM_ID);
    assert_eq!(
        accounts[3].pubkey, expected_futarchy_authority,
        "futarchy_authority PDA"
    );
    assert!(!accounts[3].is_writable);
    assert!(!accounts[3].is_signer);

    // Token in vault
    assert!(accounts[4].is_writable);
    assert!(!accounts[4].is_signer);

    // Token out vault
    assert!(accounts[5].is_writable);
    assert!(!accounts[5].is_signer);

    let pair = accounts[1].pubkey;
    let token_in_mint = accounts[8].pubkey;
    let token_out_mint = accounts[9].pubkey;

    let expected_token_in_vault = Pubkey::find_program_address(
        &[b"reserve_vault", pair.as_ref(), token_in_mint.as_ref()],
        &OMNIPAIR_PROGRAM_ID,
    )
    .0;
    let expected_token_out_vault = Pubkey::find_program_address(
        &[b"reserve_vault", pair.as_ref(), token_out_mint.as_ref()],
        &OMNIPAIR_PROGRAM_ID,
    )
    .0;
    assert_eq!(
        accounts[4].pubkey, expected_token_in_vault,
        "token_in_vault PDA"
    );
    assert_eq!(
        accounts[5].pubkey, expected_token_out_vault,
        "token_out_vault PDA"
    );

    // User ATAs
    let expected_wsol_ata =
        beethoven_client::get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    let expected_usdc_ata =
        beethoven_client::get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[6].pubkey, expected_wsol_ata,
        "user_token_in_account ATA"
    );
    assert_eq!(
        accounts[7].pubkey, expected_usdc_ata,
        "user_token_out_account ATA"
    );
    assert!(accounts[6].is_writable);
    assert!(accounts[7].is_writable);

    // Token in mint and token out mint
    assert_eq!(accounts[8].pubkey, WSOL_MINT, "token_in_mint");
    assert_eq!(accounts[9].pubkey, USDC_MINT, "token_out_mint");
    assert!(!accounts[8].is_writable);
    assert!(!accounts[9].is_writable);

    // User
    assert_eq!(accounts[10].pubkey, user);
    assert!(accounts[10].is_signer);
    assert!(!accounts[10].is_writable);

    // Token program
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "token_program");
    assert!(!accounts[11].is_writable);

    // Token 2022 program
    assert_eq!(
        accounts[12].pubkey, TOKEN_2022_PROGRAM_ID,
        "token_2022_program"
    );
    assert!(!accounts[12].is_writable);

    // Event authority
    let (expected_event_authority, _) =
        Pubkey::find_program_address(&[b"__event_authority"], &OMNIPAIR_PROGRAM_ID);
    assert_eq!(
        accounts[13].pubkey, expected_event_authority,
        "event_authority PDA"
    );
    assert!(!accounts[13].is_writable);
    assert!(!accounts[13].is_signer);

    // Omnipair program itself
    assert_eq!(
        accounts[14].pubkey, OMNIPAIR_PROGRAM_ID,
        "program self-reference"
    );
    assert!(!accounts[14].is_writable);
    assert!(!accounts[14].is_signer);

    // Omnipair has no extra data
    assert!(data.is_empty());
}

#[tokio::test]
async fn test_omnipair_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Pubkey::from_str_const("11111111111111111111111111111112");

    // Selling USDC for WSOL — vaults and mints should be flipped
    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Omnipair { pair: None },
        &USDC_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 15);
    assert_eq!(accounts[0].pubkey, OMNIPAIR_PROGRAM_ID);

    // When mint_a=USDC, mints should be flipped vs canonical order
    assert_eq!(accounts[8].pubkey, USDC_MINT, "token_in_mint");
    assert_eq!(accounts[9].pubkey, WSOL_MINT, "token_out_mint");

    // Vaults should be derived for the flipped direction
    let pair = accounts[1].pubkey;
    let expected_token_in_vault = Pubkey::find_program_address(
        &[b"reserve_vault", pair.as_ref(), USDC_MINT.as_ref()],
        &OMNIPAIR_PROGRAM_ID,
    )
    .0;
    let expected_token_out_vault = Pubkey::find_program_address(
        &[b"reserve_vault", pair.as_ref(), WSOL_MINT.as_ref()],
        &OMNIPAIR_PROGRAM_ID,
    )
    .0;
    assert_eq!(
        accounts[4].pubkey, expected_token_in_vault,
        "token_in_vault (USDC vault)"
    );
    assert_eq!(
        accounts[5].pubkey, expected_token_out_vault,
        "token_out_vault (WSOL vault)"
    );

    // User ATAs should also be flipped
    let expected_usdc_ata =
        beethoven_client::get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    let expected_wsol_ata =
        beethoven_client::get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[6].pubkey, expected_usdc_ata,
        "user_token_in_account ATA"
    );
    assert_eq!(
        accounts[7].pubkey, expected_wsol_ata,
        "user_token_out_account ATA"
    );

    assert!(data.is_empty());
}
