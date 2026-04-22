use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::ore_lst::{
            ORE_LST_PROGRAM_ID, ORE_MINT, ORE_STAKE_PROGRAM_ID, STAKE, STAKE_TOKENS, STORE_MINT,
            TREASURY, TREASURY_TOKENS, VAULT, VAULT_TOKENS,
        },
        SwapProtocol, ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_ore_lst_resolve_wrap() {
    let rpc: RpcClient = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::OreLst { is_wrap: true },
        &ORE_MINT,
        &STORE_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 17, "ore lst requires 17 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, ORE_LST_PROGRAM_ID, "ore lst program");

    // Signer
    assert_eq!(accounts[1].pubkey, user, "signer");
    assert!(accounts[1].is_signer);

    // Payer
    assert_eq!(accounts[2].pubkey, user, "payer");
    assert!(accounts[2].is_signer);
    assert!(accounts[2].is_writable);

    // Sender ore
    let expected_sender_ore = get_associated_token_address(&user, &ORE_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[3].pubkey, expected_sender_ore, "sender ore");
    assert!(accounts[3].is_writable);

    // Sender stORE
    let expected_sender_store = get_associated_token_address(&user, &STORE_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[4].pubkey, expected_sender_store, "sender store");
    assert!(accounts[4].is_writable);

    // ORE mint
    assert_eq!(accounts[5].pubkey, ORE_MINT, "ore mint");
    assert!(accounts[5].is_writable);

    // stORE mint
    assert_eq!(accounts[6].pubkey, STORE_MINT, "store mint");
    assert!(accounts[6].is_writable);

    // Stake
    assert_eq!(accounts[7].pubkey, STAKE, "stake");
    assert!(accounts[7].is_writable);

    // Stake tokens
    assert_eq!(accounts[8].pubkey, STAKE_TOKENS, "stake tokens");
    assert!(accounts[8].is_writable);

    // Treasury
    assert_eq!(accounts[9].pubkey, TREASURY, "treasury");
    assert!(accounts[9].is_writable);

    // Treasury tokens
    assert_eq!(accounts[10].pubkey, TREASURY_TOKENS, "treasury tokens");
    assert!(accounts[10].is_writable);

    // Vault
    assert_eq!(accounts[11].pubkey, VAULT, "vault");
    assert!(accounts[11].is_writable);

    // Vault tokens
    assert_eq!(accounts[12].pubkey, VAULT_TOKENS, "vault tokens");
    assert!(accounts[12].is_writable);

    // System program
    assert_eq!(accounts[13].pubkey, SYSTEM_PROGRAM_ID, "system program");

    // Token program
    assert_eq!(accounts[14].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Associated token program
    assert_eq!(
        accounts[15].pubkey, ASSOCIATED_TOKEN_PROGRAM_ID,
        "associated token program"
    );

    // ORE stake program
    assert_eq!(
        accounts[16].pubkey, ORE_STAKE_PROGRAM_ID,
        "ore stake program"
    );

    // is_wrap
    assert_eq!(data[0], 1, "is wrap");
}

#[tokio::test]
async fn test_ore_lst_resolve_unwrap() {
    let rpc: RpcClient = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::OreLst { is_wrap: false },
        &ORE_MINT,
        &STORE_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 17, "ore lst requires 17 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, ORE_LST_PROGRAM_ID, "ore lst program");

    // Signer
    assert_eq!(accounts[1].pubkey, user, "signer");
    assert!(accounts[1].is_signer);

    // Payer
    assert_eq!(accounts[2].pubkey, user, "payer");
    assert!(accounts[2].is_signer);
    assert!(accounts[2].is_writable);

    // Sender ore
    let expected_sender_ore = get_associated_token_address(&user, &ORE_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[3].pubkey, expected_sender_ore, "sender ore");
    assert!(accounts[3].is_writable);

    // Sender stORE
    let expected_sender_store = get_associated_token_address(&user, &STORE_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[4].pubkey, expected_sender_store, "sender store");
    assert!(accounts[4].is_writable);

    // ORE mint
    assert_eq!(accounts[5].pubkey, ORE_MINT, "ore mint");
    assert!(accounts[5].is_writable);

    // stORE mint
    assert_eq!(accounts[6].pubkey, STORE_MINT, "store mint");
    assert!(accounts[6].is_writable);

    // Stake
    assert_eq!(accounts[7].pubkey, STAKE, "stake");
    assert!(accounts[7].is_writable);

    // Stake tokens
    assert_eq!(accounts[8].pubkey, STAKE_TOKENS, "stake tokens");
    assert!(accounts[8].is_writable);

    // Treasury
    assert_eq!(accounts[9].pubkey, TREASURY, "treasury");
    assert!(accounts[9].is_writable);

    // Treasury tokens
    assert_eq!(accounts[10].pubkey, TREASURY_TOKENS, "treasury tokens");
    assert!(accounts[10].is_writable);

    // Vault
    assert_eq!(accounts[11].pubkey, VAULT, "vault");
    assert!(accounts[11].is_writable);

    // Vault tokens
    assert_eq!(accounts[12].pubkey, VAULT_TOKENS, "vault tokens");
    assert!(accounts[12].is_writable);

    // System program
    assert_eq!(accounts[13].pubkey, SYSTEM_PROGRAM_ID, "system program");

    // Token program
    assert_eq!(accounts[14].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Associated token program
    assert_eq!(
        accounts[15].pubkey, ASSOCIATED_TOKEN_PROGRAM_ID,
        "associated token program"
    );

    // ORE stake program
    assert_eq!(
        accounts[16].pubkey, ORE_STAKE_PROGRAM_ID,
        "ore stake program"
    );

    // is_wrap
    assert_eq!(data[0], 0, "is wrap");
}
