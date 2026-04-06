use {
    beethoven_client::{
        deposit::{perena_bankineco::BANKINECO_PROGRAM_ID, DepositProtocol},
        get_associated_token_address, resolve_deposit, ASSOCIATED_TOKEN_PROGRAM_ID,
        SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USD_STAR_MINT: Address = address!("star9agSpjiFe3M49B3RniVU4CMBBEK3Qnaqn3RGiFM");
const VAULT_STATE: Address = address!("3bZ1qY6wfzyDH7QMPiRKLr6k8p1asdtyjvJyJsJBdv23");
const BANK_STATE: Address = address!("sM6P4mh53CnG4faN4Fo3seY7wMSAiHdy8o6gKjwQF7A");
const ORACLE_STATE: Address = address!("CmKFP4YJg5QpAryUm9xk5QD611bccYMzZvpvQDJkMwt6");
const YIELDING_VAULT_TA: Address = address!("HvG7HSrNHVAcjzgwt3UVtnY9srkrY7qnMG4zS1SnPQT2");
const TEAM_STATE: Address = address!("6tqLkhbqJSx4KG616VhNCvsaFqcDPok7wdbzU2DmEAub");
const FEE_TEAM_ATA: Address = address!("3msJbxNbSeosztbNEB1eFPitMFnP8ogCszegPUswipdL");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_perena_bankineco_resolve_with_known_vault() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let min_bank_mint_minted = 1_000_000;

    let (accounts, data) = resolve_deposit(
        &rpc,
        &DepositProtocol::PerenaBankineco {
            vault: VAULT_STATE,
            min_bank_mint_minted,
        },
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 16, "bankineco requires 16 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, BANKINECO_PROGRAM_ID,
        "bankineco program"
    );

    // User
    assert_eq!(accounts[1].pubkey, user, "user");
    assert!(accounts[1].is_writable);
    assert!(accounts[1].is_signer);

    // Bank state
    assert_eq!(accounts[2].pubkey, BANK_STATE, "bank state");
    assert!(accounts[2].is_writable);

    // Vault state
    assert_eq!(accounts[3].pubkey, VAULT_STATE, "vault state");
    assert!(accounts[3].is_writable);

    // Oracle state
    assert_eq!(accounts[4].pubkey, ORACLE_STATE, "oracle state");

    // Yielding mint
    assert_eq!(accounts[5].pubkey, USDC_MINT, "yielding mint");

    // Bank mint
    assert_eq!(accounts[6].pubkey, USD_STAR_MINT, "bank mint");
    assert!(accounts[6].is_writable);

    // Yielding user TA
    let expected_yielding_user_ta =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[7].pubkey, expected_yielding_user_ta,
        "yielding user TA"
    );
    assert!(accounts[7].is_writable);

    // Bank mint user TA
    let expected_bank_mint_user_ta =
        get_associated_token_address(&user, &USD_STAR_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[8].pubkey, expected_bank_mint_user_ta,
        "bank mint user TA"
    );
    assert!(accounts[8].is_writable);

    // Yielding vault ATA
    assert_eq!(accounts[9].pubkey, YIELDING_VAULT_TA, "yielding vault ATA");
    assert!(accounts[9].is_writable);

    // Team state
    assert_eq!(accounts[10].pubkey, TEAM_STATE, "team state");
    assert!(accounts[10].is_writable);

    // Fee team ATA
    assert_eq!(accounts[11].pubkey, FEE_TEAM_ATA, "fee team ATA");
    assert!(accounts[11].is_writable);

    // System program
    assert_eq!(accounts[12].pubkey, SYSTEM_PROGRAM_ID, "system program");

    // Token program
    assert_eq!(accounts[13].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Yielding mint program
    assert_eq!(
        accounts[14].pubkey, TOKEN_PROGRAM_ID,
        "yielding mint program"
    );

    // Associated token program
    assert_eq!(
        accounts[15].pubkey, ASSOCIATED_TOKEN_PROGRAM_ID,
        "associated token program"
    );

    // min_bank_mint_minted
    assert_eq!(
        u64::from_le_bytes(data[0..8].try_into().unwrap()),
        min_bank_mint_minted,
        "min_bank_mint_minted"
    );
}
