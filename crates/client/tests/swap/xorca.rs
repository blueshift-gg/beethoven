use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::xorca::{ORCA_MINT, ORCA_VAULT, STATE_ACCOUNT, XORCA_MINT, XORCA_PROGRAM_ID},
        SwapProtocol, TOKEN_PROGRAM_ID,
    },
    solana_address::address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_xorca() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(&rpc, &SwapProtocol::Xorca, &ORCA_MINT, &XORCA_MINT, &user)
        .await
        .unwrap();

    assert_eq!(accounts.len(), 9, "xorca requires 9 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, XORCA_PROGRAM_ID);

    // Staker account
    assert_eq!(accounts[1].pubkey, user);
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Vault account
    assert_eq!(accounts[2].pubkey, ORCA_VAULT);
    assert!(accounts[2].is_writable);

    // Staker ORCA ata account
    let expected_staker_orca_ata =
        get_associated_token_address(&user, &ORCA_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[3].pubkey, expected_staker_orca_ata);
    assert!(accounts[3].is_writable);

    // Staker xORCA ata account
    let expected_staker_xorca_ata =
        get_associated_token_address(&user, &XORCA_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[4].pubkey, expected_staker_xorca_ata);
    assert!(accounts[4].is_writable);

    // xORCA mint
    assert_eq!(accounts[5].pubkey, XORCA_MINT);
    assert!(accounts[5].is_writable);

    // State account
    assert_eq!(accounts[6].pubkey, STATE_ACCOUNT);

    // ORCA mint account
    assert_eq!(accounts[7].pubkey, ORCA_MINT);

    // Token program account
    assert_eq!(accounts[8].pubkey, TOKEN_PROGRAM_ID);

    // No extra data
    assert!(data.is_empty());
}
