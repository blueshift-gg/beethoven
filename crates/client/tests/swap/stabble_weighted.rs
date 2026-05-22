use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::stabble_weighted::{
            get_vault_authority_address, get_withdraw_authority_address, FEE_VAULT_AUTHORITY,
            STABBLE_WEIGHTED_SWAP_PROGRAM_ID, VAULT_PROGRAM, VAULT_STATE,
        },
        SwapProtocol, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const POOL: Address = address!("HZzitoVgr9PWWUr2mchGRRp2RkJVUseeVSFgzbuHmMeC");
const SOL_VAULT: Address = address!("HoAHDQss5qzYkoKPXtRJRHCQrUWxcHvs4vmZ8QsN4nSq");
const USDC_VAULT: Address = address!("2PkFYJpyum86qkAM46hZ7bNvUGq157RoaPKFrgTAWLub");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_stabble_weighted_resolve_with_known_pool() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::StabbleWeighted { pool: POOL },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 16, "stabble weighted requires 16 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, STABBLE_WEIGHTED_SWAP_PROGRAM_ID,
        "stabble weighted program"
    );

    // User
    assert_eq!(accounts[1].pubkey, user, "user");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Mint in
    assert_eq!(accounts[2].pubkey, WSOL_MINT, "mint in");

    // Mint out
    assert_eq!(accounts[3].pubkey, USDC_MINT, "mint out");

    // User token in
    let expected_user_token_in = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[4].pubkey, expected_user_token_in, "user token in");
    assert!(accounts[4].is_writable);

    // User token out
    let expected_user_token_out =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[5].pubkey, expected_user_token_out,
        "user token out"
    );
    assert!(accounts[5].is_writable);

    // Vault token in
    assert_eq!(accounts[6].pubkey, SOL_VAULT, "vault token in");
    assert!(accounts[6].is_writable);

    // Vault token out
    assert_eq!(accounts[7].pubkey, USDC_VAULT, "vault token out");
    assert!(accounts[7].is_writable);

    // Beneficiary token out
    let expected_beneficiary_token_out =
        get_associated_token_address(&FEE_VAULT_AUTHORITY, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[8].pubkey, expected_beneficiary_token_out,
        "beneficiary token out"
    );
    assert!(accounts[8].is_writable);

    // Pool
    assert_eq!(accounts[9].pubkey, POOL, "pool");
    assert!(accounts[9].is_writable);

    // Withdraw authority
    assert_eq!(
        accounts[10].pubkey,
        get_withdraw_authority_address(&POOL),
        "withdraw authority"
    );

    // Vault
    assert_eq!(accounts[11].pubkey, VAULT_STATE, "vault");

    // Vault authority
    assert_eq!(
        accounts[12].pubkey,
        get_vault_authority_address(&VAULT_STATE)
    );

    // Vault program
    assert_eq!(accounts[13].pubkey, VAULT_PROGRAM, "vault program");

    // Token program
    assert_eq!(accounts[14].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Token 2022 program
    assert_eq!(
        accounts[15].pubkey, TOKEN_2022_PROGRAM_ID,
        "token 2022 program"
    );

    assert!(data.is_empty());
}

#[tokio::test]
async fn test_stabble_weighted_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::StabbleWeighted { pool: POOL },
        &USDC_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 16, "stabble weighted requires 16 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, STABBLE_WEIGHTED_SWAP_PROGRAM_ID,
        "stabble weighted program"
    );

    // User
    assert_eq!(accounts[1].pubkey, user, "user");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Mint in
    assert_eq!(accounts[2].pubkey, USDC_MINT, "mint in");

    // Mint out
    assert_eq!(accounts[3].pubkey, WSOL_MINT, "mint out");

    // User token in
    let expected_user_token_in = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[4].pubkey, expected_user_token_in, "user token in");
    assert!(accounts[4].is_writable);

    // User token out
    let expected_user_token_out =
        get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[5].pubkey, expected_user_token_out,
        "user token out"
    );
    assert!(accounts[5].is_writable);

    // Vault token in
    assert_eq!(accounts[6].pubkey, USDC_VAULT, "vault token in");
    assert!(accounts[6].is_writable);

    // Vault token out
    assert_eq!(accounts[7].pubkey, SOL_VAULT, "vault token out");
    assert!(accounts[7].is_writable);

    // Beneficiary token out
    let expected_beneficiary_token_out =
        get_associated_token_address(&FEE_VAULT_AUTHORITY, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[8].pubkey, expected_beneficiary_token_out,
        "beneficiary token out"
    );
    assert!(accounts[8].is_writable);

    // Pool
    assert_eq!(accounts[9].pubkey, POOL, "pool");
    assert!(accounts[9].is_writable);

    // Withdraw authority
    assert_eq!(
        accounts[10].pubkey,
        get_withdraw_authority_address(&POOL),
        "withdraw authority"
    );

    // Vault
    assert_eq!(accounts[11].pubkey, VAULT_STATE, "vault");

    // Vault authority
    assert_eq!(
        accounts[12].pubkey,
        get_vault_authority_address(&VAULT_STATE)
    );

    // Vault program
    assert_eq!(accounts[13].pubkey, VAULT_PROGRAM, "vault program");

    // Token program
    assert_eq!(accounts[14].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Token 2022 program
    assert_eq!(
        accounts[15].pubkey, TOKEN_2022_PROGRAM_ID,
        "token 2022 program"
    );

    assert!(data.is_empty());
}
