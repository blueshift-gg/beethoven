use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::{
            sanctum_staking::{
                BOND_MINT_AUTHORITY, BOND_POOL, CLOUD_MINT, SANCTUM_STAKING_PROGRAM_ID,
                SCLOUD_MINT, VAULT,
            },
            SwapProtocol,
        },
        TOKEN_PROGRAM_ID,
    },
    solana_address::address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_sanctum_staking() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::SanctumStaking,
        &CLOUD_MINT,
        &SCLOUD_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 9, "sanctum_staking requires 9 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, SANCTUM_STAKING_PROGRAM_ID);

    // Authority
    assert_eq!(accounts[1].pubkey, user);
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Deposit from
    let expected_deposit_from = get_associated_token_address(&user, &CLOUD_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[2].pubkey, expected_deposit_from, "deposit from");
    assert!(accounts[2].is_writable);

    // Mint to
    let expected_mint_to = get_associated_token_address(&user, &SCLOUD_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[3].pubkey, expected_mint_to, "mint to");
    assert!(accounts[3].is_writable);

    // Vault
    assert_eq!(accounts[4].pubkey, VAULT);
    assert!(accounts[4].is_writable);

    // Bonded mint
    assert_eq!(accounts[5].pubkey, SCLOUD_MINT);
    assert!(accounts[5].is_writable);

    // Bond mint authority
    assert_eq!(accounts[6].pubkey, BOND_MINT_AUTHORITY);

    // Bond pool
    assert_eq!(accounts[7].pubkey, BOND_POOL);

    // Token program
    assert_eq!(accounts[8].pubkey, TOKEN_PROGRAM_ID);

    // No extra data
    assert!(data.is_empty());
}
