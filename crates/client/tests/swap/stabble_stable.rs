use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::stabble_stable::{
            get_vault_authority_address, get_withdraw_authority_address, FEE_VAULT_AUTHORITY,
            STABBLE_STABLE_SWAP_PROGRAM_ID, VAULT_PROGRAM, VAULT_STATE,
        },
        SwapProtocol, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDT_MINT: Address = address!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
const POOL: Address = address!("CXeH7npzb5UPWfB1TesjwxjXMT31XT4pjeUAQ4z65Wpg");
const USDC_VAULT: Address = address!("AioJRQXvcDLRhHMd6DAkTbbMpgVx63qSGQYmRBS2vHYA");
const USDT_VAULT: Address = address!("95QUtvDkuoDZrNJiuh9MdahkpRNtSVhZRe83oepd8AM7");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_stabble_stable_resolve_with_known_pool() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::StabbleStable { pool: POOL },
        &USDC_MINT,
        &USDT_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 16, "stabble stable requires 16 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, STABBLE_STABLE_SWAP_PROGRAM_ID,
        "stabble stable program"
    );

    // User
    assert_eq!(accounts[1].pubkey, user, "user");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Mint in
    assert_eq!(accounts[2].pubkey, USDC_MINT, "mint in");

    // Mint out
    assert_eq!(accounts[3].pubkey, USDT_MINT, "mint out");

    // User token in
    let expected_user_token_in = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[4].pubkey, expected_user_token_in, "user token in");
    assert!(accounts[4].is_writable);

    // User token out
    let expected_user_token_out =
        get_associated_token_address(&user, &USDT_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[5].pubkey, expected_user_token_out,
        "user token out"
    );
    assert!(accounts[5].is_writable);

    // Vault token in
    assert_eq!(accounts[6].pubkey, USDC_VAULT, "vault token in");
    assert!(accounts[6].is_writable);

    // Vault token out
    assert_eq!(accounts[7].pubkey, USDT_VAULT, "vault token out");
    assert!(accounts[7].is_writable);

    // Beneficiary token out
    let expected_beneficiary_token_out =
        get_associated_token_address(&FEE_VAULT_AUTHORITY, &USDT_MINT, &TOKEN_PROGRAM_ID);
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
        get_vault_authority_address(&VAULT_STATE),
        "vault authority"
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

    // Stabble Stable has no extra data
    assert!(data.is_empty());
}

#[tokio::test]
async fn test_stabble_stable_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::StabbleStable { pool: POOL },
        &USDT_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 16, "stabble stable requires 16 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, STABBLE_STABLE_SWAP_PROGRAM_ID,
        "stabble stable program"
    );

    // User
    assert_eq!(accounts[1].pubkey, user, "user");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Mint in
    assert_eq!(accounts[2].pubkey, USDT_MINT, "mint in");

    // Mint out
    assert_eq!(accounts[3].pubkey, USDC_MINT, "mint out");

    // User token in
    let expected_user_token_in = get_associated_token_address(&user, &USDT_MINT, &TOKEN_PROGRAM_ID);
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
    assert_eq!(accounts[6].pubkey, USDT_VAULT, "vault token in");
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
        get_vault_authority_address(&VAULT_STATE),
        "vault authority"
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

    // Stabble Stable has no extra data
    assert!(data.is_empty());
}
