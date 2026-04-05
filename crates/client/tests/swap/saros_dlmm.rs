use {
    beethoven_client::{
        get_associated_token_address, resolve_swap,
        swap::saros_dlmm::{SAROS_DLMM_PROGRAM_ID, SAROS_MDMA_HOOKS_PROGRAM_ID},
        SwapProtocol, MEMO_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USD1_MINT: Address = address!("USD1ttGY1N17NEEHLmELoaybftRBUSErhqYiQzvEmuB");
const PAIR: Address = address!("8yrUdy1XufCuupHgbpptcer1npNkQDVh95sLnc67CfR2");
const BIN_ARRAY_LOWER: Address = address!("4PvQBrRimmeHiKQs2op4upDm3nEPDbWbUrnoUrkSvoDD");
const BIN_ARRAY_UPPER: Address = address!("AxPaxEzyyk1MFg5Xhk8u2PUvKR5ppYRBA2rjaT8NYbQc");
const TOKEN_VAULT_X: Address = address!("GJDXcwHfdJ1AbRZ1RoR9CLPXkiUwvf7zdWEnMnY1Cibp");
const TOKEN_VAULT_Y: Address = address!("A1rGSThS9uSgkb5SiDa5Fo479Lg1r4vFv3UcWPCMh9hm");
const HOOK: Address = address!("FBsXR7JfRyMsyoSpcGDaoax7XbJZS2Cj3aaoSAm8L7uH");
const EVENT_AUTHORITY: Address = address!("AQjz6RZK93SLjxfDGKL9nCYQNSjEbQSdETxwR63jXV8m");
const HOOK_BIN_ARRAY_0: Address = address!("4JZ5GA1xPP5o1FSe7H8kzSK5dZS8LX1yDf2zPVuueTho");
const HOOK_BIN_ARRAY_1: Address = address!("HHxjVEz8KW79C1CAchvUyrbJJaf6ydwBjTz5DGi1HQfM");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

#[tokio::test]
async fn test_saros_dlmm_resolve_with_known_pair() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::SarosDlmm {
            pair: Some(PAIR),
            swap_for_y: true,
            swap_type: 0,
        },
        &USDC_MINT,
        &USD1_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(
        accounts.len(),
        20,
        "saros dlmm: 18 fixed accounts + 2 active hook bin arrays when pair has hook"
    );

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, SAROS_DLMM_PROGRAM_ID,
        "saros dlmm program"
    );

    // Pair
    assert_eq!(accounts[1].pubkey, PAIR, "pair");
    assert!(accounts[1].is_writable);

    // Token mint X
    assert_eq!(accounts[2].pubkey, USD1_MINT, "token mint x");

    // Token mint Y
    assert_eq!(accounts[3].pubkey, USDC_MINT, "token mint y");

    // Bin array lower
    assert_eq!(accounts[4].pubkey, BIN_ARRAY_LOWER, "bin array lower");
    assert!(accounts[4].is_writable);

    // Bin array upper
    assert_eq!(accounts[5].pubkey, BIN_ARRAY_UPPER, "bin array upper");
    assert!(accounts[5].is_writable);

    // Token vault X
    assert_eq!(accounts[6].pubkey, TOKEN_VAULT_X, "token vault x");
    assert!(accounts[6].is_writable);

    // Token vault Y
    assert_eq!(accounts[7].pubkey, TOKEN_VAULT_Y, "token vault y");
    assert!(accounts[7].is_writable);

    // User vault X
    let expected_user_vault_x = get_associated_token_address(&user, &USD1_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[8].pubkey, expected_user_vault_x, "user vault x");
    assert!(accounts[8].is_writable);

    // User vault Y
    let expected_user_vault_y = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[9].pubkey, expected_user_vault_y, "user vault y");
    assert!(accounts[9].is_writable);

    // User
    assert_eq!(accounts[10].pubkey, user, "user");
    assert!(accounts[10].is_signer);
    assert!(accounts[10].is_writable);

    // Token program X
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "token program x");

    // Token program Y
    assert_eq!(accounts[12].pubkey, TOKEN_PROGRAM_ID, "token program y");

    // Memo program
    assert_eq!(accounts[13].pubkey, MEMO_PROGRAM_ID, "memo program");

    // Hook
    assert_eq!(accounts[14].pubkey, HOOK, "hook");
    assert!(accounts[14].is_writable);

    // Hooks program
    assert_eq!(
        accounts[15].pubkey, SAROS_MDMA_HOOKS_PROGRAM_ID,
        "hooks program"
    );

    // Event authority
    assert_eq!(accounts[16].pubkey, EVENT_AUTHORITY, "event authority");

    // Program
    assert_eq!(
        accounts[17].pubkey, SAROS_DLMM_PROGRAM_ID,
        "saros dlmm program"
    );

    // Active hook bin array lower
    assert_eq!(
        accounts[18].pubkey, HOOK_BIN_ARRAY_0,
        "active hook bin array lower"
    );
    assert!(accounts[18].is_writable);

    // Active hook bin array upper
    assert_eq!(
        accounts[19].pubkey, HOOK_BIN_ARRAY_1,
        "active hook bin array upper"
    );
    assert!(accounts[19].is_writable);

    // swap_for_y = true, swap_type = ExactInput
    assert_eq!(data, vec![1, 0]);
}

#[tokio::test]
async fn test_saros_dlmm_resolve_flipped_mints() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = address!("11111111111111111111111111111112");

    // Selling USDC for USD1 — `swap_for_y` differs; mint layout on pair unchanged
    let (accounts, data) = resolve_swap(
        &rpc,
        &SwapProtocol::SarosDlmm {
            pair: Some(PAIR),
            swap_for_y: false,
            swap_type: 0,
        },
        &USDC_MINT,
        &USD1_MINT,
        &user,
    )
    .await
    .unwrap();

    assert_eq!(
        accounts.len(),
        20,
        "saros dlmm: 18 fixed accounts + 2 active hook bin arrays when pair has hook"
    );

    // Protocol program ID
    assert_eq!(
        accounts[0].pubkey, SAROS_DLMM_PROGRAM_ID,
        "saros dlmm program"
    );

    // Pair
    assert_eq!(accounts[1].pubkey, PAIR, "pair");
    assert!(accounts[1].is_writable);

    // Token mint X
    assert_eq!(accounts[2].pubkey, USD1_MINT, "token mint x");

    // Token mint Y
    assert_eq!(accounts[3].pubkey, USDC_MINT, "token mint y");

    // Bin array lower
    assert_eq!(accounts[4].pubkey, BIN_ARRAY_LOWER, "bin array lower");
    assert!(accounts[4].is_writable);

    // Bin array upper
    assert_eq!(accounts[5].pubkey, BIN_ARRAY_UPPER, "bin array upper");
    assert!(accounts[5].is_writable);

    // Token vault X
    assert_eq!(accounts[6].pubkey, TOKEN_VAULT_X, "token vault x");
    assert!(accounts[6].is_writable);

    // Token vault Y
    assert_eq!(accounts[7].pubkey, TOKEN_VAULT_Y, "token vault y");
    assert!(accounts[7].is_writable);

    // User vault X
    let expected_user_vault_x = get_associated_token_address(&user, &USD1_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[8].pubkey, expected_user_vault_x, "user vault x");
    assert!(accounts[8].is_writable);

    // User vault Y
    let expected_user_vault_y = get_associated_token_address(&user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[9].pubkey, expected_user_vault_y, "user vault y");
    assert!(accounts[9].is_writable);

    // User
    assert_eq!(accounts[10].pubkey, user, "user");
    assert!(accounts[10].is_signer);
    assert!(accounts[10].is_writable);

    // Token program X
    assert_eq!(accounts[11].pubkey, TOKEN_PROGRAM_ID, "token program x");

    // Token program Y
    assert_eq!(accounts[12].pubkey, TOKEN_PROGRAM_ID, "token program y");

    // Memo program
    assert_eq!(accounts[13].pubkey, MEMO_PROGRAM_ID, "memo program");

    // Hook
    assert_eq!(accounts[14].pubkey, HOOK, "hook");
    assert!(accounts[14].is_writable);

    // Hooks program
    assert_eq!(
        accounts[15].pubkey, SAROS_MDMA_HOOKS_PROGRAM_ID,
        "hooks program"
    );

    // Event authority
    assert_eq!(accounts[16].pubkey, EVENT_AUTHORITY, "event authority");

    // Program
    assert_eq!(
        accounts[17].pubkey, SAROS_DLMM_PROGRAM_ID,
        "saros dlmm program"
    );

    // Active hook bin array lower
    assert_eq!(
        accounts[18].pubkey, HOOK_BIN_ARRAY_0,
        "active hook bin array lower"
    );
    assert!(accounts[18].is_writable);

    // Active hook bin array upper
    assert_eq!(
        accounts[19].pubkey, HOOK_BIN_ARRAY_1,
        "active hook bin array upper"
    );
    assert!(accounts[19].is_writable);

    // swap_for_y = false, swap_type = ExactInput
    assert_eq!(data, vec![0, 0]);
}
