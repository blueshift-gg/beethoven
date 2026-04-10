use {
    beethoven_client::{
        deposit::{
            defi_tuna::{DEFI_TUNA_PROGRAM_ID, TUNA_CONFIG},
            DepositProtocol,
        },
        get_associated_token_address, resolve_deposit, MEMO_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const VAULT: Address = address!("D76dDcSU5HnAGqVEZCDLyGgLpTp4xZuqeZyVDtUdDv55");
const VAULT_ATA: Address = address!("4iTbtBmr4fXpkUD4kTW9pujvXbCT3AkWya6h3dbNP7a6");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_defi_tuna_resolve_with_known_vault() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) =
        resolve_deposit(&rpc, &DepositProtocol::DefiTuna { vault: VAULT }, &user)
            .await
            .unwrap();

    assert_eq!(accounts.len(), 10, "defi tuna requires 10 accounts");

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, DEFI_TUNA_PROGRAM_ID,
        "defi tuna program"
    );

    // Authority
    assert_eq!(accounts[1].pubkey, user, "authority");
    assert!(accounts[1].is_signer);
    assert!(accounts[1].is_writable);

    // Mint
    assert_eq!(accounts[2].pubkey, USDC_MINT, "mint");

    // Tuna config
    assert_eq!(accounts[3].pubkey, TUNA_CONFIG, "tuna config");

    // Lending position
    let lending_position = Address::find_program_address(
        &[b"lending_position", user.as_ref(), VAULT.as_ref()],
        &DEFI_TUNA_PROGRAM_ID,
    )
    .0;
    assert_eq!(accounts[4].pubkey, lending_position, "lending position");
    assert!(accounts[4].is_writable);

    // Vault
    assert_eq!(accounts[5].pubkey, VAULT, "vault");
    assert!(accounts[5].is_writable);

    // Vault ATA
    assert_eq!(accounts[6].pubkey, VAULT_ATA, "vault ata");
    assert!(accounts[6].is_writable);

    // Authority ATA
    let authority_ata = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[7].pubkey, authority_ata, "authority ata");
    assert!(accounts[7].is_writable);

    // Token program
    assert_eq!(accounts[8].pubkey, TOKEN_PROGRAM_ID, "token program");

    // Memo program
    assert_eq!(accounts[9].pubkey, MEMO_PROGRAM_ID, "memo program");

    // No extra data
    assert!(data.is_empty());
}
