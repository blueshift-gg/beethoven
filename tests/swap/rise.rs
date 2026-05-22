use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, rise_fixtures_dir,
        send_transaction, setup_svm, MAYFLOWER_PROGRAM_ID, RISE_PROGRAM_ID, TEST_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    litesvm::LiteSVM,
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
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
const MAY_LOG_ACCOUNT: Address = address!("EKVkmuwDKRKHw85NPTbKSKuS75EY4NLcxe1qzSPixLdy");
const TENANT_SEED: Address = address!("Eg4Akr8HRv3gy4MaSp3zgKgC5qnN1V5ZTqAjhT54xJ9L");
const LIQ_VAULT_MAIN: Address = address!("4jcJALKPqj8HJLVqyaoZHWgmPaj3NrUAqKbRzJhgK59A");
const REV_ESCROW_GROUP: Address = address!("B5RN6yCA7BpuSE6sLXrTF9jr3xYppAAXZ916YM6az1tD");
const REV_ESCROW_TENANT: Address = address!("7rQy1MP7MRcxdyfBi2UZmFDCSUxoBaZt85vFPNCcDFvG");
const CREATOR_ESCROW: Address = address!("kiupjCCSLu5CQ2vQpBwZcpLJmT4ch9uZ6H8X2BAaq6H");
const TEAM_ESCROW: Address = address!("42ppjEacskgn6oucmLD1fthbzp28EXiyQDorC9si6PW7");

fn load_program_and_fixtures(svm: &mut LiteSVM) {
    load_program(svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Rise program
    load_program(
        svm,
        RISE_PROGRAM_ID,
        &format!("{}/rise.so", rise_fixtures_dir()),
    );
    load_program(
        svm,
        MAYFLOWER_PROGRAM_ID,
        &format!("{}/mayflower.so", rise_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(svm, &format!("{}/wsol_mint.json", common_fixtures_dir()));
    load_and_set_json_fixture(svm, &format!("{}/mint_token.json", rise_fixtures_dir()));
    load_and_set_json_fixture(svm, &format!("{}/tenant.json", rise_fixtures_dir()));
    load_and_set_json_fixture(svm, &format!("{}/market.json", rise_fixtures_dir()));
    load_and_set_json_fixture(svm, &format!("{}/cash_escrow.json", rise_fixtures_dir()));
    load_and_set_json_fixture(svm, &format!("{}/may_tenant.json", rise_fixtures_dir()));
    load_and_set_json_fixture(
        svm,
        &format!("{}/may_market_group.json", rise_fixtures_dir()),
    );
    load_and_set_json_fixture(svm, &format!("{}/market_meta.json", rise_fixtures_dir()));
    load_and_set_json_fixture(svm, &format!("{}/may_market.json", rise_fixtures_dir()));
    load_and_set_json_fixture(svm, &format!("{}/liq_vault_main.json", rise_fixtures_dir()));
    load_and_set_json_fixture(
        svm,
        &format!("{}/rev_escrow_group.json", rise_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/rev_escrow_tenant.json", rise_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/may_log_account.json", rise_fixtures_dir()),
    );
    load_and_set_json_fixture(svm, &format!("{}/creator_escrow.json", rise_fixtures_dir()));
    load_and_set_json_fixture(svm, &format!("{}/team_escrow.json", rise_fixtures_dir()));
}

#[test]
fn test_rise_swap_cpi_buy() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program_and_fixtures(&mut svm);

    // Jump ahead to market last_floor_rise_timestamp (1_776_524_218) + market gov floor_raise_cooldown_seconds (200)
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_776_524_218 + 200;
    svm.set_sysvar::<Clock>(&clock);

    // Create trader token accounts with initial balances
    // Selling SOL (input=WSOL) for base mint (output)
    let initial_wsol = 1_000_000_000u64; // 1 SOL
    let initial_base_mint = 0u64;
    let trader_input =
        create_token_account(&mut svm, &payer.pubkey(), &MINT_MAIN, initial_wsol, false);
    let trader_output = create_token_account(
        &mut svm,
        &payer.pubkey(),
        &MINT_TOKEN,
        initial_base_mint,
        false,
    );

    // Build swap instruction: sell 0.001 SOL for base mint
    let in_amount = 1_000_000u64; // 0.001 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    // Rise base accounts layout (23 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(RISE_PROGRAM_ID, false), // rise_program
        AccountMeta::new(payer.pubkey(), true),            // buyer
        AccountMeta::new(TENANT, false),                   // tenant
        AccountMeta::new(MARKET, false),                   // market
        AccountMeta::new(CASH_ESCROW, false),              // cash_escrow
        AccountMeta::new_readonly(MAY_TENANT, false),      // may_tenant
        AccountMeta::new(MAY_MARKET_GROUP, false),         // may_market_group
        AccountMeta::new(MARKET_META, false),              // market_meta
        AccountMeta::new(MAY_MARKET, false),               // may_market
        AccountMeta::new(TENANT_SEED, false),              // tenant_seed
        AccountMeta::new(MINT_TOKEN, false),               // mint_token
        AccountMeta::new_readonly(MINT_MAIN, false),       // mint_main
        AccountMeta::new(trader_output, false),            // token_dst
        AccountMeta::new(trader_input, false),             // main_src
        AccountMeta::new(LIQ_VAULT_MAIN, false),           // liq_vault_main
        AccountMeta::new(REV_ESCROW_GROUP, false),         // rev_escrow_group
        AccountMeta::new(REV_ESCROW_TENANT, false),        // rev_escrow_tenant
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false), // token_program_main
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false), // token_program
        AccountMeta::new_readonly(MAYFLOWER_PROGRAM_ID, false), // mayflower_program
        AccountMeta::new(MAY_LOG_ACCOUNT, false),          // may_log_account
        AccountMeta::new(CREATOR_ESCROW, false),           // creator_escrow
        AccountMeta::new(TEAM_ESCROW, false),              // team_escrow
    ];

    let mut extra_data = Vec::with_capacity(64);
    // new_shoulder_end
    extra_data.extend_from_slice(&0_u64.to_le_bytes());
    // floor_increase_ratio
    extra_data.extend_from_slice(&[0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    // max_new_floor
    extra_data.extend_from_slice(&[0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    // max_area_shrinkage_tolerance_units
    extra_data.extend_from_slice(&100_000_000_u64.to_le_bytes());
    // min_liq_ratio
    extra_data.extend_from_slice(&[0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::Rise,
        &extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_wsol = get_token_balance(&svm, &trader_input);
            let final_base_mint = get_token_balance(&svm, &trader_output);

            assert!(
                final_wsol < initial_wsol,
                "WSOL should have decreased: {} -> {}",
                initial_wsol,
                final_wsol
            );
            assert!(
                final_base_mint > initial_base_mint,
                "Base mint should have increased: {} -> {}",
                initial_base_mint,
                final_base_mint
            );

            println!(
                "Rise swap successful! WSOL: {} -> {}, base mint: {} -> {}",
                initial_wsol, final_wsol, initial_base_mint, final_base_mint
            );
        }
        Err(e) => {
            panic!("Rise swap CPI failed: {}", e);
        }
    }
}

#[test]
fn test_rise_swap_cpi_sell() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program_and_fixtures(&mut svm);

    // Jump ahead to market last_floor_rise_timestamp (1_776_524_218) + market gov floor_raise_cooldown_seconds (200)
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_776_524_218 + 200;
    svm.set_sysvar::<Clock>(&clock);

    // Create trader token accounts with initial balances
    // Selling base mint (input) for SOL (output)
    let initial_base_mint = 100_000_000_000u64; // 100 * 10 ^ 9 units
    let initial_wsol = 1_000_000_000u64; // 1 SOL
    let trader_input = create_token_account(
        &mut svm,
        &payer.pubkey(),
        &MINT_TOKEN,
        initial_base_mint,
        false,
    );
    let trader_output =
        create_token_account(&mut svm, &payer.pubkey(), &MINT_MAIN, initial_wsol, false);

    // Build swap instruction: sell 1 unit of base mint for SOL
    let in_amount = 1_000_000_000u64; // 1 unit
    let min_out_amount = 1u64; // Very loose slippage for test

    // Rise base accounts layout (22 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(RISE_PROGRAM_ID, false), // rise_program
        AccountMeta::new(payer.pubkey(), true),            // buyer
        AccountMeta::new(TENANT, false),                   // tenant
        AccountMeta::new(MARKET, false),                   // market
        AccountMeta::new(CASH_ESCROW, false),              // cash_escrow
        AccountMeta::new_readonly(MAY_TENANT, false),      // may_tenant
        AccountMeta::new(MAY_MARKET_GROUP, false),         // may_market_group
        AccountMeta::new(MARKET_META, false),              // market_meta
        AccountMeta::new(MAY_MARKET, false),               // may_market
        AccountMeta::new(MINT_TOKEN, false),               // mint_token
        AccountMeta::new_readonly(MINT_MAIN, false),       // mint_main
        AccountMeta::new(trader_input, false),             // token_src
        AccountMeta::new(trader_output, false),            // main_dst
        AccountMeta::new(LIQ_VAULT_MAIN, false),           // liq_vault_main
        AccountMeta::new(REV_ESCROW_GROUP, false),         // rev_escrow_group
        AccountMeta::new(REV_ESCROW_TENANT, false),        // rev_escrow_tenant
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false), // token_program_main
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false), // token_program
        AccountMeta::new_readonly(MAYFLOWER_PROGRAM_ID, false), // mayflower_program
        AccountMeta::new(MAY_LOG_ACCOUNT, false),          // may_log_account
        AccountMeta::new(CREATOR_ESCROW, false),           // creator_escrow
        AccountMeta::new(TEAM_ESCROW, false),              // team_escrow
    ];

    // sell has no extra data
    let extra_data = vec![];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::Rise,
        &extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_base_mint = get_token_balance(&svm, &trader_input);
            let final_wsol = get_token_balance(&svm, &trader_output);

            assert!(
                final_base_mint < initial_base_mint,
                "Base mint should have decreased: {} -> {}",
                initial_base_mint,
                final_base_mint
            );
            assert!(
                final_wsol > initial_wsol,
                "WSOL should have increased: {} -> {}",
                initial_wsol,
                final_wsol
            );

            println!(
                "Rise swap successful! base mint: {} -> {}, WSOL: {} -> {}",
                initial_base_mint, final_base_mint, initial_wsol, final_wsol
            );
        }
        Err(e) => {
            panic!("Rise swap CPI failed: {}", e);
        }
    }
}
