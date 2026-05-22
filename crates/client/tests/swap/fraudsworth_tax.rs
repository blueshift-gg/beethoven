use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::fraudsworth_tax::{
            CARNAGE_VAULT, CRIME_MINT, EPOCH_STATE, EXTRA_ACCOUNT_META_LIST_CRIME,
            EXTRA_ACCOUNT_META_LIST_FRAUD, FRAUDSWORTH_AMM_PROGRAM_ID,
            FRAUDSWORTH_STAKING_PROGRAM_ID, FRAUDSWORTH_TAX_PROGRAM_ID,
            FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID, FRAUD_MINT, POOL_WSOL_CRIME,
            POOL_WSOL_CRIME_VAULT_A, POOL_WSOL_CRIME_VAULT_B, POOL_WSOL_FRAUD,
            POOL_WSOL_FRAUD_VAULT_A, POOL_WSOL_FRAUD_VAULT_B, STAKE_POOL, STAKING_ESCROW,
            SWAP_AUTHORITY, TAX_AUTHORITY, TREASURY, WSOL_INTERMEDIARY, WSOL_MINT,
        },
        SwapProtocol, SYSTEM_PROGRAM_ID, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::Address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_fraudsworth_tax_is_buy() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::FraudsworthTax {
            is_buy: true,
            is_crime: true,
        },
        &WSOL_MINT,
        &CRIME_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 25, "fraudsworth tax requires 25 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, FRAUDSWORTH_TAX_PROGRAM_ID,
        "fraudsworth tax program ID"
    );
    assert!(!accounts[0].is_signer);
    assert!(!accounts[0].is_writable);

    // User
    assert_eq!(accounts[1].pubkey, user, "user");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Epoch state
    assert_eq!(accounts[2].pubkey, EPOCH_STATE, "epoch state");

    // Swap authority
    assert_eq!(accounts[3].pubkey, SWAP_AUTHORITY, "swap authority");

    // Tax authority
    assert_eq!(accounts[4].pubkey, TAX_AUTHORITY, "tax authority");

    // Pool
    assert_eq!(accounts[5].pubkey, POOL_WSOL_CRIME, "pool");
    assert!(accounts[5].is_writable);

    // Pool vault A
    assert_eq!(accounts[6].pubkey, POOL_WSOL_CRIME_VAULT_A, "pool vault A");
    assert!(accounts[6].is_writable);

    // Pool vault B
    assert_eq!(accounts[7].pubkey, POOL_WSOL_CRIME_VAULT_B, "pool vault B");
    assert!(accounts[7].is_writable);

    // Mint A
    assert_eq!(accounts[8].pubkey, WSOL_MINT, "mint A");

    // Mint B
    assert_eq!(accounts[9].pubkey, CRIME_MINT, "mint B");

    // User token A
    let expected_user_token_a = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[10].pubkey, expected_user_token_a, "user token A");
    assert!(accounts[10].is_writable);

    // User token B
    let expected_user_token_b =
        get_associated_token_address(&user, &CRIME_MINT, &TOKEN_2022_PROGRAM_ID);
    assert_eq!(accounts[11].pubkey, expected_user_token_b, "user token B");
    assert!(accounts[11].is_writable);

    // Stake pool
    assert_eq!(accounts[12].pubkey, STAKE_POOL, "stake pool");
    assert!(accounts[12].is_writable);

    // Staking escrow
    assert_eq!(accounts[13].pubkey, STAKING_ESCROW, "staking escrow");
    assert!(accounts[13].is_writable);

    // Carnage vault
    assert_eq!(accounts[14].pubkey, CARNAGE_VAULT, "carnage vault");
    assert!(accounts[14].is_writable);

    // Treasury
    assert_eq!(accounts[15].pubkey, TREASURY, "treasury");
    assert!(accounts[15].is_writable);

    // AMM program
    assert_eq!(
        accounts[16].pubkey, FRAUDSWORTH_AMM_PROGRAM_ID,
        "amm program"
    );

    // Token program a
    assert_eq!(accounts[17].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Token program b
    assert_eq!(
        accounts[18].pubkey, TOKEN_2022_PROGRAM_ID,
        "token program b"
    );

    // System program
    assert_eq!(accounts[19].pubkey, SYSTEM_PROGRAM_ID, "system program");

    // Staking program
    assert_eq!(
        accounts[20].pubkey, FRAUDSWORTH_STAKING_PROGRAM_ID,
        "staking program"
    );

    // Extra account meta list
    assert_eq!(
        accounts[21].pubkey, EXTRA_ACCOUNT_META_LIST_CRIME,
        "extra account meta list"
    );

    // Whitelist source
    let expected_whitelist_source = Address::find_program_address(
        &[b"whitelist", POOL_WSOL_CRIME_VAULT_B.as_ref()],
        &FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
    )
    .0;
    assert_eq!(
        accounts[22].pubkey, expected_whitelist_source,
        "whitelist source"
    );

    // Whitelist destination
    let expected_whitelist_destination = Address::find_program_address(
        &[b"whitelist", expected_user_token_b.as_ref()],
        &FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
    )
    .0;
    assert_eq!(
        accounts[23].pubkey, expected_whitelist_destination,
        "whitelist destination"
    );

    // Transfer hook program
    assert_eq!(
        accounts[24].pubkey, FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
        "transfer hook program"
    );

    // is_buy
    assert_eq!(data[0], 1_u8);
}

#[tokio::test]
async fn test_fraudsworth_tax_is_sell() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::FraudsworthTax {
            is_buy: false,
            is_crime: false,
        },
        &WSOL_MINT,
        &FRAUD_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 26, "fraudsworth tax requires 26 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, FRAUDSWORTH_TAX_PROGRAM_ID,
        "fraudsworth tax program ID"
    );
    assert!(!accounts[0].is_signer);
    assert!(!accounts[0].is_writable);

    // User
    assert_eq!(accounts[1].pubkey, user, "user");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Epoch state
    assert_eq!(accounts[2].pubkey, EPOCH_STATE, "epoch state");

    // Swap authority
    assert_eq!(accounts[3].pubkey, SWAP_AUTHORITY, "swap authority");
    assert!(accounts[3].is_writable);

    // Tax authority
    assert_eq!(accounts[4].pubkey, TAX_AUTHORITY, "tax authority");

    // Pool
    assert_eq!(accounts[5].pubkey, POOL_WSOL_FRAUD, "pool");
    assert!(accounts[5].is_writable);

    // Pool vault A
    assert_eq!(accounts[6].pubkey, POOL_WSOL_FRAUD_VAULT_A, "pool vault A");
    assert!(accounts[6].is_writable);

    // Pool vault B
    assert_eq!(accounts[7].pubkey, POOL_WSOL_FRAUD_VAULT_B, "pool vault B");
    assert!(accounts[7].is_writable);

    // Mint A
    assert_eq!(accounts[8].pubkey, WSOL_MINT, "mint A");

    // Mint B
    assert_eq!(accounts[9].pubkey, FRAUD_MINT, "mint B");

    // User token A
    let expected_user_token_a = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[10].pubkey, expected_user_token_a, "user token A");
    assert!(accounts[10].is_writable);

    // User token B
    let expected_user_token_b =
        get_associated_token_address(&user, &FRAUD_MINT, &TOKEN_2022_PROGRAM_ID);
    assert_eq!(accounts[11].pubkey, expected_user_token_b, "user token B");
    assert!(accounts[11].is_writable);

    // Stake pool
    assert_eq!(accounts[12].pubkey, STAKE_POOL, "stake pool");
    assert!(accounts[12].is_writable);

    // Staking escrow
    assert_eq!(accounts[13].pubkey, STAKING_ESCROW, "staking escrow");
    assert!(accounts[13].is_writable);

    // Carnage vault
    assert_eq!(accounts[14].pubkey, CARNAGE_VAULT, "carnage vault");
    assert!(accounts[14].is_writable);

    // Treasury
    assert_eq!(accounts[15].pubkey, TREASURY, "treasury");
    assert!(accounts[15].is_writable);

    // WSOL intermediary
    assert_eq!(accounts[16].pubkey, WSOL_INTERMEDIARY, "wsol intermediary");
    assert!(accounts[16].is_writable);

    // AMM program
    assert_eq!(
        accounts[17].pubkey, FRAUDSWORTH_AMM_PROGRAM_ID,
        "amm program"
    );

    // Token program a
    assert_eq!(accounts[18].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Token program b
    assert_eq!(
        accounts[19].pubkey, TOKEN_2022_PROGRAM_ID,
        "token program b"
    );

    // System program
    assert_eq!(accounts[20].pubkey, SYSTEM_PROGRAM_ID, "system program");

    // Staking program
    assert_eq!(
        accounts[21].pubkey, FRAUDSWORTH_STAKING_PROGRAM_ID,
        "staking program"
    );

    // Extra account meta list
    assert_eq!(
        accounts[22].pubkey, EXTRA_ACCOUNT_META_LIST_FRAUD,
        "extra account meta list"
    );

    // Whitelist source
    let expected_whitelist_source = Address::find_program_address(
        &[b"whitelist", expected_user_token_b.as_ref()],
        &FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
    )
    .0;
    assert_eq!(
        accounts[23].pubkey, expected_whitelist_source,
        "whitelist source"
    );

    // Whitelist destination
    let expected_whitelist_destination = Address::find_program_address(
        &[b"whitelist", POOL_WSOL_FRAUD_VAULT_B.as_ref()],
        &FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
    )
    .0;
    assert_eq!(
        accounts[24].pubkey, expected_whitelist_destination,
        "whitelist destination"
    );

    // Transfer hook program
    assert_eq!(
        accounts[25].pubkey, FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
        "transfer hook program"
    );

    // is_buy
    assert_eq!(data[0], 0_u8);
}
