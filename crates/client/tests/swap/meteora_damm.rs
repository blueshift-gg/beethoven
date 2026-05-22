use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::meteora_damm::{METEORA_DAMM_PROGRAM_ID, METEORA_DYNAMIC_VAULT_PROGRAM_ID},
        SwapProtocol, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDT_MINT: Address = address!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
const POOL: Address = address!("32D4zRxNc1EssbJieVHfPhZM3rH6CzfUPrWUuWxD9prG");
const A_VAULT: Address = address!("3ESUFCnRNgZ7Mn2mPPUMmXYaKU8jpnV9VtA17M7t2mHQ");
const B_VAULT: Address = address!("5XCP3oD3JAuQyDpfBFFVUxsBxNjPQojpKuL4aVhHsDok");
const A_TOKEN_VAULT: Address = address!("C2QoQ111jGHEy5918XkNXQro7gGwC9PKLXd1LqBiYNwA");
const B_TOKEN_VAULT: Address = address!("DQjGWHN9ERn1zSMpWLNvSpTFUSfnxbanBt9A7xyU2bVE");
const A_VAULT_LP_MINT: Address = address!("3RpEekjLE5cdcG15YcXJUpxSepemvq2FpmMcgo342BwC");
const B_VAULT_LP_MINT: Address = address!("EZun6G5514FeqYtUv26cBHWLqXjAEdjGuoX6ThBpBtKj");
const A_VAULT_LP: Address = address!("24NYE3hHQyUTrHUT4n1CcVrMP9Xy3ULuT1Uurw1HDeck");
const B_VAULT_LP: Address = address!("Hv5ogVb2BZCF3ET2KnaEYj2seKHN5ffGDazm6BGt5DD9");
const PROTOCOL_TOKEN_FEE: Address = address!("4Qjrnzp5jXPSBhyv495ApB1SdDbXdZ5Pc9ZSiabf9NmJ");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_meteora_damm_resolve_with_known_pool() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("1111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::MeteoraDamm { pool: Some(POOL) },
        &USDC_MINT,
        &USDT_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 16, "meteora damm requires 16 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, METEORA_DAMM_PROGRAM_ID,
        "meteora damm program"
    );

    // Pool
    assert_eq!(accounts[1].pubkey, POOL, "pool");
    assert!(accounts[1].is_writable);

    // User source token
    let expected_user_source_token =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[2].pubkey, expected_user_source_token,
        "user source token"
    );
    assert!(accounts[2].is_writable);

    // User destination token
    let expected_user_destination_token =
        get_associated_token_address(&user, &USDT_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[3].pubkey, expected_user_destination_token,
        "user destination token"
    );
    assert!(accounts[3].is_writable);

    // A vault
    assert_eq!(accounts[4].pubkey, A_VAULT, "a vault");
    assert!(accounts[4].is_writable);

    // B vault
    assert_eq!(accounts[5].pubkey, B_VAULT, "b vault");
    assert!(accounts[5].is_writable);

    // A token vault
    assert_eq!(accounts[6].pubkey, A_TOKEN_VAULT, "a token vault");
    assert!(accounts[6].is_writable);

    // B token vault
    assert_eq!(accounts[7].pubkey, B_TOKEN_VAULT, "b token vault");
    assert!(accounts[7].is_writable);

    // A vault LP mint
    assert_eq!(accounts[8].pubkey, A_VAULT_LP_MINT, "a vault LP mint");
    assert!(accounts[8].is_writable);

    // B vault LP mint
    assert_eq!(accounts[9].pubkey, B_VAULT_LP_MINT, "b vault LP mint");
    assert!(accounts[9].is_writable);

    // A vault LP
    assert_eq!(accounts[10].pubkey, A_VAULT_LP, "a vault LP");
    assert!(accounts[10].is_writable);

    // B vault LP
    assert_eq!(accounts[11].pubkey, B_VAULT_LP, "b vault LP");
    assert!(accounts[11].is_writable);

    // Protocol token fee
    assert_eq!(
        accounts[12].pubkey, PROTOCOL_TOKEN_FEE,
        "protocol token fee"
    );
    assert!(accounts[12].is_writable);

    // User
    assert_eq!(accounts[13].pubkey, user, "user");
    assert!(accounts[13].is_signer);
    assert!(accounts[13].is_writable);

    // Vault program
    assert_eq!(
        accounts[14].pubkey, METEORA_DYNAMIC_VAULT_PROGRAM_ID,
        "vault program"
    );

    // Token program
    assert_eq!(accounts[15].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Swap has no extra data
    assert!(data.is_empty());
}
