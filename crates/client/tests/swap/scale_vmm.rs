use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::scale_vmm::{Side, SCALE_AMM_PROGRAM_ID, SCALE_VMM_PROGRAM_ID},
        SwapProtocol, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const MINT_B: Address = address!("3CYBUFXzQ7GJiYoxMfrFjsPNVaSkp2vVFXXLefb57chr");
const PAIR: Address = address!("BWnowWbMBTfsLzTKgZM7vnh8SxbutJA2HB4z7Labswkb");
const PLATFORM_FEE_TA_A: Address = address!("5otzrfbppNE1j6m7ptWkAcu5gs1nwj5ZQKQVwFFHQAHv");
const VAULT_A: Address = address!("Tf8aks7NB82QXob8NAoorzrPFHwkuPoZ9GVLYNLjYYZ");
const VAULT_B: Address = address!("AaP7zPXf22rpmX27n4t6ohRDPLx7mcXDeodFN9rmbhQh");
const PLATFORM_CONFIG: Address = address!("8DxXv6ikV38rCepX3esVCHMb2wMnnnXpp7xYasGSc6bo");
const AMM_POOL: Address = address!("2Lt3pqPLCDzxizyHM8cx1auySxNYxFwmf1JwnEfJtHbw");
const AMM_VAULT_A: Address = address!("8drJbd2DZxcMqDcv8AZf1C1Y7wKBHS2JWXfEgUkjKwtz");
const AMM_VAULT_B: Address = address!("6K5ekuXPF7iAyWpMkTijenLQxtQYZJozY7PJjfFCZSp4");
const AMM_CONFIG: Address = address!("232KbYciAe6ma2VCB6gQyofix8qQwyZd2WYVhpNx8SyR");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_scale_vmm_resolve_with_known_pair() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, _data) = resolve_swap(
        &rpc,
        &SwapProtocol::ScaleVmm {
            pair: Some(PAIR),
            side: Side::Buy,
        },
        &WSOL_MINT,
        &MINT_B,
        &user,
    )
    .await
    .unwrap();

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, SCALE_VMM_PROGRAM_ID,
        "scale_vmm program"
    );

    // Pair
    assert_eq!(accounts[1].pubkey, PAIR, "pair");
    assert!(accounts[1].is_writable);

    // User
    assert_eq!(accounts[2].pubkey, user, "user");
    assert!(accounts[2].is_signer);
    assert!(accounts[2].is_writable);

    // Mint a
    assert_eq!(accounts[3].pubkey, WSOL_MINT, "mint_a");

    // Mint b
    assert_eq!(accounts[4].pubkey, MINT_B, "mint_b");

    // User ta a
    let expected_user_ta_a = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_user_ta_a, "user_ta_a ATA");
    assert!(accounts[5].is_writable);

    // User ta b
    let expected_user_ta_b = get_associated_token_address(&user, &MINT_B, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[6].pubkey, expected_user_ta_b, "user_ta_b ATA");
    assert!(accounts[6].is_writable);

    // Vault a
    assert_eq!(accounts[7].pubkey, VAULT_A, "vault_a ATA");
    assert!(accounts[7].is_writable);

    // Vault b
    assert_eq!(accounts[8].pubkey, VAULT_B, "vault_b ATA");
    assert!(accounts[8].is_writable);

    // Platform fee ta a
    assert_eq!(
        accounts[9].pubkey, PLATFORM_FEE_TA_A,
        "platform_fee_ta_a ATA"
    );
    assert!(accounts[9].is_writable);

    // Token program a
    assert_eq!(accounts[10].pubkey, TOKEN_PROGRAM_ID, "token_program_a");

    // Token program b
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "token_program_b");

    // System program
    assert_eq!(accounts[12].pubkey, SYSTEM_PROGRAM_ID, "system program");

    // Config
    assert_eq!(accounts[13].pubkey, PLATFORM_CONFIG, "config");

    // AMM program
    assert_eq!(accounts[14].pubkey, SCALE_AMM_PROGRAM_ID, "amm program");

    // AMM pool
    assert_eq!(accounts[15].pubkey, AMM_POOL, "amm pool");

    // AMM vault a
    assert_eq!(accounts[16].pubkey, AMM_VAULT_A, "amm_vault_a ATA");
    assert!(accounts[16].is_writable);

    // AMM vault b
    assert_eq!(accounts[17].pubkey, AMM_VAULT_B, "amm_vault_b ATA");
    assert!(accounts[17].is_writable);

    // AMM config
    assert_eq!(accounts[18].pubkey, AMM_CONFIG, "amm config");
}

#[tokio::test]
async fn test_scale_vmm_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::ScaleVmm {
            pair: Some(PAIR),
            side: Side::Sell,
        },
        &WSOL_MINT,
        &MINT_B,
        &user,
    )
    .await
    .unwrap();

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, SCALE_VMM_PROGRAM_ID,
        "scale_vmm program"
    );

    // Pair
    assert_eq!(accounts[1].pubkey, PAIR, "pair");
    assert!(accounts[1].is_writable);

    // User
    assert_eq!(accounts[2].pubkey, user, "user");
    assert!(accounts[2].is_signer);
    assert!(accounts[2].is_writable);

    // Mint a
    assert_eq!(accounts[3].pubkey, WSOL_MINT, "mint_a");

    // Mint b
    assert_eq!(accounts[4].pubkey, MINT_B, "mint_b");

    // User ta a
    let expected_user_ta_a = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_user_ta_a, "user_ta_a ATA");
    assert!(accounts[5].is_writable);

    // User ta b
    let expected_user_ta_b = get_associated_token_address(&user, &MINT_B, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[6].pubkey, expected_user_ta_b, "user_ta_b ATA");
    assert!(accounts[6].is_writable);

    // Vault a
    assert_eq!(accounts[7].pubkey, VAULT_A, "vault_a ATA");
    assert!(accounts[7].is_writable);

    // Vault b
    assert_eq!(accounts[8].pubkey, VAULT_B, "vault_b ATA");
    assert!(accounts[8].is_writable);

    // Platform fee ta a
    assert_eq!(
        accounts[9].pubkey, PLATFORM_FEE_TA_A,
        "platform_fee_ta_a ATA"
    );
    assert!(accounts[9].is_writable);

    // Token program a
    assert_eq!(accounts[10].pubkey, TOKEN_PROGRAM_ID, "token_program_a");

    // Token program b
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "token_program_b");

    // System program
    assert_eq!(accounts[12].pubkey, SYSTEM_PROGRAM_ID, "system program");

    // Config
    assert_eq!(accounts[13].pubkey, PLATFORM_CONFIG, "config");

    // AMM program
    assert_eq!(accounts[14].pubkey, SCALE_AMM_PROGRAM_ID, "amm program");

    // AMM pool
    assert_eq!(accounts[15].pubkey, AMM_POOL, "amm pool");

    // AMM vault a
    assert_eq!(accounts[16].pubkey, AMM_VAULT_A, "amm_vault_a ATA");
    assert!(accounts[16].is_writable);

    // AMM vault b
    assert_eq!(accounts[17].pubkey, AMM_VAULT_B, "amm_vault_b ATA");
    assert!(accounts[17].is_writable);

    // AMM config
    assert_eq!(accounts[18].pubkey, AMM_CONFIG, "amm config");

    // side
    assert_eq!(data, vec![Side::Sell as u8]);
}
