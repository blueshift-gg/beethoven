use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::onre::{
            MINT_AUTHORITY, ONRE_MINT, ONRE_PROGRAM_ID, PERMISSIONLESS_AUTHORITY,
            PERMISSIONLESS_TOKEN_IN_ACCOUNT, PERMISSIONLESS_TOKEN_OUT_ACCOUNT, STATE,
            VAULT_AUTHORITY, VAULT_TOKEN_IN_ACCOUNT, VAULT_TOKEN_OUT_ACCOUNT,
        },
        SwapProtocol, ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID, SYSVAR_INSTRUCTIONS_ID,
        TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const OFFER: Address = address!("E88zkA9Pxb1i8EfSHrEW5ZUe6hiQbo8DHWQ3WhDFw7p6");
const BOSS: Address = address!("45YnzauhsBM8CpUz96Djf8UG5vqq2Dua62wuW9H3jaJ5");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_onre_resolve_with_known_offer_take_offer_permissionless() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Onre { offer: Some(OFFER) },
        &USDC_MINT,
        &ONRE_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 22, "onre requires 22 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, ONRE_PROGRAM_ID, "onre program id");

    // Offer
    assert_eq!(accounts[1].pubkey, OFFER, "offer");
    assert!(accounts[1].is_writable);

    // State
    assert_eq!(accounts[2].pubkey, STATE, "state");

    // Boss
    assert_eq!(accounts[3].pubkey, BOSS, "boss");

    // Vault authority
    assert_eq!(accounts[4].pubkey, VAULT_AUTHORITY, "vault authority");

    // Vault token in account
    assert_eq!(
        accounts[5].pubkey, VAULT_TOKEN_IN_ACCOUNT,
        "vault token in account"
    );
    assert!(accounts[5].is_writable);

    // Vault token out account
    assert_eq!(
        accounts[6].pubkey, VAULT_TOKEN_OUT_ACCOUNT,
        "vault token out account"
    );
    assert!(accounts[6].is_writable);

    // Permissionless authority
    assert_eq!(
        accounts[7].pubkey, PERMISSIONLESS_AUTHORITY,
        "permissionless authority"
    );

    // Permissionless token in account
    assert_eq!(
        accounts[8].pubkey, PERMISSIONLESS_TOKEN_IN_ACCOUNT,
        "permissionless token in account"
    );
    assert!(accounts[8].is_writable);

    // Permissionless token out account
    assert_eq!(
        accounts[9].pubkey, PERMISSIONLESS_TOKEN_OUT_ACCOUNT,
        "permissionless token out account"
    );
    assert!(accounts[9].is_writable);

    // Token in mint
    assert_eq!(accounts[10].pubkey, USDC_MINT, "token in mint");

    // Token in program
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "token in program");

    // Token out mint
    assert_eq!(accounts[12].pubkey, ONRE_MINT, "token out mint");

    // Token out program
    assert_eq!(accounts[13].pubkey, TOKEN_PROGRAM_ID, "token out program");

    // User token in account
    let expected_user_token_in_account =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[14].pubkey, expected_user_token_in_account,
        "user token in account"
    );
    assert!(accounts[14].is_writable);

    // User token out account
    let expected_user_token_out_account =
        get_associated_token_address(&user, &ONRE_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[15].pubkey, expected_user_token_out_account,
        "user token out account"
    );
    assert!(accounts[15].is_writable);

    // Boss token in account
    let expected_boss_token_in_account =
        get_associated_token_address(&BOSS, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[16].pubkey, expected_boss_token_in_account,
        "boss token in account"
    );
    assert!(accounts[16].is_writable);

    // Mint authority
    assert_eq!(accounts[17].pubkey, MINT_AUTHORITY, "mint authority");

    // Instructions sysvar
    assert_eq!(
        accounts[18].pubkey, SYSVAR_INSTRUCTIONS_ID,
        "instructions sysvar"
    );

    // User
    assert_eq!(accounts[19].pubkey, user, "user");
    assert!(accounts[19].is_signer);
    assert!(accounts[19].is_writable);

    // Associated token program
    assert_eq!(
        accounts[20].pubkey, ASSOCIATED_TOKEN_PROGRAM_ID,
        "associated token program"
    );

    // System program
    assert_eq!(accounts[21].pubkey, SYSTEM_PROGRAM_ID, "system program");

    // data is empty
    assert!(data.is_empty());
}

#[test]
#[ignore = "redemption instructions not tested due to KYB/KYC gating"]
fn test_onre_resolve_with_known_offer_create_redemption_request() {}
