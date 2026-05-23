use {
    beethoven_client::{
        get_associated_token_address, read_pubkey, resolve_swap,
        swap::pump_amm::{
            EVENT_AUTHORITY, FEE_CONFIG, FEE_PROGRAM_ID, FEE_RECIPIENT, GLOBAL_CONFIG,
            GLOBAL_VOLUME_ACCUMULATOR, OFFSET_FIRST_PROTOCOL_FEE_RECIPIENT, PUMP_AMM_PROGRAM_ID,
        },
        SwapProtocol, ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const POOL_STATE: Address = address!("Gf7sXMoP8iRw4iiXmJ1nq4vxcRycbGXy5RL8a8LnTd3v");
const POOL_BASE_TOKEN_ACCOUNT: Address = address!("nML7msD1MiJHxFvhv4po1u6C4KpWr64ugKqc75DMuD2");
const POOL_QUOTE_TOKEN_ACCOUNT: Address = address!("EjHirXt2bQd2DDNveagHHCWYzUwtY1iwNbBrV5j84e6j");
const COIN_CREATOR_VAULT_ATA: Address = address!("Ei6iux5MMYG8JxCTr58goADqFTtMroL9TXJityF3fAQc");
const COIN_CREATOR_VAULT_AUTHORITY: Address =
    address!("8N3GDaZ2iwN65oxVatKTLPNooAVUJTbfiVJ1ahyqwjSk");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_pump_amm_resolve_with_known_pool_buy() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::PumpAmm {
            // explicitly passed, pool cannot be derived using base and quote mints alone
            pool: Some(POOL_STATE),
            track_volume: None,
            is_buy: true,
        },
        &USDC_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert!(
        accounts.len() >= 24,
        "pump amm requires at least 24 accounts"
    );

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, PUMP_AMM_PROGRAM_ID, "pump AMM program");

    // Pool
    assert_eq!(accounts[1].pubkey, POOL_STATE, "pool");
    assert!(accounts[1].is_writable);

    // User
    assert_eq!(accounts[2].pubkey, user, "user");
    assert!(accounts[2].is_signer);
    assert!(accounts[2].is_writable);

    // Global config
    assert_eq!(accounts[3].pubkey, GLOBAL_CONFIG, "global config");

    // Base mint
    assert_eq!(accounts[4].pubkey, USDC_MINT, "base mint");

    // Quote mint
    assert_eq!(accounts[5].pubkey, WSOL_MINT, "quote mint");

    // User base token account
    let expected_user_base_ata = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[6].pubkey, expected_user_base_ata,
        "user base token account"
    );
    assert!(accounts[6].is_writable);

    // User quote token account
    let expected_user_quote_ata =
        get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[7].pubkey, expected_user_quote_ata,
        "user quote token account"
    );
    assert!(accounts[7].is_writable);

    // Pool base token account
    assert_eq!(
        accounts[8].pubkey, POOL_BASE_TOKEN_ACCOUNT,
        "pool base token account"
    );
    assert!(accounts[8].is_writable);

    // Pool quote token account
    assert_eq!(
        accounts[9].pubkey, POOL_QUOTE_TOKEN_ACCOUNT,
        "pool quote token account"
    );
    assert!(accounts[9].is_writable);

    // Protocol fee recipient
    let gc_data = rpc.get_account(&GLOBAL_CONFIG).await.unwrap().data;
    let protocol_fee_recipient =
        read_pubkey(&gc_data, OFFSET_FIRST_PROTOCOL_FEE_RECIPIENT).unwrap();
    assert_eq!(
        accounts[10].pubkey, protocol_fee_recipient,
        "protocol fee recipient"
    );

    // Protocol fee recipient token account
    let expected_protocol_fee_recipient_token_account =
        get_associated_token_address(&protocol_fee_recipient, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[11].pubkey, expected_protocol_fee_recipient_token_account,
        "protocol fee recipient token account"
    );
    assert!(accounts[11].is_writable);

    // Base token program
    assert_eq!(accounts[12].pubkey, TOKEN_PROGRAM_ID, "base token program");

    // Quote token program
    assert_eq!(accounts[13].pubkey, TOKEN_PROGRAM_ID, "quote token program");

    // System program
    assert_eq!(accounts[14].pubkey, SYSTEM_PROGRAM_ID, "system program");

    // Associated token program
    assert_eq!(
        accounts[15].pubkey, ASSOCIATED_TOKEN_PROGRAM_ID,
        "associated token program"
    );

    // Event authority
    assert_eq!(accounts[16].pubkey, EVENT_AUTHORITY, "event authority");

    // Program (self)
    assert_eq!(accounts[17].pubkey, PUMP_AMM_PROGRAM_ID, "program (self)");

    // Coin creator vault ATA
    assert_eq!(
        accounts[18].pubkey, COIN_CREATOR_VAULT_ATA,
        "coin creator vault ATA"
    );
    assert!(accounts[18].is_writable);

    // Coin creator vault authority
    assert_eq!(
        accounts[19].pubkey, COIN_CREATOR_VAULT_AUTHORITY,
        "coin creator vault authority"
    );

    // Global volume accumulator
    assert_eq!(
        accounts[20].pubkey, GLOBAL_VOLUME_ACCUMULATOR,
        "global volume accumulator"
    );

    // User volume accumulator
    let (expected_user_volume_accumulator, _) = Address::find_program_address(
        &[b"user_volume_accumulator", user.as_ref()],
        &PUMP_AMM_PROGRAM_ID,
    );
    assert_eq!(
        accounts[21].pubkey, expected_user_volume_accumulator,
        "user volume accumulator"
    );
    assert!(accounts[21].is_writable);

    // Fee config
    assert_eq!(accounts[22].pubkey, FEE_CONFIG, "fee config");

    // Fee program
    assert_eq!(accounts[23].pubkey, FEE_PROGRAM_ID, "fee program");

    let accounts_len = accounts.len();

    // pool-v2
    // let expected_pool_v2 = Address::find_program_address(
    //     &[b"pool-v2", accounts[4].pubkey.as_ref()],
    //     &PUMP_AMM_PROGRAM_ID,
    // )
    // .0;
    // assert_eq!(
    //     accounts[accounts_len - 3].pubkey,
    //     expected_pool_v2,
    //     "pool-v2"
    // );

    // Fee recipient
    assert_eq!(
        accounts[accounts_len - 2].pubkey,
        FEE_RECIPIENT,
        "fee recipient"
    );

    // Fee recipient quote mint ATA
    let expected_fee_recipient_quote_mint_ata =
        get_associated_token_address(&FEE_RECIPIENT, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[accounts_len - 1].pubkey,
        expected_fee_recipient_quote_mint_ata,
        "fee recipient quote mint ATA"
    );

    // track_volume = None
    assert_eq!(data, vec![0, 0]);
}

#[tokio::test]
async fn test_pump_amm_resolve_with_known_pool_sell() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::PumpAmm {
            pool: Some(POOL_STATE),
            track_volume: Some(true),
            is_buy: false,
        },
        &USDC_MINT,
        &WSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert!(
        accounts.len() >= 24,
        "pump amm requires at least 24 accounts"
    );

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, PUMP_AMM_PROGRAM_ID, "pump AMM program");

    // Pool
    assert_eq!(accounts[1].pubkey, POOL_STATE, "pool");
    assert!(accounts[1].is_writable);

    // User
    assert_eq!(accounts[2].pubkey, user, "user");
    assert!(accounts[2].is_signer);
    assert!(accounts[2].is_writable);

    // Global config
    assert_eq!(accounts[3].pubkey, GLOBAL_CONFIG, "global config");

    // Base mint
    assert_eq!(accounts[4].pubkey, USDC_MINT, "base mint");

    // Quote mint
    assert_eq!(accounts[5].pubkey, WSOL_MINT, "quote mint");

    // User base token account
    let expected_user_base_ata = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[6].pubkey, expected_user_base_ata,
        "user base token account"
    );
    assert!(accounts[6].is_writable);

    // User quote token account
    let expected_user_quote_ata =
        get_associated_token_address(&user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[7].pubkey, expected_user_quote_ata,
        "user quote token account"
    );
    assert!(accounts[7].is_writable);

    // Pool base token account
    assert_eq!(
        accounts[8].pubkey, POOL_BASE_TOKEN_ACCOUNT,
        "pool base token account"
    );
    assert!(accounts[8].is_writable);

    // Pool quote token account
    assert_eq!(
        accounts[9].pubkey, POOL_QUOTE_TOKEN_ACCOUNT,
        "pool quote token account"
    );
    assert!(accounts[9].is_writable);

    // Protocol fee recipient
    let gc_data = rpc.get_account(&GLOBAL_CONFIG).await.unwrap().data;
    let protocol_fee_recipient =
        read_pubkey(&gc_data, OFFSET_FIRST_PROTOCOL_FEE_RECIPIENT).unwrap();
    assert_eq!(
        accounts[10].pubkey, protocol_fee_recipient,
        "protocol fee recipient"
    );

    // Protocol fee recipient token account
    let expected_protocol_fee_recipient_token_account =
        get_associated_token_address(&protocol_fee_recipient, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[11].pubkey, expected_protocol_fee_recipient_token_account,
        "protocol fee recipient token account"
    );
    assert!(accounts[11].is_writable);

    // Base token program
    assert_eq!(accounts[12].pubkey, TOKEN_PROGRAM_ID, "base token program");

    // Quote token program
    assert_eq!(accounts[13].pubkey, TOKEN_PROGRAM_ID, "quote token program");

    // System program
    assert_eq!(accounts[14].pubkey, SYSTEM_PROGRAM_ID, "system program");

    // Associated token program
    assert_eq!(
        accounts[15].pubkey, ASSOCIATED_TOKEN_PROGRAM_ID,
        "associated token program"
    );

    // Event authority
    assert_eq!(accounts[16].pubkey, EVENT_AUTHORITY, "event authority");

    // Program (self)
    assert_eq!(accounts[17].pubkey, PUMP_AMM_PROGRAM_ID, "program (self)");

    // Coin creator vault ATA
    assert_eq!(
        accounts[18].pubkey, COIN_CREATOR_VAULT_ATA,
        "coin creator vault ATA"
    );
    assert!(accounts[18].is_writable);

    // Coin creator vault authority
    assert_eq!(
        accounts[19].pubkey, COIN_CREATOR_VAULT_AUTHORITY,
        "coin creator vault authority"
    );

    // Fee config
    assert_eq!(accounts[20].pubkey, FEE_CONFIG, "fee config");

    // Fee program
    assert_eq!(accounts[21].pubkey, FEE_PROGRAM_ID, "fee program");

    let accounts_len = accounts.len();

    // pool-v2
    // let expected_pool_v2 = Address::find_program_address(
    //     &[b"pool-v2", accounts[4].pubkey.as_ref()],
    //     &PUMP_AMM_PROGRAM_ID,
    // )
    // .0;
    // assert_eq!(
    //     accounts[accounts_len - 3].pubkey,
    //     expected_pool_v2,
    //     "pool-v2"
    // );

    // Fee recipient
    assert_eq!(
        accounts[accounts_len - 2].pubkey,
        FEE_RECIPIENT,
        "fee recipient"
    );

    // Fee recipient quote mint ATA
    let expected_fee_recipient_quote_mint_ata =
        get_associated_token_address(&FEE_RECIPIENT, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[accounts_len - 1].pubkey,
        expected_fee_recipient_quote_mint_ata,
        "fee recipient quote mint ATA"
    );

    // track_volume = Some(true)
    assert_eq!(data, vec![1, 1]);
}
