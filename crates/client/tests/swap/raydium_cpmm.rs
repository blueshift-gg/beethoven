use {
    beethoven_client::{
        get_associated_token_address, resolve_swap, swap::raydium_cpmm::RAYDIUM_CPMM_PROGRAM_ID,
        SwapProtocol, TOKEN_PROGRAM_ID,
    },
    solana_address::Address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = Address::from_str_const("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = Address::from_str_const("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const AMM_CONFIG: Address = Address::from_str_const("D4FPEruKEHrG5TenZ2mpDGEfu1iUvTiqBxvpU8HLBvC2");
const POOL_STATE: Address = Address::from_str_const("7JuwJuNU88gurFnyWeiyGKbFmExMWcmRZntn9imEzdny");
const SOL_VAULT: Address = Address::from_str_const("7VLUXrnSSDo9BfCa4NWaQs68g7ddDY1sdXBKW6Xswj9Y");
const USDC_VAULT: Address = Address::from_str_const("3rzbbW5Q8MA7sCaowf28hNgACNPecdS2zceWy7Ptzua9");
const OBSERVATION_STATE: Address =
    Address::from_str_const("4MYrPgjgFceyhtwhG1ZX8UVb4wn1aQB5wzMimtFqg7U8");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_raydium_cpmm_resolve_with_known_pool() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        // explicitly passed, pool state cannot be derived using input and output token mints alone
        &SwapProtocol::RaydiumCpmm {
            pool: Some(POOL_STATE),
        },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 14, "raydium cpmm requires 14 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, RAYDIUM_CPMM_PROGRAM_ID,
        "raydium cpmm program"
    );

    // Payer
    assert_eq!(accounts[1].pubkey, user, "payer");
    assert!(accounts[1].is_signer);

    // authority
    let (expected_authority, _) =
        Address::find_program_address(&[b"vault_and_lp_mint_auth_seed"], &RAYDIUM_CPMM_PROGRAM_ID);
    assert_eq!(accounts[2].pubkey, expected_authority, "authority");

    // AMM config
    assert_eq!(accounts[3].pubkey, AMM_CONFIG, "amm config");

    // Pool state
    assert_eq!(accounts[4].pubkey, POOL_STATE, "pool state");
    assert!(accounts[4].is_writable);

    // Input token account
    let expected_wsol_ata = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_wsol_ata, "input token account");
    assert!(accounts[5].is_writable);

    // Output token account
    let expected_usdc_ata = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[6].pubkey, expected_usdc_ata,
        "output token account"
    );
    assert!(accounts[6].is_writable);

    // Input vault
    assert_eq!(accounts[7].pubkey, SOL_VAULT, "input vault");
    assert!(accounts[7].is_writable);

    // Output vault
    assert_eq!(accounts[8].pubkey, USDC_VAULT, "output vault");
    assert!(accounts[8].is_writable);

    // Input token program
    assert_eq!(accounts[9].pubkey, TOKEN_PROGRAM_ID, "input token program");

    // Output token program
    assert_eq!(
        accounts[10].pubkey, TOKEN_PROGRAM_ID,
        "output token program"
    );

    // Input token mint
    assert_eq!(accounts[11].pubkey, WSOL_MINT, "input token mint");

    // Output token mint
    assert_eq!(accounts[12].pubkey, USDC_MINT, "output token mint");

    // Observation state
    assert_eq!(accounts[13].pubkey, OBSERVATION_STATE, "observation state");
    assert!(accounts[13].is_writable);

    // Raydium CPMM swap_base_input has no extra data
    assert!(data.is_empty());
}

#[tokio::test]
async fn test_raydium_cpmm_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    // Selling USDC for WSOL — mints and ATAs should be flipped
    let (accounts, data) = resolve_swap(
        &rpc,
        // explicitly passed, pool state cannot be derived using input and output token mints alone
        &SwapProtocol::RaydiumCpmm {
            pool: Some(POOL_STATE),
        },
        &USDC_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 14, "raydium cpmm requires 14 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, RAYDIUM_CPMM_PROGRAM_ID,
        "raydium cpmm program"
    );

    // Payer
    assert_eq!(accounts[1].pubkey, user, "payer");
    assert!(accounts[1].is_signer);

    // authority
    let (expected_authority, _) =
        Address::find_program_address(&[b"vault_and_lp_mint_auth_seed"], &RAYDIUM_CPMM_PROGRAM_ID);
    assert_eq!(accounts[2].pubkey, expected_authority, "authority");

    // AMM config
    assert_eq!(accounts[3].pubkey, AMM_CONFIG, "amm config");

    // Pool state
    assert_eq!(accounts[4].pubkey, POOL_STATE, "pool state");
    assert!(accounts[4].is_writable);

    // Input token account (USDC)
    let expected_usdc_ata = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_usdc_ata, "input token account");
    assert!(accounts[5].is_writable);

    // Output token account (WSOL)
    let expected_wsol_ata = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[6].pubkey, expected_wsol_ata,
        "output token account"
    );
    assert!(accounts[6].is_writable);

    // Input vault (USDC reserve)
    assert_eq!(accounts[7].pubkey, USDC_VAULT, "input vault");
    assert!(accounts[7].is_writable);

    // Output vault (SOL reserve)
    assert_eq!(accounts[8].pubkey, SOL_VAULT, "output vault");
    assert!(accounts[8].is_writable);

    // Input token program
    assert_eq!(accounts[9].pubkey, TOKEN_PROGRAM_ID, "input token program");

    // Output token program
    assert_eq!(
        accounts[10].pubkey, TOKEN_PROGRAM_ID,
        "output token program"
    );

    // Input token mint
    assert_eq!(accounts[11].pubkey, USDC_MINT, "input token mint");

    // Output token mint
    assert_eq!(accounts[12].pubkey, WSOL_MINT, "output token mint");

    // Observation state
    assert_eq!(accounts[13].pubkey, OBSERVATION_STATE, "observation state");
    assert!(accounts[13].is_writable);

    // Raydium CPMM swap_base_input has no extra data
    assert!(data.is_empty());
}
