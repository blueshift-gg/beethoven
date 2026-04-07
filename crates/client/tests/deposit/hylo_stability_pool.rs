use {
    beethoven_client::{
        deposit::{hylo_stability_pool::HYLO_STABILITY_PROGRAM_ID, DepositProtocol},
        get_associated_token_address, resolve_deposit, ASSOCIATED_TOKEN_PROGRAM_ID,
        SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const POOL_CONFIG: Address = address!("2jk7miWrsTbt5hUSaCXPkEQPvuUMgbFLpgMzMQw3Z6ar");
const HYLO: Address = address!("9cd2sAfbBvKs4SX9YKo4dcjwP3TgTVQ8dT5koshGcDND");
const HYUSD_MINT: Address = address!("5YMkXAYccHSGnHn9nob9xEvv6Pvka9DZWH7nTbotTu9E");
const XSOL_MINT: Address = address!("4sWNB8zGWHkh6UnmwiEtzNxL4XrN7uK9tosbESbJFfVs");
const POOL_AUTH: Address = address!("5YrRAQag9BbJkauDtJkd1vsTquXT6N46oU8rJ66GDxHd");
const STABLECOIN_POOL: Address = address!("EqozKyMj7FVnLHc2cJj3VC25aBr4AhVh1cGM2WDajGe9");
const LEVERCOIN_POOL: Address = address!("4GPXVXuzk8ABAUkoXeBJg8r9kccEXQjoi5vqSxE9rhk1");
const LP_TOKEN_AUTH: Address = address!("5YWerkcqAXTSCzKC1X52BXtfv2aoNCB6wzv7wEXuGWpq");
const SHYUSD_MINT: Address = address!("HnnGv3HrSqjRpgdFmx7vQGjntNEoex1SU4e9Lxcxuihz");
const SOL_PRICE_UPDATE_V2: Address = address!("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");
const EVENT_AUTHORITY: Address = address!("8fjUWoZTb8ox8JFRJTb7WznL1V8oJT9o21kQKHJzbTS8");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_hylo_stability_pool_resolve() {
    let rpc: RpcClient = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_deposit(&rpc, &DepositProtocol::HyloStabilityPool, &user)
        .await
        .unwrap();

    assert_eq!(
        accounts.len(),
        19,
        "hylo stability pool requires 19 accounts"
    );

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, HYLO_STABILITY_PROGRAM_ID,
        "hylo stability program"
    );

    // User
    assert_eq!(accounts[1].pubkey, user, "user");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Pool config
    assert_eq!(accounts[2].pubkey, POOL_CONFIG, "pool config");

    // Hylo
    assert_eq!(accounts[3].pubkey, HYLO, "hylo");

    // Stablecoin mint
    assert_eq!(accounts[4].pubkey, HYUSD_MINT, "stablecoin mint");

    // Levercoin mint
    assert_eq!(accounts[5].pubkey, XSOL_MINT, "levercoin mint");

    // User stablecoin token account
    let expected_user_stablecoin_ta =
        get_associated_token_address(&user, &HYUSD_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[6].pubkey, expected_user_stablecoin_ta,
        "user stablecoin token account"
    );
    assert!(accounts[6].is_writable);

    // User lp token token account
    let expected_user_lp_token_ta =
        get_associated_token_address(&user, &SHYUSD_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[7].pubkey, expected_user_lp_token_ta,
        "user lp token token account"
    );
    assert!(accounts[7].is_writable);

    // Pool auth
    assert_eq!(accounts[8].pubkey, POOL_AUTH, "pool auth");

    // Stablecoin pool
    assert_eq!(accounts[9].pubkey, STABLECOIN_POOL, "stablecoin pool");
    assert!(accounts[9].is_writable);

    // Levercoin pool
    assert_eq!(accounts[10].pubkey, LEVERCOIN_POOL, "levercoin pool");

    // LP token auth
    assert_eq!(accounts[11].pubkey, LP_TOKEN_AUTH, "lp token auth");

    // LP token mint
    assert_eq!(accounts[12].pubkey, SHYUSD_MINT, "lp token mint");
    assert!(accounts[12].is_writable);

    // SOL price update v2
    assert_eq!(
        accounts[13].pubkey, SOL_PRICE_UPDATE_V2,
        "sol price update v2"
    );

    // System program
    assert_eq!(accounts[14].pubkey, SYSTEM_PROGRAM_ID, "system program");

    // Token program
    assert_eq!(accounts[15].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Associated token program
    assert_eq!(
        accounts[16].pubkey, ASSOCIATED_TOKEN_PROGRAM_ID,
        "associated token program"
    );

    // Event authority
    assert_eq!(accounts[17].pubkey, EVENT_AUTHORITY, "event authority");

    // Program
    assert_eq!(
        accounts[18].pubkey, HYLO_STABILITY_PROGRAM_ID,
        "hylo stability program"
    );

    // Hylo stability pool deposit has no extra data
    assert!(data.is_empty());
}
