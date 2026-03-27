use {
    beethoven_client::{resolve_swap, SwapProtocol},
    solana_address::Address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = Address::from_str_const("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = Address::from_str_const("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const GAMMA_POOL: Address = Address::from_str_const("Hjm1F98vgVdN7Y9L46KLqcZZWyTKS9tj9ybYKJcXnSng");
const GAMMA_AMM_CONFIG: Address =
    Address::from_str_const("68yDnv1sDzU3L2cek5kNEszKFPaK9yUJaC4ghV5LAXW6");
const GAMMA_VAULT_0: Address =
    Address::from_str_const("61Xc2EKCL6SnqyMjWujTmcsFvBbRh5717MwrD3EMwaaw");
const GAMMA_VAULT_1: Address =
    Address::from_str_const("7Aihr5kSURKgUtvnAEAkQyZzfJ7vq5WiYLeCd4o78xLW");
const GAMMA_OBSERVATION: Address =
    Address::from_str_const("6qFaCY5Ws9bcagcvJoZnUpH9qLv8MkKWmUszvhX9QW3V");

const GAMMA_PROGRAM_ID: Address =
    Address::from_str_const("GAMMA7meSFWaBXF25oSUgmGRwaW6sCMFLmBNiMSdbHVT");
const TOKEN_PROGRAM_ID: Address =
    Address::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_gamma_resolve_with_known_pool() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Gamma {
            pool: Some(GAMMA_POOL),
        },
        &WSOL_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 14, "gamma requires 14 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, GAMMA_PROGRAM_ID);

    // User (signer, readonly)
    assert_eq!(accounts[1].pubkey, user);
    assert!(accounts[1].is_signer);

    // Authority PDA
    let (expected_authority, _) =
        Address::find_program_address(&[b"vault_and_lp_mint_auth_seed"], &GAMMA_PROGRAM_ID);
    assert_eq!(accounts[2].pubkey, expected_authority, "authority PDA");

    // Pool accounts from on-chain state
    assert_eq!(accounts[3].pubkey, GAMMA_AMM_CONFIG, "amm_config");
    assert_eq!(accounts[4].pubkey, GAMMA_POOL, "pool_state");

    // User ATAs — derived from user
    let expected_input_ata =
        beethoven_client::get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    let expected_output_ata =
        beethoven_client::get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[5].pubkey, expected_input_ata, "user_input_ata");
    assert_eq!(accounts[6].pubkey, expected_output_ata, "user_output_ata");

    // Vaults (read from on-chain pool state)
    assert_eq!(accounts[7].pubkey, GAMMA_VAULT_0, "input_vault");
    assert_eq!(accounts[8].pubkey, GAMMA_VAULT_1, "output_vault");

    // Token programs
    assert_eq!(accounts[9].pubkey, TOKEN_PROGRAM_ID, "input_token_program");
    assert_eq!(
        accounts[10].pubkey, TOKEN_PROGRAM_ID,
        "output_token_program"
    );

    // Mints
    assert_eq!(accounts[11].pubkey, WSOL_MINT, "input_mint");
    assert_eq!(accounts[12].pubkey, USDC_MINT, "output_mint");

    // Observation key
    assert_eq!(accounts[13].pubkey, GAMMA_OBSERVATION, "observation_key");

    // Gamma has no extra data
    assert!(data.is_empty());
}

#[tokio::test]
async fn test_gamma_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    // Selling USDC for WSOL — vaults should be flipped vs the canonical pool order
    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Gamma {
            pool: Some(GAMMA_POOL),
        },
        &USDC_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 14);
    assert_eq!(accounts[0].pubkey, GAMMA_PROGRAM_ID);

    // When mint_a=USDC (token_1), vaults should be flipped:
    // input_vault = token_1_vault, output_vault = token_0_vault
    assert_eq!(
        accounts[7].pubkey, GAMMA_VAULT_1,
        "input_vault (USDC vault)"
    );
    assert_eq!(
        accounts[8].pubkey, GAMMA_VAULT_0,
        "output_vault (WSOL vault)"
    );

    // Mints should also be flipped
    assert_eq!(accounts[11].pubkey, USDC_MINT, "input_mint");
    assert_eq!(accounts[12].pubkey, WSOL_MINT, "output_mint");

    assert!(data.is_empty());
}
