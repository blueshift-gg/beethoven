use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::fraudsworth_conversion_vault::{
            CRIME_MINT, EXTRA_ACCOUNT_META_LIST_CRIME, EXTRA_ACCOUNT_META_LIST_FRAUD,
            EXTRA_ACCOUNT_META_LIST_PROFIT, FRAUDSWORTH_CONVERSION_VAULT_PROGRAM_ID,
            FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID, FRAUD_MINT, PROFIT_MINT, VAULT_CONFIG,
            VAULT_CRIME, VAULT_FRAUD, VAULT_PROFIT,
        },
        SwapProtocol, TOKEN_2022_PROGRAM_ID,
    },
    solana_address::Address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_fraudsworth_conversion_vault_crime_to_profit() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let pre_balance = 0u64;

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::FraudsworthConversionVault { pre_balance },
        &CRIME_MINT,
        &PROFIT_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(
        accounts.len(),
        18,
        "conversion_vault convert_v2 + 2 hook groups"
    );

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, FRAUDSWORTH_CONVERSION_VAULT_PROGRAM_ID,
        "fraudsworth conversion vault program"
    );

    // User
    assert_eq!(accounts[1].pubkey, user, "user");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Vault config
    assert_eq!(accounts[2].pubkey, VAULT_CONFIG, "vault config");

    // User input
    let expected_user_crime =
        get_associated_token_address(&user, &CRIME_MINT, &TOKEN_2022_PROGRAM_ID);
    assert_eq!(
        accounts[3].pubkey, expected_user_crime,
        "user input account"
    );
    assert!(accounts[3].is_writable);

    // User output
    let expected_user_profit =
        get_associated_token_address(&user, &PROFIT_MINT, &TOKEN_2022_PROGRAM_ID);
    assert_eq!(
        accounts[4].pubkey, expected_user_profit,
        "user output account"
    );
    assert!(accounts[4].is_writable);

    // Input mint
    assert_eq!(accounts[5].pubkey, CRIME_MINT, "input mint");

    // Output mint
    assert_eq!(accounts[6].pubkey, PROFIT_MINT, "output mint");

    // Vault input
    assert_eq!(accounts[7].pubkey, VAULT_CRIME, "vault input");
    assert!(accounts[7].is_writable);

    // Vault output
    assert_eq!(accounts[8].pubkey, VAULT_PROFIT, "vault output");
    assert!(accounts[8].is_writable);

    // Token program
    assert_eq!(accounts[9].pubkey, TOKEN_2022_PROGRAM_ID, "token program");

    // Input mint extra account meta list
    assert_eq!(
        accounts[10].pubkey, EXTRA_ACCOUNT_META_LIST_CRIME,
        "input mint extra account meta list"
    );

    // Input mint whitelist source
    let expected_input_whitelist_source = Address::find_program_address(
        &[b"whitelist", expected_user_crime.as_ref()],
        &FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
    )
    .0;
    assert_eq!(
        accounts[11].pubkey, expected_input_whitelist_source,
        "input mint whitelist source"
    );

    // Input mint whitelist destination
    let expected_input_whitelist_destination = Address::find_program_address(
        &[b"whitelist", VAULT_CRIME.as_ref()],
        &FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
    )
    .0;
    assert_eq!(
        accounts[12].pubkey, expected_input_whitelist_destination,
        "input mint whitelist destination"
    );

    // Input mint transfer hook program
    assert_eq!(
        accounts[13].pubkey, FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
        "input mint transfer hook program"
    );

    // Output mint extra account meta list
    assert_eq!(
        accounts[14].pubkey, EXTRA_ACCOUNT_META_LIST_PROFIT,
        "output mint extra account meta list"
    );

    // Output mint whitelist source
    let expected_output_whitelist_source = Address::find_program_address(
        &[b"whitelist", VAULT_PROFIT.as_ref()],
        &FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
    )
    .0;
    assert_eq!(
        accounts[15].pubkey, expected_output_whitelist_source,
        "output mint whitelist source"
    );

    // Output mint whitelist destination
    let expected_output_whitelist_destination = Address::find_program_address(
        &[b"whitelist", expected_user_profit.as_ref()],
        &FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
    )
    .0;
    assert_eq!(
        accounts[16].pubkey, expected_output_whitelist_destination,
        "output mint whitelist destination"
    );

    // Output mint transfer hook program
    assert_eq!(
        accounts[17].pubkey, FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
        "output mint transfer hook program"
    );

    // pre balance
    assert_eq!(data, pre_balance.to_le_bytes(), "pre balance");
}

#[tokio::test]
async fn test_fraudsworth_conversion_vault_resolve_profit_to_fraud() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::FraudsworthConversionVault { pre_balance: 0u64 },
        &PROFIT_MINT,
        &FRAUD_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(
        accounts.len(),
        18,
        "conversion_vault convert_v2 + 2 hook groups"
    );

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, FRAUDSWORTH_CONVERSION_VAULT_PROGRAM_ID,
        "conversion_vault program"
    );

    // User
    assert_eq!(accounts[1].pubkey, user, "user");
    assert!(accounts[1].is_signer);

    // Vault config
    assert_eq!(accounts[2].pubkey, VAULT_CONFIG, "vault config");

    let expected_user_profit =
        get_associated_token_address(&user, &PROFIT_MINT, &TOKEN_2022_PROGRAM_ID);
    let expected_user_fraud =
        get_associated_token_address(&user, &FRAUD_MINT, &TOKEN_2022_PROGRAM_ID);
    assert_eq!(
        accounts[3].pubkey, expected_user_profit,
        "user input account"
    );
    assert_eq!(
        accounts[4].pubkey, expected_user_fraud,
        "user output account"
    );

    // Mints
    assert_eq!(accounts[5].pubkey, PROFIT_MINT, "input mint");
    assert_eq!(accounts[6].pubkey, FRAUD_MINT, "output mint");

    // Vaults
    assert_eq!(accounts[7].pubkey, VAULT_PROFIT, "vault input");
    assert_eq!(accounts[8].pubkey, VAULT_FRAUD, "vault output");

    // Token program
    assert_eq!(accounts[9].pubkey, TOKEN_2022_PROGRAM_ID, "token program");

    // Input transfer hook accounts
    assert_eq!(
        accounts[10].pubkey, EXTRA_ACCOUNT_META_LIST_PROFIT,
        "input extra account meta list"
    );

    // Output transfer hook accounts
    assert_eq!(
        accounts[14].pubkey, EXTRA_ACCOUNT_META_LIST_FRAUD,
        "output extra account meta list"
    );

    assert_eq!(data, 0u64.to_le_bytes(), "pre_balance only");
}
