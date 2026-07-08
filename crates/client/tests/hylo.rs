use {
    beethoven_client::{
        deposit::{
            hylo::{HYLO_PROGRAM_ID, HYUSD_MINT, SOL_USD_PYTH_FEED, XSOL_MINT},
            DepositProtocol,
        },
        get_associated_token_address, resolve_deposit, ASSOCIATED_TOKEN_PROGRAM_ID,
        SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

const JITOSOL_MINT: Address = address!("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn");

fn get_rpc_url() -> String {
    std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

fn derive_pda(seeds: &[&[u8]]) -> Address {
    let (addr, _) = Address::find_program_address(seeds, &HYLO_PROGRAM_ID);
    addr
}

/// Run with: cargo test -p beethoven-client --features hylo,resolve -- print_hylo_dump_commands --ignored --nocapture
/// Then copy-paste the output commands (requires `solana` CLI configured for mainnet).
#[test]
#[ignore]
fn print_hylo_dump_commands() {
    let fixtures_dir = "fixtures/deposit/hylo";

    let hylo_state = derive_pda(&[b"hylo"]);
    let fee_auth = derive_pda(&[b"fee_auth", JITOSOL_MINT.as_ref()]);
    let vault_auth = derive_pda(&[b"vault_auth", JITOSOL_MINT.as_ref()]);
    let stablecoin_auth = derive_pda(&[b"mint_auth", HYUSD_MINT.as_ref()]);
    let levercoin_auth = derive_pda(&[b"mint_auth", XSOL_MINT.as_ref()]);
    let lst_header = derive_pda(&[b"lst_header", JITOSOL_MINT.as_ref()]);
    let event_authority = derive_pda(&[b"__event_authority"]);

    let fee_vault = get_associated_token_address(&fee_auth, &JITOSOL_MINT, &TOKEN_PROGRAM_ID);
    let lst_vault = get_associated_token_address(&vault_auth, &JITOSOL_MINT, &TOKEN_PROGRAM_ID);

    println!("# Hylo fixture dump commands (JitoSOL as LST)");
    println!("# Run from repo root with mainnet RPC\n");
    println!("mkdir -p {fixtures_dir}\n");
    println!("# Program binary");
    println!(
        "solana program dump {} {fixtures_dir}/hylo_program.so\n",
        HYLO_PROGRAM_ID
    );
    println!("# State PDAs (have on-chain data)");
    println!("solana account {hylo_state} --output json-compact > {fixtures_dir}/hylo_state.json");
    println!("solana account {lst_header} --output json-compact > {fixtures_dir}/lst_header.json");
    println!("\n# Authority PDAs (signing-only, no on-chain data — created as mocks in tests)");
    println!("# fee_auth:        {fee_auth}");
    println!("# vault_auth:      {vault_auth}");
    println!("# stablecoin_auth: {stablecoin_auth}");
    println!("# levercoin_auth:  {levercoin_auth}");
    println!("# event_authority: {event_authority}");
    println!("\n# ATAs (vaults)");
    println!("solana account {fee_vault} --output json-compact > {fixtures_dir}/fee_vault.json");
    println!("solana account {lst_vault} --output json-compact > {fixtures_dir}/lst_vault.json");
    println!("\n# Mints");
    println!(
        "solana account {} --output json-compact > {fixtures_dir}/jitosol_mint.json",
        JITOSOL_MINT
    );
    println!(
        "solana account {} --output json-compact > {fixtures_dir}/hyusd_mint.json",
        HYUSD_MINT
    );
    println!(
        "solana account {} --output json-compact > {fixtures_dir}/xsol_mint.json",
        XSOL_MINT
    );
    println!("\n# Oracle");
    println!(
        "solana account {} --output json-compact > {fixtures_dir}/sol_usd_pyth_feed.json",
        SOL_USD_PYTH_FEED
    );
}

#[tokio::test]
async fn test_hylo_stablecoin_resolve() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_deposit(
        &rpc,
        &DepositProtocol::Hylo {
            lst_mint: JITOSOL_MINT,
            mint_type: 0,
            expected_token_out: 0,
            slippage_tolerance: 0,
        },
        &user,
    )
    .await
    .unwrap();

    // stablecoin: 1 detection + 17 CPI = 18 total + 5 tail = 18 + additional anchor accounts
    assert_eq!(
        accounts.len(),
        19,
        "stablecoin requires 19 accounts (1 detection + 18 CPI)"
    );

    // Detection prefix
    assert_eq!(accounts[0].pubkey, HYLO_PROGRAM_ID);

    // User (signer)
    assert_eq!(accounts[1].pubkey, user);
    assert!(accounts[1].is_writable);
    assert!(accounts[1].is_signer);

    // Hylo state PDA
    let hylo_state = derive_pda(&[b"hylo"]);
    assert_eq!(accounts[2].pubkey, hylo_state);
    assert!(accounts[2].is_writable);

    // fee_auth PDA
    let fee_auth = derive_pda(&[b"fee_auth", JITOSOL_MINT.as_ref()]);
    assert_eq!(accounts[3].pubkey, fee_auth);

    // vault_auth PDA
    let vault_auth = derive_pda(&[b"vault_auth", JITOSOL_MINT.as_ref()]);
    assert_eq!(accounts[4].pubkey, vault_auth);

    // stablecoin_auth PDA
    let stablecoin_auth = derive_pda(&[b"mint_auth", HYUSD_MINT.as_ref()]);
    assert_eq!(accounts[5].pubkey, stablecoin_auth);

    // fee_vault (ATA of fee_auth for JitoSOL)
    let fee_vault = get_associated_token_address(&fee_auth, &JITOSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[6].pubkey, fee_vault);
    assert!(accounts[6].is_writable);

    // lst_vault (ATA of vault_auth for JitoSOL)
    let lst_vault = get_associated_token_address(&vault_auth, &JITOSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[7].pubkey, lst_vault);
    assert!(accounts[7].is_writable);

    // lst_header PDA
    let lst_header = derive_pda(&[b"lst_header", JITOSOL_MINT.as_ref()]);
    assert_eq!(accounts[8].pubkey, lst_header);

    // user_lst_ta
    let user_lst_ta = get_associated_token_address(&user, &JITOSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[9].pubkey, user_lst_ta);
    assert!(accounts[9].is_writable);

    // user_stablecoin_ta
    let user_hyusd_ta = get_associated_token_address(&user, &HYUSD_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[10].pubkey, user_hyusd_ta);
    assert!(accounts[10].is_writable);

    // lst_mint
    assert_eq!(accounts[11].pubkey, JITOSOL_MINT);

    // output_mint (hyUSD)
    assert_eq!(accounts[12].pubkey, HYUSD_MINT);
    assert!(accounts[12].is_writable);

    // sol_usd_pyth_feed
    assert_eq!(accounts[13].pubkey, SOL_USD_PYTH_FEED);

    // token_program
    assert_eq!(accounts[14].pubkey, TOKEN_PROGRAM_ID);

    // associated_token_program
    assert_eq!(accounts[15].pubkey, ASSOCIATED_TOKEN_PROGRAM_ID);

    // system_program
    assert_eq!(accounts[16].pubkey, SYSTEM_PROGRAM_ID);

    // event_authority PDA
    let event_auth = derive_pda(&[b"__event_authority"]);
    assert_eq!(accounts[17].pubkey, event_auth);

    // program (Hylo itself, for Anchor CPI)
    assert_eq!(accounts[18].pubkey, HYLO_PROGRAM_ID);

    // Verify extra data
    assert_eq!(data.len(), 17);
    assert_eq!(data[0], 0); // mint_type = stablecoin
}

#[tokio::test]
async fn test_hylo_levercoin_resolve() {
    let rpc = RpcClient::new(get_rpc_url());
    let user = Address::from_str_const("11111111111111111111111111111112");

    let (accounts, data) = resolve_deposit(
        &rpc,
        &DepositProtocol::Hylo {
            lst_mint: JITOSOL_MINT,
            mint_type: 1,
            expected_token_out: 0,
            slippage_tolerance: 0,
        },
        &user,
    )
    .await
    .unwrap();

    // levercoin: 1 detection + 18 CPI + 1 extra (stablecoin_mint) = 20 total
    assert_eq!(
        accounts.len(),
        20,
        "levercoin requires 20 accounts (1 detection + 19 CPI)"
    );

    // Detection prefix
    assert_eq!(accounts[0].pubkey, HYLO_PROGRAM_ID);

    // levercoin_auth PDA (instead of stablecoin_auth)
    let levercoin_auth = derive_pda(&[b"mint_auth", XSOL_MINT.as_ref()]);
    assert_eq!(accounts[5].pubkey, levercoin_auth);

    // user_levercoin_ta
    let user_xsol_ta = get_associated_token_address(&user, &XSOL_MINT, &TOKEN_PROGRAM_ID);
    assert_eq!(accounts[10].pubkey, user_xsol_ta);

    // output_mint (xSOL)
    assert_eq!(accounts[12].pubkey, XSOL_MINT);
    assert!(accounts[12].is_writable);

    // stablecoin_mint (extra account for levercoin)
    assert_eq!(accounts[13].pubkey, HYUSD_MINT);
    assert!(!accounts[13].is_writable);

    // sol_usd_pyth_feed shifted by 1
    assert_eq!(accounts[14].pubkey, SOL_USD_PYTH_FEED);

    // program at the end
    assert_eq!(accounts[19].pubkey, HYLO_PROGRAM_ID);

    // Verify extra data
    assert_eq!(data.len(), 17);
    assert_eq!(data[0], 1); // mint_type = levercoin
}
