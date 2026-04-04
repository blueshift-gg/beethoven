use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::heaven::{CHAINLINK_SOL_USD_FEED, CHAINLINK_STORE_PROGRAM_ID, HEAVEN_PROGRAM_ID},
        SwapProtocol, ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID, SYSVAR_INSTRUCTIONS_ID,
        TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::Address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = Address::from_str_const("So11111111111111111111111111111111111111112");
const LIGHT_MINT: Address = Address::from_str_const("LiGHtkg3uTa9836RaNkKLLriqTNRcMdRAhqjGWNv777");
const SOL_LIGHT_LIQUIDITY_POOL_STATE: Address =
    Address::from_str_const("EkU9zGSkUnVVK6nhmPSqnxqcKPzt1PicrCjdxSbWo9uA");
const PROTOCOL_CONFIG: Address =
    Address::from_str_const("KpXrCt3pjJYFind2kgk7nQ3dS6bqjC2Ze3zzE5MQ78v");
const WSOL_VAULT: Address = Address::from_str_const("HBw4rhjiJ1cXDNQz7395QJ51DskLknwHRAjxYzgBsYnK");
const LIGHT_VAULT: Address =
    Address::from_str_const("FjCZrwymiMvdufnrPZLP6NvgZDY8j9KGnLakRic3vQi7");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_heaven_resolve_with_known_pair() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");
    let encoded_user_defined_event_data: Vec<u8> = vec![];

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Heaven {
            pool: None,
            direction: 0,
            encoded_user_defined_event_data: encoded_user_defined_event_data.clone(),
        },
        &WSOL_MINT,
        &LIGHT_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 17, "heaven requires 17 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, HEAVEN_PROGRAM_ID, "heaven program");

    // Token 2022 program
    assert_eq!(
        accounts[1].pubkey, TOKEN_2022_PROGRAM_ID,
        "token 2022 program"
    );

    // Token program
    assert_eq!(accounts[2].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Associated token program
    assert_eq!(
        accounts[3].pubkey, ASSOCIATED_TOKEN_PROGRAM_ID,
        "associated token program"
    );

    // System program
    assert_eq!(accounts[4].pubkey, SYSTEM_PROGRAM_ID, "system program");

    // Liquidity pool state
    assert_eq!(
        accounts[5].pubkey, SOL_LIGHT_LIQUIDITY_POOL_STATE,
        "liquidity pool state"
    );
    assert!(accounts[5].is_writable);

    // User
    assert_eq!(accounts[6].pubkey, user, "user");
    assert!(accounts[6].is_signer);
    assert!(accounts[6].is_writable);

    // Token a mint
    assert_eq!(accounts[7].pubkey, LIGHT_MINT, "token a mint");

    // Token b mint
    assert_eq!(accounts[8].pubkey, WSOL_MINT, "token b mint");

    // User token a vault
    let expected_light_ata =
        get_associated_token_address(&user, &LIGHT_MINT, &TOKEN_2022_PROGRAM_ID);
    assert_eq!(accounts[9].pubkey, expected_light_ata, "user token a vault");
    assert!(accounts[9].is_writable);

    // User token b vault
    let expected_wsol_ata = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[10].pubkey, expected_wsol_ata, "user token b vault");
    assert!(accounts[10].is_writable);

    // Token a vault
    assert_eq!(accounts[11].pubkey, LIGHT_VAULT, "token a vault");
    assert!(accounts[11].is_writable);

    // Token b vault
    assert_eq!(accounts[12].pubkey, WSOL_VAULT, "token b vault");
    assert!(accounts[12].is_writable);

    // Protocol config
    assert_eq!(accounts[13].pubkey, PROTOCOL_CONFIG, "protocol config");

    // Instructions sysvar
    assert_eq!(
        accounts[14].pubkey, SYSVAR_INSTRUCTIONS_ID,
        "instructions sysvar"
    );

    // Chainlink store program
    assert_eq!(
        accounts[15].pubkey, CHAINLINK_STORE_PROGRAM_ID,
        "chainlink store program"
    );

    // Chainlink transmissions SOL USD feed
    assert_eq!(
        accounts[16].pubkey, CHAINLINK_SOL_USD_FEED,
        "chainlink transmissions SOL USD feed"
    );

    // direction = u8
    let direction = data[0];
    assert_eq!(direction, 0);

    // encoded_user_defined_event_data
    assert_eq!(&data[1..], encoded_user_defined_event_data);
}

#[tokio::test]
async fn test_heaven_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");
    let encoded_user_defined_event_data: Vec<u8> = vec![];

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Heaven {
            pool: None,
            direction: 0,
            encoded_user_defined_event_data: encoded_user_defined_event_data.clone(),
        },
        &LIGHT_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 17, "heaven requires 17 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, HEAVEN_PROGRAM_ID, "heaven program");

    // Token 2022 program
    assert_eq!(
        accounts[1].pubkey, TOKEN_2022_PROGRAM_ID,
        "token 2022 program"
    );

    // Token program
    assert_eq!(accounts[2].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Associated token program
    assert_eq!(
        accounts[3].pubkey, ASSOCIATED_TOKEN_PROGRAM_ID,
        "associated token program"
    );

    // System program
    assert_eq!(accounts[4].pubkey, SYSTEM_PROGRAM_ID, "system program");

    // Liquidity pool state
    assert_eq!(
        accounts[5].pubkey, SOL_LIGHT_LIQUIDITY_POOL_STATE,
        "liquidity pool state"
    );
    assert!(accounts[5].is_writable);

    // User
    assert_eq!(accounts[6].pubkey, user, "user");
    assert!(accounts[6].is_signer);
    assert!(accounts[6].is_writable);

    // Token a mint
    assert_eq!(accounts[7].pubkey, LIGHT_MINT, "token a mint");

    // Token b mint
    assert_eq!(accounts[8].pubkey, WSOL_MINT, "token b mint");

    // User token a vault
    let expected_light_ata =
        get_associated_token_address(&user, &LIGHT_MINT, &TOKEN_2022_PROGRAM_ID);
    assert_eq!(accounts[9].pubkey, expected_light_ata, "user token a vault");
    assert!(accounts[9].is_writable);

    // User token b vault
    let expected_wsol_ata = get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[10].pubkey, expected_wsol_ata, "user token b vault");
    assert!(accounts[10].is_writable);

    // Token a vault
    assert_eq!(accounts[11].pubkey, LIGHT_VAULT, "token a vault");
    assert!(accounts[11].is_writable);

    // Token b vault
    assert_eq!(accounts[12].pubkey, WSOL_VAULT, "token b vault");
    assert!(accounts[12].is_writable);

    // Protocol config
    assert_eq!(accounts[13].pubkey, PROTOCOL_CONFIG, "protocol config");

    // Instructions sysvar
    assert_eq!(
        accounts[14].pubkey, SYSVAR_INSTRUCTIONS_ID,
        "instructions sysvar"
    );

    // Chainlink store program
    assert_eq!(
        accounts[15].pubkey, CHAINLINK_STORE_PROGRAM_ID,
        "chainlink store program"
    );

    // Chainlink transmissions SOL USD feed
    assert_eq!(
        accounts[16].pubkey, CHAINLINK_SOL_USD_FEED,
        "chainlink transmissions SOL USD feed"
    );

    // direction = u8
    let direction = data[0];
    assert_eq!(direction, 0);

    // encoded_user_defined_event_data
    assert_eq!(&data[1..], encoded_user_defined_event_data);
}
