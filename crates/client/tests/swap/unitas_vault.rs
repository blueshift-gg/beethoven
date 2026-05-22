use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::unitas_vault::{
            ACCESS_REGISTRY, SUSDU_CONFIG, SUSDU_MINT, SUSDU_MINTER, SUSDU_PROGRAM_ID,
            UNITAS_VAULT_PROGRAM_ID, USDU_MINT, VAULT_CONFIG, VAULT_STAKE_POOL_USDU_TOKEN_ACCOUNT,
            VAULT_STATE,
        },
        SwapProtocol, ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_2022_PROGRAM_ID,
    },
    solana_address::address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_unitas_vault() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::UnitasVault,
        &USDU_MINT,
        &SUSDU_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 18, "unitas_vault requires 18 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, UNITAS_VAULT_PROGRAM_ID);

    // Caller account
    assert_eq!(accounts[1].pubkey, user);
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Receiver account
    assert_eq!(accounts[2].pubkey, user);
    assert!(accounts[2].is_signer);
    assert!(accounts[2].is_writable);

    // Receiver sUSDU token account
    let expected_receiver_susdu_token_account =
        get_associated_token_address(&user, &SUSDU_MINT, &TOKEN_2022_PROGRAM_ID);
    assert_eq!(accounts[3].pubkey, expected_receiver_susdu_token_account);
    assert!(accounts[3].is_writable);

    // Caller USDU token account
    let expected_caller_usdu_token_account =
        get_associated_token_address(&user, &USDU_MINT, &TOKEN_2022_PROGRAM_ID);
    assert_eq!(accounts[4].pubkey, expected_caller_usdu_token_account);
    assert!(accounts[4].is_writable);

    // Access registry
    assert_eq!(accounts[5].pubkey, ACCESS_REGISTRY);

    // Vault stake pool USDU token account
    assert_eq!(accounts[6].pubkey, VAULT_STAKE_POOL_USDU_TOKEN_ACCOUNT);
    assert!(accounts[6].is_writable);

    // sUSDU minter
    assert_eq!(accounts[7].pubkey, SUSDU_MINTER);

    // USDU token mint
    assert_eq!(accounts[8].pubkey, USDU_MINT);
    assert!(accounts[8].is_writable);

    // sUSDU token mint
    assert_eq!(accounts[9].pubkey, SUSDU_MINT);
    assert!(accounts[9].is_writable);

    // Vault state
    assert_eq!(accounts[10].pubkey, VAULT_STATE);

    // Vault config
    assert_eq!(accounts[11].pubkey, VAULT_CONFIG);
    assert!(accounts[11].is_writable);

    // sUSDU config
    assert_eq!(accounts[12].pubkey, SUSDU_CONFIG);
    assert!(accounts[12].is_writable);

    // sUSDU program
    assert_eq!(accounts[13].pubkey, SUSDU_PROGRAM_ID);

    // USDU token program
    assert_eq!(accounts[14].pubkey, TOKEN_2022_PROGRAM_ID);

    // sUSDU token program
    assert_eq!(accounts[15].pubkey, TOKEN_2022_PROGRAM_ID);

    // Associated token program
    assert_eq!(accounts[16].pubkey, ASSOCIATED_TOKEN_PROGRAM_ID);

    // System program
    assert_eq!(accounts[17].pubkey, SYSTEM_PROGRAM_ID);

    // No extra data
    assert!(data.is_empty());
}
