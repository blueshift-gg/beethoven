use {
    beethoven_client::{
        get_associated_token_address, resolve_swap, swap::solv_finance::SOLV_FINANCE_PROGRAM_ID,
        SwapProtocol, ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const SOLVBTC_MINT: Address = address!("SoLvHDFVstC74Jr9eNLTDoG4goSUsn1RENmjNtFKZvW");
const BTCPLUS_MINT: Address = address!("soLvpPEDkN8D1Wgjezrb1oj4WjGtj17vynGm6t3jah6");
const TREASURER_TOKEN_TA: Address = address!("4JjcZvMzgcDxxd9YZbdbFQciWcSakwAVe3zT1kNoJqav");
const MULTISIG: Address = address!("msigaXYhoZ6qELubRGf6N6Uj3kJ14WF82PDbV5HL172");
const VAULT: Address = address!("B3ct2h3iCWKZmErPQ8PtZ51qBU98Zfci2TSnjvXNUbUa");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_solv_finance() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::SolvFinance { vault: Some(VAULT) },
        &SOLVBTC_MINT,
        &BTCPLUS_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 11, "solv finance requires 11 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, SOLV_FINANCE_PROGRAM_ID,
        "solv finance program"
    );

    // User
    assert_eq!(accounts[1].pubkey, user, "user");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // User token ta
    let expected_user_token_ta =
        get_associated_token_address(&user, &SOLVBTC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[2].pubkey, expected_user_token_ta, "user token ta");
    assert!(accounts[2].is_writable);

    // User target ta
    let expected_user_target_ta =
        get_associated_token_address(&user, &BTCPLUS_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[3].pubkey, expected_user_target_ta,
        "user target ta"
    );
    assert!(accounts[3].is_writable);

    // Treasurer token ta
    assert_eq!(accounts[4].pubkey, TREASURER_TOKEN_TA, "treasurer token ta");
    assert!(accounts[4].is_writable);

    // Multisig
    assert_eq!(accounts[5].pubkey, MULTISIG, "multisig");

    // Mint token
    assert_eq!(accounts[6].pubkey, SOLVBTC_MINT, "mint token");

    // Mint target
    assert_eq!(accounts[7].pubkey, BTCPLUS_MINT, "mint target");
    assert!(accounts[7].is_writable);

    // Vault
    assert_eq!(accounts[8].pubkey, VAULT, "vault");
    assert!(accounts[8].is_writable);

    // Token program
    assert_eq!(accounts[9].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Associated token program
    assert_eq!(
        accounts[10].pubkey, ASSOCIATED_TOKEN_PROGRAM_ID,
        "associated token program"
    );

    // no extra data
    assert_eq!(data.len(), 0);
}
