use {
    beethoven_client::{
        deposit::{carrot_boost::CARROT_BOOST_PROGRAM_ID, DepositProtocol},
        get_associated_token_address, resolve_deposit, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const CLEND_GROUP: Address = address!("9PWf4kEwa3E4WCMnPp4SQoUGWNaA8Zn427g33n6jcmMb");
const CLEND_ACCOUNT: Address = address!("HwEujdhizP5gpHC63a6xF9qWjo2NvKvdTJdNEhHY9hhK");
const USDC_BANK: Address = address!("4a74Z8rY6JuuTUeVv7i8kB7LQRANb72jMtweFTUoQM81");
const USDC_VAULT: Address = address!("4ZU6vJULZNxP9BQzRgc5UFtzrSJhs77An9iA6W9ceUEq");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_carrot_boost_resolve_with_known_clend_account_and_bank() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let deposit_up_to_amount = 1;

    let (accounts, data) = resolve_deposit(
        &rpc,
        &DepositProtocol::CarrotBoost {
            clend_account: CLEND_ACCOUNT,
            bank: USDC_BANK,
            deposit_up_to_amount,
        },
        &user,
    )
    .await
    .unwrap();

    assert_eq!(accounts.len(), 8, "carrot boost requires 8 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, CARROT_BOOST_PROGRAM_ID,
        "carrot boost program"
    );

    // Clend group
    assert_eq!(accounts[1].pubkey, CLEND_GROUP, "clend group");

    // Clend account
    assert_eq!(accounts[2].pubkey, CLEND_ACCOUNT, "clend account");

    // Signer
    assert_eq!(accounts[3].pubkey, user, "signer");
    assert!(accounts[3].is_signer);
    assert!(accounts[3].is_writable);

    // Bank
    assert_eq!(accounts[4].pubkey, USDC_BANK, "bank");
    assert!(accounts[4].is_writable);

    // Signer token account
    let expected_signer_token_account =
        get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(
        accounts[5].pubkey, expected_signer_token_account,
        "signer token account"
    );
    assert!(accounts[5].is_writable);

    // Bank liquidity vault
    assert_eq!(accounts[6].pubkey, USDC_VAULT, "bank liquidity vault");
    assert!(accounts[6].is_writable);

    // Token program
    assert_eq!(accounts[7].pubkey, TOKEN_PROGRAM_ID, "token program");

    // deposit_up_to_amount
    assert_eq!(data[0], deposit_up_to_amount, "deposit_up_to_amount");
}
