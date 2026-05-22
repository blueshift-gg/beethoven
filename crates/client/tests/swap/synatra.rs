use {
    beethoven_client::{
        get_associated_token_address, resolve_swap, swap::synatra::SYNATRA_PROGRAM_ID,
        SwapProtocol, ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111111");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const YSOL_MINT: Address = address!("yso11zxLbHA3wBJ9HAtVu6wnesqz9A2qxnhxanasZ4N");
const YUSD_MINT: Address = address!("yUSDX7W89jXWn4zzDPLnhykDymSjQSmpaJ8e4fjC1fg");
const YSOL_POOL: Address = address!("2wMDWx7a1PpbrsnNAHGJLPMgRs7H3pcYxqmmkQrzLxHg");
const YUSD_POOL: Address = address!("Fm8E4fEAiRraWP2EhMfycyYzYdvNgzQiKUwhxCCUB4ho");
const YUSD_POOL_ATA: Address = address!("DME9KG2K16wTWvpMrijFHDzsENZAf1YS7DLwzWG2AiHU");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_synatra_resolve_with_known_pool_stake_sol() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Synatra {
            pool: Some(YSOL_POOL),
        },
        &WSOL_MINT,
        &YSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 9, "synatra stake sol requires 9 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, SYNATRA_PROGRAM_ID);

    // Signer
    assert_eq!(accounts[1].pubkey, user);
    assert!(accounts[1].is_signer);

    // Payer
    assert_eq!(accounts[2].pubkey, user);
    assert!(accounts[2].is_writable);

    // Pool
    assert_eq!(accounts[3].pubkey, YSOL_POOL);
    assert!(accounts[3].is_writable);

    // Receipt token
    assert_eq!(accounts[4].pubkey, YSOL_MINT);
    assert!(accounts[4].is_writable);

    // User receipt ATA
    let expected_user_receipt_ata =
        get_associated_token_address(&user, &YSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_user_receipt_ata);
    assert!(accounts[5].is_writable);

    // Associated token program
    assert_eq!(accounts[6].pubkey, ASSOCIATED_TOKEN_PROGRAM_ID);

    // Token program
    assert_eq!(accounts[7].pubkey, TOKEN_PROGRAM_ID);

    // System program
    assert_eq!(accounts[8].pubkey, SYSTEM_PROGRAM_ID);

    // No extra data
    assert!(data.is_empty());
}

#[tokio::test]
async fn test_omnipair_resolve_with_known_pool_stake_token() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Synatra {
            pool: Some(YUSD_POOL),
        },
        &USDC_MINT,
        &YUSD_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(
        accounts.len(),
        12,
        "synatra stake token requires 12 accounts"
    );

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, SYNATRA_PROGRAM_ID);

    // Signer
    assert_eq!(accounts[1].pubkey, user);
    assert!(accounts[1].is_signer);

    // Payer
    assert_eq!(accounts[2].pubkey, user);
    assert!(accounts[2].is_writable);

    // Pool
    assert_eq!(accounts[3].pubkey, YUSD_POOL);
    assert!(accounts[3].is_writable);

    // Stake token
    assert_eq!(accounts[4].pubkey, USDC_MINT);
    assert!(accounts[4].is_writable);

    // Receipt token
    assert_eq!(accounts[5].pubkey, YUSD_MINT);
    assert!(accounts[5].is_writable);

    // User token ATA
    let expected_user_token_ata =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[6].pubkey, expected_user_token_ata);
    assert!(accounts[6].is_writable);

    // User receipt ATA
    let expected_user_receipt_ata =
        get_associated_token_address(&user, &YUSD_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[7].pubkey, expected_user_receipt_ata);
    assert!(accounts[7].is_writable);

    // Pool token ATA
    assert_eq!(accounts[8].pubkey, YUSD_POOL_ATA);
    assert!(accounts[8].is_writable);

    // Associated token program
    assert_eq!(accounts[9].pubkey, ASSOCIATED_TOKEN_PROGRAM_ID);

    // Token program
    assert_eq!(accounts[10].pubkey, TOKEN_PROGRAM_ID);

    // System program
    assert_eq!(accounts[11].pubkey, SYSTEM_PROGRAM_ID);

    // No extra data
    assert!(data.is_empty());
}
