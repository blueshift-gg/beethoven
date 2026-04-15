use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::futarchy::{SwapType, FUTARCHY_PROGRAM_ID},
        SwapProtocol, TOKEN_PROGRAM_ID,
    },
    solana_address::Address,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const USDC_MINT: Address = Address::from_str_const("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const METADAO_MINT: Address =
    Address::from_str_const("METAwkXcqyXKy1AtsSgJ8JiUHwGCafnZL38n3vYmeta");
const METADAO_DAO: Address =
    Address::from_str_const("CUPoiqkK4hxyCiJcLC4yE9AtJP1MoV1vFV2vx3jqwWeS");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_futarchy_resolve_with_known_pair() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Futarchy {
            dao: Some(METADAO_DAO),
            swap_type: SwapType::Buy,
        },
        &USDC_MINT,
        &METADAO_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 10, "futarchy requires 10 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, FUTARCHY_PROGRAM_ID, "futarchy program");

    // Dao
    assert_eq!(accounts[1].pubkey, METADAO_DAO, "dao");
    assert!(accounts[1].is_writable);

    // User base account
    let expected_metadao_ata =
        get_associated_token_address(&user, &METADAO_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[2].pubkey, expected_metadao_ata,
        "user base account"
    );
    assert!(accounts[2].is_writable);

    // User quote account
    let expected_usdc_ata = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[3].pubkey, expected_usdc_ata, "user quote account");
    assert!(accounts[3].is_writable);

    let dao = accounts[1].pubkey;

    // AMM base vault
    let expected_amm_base_vault =
        get_associated_token_address(&dao, &METADAO_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[4].pubkey, expected_amm_base_vault,
        "amm base vault"
    );
    assert!(accounts[4].is_writable);

    // AMM quote vault
    let expected_amm_quote_vault =
        beethoven_client::get_associated_token_address(&dao, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[5].pubkey, expected_amm_quote_vault,
        "amm quote vault"
    );
    assert!(accounts[5].is_writable);

    // User
    assert_eq!(accounts[6].pubkey, user, "user");
    assert!(accounts[6].is_signer);

    // Token program
    assert_eq!(accounts[7].pubkey, TOKEN_PROGRAM_ID, "token_program");

    // swap type = buy
    assert_eq!(data, vec![SwapType::Buy as u8]);
}

#[tokio::test]
async fn test_futarchy_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::Futarchy {
            dao: Some(METADAO_DAO),
            swap_type: SwapType::Sell,
        },
        &METADAO_MINT,
        &USDC_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 10, "futarchy requires 10 accounts");

    // Protocol program ID
    assert_eq!(accounts[0].pubkey, FUTARCHY_PROGRAM_ID);

    // Dao
    assert_eq!(accounts[1].pubkey, METADAO_DAO, "dao");
    assert!(accounts[1].is_writable);

    // User base account
    let expected_metadao_ata =
        get_associated_token_address(&user, &METADAO_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[2].pubkey, expected_metadao_ata,
        "user quote account"
    );
    assert!(accounts[2].is_writable);

    // User quote account
    let expected_usdc_ata = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[3].pubkey, expected_usdc_ata, "user base account");
    assert!(accounts[3].is_writable);

    let dao = accounts[1].pubkey;

    // AMM base vault
    let expected_amm_base_vault =
        get_associated_token_address(&dao, &METADAO_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[4].pubkey, expected_amm_base_vault,
        "amm base vault"
    );
    assert!(accounts[4].is_writable);

    // AMM quote vault
    let expected_amm_quote_vault =
        get_associated_token_address(&dao, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[5].pubkey, expected_amm_quote_vault,
        "amm quote vault"
    );
    assert!(accounts[5].is_writable);

    // User
    assert_eq!(accounts[6].pubkey, user, "user");
    assert!(accounts[6].is_signer);

    // Token program
    assert_eq!(accounts[7].pubkey, TOKEN_PROGRAM_ID, "token_program");

    // swap type = sell
    assert_eq!(data, vec![SwapType::Sell as u8]);
}
