use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::hylo_exchange::{
            EVENT_AUTHORITY, HYLO, HYLO_EXCHANGE_PROGRAM_ID, HYUSD_FEE_AUTH, HYUSD_FEE_VAULT,
            HYUSD_MINT, HYUSD_STABLECOIN_AUTH, SOL_PRICE_UPDATE_V2,
        },
        SwapProtocol, ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const XSOL_MINT: Address = address!("4sWNB8zGWHkh6UnmwiEtzNxL4XrN7uK9tosbESbJFfVs");
const XSOL_LEVERCOIN_AUTH: Address = address!("J8rGkrzsvqinX9kfwD8SkP3mRzXAk3uiDRUaiZKXM4as");
const JITOSOL_MINT: Address = address!("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn");
const JITOSOL_FEE_AUTH: Address = address!("FpLaqELxKRm6S3bjfNSknwZu43TL89VYkwuMDwsRMj59");
const JITOSOL_VAULT_AUTH: Address = address!("82MNhUCha26wY4kohTUEC965b4ypEe7RPa4itp9UMrKK");
const JITOSOL_FEE_VAULT: Address = address!("3JENUTyYnMMtZUSg5ErSHEvowjQteYD7wr7RDNw12bei");
const JITOSOL_VAULT: Address = address!("2Y3TLkdGoJwbdizxqrZmQwNLYJyGKTgzC4tbetbkvQ43");
const JITOSOL_LST_HEADER: Address = address!("8Ri52tZXZehgAHKbx1MQiXhWXXkVsvAL9op6C5HytDKF");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_hylo_exchange_resolve_mint_stablecoin() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::HyloExchange { swap_type: 0 },
        &JITOSOL_MINT,
        &HYUSD_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(
        accounts.len(),
        19,
        "hylo exchange mint_stablecoin requires 19 accounts"
    );

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, HYLO_EXCHANGE_PROGRAM_ID,
        "hylo exchange program"
    );

    // User
    assert_eq!(accounts[1].pubkey, user, "user");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Hylo
    assert_eq!(accounts[2].pubkey, HYLO, "hylo");
    assert!(accounts[2].is_writable);

    // Fee auth
    assert_eq!(accounts[3].pubkey, JITOSOL_FEE_AUTH, "fee auth");

    // Vault auth
    assert_eq!(accounts[4].pubkey, JITOSOL_VAULT_AUTH, "vault auth");

    // Stablecoin auth
    assert_eq!(accounts[5].pubkey, HYUSD_STABLECOIN_AUTH, "stablecoin auth");

    // Fee vault
    assert_eq!(accounts[6].pubkey, JITOSOL_FEE_VAULT, "fee vault");
    assert!(accounts[6].is_writable);

    // Lst vault
    assert_eq!(accounts[7].pubkey, JITOSOL_VAULT, "lst vault");
    assert!(accounts[7].is_writable);

    // Lst header
    assert_eq!(accounts[8].pubkey, JITOSOL_LST_HEADER, "lst header");

    // User lst ta
    let expected_user_lst_ta =
        get_associated_token_address(&user, &JITOSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[9].pubkey, expected_user_lst_ta, "user lst ta");
    assert!(accounts[9].is_writable);

    // User stablecoin ta
    let expected_user_stablecoin_ta =
        get_associated_token_address(&user, &HYUSD_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[10].pubkey, expected_user_stablecoin_ta,
        "user stablecoin ta"
    );
    assert!(accounts[10].is_writable);

    // Lst mint
    assert_eq!(accounts[11].pubkey, JITOSOL_MINT, "lst mint");

    // Stablecoin mint
    assert_eq!(accounts[12].pubkey, HYUSD_MINT, "stablecoin mint");
    assert!(accounts[12].is_writable);

    // Sol usd pyth feed
    assert_eq!(
        accounts[13].pubkey, SOL_PRICE_UPDATE_V2,
        "sol usd pyth feed"
    );

    // Token program
    assert_eq!(accounts[14].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Associated token program
    assert_eq!(
        accounts[15].pubkey, ASSOCIATED_TOKEN_PROGRAM_ID,
        "associated token program"
    );

    // System program
    assert_eq!(accounts[16].pubkey, SYSTEM_PROGRAM_ID, "system program");

    // Event authority
    assert_eq!(accounts[17].pubkey, EVENT_AUTHORITY, "event authority");

    // Program
    assert_eq!(accounts[18].pubkey, HYLO_EXCHANGE_PROGRAM_ID, "program");

    // swap_type
    assert_eq!(data, vec![0_u8]);
}

#[tokio::test]
async fn test_hylo_exchange_resolve_redeem_stablecoin() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::HyloExchange { swap_type: 1 },
        &HYUSD_MINT,
        &JITOSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(
        accounts.len(),
        18,
        "hylo exchange redeem_stablecoin requires 18 accounts"
    );

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, HYLO_EXCHANGE_PROGRAM_ID,
        "hylo exchange program"
    );

    // User
    assert_eq!(accounts[1].pubkey, user, "user");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Hylo
    assert_eq!(accounts[2].pubkey, HYLO, "hylo");
    assert!(accounts[2].is_writable);

    // Fee auth
    assert_eq!(accounts[3].pubkey, JITOSOL_FEE_AUTH, "fee auth");

    // Vault auth
    assert_eq!(accounts[4].pubkey, JITOSOL_VAULT_AUTH, "vault auth");

    // Fee vault
    assert_eq!(accounts[5].pubkey, JITOSOL_FEE_VAULT, "fee vault");
    assert!(accounts[5].is_writable);

    // Lst vault
    assert_eq!(accounts[6].pubkey, JITOSOL_VAULT, "lst vault");
    assert!(accounts[6].is_writable);

    // Lst header
    assert_eq!(accounts[7].pubkey, JITOSOL_LST_HEADER, "lst header");

    // User stablecoin ta
    let expected_user_stablecoin_ta =
        get_associated_token_address(&user, &HYUSD_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[8].pubkey, expected_user_stablecoin_ta,
        "user stablecoin ta"
    );
    assert!(accounts[8].is_writable);

    // User lst ta
    let expected_user_lst_ta =
        get_associated_token_address(&user, &JITOSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[9].pubkey, expected_user_lst_ta, "user lst ta");
    assert!(accounts[9].is_writable);

    // Stablecoin mint
    assert_eq!(accounts[10].pubkey, HYUSD_MINT, "stablecoin mint");
    assert!(accounts[10].is_writable);

    // Lst mint
    assert_eq!(accounts[11].pubkey, JITOSOL_MINT, "lst mint");

    // Sol usd pyth feed
    assert_eq!(
        accounts[12].pubkey, SOL_PRICE_UPDATE_V2,
        "sol usd pyth feed"
    );

    // System program
    assert_eq!(accounts[13].pubkey, SYSTEM_PROGRAM_ID, "system program");

    // Token program
    assert_eq!(accounts[14].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Associated token program
    assert_eq!(
        accounts[15].pubkey, ASSOCIATED_TOKEN_PROGRAM_ID,
        "associated token program"
    );

    // Event authority
    assert_eq!(accounts[16].pubkey, EVENT_AUTHORITY, "event authority");

    // Program
    assert_eq!(accounts[17].pubkey, HYLO_EXCHANGE_PROGRAM_ID, "program");

    // swap_type
    assert_eq!(data, vec![1_u8]);
}

#[tokio::test]
async fn test_hylo_exchange_resolve_mint_levercoin() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::HyloExchange { swap_type: 2 },
        &JITOSOL_MINT,
        &XSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(
        accounts.len(),
        20,
        "hylo exchange mint_levercoin requires 20 accounts"
    );

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, HYLO_EXCHANGE_PROGRAM_ID,
        "hylo exchange program"
    );

    // User
    assert_eq!(accounts[1].pubkey, user, "user");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Hylo
    assert_eq!(accounts[2].pubkey, HYLO, "hylo");
    assert!(accounts[2].is_writable);

    // Fee auth
    assert_eq!(accounts[3].pubkey, JITOSOL_FEE_AUTH, "fee auth");

    // Vault auth
    assert_eq!(accounts[4].pubkey, JITOSOL_VAULT_AUTH, "vault auth");

    // Levercoin auth (mint_auth PDA derived from levercoin mint)
    let expected_levercoin_auth = Address::find_program_address(
        &[b"mint_auth", XSOL_MINT.as_ref()],
        &HYLO_EXCHANGE_PROGRAM_ID,
    )
    .0;
    assert_eq!(
        accounts[5].pubkey, expected_levercoin_auth,
        "levercoin auth"
    );

    // Fee vault
    assert_eq!(accounts[6].pubkey, JITOSOL_FEE_VAULT, "fee vault");
    assert!(accounts[6].is_writable);

    // Lst vault
    assert_eq!(accounts[7].pubkey, JITOSOL_VAULT, "lst vault");
    assert!(accounts[7].is_writable);

    // Lst header
    assert_eq!(accounts[8].pubkey, JITOSOL_LST_HEADER, "lst header");

    // User lst ta
    let expected_user_lst_ta =
        get_associated_token_address(&user, &JITOSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[9].pubkey, expected_user_lst_ta, "user lst ta");
    assert!(accounts[9].is_writable);

    // User levercoin ta
    let expected_user_levercoin_ta =
        get_associated_token_address(&user, &XSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[10].pubkey, expected_user_levercoin_ta,
        "user levercoin ta"
    );
    assert!(accounts[10].is_writable);

    // Lst mint
    assert_eq!(accounts[11].pubkey, JITOSOL_MINT, "lst mint");

    // Levercoin mint
    assert_eq!(accounts[12].pubkey, XSOL_MINT, "levercoin mint");
    assert!(accounts[12].is_writable);

    // Stablecoin mint
    assert_eq!(accounts[13].pubkey, HYUSD_MINT, "stablecoin mint");

    // Sol usd pyth feed
    assert_eq!(
        accounts[14].pubkey, SOL_PRICE_UPDATE_V2,
        "sol usd pyth feed"
    );

    // Token program
    assert_eq!(accounts[15].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Associated token program
    assert_eq!(
        accounts[16].pubkey, ASSOCIATED_TOKEN_PROGRAM_ID,
        "associated token program"
    );

    // System program
    assert_eq!(accounts[17].pubkey, SYSTEM_PROGRAM_ID, "system program");

    // Event authority
    assert_eq!(accounts[18].pubkey, EVENT_AUTHORITY, "event authority");

    // Program
    assert_eq!(accounts[19].pubkey, HYLO_EXCHANGE_PROGRAM_ID, "program");

    // swap_type
    assert_eq!(data, vec![2_u8]);
}

#[tokio::test]
async fn test_hylo_exchange_resolve_redeem_levercoin() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::HyloExchange { swap_type: 3 },
        &XSOL_MINT,
        &JITOSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(
        accounts.len(),
        19,
        "hylo exchange redeem_levercoin requires 19 accounts"
    );

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, HYLO_EXCHANGE_PROGRAM_ID,
        "hylo exchange program"
    );

    // User
    assert_eq!(accounts[1].pubkey, user, "user");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Hylo
    assert_eq!(accounts[2].pubkey, HYLO, "hylo");
    assert!(accounts[2].is_writable);

    // Fee auth
    assert_eq!(accounts[3].pubkey, JITOSOL_FEE_AUTH, "fee auth");

    // Vault auth
    assert_eq!(accounts[4].pubkey, JITOSOL_VAULT_AUTH, "vault auth");

    // Fee vault
    assert_eq!(accounts[5].pubkey, JITOSOL_FEE_VAULT, "fee vault");
    assert!(accounts[5].is_writable);

    // Lst vault
    assert_eq!(accounts[6].pubkey, JITOSOL_VAULT, "lst vault");
    assert!(accounts[6].is_writable);

    // Lst header
    assert_eq!(accounts[7].pubkey, JITOSOL_LST_HEADER, "lst header");

    // User levercoin ta
    let expected_user_levercoin_ta =
        get_associated_token_address(&user, &XSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[8].pubkey, expected_user_levercoin_ta,
        "user levercoin ta"
    );
    assert!(accounts[8].is_writable);

    // User lst ta
    let expected_user_lst_ta =
        get_associated_token_address(&user, &JITOSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[9].pubkey, expected_user_lst_ta, "user lst ta");
    assert!(accounts[9].is_writable);

    // Levercoin mint
    assert_eq!(accounts[10].pubkey, XSOL_MINT, "levercoin mint");
    assert!(accounts[10].is_writable);

    // Stablecoin mint
    assert_eq!(accounts[11].pubkey, HYUSD_MINT, "stablecoin mint");

    // Lst mint
    assert_eq!(accounts[12].pubkey, JITOSOL_MINT, "lst mint");

    // Sol usd pyth feed
    assert_eq!(
        accounts[13].pubkey, SOL_PRICE_UPDATE_V2,
        "sol usd pyth feed"
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
    assert_eq!(accounts[18].pubkey, HYLO_EXCHANGE_PROGRAM_ID, "program");

    // swap_type
    assert_eq!(data, vec![3_u8]);
}

#[tokio::test]
async fn test_hylo_exchange_resolve_lever_to_stable() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::HyloExchange { swap_type: 5 },
        &XSOL_MINT,
        &HYUSD_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(
        accounts.len(),
        15,
        "hylo exchange leverage lever_to_stable requires 15 accounts"
    );

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, HYLO_EXCHANGE_PROGRAM_ID,
        "hylo exchange program"
    );

    // User
    assert_eq!(accounts[1].pubkey, user, "user");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Hylo
    assert_eq!(accounts[2].pubkey, HYLO, "hylo");

    // Sol usd pyth feed
    assert_eq!(accounts[3].pubkey, SOL_PRICE_UPDATE_V2, "sol usd pyth feed");

    // Stablecoin mint
    assert_eq!(accounts[4].pubkey, HYUSD_MINT, "stablecoin mint");
    assert!(accounts[4].is_writable);

    // Stablecoin auth
    assert_eq!(accounts[5].pubkey, HYUSD_STABLECOIN_AUTH, "stablecoin auth");

    // Fee auth
    assert_eq!(accounts[6].pubkey, HYUSD_FEE_AUTH, "fee auth");

    // Fee vault
    assert_eq!(accounts[7].pubkey, HYUSD_FEE_VAULT, "fee vault");

    // User stablecoin ta
    let expected_user_stablecoin_ta =
        get_associated_token_address(&user, &HYUSD_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[8].pubkey, expected_user_stablecoin_ta,
        "user stablecoin ta"
    );
    assert!(accounts[8].is_writable);

    // Levercoin mint
    assert_eq!(accounts[9].pubkey, XSOL_MINT, "levercoin mint");
    assert!(accounts[9].is_writable);

    // Levercoin auth
    assert_eq!(accounts[10].pubkey, XSOL_LEVERCOIN_AUTH, "levercoin auth");

    // User levercoin ta
    let expected_user_levercoin_ta =
        get_associated_token_address(&user, &XSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[11].pubkey, expected_user_levercoin_ta,
        "user levercoin ta"
    );
    assert!(accounts[11].is_writable);

    // Token program
    assert_eq!(accounts[12].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Event authority
    assert_eq!(accounts[13].pubkey, EVENT_AUTHORITY, "event authority");

    // Program
    assert_eq!(accounts[14].pubkey, HYLO_EXCHANGE_PROGRAM_ID, "program");

    // swap_type
    assert_eq!(data, vec![5_u8]);
}

#[tokio::test]
async fn test_hylo_exchange_resolve_stable_to_lever() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::HyloExchange { swap_type: 4 },
        &HYUSD_MINT,
        &XSOL_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(
        accounts.len(),
        15,
        "hylo exchange leverage stable_to_lever requires 15 accounts"
    );

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, HYLO_EXCHANGE_PROGRAM_ID,
        "hylo exchange program"
    );

    // User
    assert_eq!(accounts[1].pubkey, user, "user");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Hylo
    assert_eq!(accounts[2].pubkey, HYLO, "hylo");

    // Sol usd pyth feed
    assert_eq!(accounts[3].pubkey, SOL_PRICE_UPDATE_V2, "sol usd pyth feed");

    // Stablecoin mint
    assert_eq!(accounts[4].pubkey, HYUSD_MINT, "stablecoin mint");
    assert!(accounts[4].is_writable);

    // Stablecoin auth
    assert_eq!(accounts[5].pubkey, HYUSD_STABLECOIN_AUTH, "stablecoin auth");

    // Fee auth
    assert_eq!(accounts[6].pubkey, HYUSD_FEE_AUTH, "fee auth");

    // Fee vault
    assert_eq!(accounts[7].pubkey, HYUSD_FEE_VAULT, "fee vault");

    // User stablecoin ta
    let expected_user_stablecoin_ta =
        get_associated_token_address(&user, &HYUSD_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[8].pubkey, expected_user_stablecoin_ta,
        "user stablecoin ta"
    );
    assert!(accounts[8].is_writable);

    // Levercoin mint
    assert_eq!(accounts[9].pubkey, XSOL_MINT, "levercoin mint");
    assert!(accounts[9].is_writable);

    // Levercoin auth
    assert_eq!(accounts[10].pubkey, XSOL_LEVERCOIN_AUTH, "levercoin auth");

    // User levercoin ta
    let expected_user_levercoin_ta =
        get_associated_token_address(&user, &XSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[11].pubkey, expected_user_levercoin_ta,
        "user levercoin ta"
    );
    assert!(accounts[11].is_writable);

    // Token program
    assert_eq!(accounts[12].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Event authority
    assert_eq!(accounts[13].pubkey, EVENT_AUTHORITY, "event authority");

    // Program
    assert_eq!(accounts[14].pubkey, HYLO_EXCHANGE_PROGRAM_ID, "program");

    // swap_type
    assert_eq!(data, vec![4_u8]);
}
