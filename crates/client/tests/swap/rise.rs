use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::rise::{MAYFLOWER_PROGRAM_ID, MAY_LOG_ACCOUNT, RISE_PROGRAM_ID},
        SwapProtocol, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const MINT_TOKEN: Address = address!("7MsJCvDi5t5U3Ya2UAs5bR75VJyVMr2FKdzGmeg2rise");
const MINT_MAIN: Address = address!("So11111111111111111111111111111111111111112");
const TENANT: Address = address!("5scY2JGWLnBubCMbWrn1gi8FQEP8SPjvQ1hfjW4ktYUb");
const MARKET: Address = address!("Dmryq83qiuGuRjd36QkY5Y2cEFZajqrhuXW8kYVG1z2E");
const CASH_ESCROW: Address = address!("992CCqsXFiwRLtNSRjHaDSsQiFd95zzxuxmWPic1AcHb");
const MAY_TENANT: Address = address!("HeBDu9g5EN6qdDJWijHHpxYuMBE6aWvy1BmzFyEa7Q7C");
const MAY_MARKET_GROUP: Address = address!("HA9pvTe8F2MLhQK1ZgHn7r2rfd2DJgA7NJBxDfKn9P7d");
const MARKET_META: Address = address!("GHqz6PrckckfmEQhA1MwMuCS5AazUytFFtLRE3DRi5sF");
const MAY_MARKET: Address = address!("XqjXrobAKCzVBS93aFb3CY1MbujtL1f3GT8NqVqQbnD");
const TENANT_SEED: Address = address!("Eg4Akr8HRv3gy4MaSp3zgKgC5qnN1V5ZTqAjhT54xJ9L");
const LIQ_VAULT_MAIN: Address = address!("4jcJALKPqj8HJLVqyaoZHWgmPaj3NrUAqKbRzJhgK59A");
const REV_ESCROW_GROUP: Address = address!("B5RN6yCA7BpuSE6sLXrTF9jr3xYppAAXZ916YM6az1tD");
const REV_ESCROW_TENANT: Address = address!("7rQy1MP7MRcxdyfBi2UZmFDCSUxoBaZt85vFPNCcDFvG");
const CREATOR_ESCROW: Address = address!("kiupjCCSLu5CQ2vQpBwZcpLJmT4ch9uZ6H8X2BAaq6H");
const TEAM_ESCROW: Address = address!("42ppjEacskgn6oucmLD1fthbzp28EXiyQDorC9si6PW7");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_rise_resolve_with_known_market_buy() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111111");

    let new_shoulder_end = 0;
    let floor_increase_ratio = [0; 16];
    let max_new_floor = [0; 16];
    let max_area_shrinkage_tolerance_units = 0;
    let min_liq_ratio = [0; 16];

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Rise {
            market: Some(MARKET),
            new_shoulder_end: Some(new_shoulder_end),
            floor_increase_ratio: Some(floor_increase_ratio),
            max_new_floor: Some(max_new_floor),
            max_area_shrinkage_tolerance_units: Some(max_area_shrinkage_tolerance_units),
            min_liq_ratio: Some(min_liq_ratio),
        },
        &MINT_TOKEN,
        &MINT_MAIN,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 23, "rise requires 23 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, RISE_PROGRAM_ID, "rise program");

    // Buyer
    assert_eq!(accounts[1].pubkey, user, "buyer");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Tenant
    assert_eq!(accounts[2].pubkey, TENANT, "tenant");
    assert!(accounts[2].is_writable);

    // Market
    assert_eq!(accounts[3].pubkey, MARKET, "market");
    assert!(accounts[3].is_writable);

    // Cash escrow
    assert_eq!(accounts[4].pubkey, CASH_ESCROW, "cash escrow");
    assert!(accounts[4].is_writable);

    // May tenant
    assert_eq!(accounts[5].pubkey, MAY_TENANT, "may tenant");

    // May market group
    assert_eq!(accounts[6].pubkey, MAY_MARKET_GROUP, "may market group");

    // Market meta
    assert_eq!(accounts[7].pubkey, MARKET_META, "market meta");
    assert!(accounts[7].is_writable);

    // May market
    assert_eq!(accounts[8].pubkey, MAY_MARKET, "may market");
    assert!(accounts[8].is_writable);

    // Tenant seed
    assert_eq!(accounts[9].pubkey, TENANT_SEED, "tenant seed");

    // Mint token
    assert_eq!(accounts[10].pubkey, MINT_TOKEN, "mint token");
    assert!(accounts[10].is_writable);

    // Mint main
    assert_eq!(accounts[11].pubkey, MINT_MAIN, "mint main");

    // Token dst
    let expected_token_dst = get_associated_token_address(&user, &MINT_TOKEN, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[12].pubkey, expected_token_dst, "token dst");
    assert!(accounts[12].is_writable);

    // Main src
    let expected_main_src = get_associated_token_address(&user, &MINT_MAIN, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[13].pubkey, expected_main_src, "main src");
    assert!(accounts[13].is_writable);

    // Liq vault main
    assert_eq!(accounts[14].pubkey, LIQ_VAULT_MAIN, "liq vault main");
    assert!(accounts[14].is_writable);

    // Rev escrow group
    assert_eq!(accounts[15].pubkey, REV_ESCROW_GROUP, "rev escrow group");
    assert!(accounts[15].is_writable);

    // Rev escrow tenant
    assert_eq!(accounts[16].pubkey, REV_ESCROW_TENANT, "rev escrow tenant");
    assert!(accounts[16].is_writable);

    // Token program main
    assert_eq!(accounts[17].pubkey, TOKEN_PROGRAM_ID, "token program main");

    // Token program
    assert_eq!(accounts[18].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Mayflower program
    assert_eq!(
        accounts[19].pubkey, MAYFLOWER_PROGRAM_ID,
        "mayflower program"
    );

    // May log account
    assert_eq!(accounts[20].pubkey, MAY_LOG_ACCOUNT, "may log account");
    assert!(accounts[20].is_writable);

    // Creator escrow
    assert_eq!(accounts[21].pubkey, CREATOR_ESCROW, "creator escrow");
    assert!(accounts[21].is_writable);

    // Team escrow
    assert_eq!(accounts[22].pubkey, TEAM_ESCROW, "team escrow");
    assert!(accounts[22].is_writable);

    // new_shoulder_end
    assert_eq!(data[0..8], new_shoulder_end.to_le_bytes());

    // floor_increase_ratio
    assert_eq!(data[8..24], floor_increase_ratio);

    // max_new_floor
    assert_eq!(data[24..40], max_new_floor);

    // max_area_shrinkage_tolerance_units
    assert_eq!(
        data[40..48],
        max_area_shrinkage_tolerance_units.to_le_bytes()
    );

    // min_liq_ratio
    assert_eq!(data[48..64], min_liq_ratio);
}

#[tokio::test]
async fn test_rise_resolve_with_known_market_sell() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111111");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Rise {
            market: Some(MARKET),
            new_shoulder_end: None,
            floor_increase_ratio: None,
            max_new_floor: None,
            max_area_shrinkage_tolerance_units: None,
            min_liq_ratio: None,
        },
        &MINT_TOKEN,
        &MINT_MAIN,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 22, "rise requires 22 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, RISE_PROGRAM_ID, "rise program");

    // Buyer
    assert_eq!(accounts[1].pubkey, user, "buyer");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Tenant
    assert_eq!(accounts[2].pubkey, TENANT, "tenant");
    assert!(accounts[2].is_writable);

    // Market
    assert_eq!(accounts[3].pubkey, MARKET, "market");
    assert!(accounts[3].is_writable);

    // Cash escrow
    assert_eq!(accounts[4].pubkey, CASH_ESCROW, "cash escrow");
    assert!(accounts[4].is_writable);

    // May tenant
    assert_eq!(accounts[5].pubkey, MAY_TENANT, "may tenant");

    // May market group
    assert_eq!(accounts[6].pubkey, MAY_MARKET_GROUP, "may market group");

    // Market meta
    assert_eq!(accounts[7].pubkey, MARKET_META, "market meta");
    assert!(accounts[7].is_writable);

    // May market
    assert_eq!(accounts[8].pubkey, MAY_MARKET, "may market");
    assert!(accounts[8].is_writable);

    // Mint token
    assert_eq!(accounts[9].pubkey, MINT_TOKEN, "mint token");
    assert!(accounts[9].is_writable);

    // Mint main
    assert_eq!(accounts[10].pubkey, MINT_MAIN, "mint main");

    // Token dst
    let expected_token_dst = get_associated_token_address(&user, &MINT_TOKEN, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[11].pubkey, expected_token_dst, "token dst");
    assert!(accounts[11].is_writable);

    // Main src
    let expected_main_src = get_associated_token_address(&user, &MINT_MAIN, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[12].pubkey, expected_main_src, "main src");
    assert!(accounts[12].is_writable);

    // Liq vault main
    assert_eq!(accounts[13].pubkey, LIQ_VAULT_MAIN, "liq vault main");
    assert!(accounts[13].is_writable);

    // Rev escrow group
    assert_eq!(accounts[14].pubkey, REV_ESCROW_GROUP, "rev escrow group");
    assert!(accounts[14].is_writable);

    // Rev escrow tenant
    assert_eq!(accounts[15].pubkey, REV_ESCROW_TENANT, "rev escrow tenant");
    assert!(accounts[15].is_writable);

    // Token program main
    assert_eq!(accounts[16].pubkey, TOKEN_PROGRAM_ID, "token program main");

    // Token program
    assert_eq!(accounts[17].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Mayflower program
    assert_eq!(
        accounts[18].pubkey, MAYFLOWER_PROGRAM_ID,
        "mayflower program"
    );

    // May log account
    assert_eq!(accounts[19].pubkey, MAY_LOG_ACCOUNT, "may log account");
    assert!(accounts[19].is_writable);

    // Creator escrow
    assert_eq!(accounts[20].pubkey, CREATOR_ESCROW, "creator escrow");
    assert!(accounts[20].is_writable);

    // Team escrow
    assert_eq!(accounts[21].pubkey, TEAM_ESCROW, "team escrow");
    assert!(accounts[21].is_writable);

    // sell has no extra data
    assert!(data.is_empty());
}
