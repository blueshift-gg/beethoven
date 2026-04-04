use {
    beethoven_client::{
        get_associated_token_address, resolve_swap, swap::omnipair::OMNIPAIR_PROGRAM_ID,
        SwapProtocol, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::Address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = Address::from_str_const("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = Address::from_str_const("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const SOL_USDC_PAIR: Address =
    Address::from_str_const("3cPJTS5kfD7414aTRPcyBrA55aSx8csCUPWsrS4mnFWV");
const RATE_MODEL: Address = Address::from_str_const("GEbFhfNcpu1gnKbyuzGZ4wfP4kXzuyKj4J8xRm2yTKeG");
const FUTARCHY_AUTHORITY: Address =
    Address::from_str_const("2SMS1Y4EAyL2dQLpXD6VJCrNbQJ2eQ2pN3qYcX1vim3E");
const SOL_USDC_PAIR_SOL_RESERVE_VAULT: Address =
    Address::from_str_const("2PXu1RN3zW5PDjAZoNBaijaGs3rEZ3bG9omRihb5C8Bi");
const SOL_USDC_PAIR_USDC_RESERVE_VAULT: Address =
    Address::from_str_const("F5c9GM9rZXPk99z6sahgSnZcyp67ck4Q694uve1RUU2Z");
const EVENT_AUTHORITY: Address =
    Address::from_str_const("FWdP9yTogKbuXvEqQNNHYw2TYm38MbinAZ2iTHeZWX8H");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_omnipair_resolve_with_known_pair() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Omnipair {
            pair: Some(SOL_USDC_PAIR),
        },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 15, "omnipair requires 15 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, OMNIPAIR_PROGRAM_ID, "omnipair program");

    // Pair
    assert_eq!(accounts[1].pubkey, SOL_USDC_PAIR, "pair");
    assert!(accounts[1].is_writable);

    // Rate model
    assert_eq!(accounts[2].pubkey, RATE_MODEL, "rate model");

    // Futarchy authority
    assert_eq!(accounts[3].pubkey, FUTARCHY_AUTHORITY, "futarchy authority");

    // Token in vault
    assert_eq!(
        accounts[4].pubkey, SOL_USDC_PAIR_SOL_RESERVE_VAULT,
        "token in vault"
    );
    assert!(accounts[4].is_writable);

    // Token out vault
    assert_eq!(
        accounts[5].pubkey, SOL_USDC_PAIR_USDC_RESERVE_VAULT,
        "token out vault"
    );
    assert!(accounts[5].is_writable);

    let pair = accounts[1].pubkey;
    let token_in_mint = accounts[8].pubkey;
    let token_out_mint = accounts[9].pubkey;

    let expected_token_in_vault = Address::find_program_address(
        &[b"reserve_vault", pair.as_ref(), token_in_mint.as_ref()],
        &OMNIPAIR_PROGRAM_ID,
    )
    .0;
    let expected_token_out_vault = Address::find_program_address(
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

    // User token in account
    let expected_wsol_ata = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[6].pubkey, expected_wsol_ata,
        "user token in account"
    );
    assert!(accounts[6].is_writable);

    // User token out account
    let expected_usdc_ata = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[7].pubkey, expected_usdc_ata,
        "user token out account"
    );
    assert!(accounts[7].is_writable);

    // Token in mint
    assert_eq!(accounts[8].pubkey, WSOL_MINT, "token in mint");

    // token out mint
    assert_eq!(accounts[9].pubkey, USDC_MINT, "token out mint");

    // User
    assert_eq!(accounts[10].pubkey, user, "user");
    assert!(accounts[10].is_writable);
    assert!(accounts[10].is_signer);

    // Token program
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Token 2022 program
    assert_eq!(
        accounts[12].pubkey, TOKEN_2022_PROGRAM_ID,
        "token 2022 program"
    );

    // Event authority
    assert_eq!(accounts[13].pubkey, EVENT_AUTHORITY, "event authority");

    // Omnipair program itself
    assert_eq!(
        accounts[14].pubkey, OMNIPAIR_PROGRAM_ID,
        "program self-reference"
    );

    // Omnipair has no extra data
    assert!(data.is_empty());
}

#[tokio::test]
async fn test_omnipair_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    // Selling USDC for WSOL — vaults and mints should be flipped
    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Omnipair {
            pair: Some(SOL_USDC_PAIR),
        },
        &USDC_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 15);

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, OMNIPAIR_PROGRAM_ID, "omnipair program");

    // Pair
    assert_eq!(accounts[1].pubkey, SOL_USDC_PAIR, "pair");
    assert!(accounts[1].is_writable);

    // Rate model
    assert_eq!(accounts[2].pubkey, RATE_MODEL, "rate model");

    // Futarchy authority
    assert_eq!(accounts[3].pubkey, FUTARCHY_AUTHORITY, "futarchy authority");

    // Token in vault
    assert_eq!(
        accounts[4].pubkey, SOL_USDC_PAIR_USDC_RESERVE_VAULT,
        "token in vault"
    );
    assert!(accounts[4].is_writable);

    // Token out vault
    assert_eq!(
        accounts[5].pubkey, SOL_USDC_PAIR_SOL_RESERVE_VAULT,
        "token out vault"
    );
    assert!(accounts[5].is_writable);

    // User token in account
    let expected_usdc_ata = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[6].pubkey, expected_usdc_ata,
        "user token in account"
    );
    assert!(accounts[6].is_writable);

    // User token out account
    let expected_wsol_ata = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[7].pubkey, expected_wsol_ata,
        "user token out account"
    );
    assert!(accounts[7].is_writable);

    // Token in mint
    assert_eq!(accounts[8].pubkey, USDC_MINT, "token in mint");

    // token out mint
    assert_eq!(accounts[9].pubkey, WSOL_MINT, "token out mint");

    // User
    assert_eq!(accounts[10].pubkey, user, "user");
    assert!(accounts[10].is_writable);
    assert!(accounts[10].is_signer);

    // Token program
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Token 2022 program
    assert_eq!(
        accounts[12].pubkey, TOKEN_2022_PROGRAM_ID,
        "token 2022 program"
    );

    // Event authority
    assert_eq!(accounts[13].pubkey, EVENT_AUTHORITY, "event authority");

    // Omnipair program itself
    assert_eq!(
        accounts[14].pubkey, OMNIPAIR_PROGRAM_ID,
        "program self-reference"
    );

    // Omnipair has no extra data
    assert!(data.is_empty());
}
