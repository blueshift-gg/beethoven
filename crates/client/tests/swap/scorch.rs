use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::scorch::{ORACLE_PROGRAM_ID, SCORCH_PROGRAM_ID},
        SwapProtocol, MEMO_PROGRAM_ID, SYSVAR_INSTRUCTIONS_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const MARKET: Address = address!("EHcege7dok1iYs7SxL2XzDPvhg6XzMVcx2V5SkMUurJP");
const SOL_MARKET_TA: Address = address!("44GW6aFire4Fd72h4QqjenNKrQLjfnApHhWYXo3S1gvp");
const USDC_MARKET_TA: Address = address!("34WpFzQ1WE2nDLLEKJuvCXxapL7ak6CigJS8Ks5NDo5K");
const ACC_1: Address = address!("HLixVmXdBqzP1sXT9au4BHcvUjDgx5ev16cEJdd9tUSM");
const STATE_A: Address = address!("85Etk23kFtyt265MQjyUgzJYZ7u5o2EVNdjDmuNySbGi");
const STATE_B: Address = address!("DmocjvFXp75asezCDKVNH2qaXbjrV7VeQk4aLZaPF88E");
const STATE_C: Address = address!("FnhxUP3dcQbypCUmGWw55ijxPBxifPT558UQSCYDfcCU");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_scorch_resolve_with_known_market() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let param = [
        0xe0, 0xbe, 0x8c, 0xae, 0x67, 0xc2, 0xbc, 0x97, 0x89, 0x0a, 0x00, 0x00, 0x0c, 0x00, 0x00,
        0xf9, 0x00,
    ];

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Scorch {
            market: Some(MARKET),
            acc1: ACC_1,
            state_a: STATE_A,
            state_b: STATE_B,
            state_c: STATE_C,
            param,
        },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 18, "scorch requires 18 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, SCORCH_PROGRAM_ID);

    // Market
    assert_eq!(accounts[1].pubkey, MARKET);

    // User
    assert_eq!(accounts[2].pubkey, user);
    assert!(accounts[2].is_signer);
    assert!(accounts[2].is_writable);

    // User ata a
    let expected_ata_a = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[3].pubkey, expected_ata_a);

    // User ata b
    let expected_ata_b = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[4].pubkey, expected_ata_b);

    // Market ta a
    assert_eq!(accounts[5].pubkey, SOL_MARKET_TA);
    assert!(accounts[5].is_writable);

    // Market ta b
    assert_eq!(accounts[6].pubkey, USDC_MARKET_TA);
    assert!(accounts[6].is_writable);

    // Mint a
    assert_eq!(accounts[7].pubkey, WSOL_MINT);

    // Mint b
    assert_eq!(accounts[8].pubkey, USDC_MINT);

    // Token program a
    assert_eq!(accounts[9].pubkey, TOKEN_PROGRAM_ID);

    // Token program b
    assert_eq!(accounts[10].pubkey, TOKEN_PROGRAM_ID);

    // Memo program
    assert_eq!(accounts[11].pubkey, MEMO_PROGRAM_ID);

    // Oracle program
    assert_eq!(accounts[12].pubkey, ORACLE_PROGRAM_ID);

    // Acc 1
    assert_eq!(accounts[13].pubkey, ACC_1);

    // State a
    assert_eq!(accounts[14].pubkey, STATE_A);
    assert!(accounts[14].is_writable);

    // State b
    assert_eq!(accounts[15].pubkey, STATE_B);
    assert!(accounts[15].is_writable);

    // State c
    assert_eq!(accounts[16].pubkey, STATE_C);

    // Sysvar instructions
    assert_eq!(accounts[17].pubkey, SYSVAR_INSTRUCTIONS_ID);

    // scorch param
    assert_eq!(data, param.to_vec());
}

#[tokio::test]
async fn test_scorch_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let param = [
        0xe0, 0xbe, 0x8c, 0xae, 0x67, 0xc2, 0xbc, 0x97, 0x89, 0x0a, 0x00, 0x00, 0x0c, 0x00, 0x00,
        0xf9, 0x00,
    ];

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Scorch {
            market: Some(MARKET),
            acc1: ACC_1,
            state_a: STATE_A,
            state_b: STATE_B,
            state_c: STATE_C,
            param,
        },
        &USDC_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 18, "scorch requires 18 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, SCORCH_PROGRAM_ID);

    // Market
    assert_eq!(accounts[1].pubkey, MARKET);

    // User
    assert_eq!(accounts[2].pubkey, user);
    assert!(accounts[2].is_signer);
    assert!(accounts[2].is_writable);

    // User ata a
    let expected_ata_a = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[3].pubkey, expected_ata_a);

    // User ata b
    let expected_ata_b = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[4].pubkey, expected_ata_b);

    // Market ta a
    assert_eq!(accounts[5].pubkey, USDC_MARKET_TA);
    assert!(accounts[5].is_writable);

    // Market ta b
    assert_eq!(accounts[6].pubkey, SOL_MARKET_TA);
    assert!(accounts[6].is_writable);

    // Mint a
    assert_eq!(accounts[7].pubkey, USDC_MINT);

    // Mint b
    assert_eq!(accounts[8].pubkey, WSOL_MINT);

    // Token program a
    assert_eq!(accounts[9].pubkey, TOKEN_PROGRAM_ID);

    // Token program b
    assert_eq!(accounts[10].pubkey, TOKEN_PROGRAM_ID);

    // Memo program
    assert_eq!(accounts[11].pubkey, MEMO_PROGRAM_ID);

    // Oracle program
    assert_eq!(accounts[12].pubkey, ORACLE_PROGRAM_ID);

    // Acc 1
    assert_eq!(accounts[13].pubkey, ACC_1);

    // State a
    assert_eq!(accounts[14].pubkey, STATE_A);
    assert!(accounts[14].is_writable);

    // State b
    assert_eq!(accounts[15].pubkey, STATE_B);
    assert!(accounts[15].is_writable);

    // State c
    assert_eq!(accounts[16].pubkey, STATE_C);

    // Sysvar instructions
    assert_eq!(accounts[17].pubkey, SYSVAR_INSTRUCTIONS_ID);

    // scorch param
    assert_eq!(data, param.to_vec());
}
