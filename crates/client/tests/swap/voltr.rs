use {
    beethoven_client::{
        get_associated_token_address, resolve_swap, swap::voltr::VOLTR_PROGRAM_ID, SwapProtocol,
        SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const PROTOCOL: Address = address!("4sycXz9Xwevedo6eiXR8QEhY8yrQrkNS4G1deY9tAD2Y");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_voltr_resolve_with_known_vault_deposit_vault() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    const RAUSDC_MINT: Address = address!("53fZaJGDMHcfku8pzZak5obVFUUjVxwqRTF63M3SQiSS");
    const VAULT: Address = address!("3maCuTJVPteZ2dFA8dADxz2EbpJHfoAG5txYhXDs6gNQ");
    const VAULT_ASSET_IDLE_ATA: Address = address!("3iKiu9CYBqNSPJ9GdNd46BGMFtwQ27N1qJpXSocpo5wm");
    const VAULT_ASSET_IDLE_AUTH: Address = address!("F5FT74NET1Y6JTJyNCioGYpyWXqEYTnvNfb6gh7aM8Yn");
    const VAULT_LP_MINT_AUTH: Address = address!("FFh6frp7DsAyCkP1275yVndhpaWfMtevk9sk6BrZB7V8");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Voltr {
            vault: Some(VAULT),
            is_amount_in_lp: None,
            is_withdraw_all: None,
        },
        &USDC_MINT,
        &RAUSDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(
        accounts.len(),
        14,
        "voltr deposit_vault requires 14 accounts"
    );

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, VOLTR_PROGRAM_ID, "voltr program ID");

    // User transfer authority
    assert_eq!(accounts[1].pubkey, user, "user transfer authority");
    assert!(accounts[1].is_signer);

    // Protocol
    assert_eq!(accounts[2].pubkey, PROTOCOL, "protocol");

    // Vault
    assert_eq!(accounts[3].pubkey, VAULT, "vault");
    assert!(accounts[3].is_writable);

    // Vault asset mint
    assert_eq!(accounts[4].pubkey, USDC_MINT, "vault asset mint");

    // Vault lp mint
    assert_eq!(accounts[5].pubkey, RAUSDC_MINT, "vault lp mint");
    assert!(accounts[5].is_writable);

    // User asset ata
    let expected_user_asset_ata =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[6].pubkey, expected_user_asset_ata,
        "user asset ata"
    );
    assert!(accounts[6].is_writable);

    // Vault asset idle ata
    assert_eq!(
        accounts[7].pubkey, VAULT_ASSET_IDLE_ATA,
        "vault asset idle ata"
    );
    assert!(accounts[7].is_writable);

    // Vault asset idle auth
    assert_eq!(
        accounts[8].pubkey, VAULT_ASSET_IDLE_AUTH,
        "vault asset idle auth"
    );

    // User lp ata
    let expected_user_lp_ata = get_associated_token_address(&user, &RAUSDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[9].pubkey, expected_user_lp_ata, "user lp ata");
    assert!(accounts[9].is_writable);

    // Vault lp mint auth
    assert_eq!(
        accounts[10].pubkey, VAULT_LP_MINT_AUTH,
        "vault lp mint auth"
    );

    // Asset token program
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "asset token program");

    // Lp token program
    assert_eq!(accounts[12].pubkey, TOKEN_PROGRAM_ID, "lp token program");

    // System program
    assert_eq!(accounts[13].pubkey, SYSTEM_PROGRAM_ID, "system program");

    // deposit_vault has no extra data
    assert!(data.is_empty());
}

#[tokio::test]
async fn test_voltr_resolve_with_known_vault_instant_withdraw_vault() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    const STWUSDC_MINT: Address = address!("KNxz3PQeNvPB2Xa4TYcwkZ2a7VCJgZLJnd6i9ho2EsE");
    const VAULT: Address = address!("AYfsjUSmHEksrqBVAyjmy4t9aDnHjc245jz2hrUWhMrc");
    const VAULT_ASSET_IDLE_ATA: Address = address!("FsBpt92rK8FmqcBais73aYqfDvDgrETbKbzWpjnuaUAf");
    const VAULT_ASSET_IDLE_AUTH: Address = address!("4MJHRWd4NR6P6vdbka2Ai9FN75hXrmv77Qqi94CWhayt");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Voltr {
            vault: Some(VAULT),
            is_amount_in_lp: Some(false),
            is_withdraw_all: Some(false),
        },
        &USDC_MINT,
        &STWUSDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(
        accounts.len(),
        13,
        "voltr deposit_vault requires 13 accounts"
    );

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, VOLTR_PROGRAM_ID, "voltr program ID");

    // User transfer authority
    assert_eq!(accounts[1].pubkey, user, "user transfer authority");
    assert!(accounts[1].is_signer);

    // Protocol
    assert_eq!(accounts[2].pubkey, PROTOCOL, "protocol");

    // Vault
    assert_eq!(accounts[3].pubkey, VAULT, "vault");
    assert!(accounts[3].is_writable);

    // Vault asset mint
    assert_eq!(accounts[4].pubkey, USDC_MINT, "vault asset mint");

    // Vault lp mint
    assert_eq!(accounts[5].pubkey, STWUSDC_MINT, "vault lp mint");
    assert!(accounts[5].is_writable);

    // User lp ata
    let expected_user_lp_ata =
        get_associated_token_address(&user, &STWUSDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[6].pubkey, expected_user_lp_ata, "user lp ata");
    assert!(accounts[6].is_writable);

    // Vault asset idle ata
    assert_eq!(
        accounts[7].pubkey, VAULT_ASSET_IDLE_ATA,
        "vault asset idle ata"
    );
    assert!(accounts[7].is_writable);

    // Vault asset idle auth
    assert_eq!(
        accounts[8].pubkey, VAULT_ASSET_IDLE_AUTH,
        "vault asset idle auth"
    );

    // User asset ata
    let expected_user_asset_ata =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[9].pubkey, expected_user_asset_ata,
        "user asset ata"
    );
    assert!(accounts[9].is_writable);

    // Asset token program
    assert_eq!(accounts[10].pubkey, TOKEN_PROGRAM_ID, "asset token program");

    // Lp token program
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "lp token program");

    // System program
    assert_eq!(accounts[12].pubkey, SYSTEM_PROGRAM_ID, "system program");

    // is_amount_in_lp
    assert_eq!(data[0], 0u8);

    // is_withdraw_all
    assert_eq!(data[1], 0u8);
}
