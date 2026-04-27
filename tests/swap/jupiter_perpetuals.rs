use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, jupiter_perpetuals_fixtures_dir, load_and_set_json_fixture,
        load_program, send_transaction, setup_svm, JUPITER_PERPETUALS_PROGRAM_ID, TEST_PROGRAM_ID,
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

pub const TRANSFER_AUTHORITY: Address = address!("AVzP2GeRmqGphJsMxWoqjpUifPpCret7LqWhD8NWQK49");
pub const PERPETUALS: Address = address!("H4ND9aYttUVLFmNypZqLjZ52FYiGvdEB45GmwNoKEjTj");
pub const POOL: Address = address!("5BUwFW4nRbftYTDMbgxykoFWqWHPzahFSNAaaaJtVKsq");
pub const SOL_VAULT: Address = address!("BUvduFTd2sWFagCunBPLupG8fBTJqweLw9DuhruNFSCm");
pub const USDC_VAULT: Address = address!("WzWUoCmtVv7eqAbU3BfKPU3fhLP6CXR8NCJH78UK9VS");
pub const JLP_MINT: Address = address!("27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4");
pub const EVENT_AUTHORITY: Address = address!("37hJBDnntwqhGbK7L6M1bLyvccj4u55CCUiLPdYkiqBN");
pub const SOL_CUSTODY: Address = address!("7xS2gz2bTp3fwCC7knJvUWTEU9Tycczu6VhJYKgi1wdz");
pub const WETH_CUSTODY: Address = address!("AQCGyheWPLeo6Qp9WpYS9m3Qj479t7R636N9ey1rEjEn");
pub const WBTC_CUSTODY: Address = address!("5Pv3gM9JrFFH883SWAhvJC9RPYmo8UNxuFtv5bMMALkm");
pub const USDC_CUSTODY: Address = address!("G18jKKXQwBbrHeiK3C9MRXhkHsLHf7XgCSisykV46EZa");
pub const USDT_CUSTODY: Address = address!("4vkNeXiYEUizLdrpdPS1eC2mccyM4NUPRtERrk6ZETkk");
pub const SOL_AG_PRICE_FEED: Address = address!("FYq2BWQ1V5P1WFBqr3qB2Kb5yHVvSv7upzKodgQE5zXh");
pub const WETH_AG_PRICE_FEED: Address = address!("AFZnHPzy4mvVCffrVwhewHbFc93uTHvDSFrVH7GtfXF1");
pub const WBTC_AG_PRICE_FEED: Address = address!("hUqAT1KQ7eW1i6Csp9CXYtpPfSAvi835V7wKi5fRfmC");
pub const USDC_AG_PRICE_FEED: Address = address!("6Jp2xZUTWdDD2ZyUPRzeMdc6AFQ5K3pFgZxk2EijfjnM");
pub const USDT_AG_PRICE_FEED: Address = address!("Fgc93D641F8N2d1xLjQ4jmShuD3GE3BsCXA56KBQbF5u");
pub const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
pub const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

fn load_programs_and_fixtures(svm: &mut LiteSVM) {
    load_program(svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Jupiter Perpetuals program
    load_program(
        svm,
        JUPITER_PERPETUALS_PROGRAM_ID,
        &format!(
            "{}/jupiter_perpetuals.so",
            jupiter_perpetuals_fixtures_dir()
        ),
    );

    // Load fixtures
    load_and_set_json_fixture(svm, &format!("{}/wsol_mint.json", common_fixtures_dir()));
    load_and_set_json_fixture(svm, &format!("{}/usdc_mint.json", common_fixtures_dir()));
    load_and_set_json_fixture(
        svm,
        &format!("{}/usdt_mint.json", jupiter_perpetuals_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/jlp_mint.json", jupiter_perpetuals_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/weth_mint.json", jupiter_perpetuals_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/wbtc_mint.json", jupiter_perpetuals_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/perpetuals.json", jupiter_perpetuals_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/pool.json", jupiter_perpetuals_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/sol_vault.json", jupiter_perpetuals_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/weth_vault.json", jupiter_perpetuals_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/wbtc_vault.json", jupiter_perpetuals_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/usdc_vault.json", jupiter_perpetuals_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/usdt_vault.json", jupiter_perpetuals_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/sol_custody.json", jupiter_perpetuals_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/weth_custody.json", jupiter_perpetuals_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/wbtc_custody.json", jupiter_perpetuals_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/usdc_custody.json", jupiter_perpetuals_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/usdt_custody.json", jupiter_perpetuals_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/sol_ag_price_feed.json",
            jupiter_perpetuals_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/weth_ag_price_feed.json",
            jupiter_perpetuals_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/wbtc_ag_price_feed.json",
            jupiter_perpetuals_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/usdc_ag_price_feed.json",
            jupiter_perpetuals_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/usdt_ag_price_feed.json",
            jupiter_perpetuals_fixtures_dir()
        ),
    );

    let mut clock = svm.get_sysvar::<Clock>();
    // Jump ahead to pool aum_usd_updated_at (1_777_275_312) + 10
    clock.unix_timestamp = 1_777_275_312 + 10;
    // Jump ahead to pool aum_usd_refreshed_at_slo (415_957_237) + 1
    clock.slot = 415_957_237 + 1;
    svm.set_sysvar::<Clock>(&clock);
}

#[test]
fn test_jupiter_perpetuals_swap_cpi_swap_2() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_programs_and_fixtures(&mut svm);

    // Create trader token accounts with initial balances
    // Selling SOL (input=WSOL) for USDC (output)
    let initial_wsol = 1_000_000_000u64; // 1 SOL
    let initial_usdc = 0u64;
    let trader_wsol =
        create_token_account(&mut svm, &payer.pubkey(), &WSOL_MINT, initial_wsol, false);
    let trader_usdc =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);

    // Build swap instruction: sell 0.001 SOL for USDC
    let in_amount = 1_000_000u64; // 0.001 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    // Jupiter Perpetuals swap2 accounts layout (18 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(JUPITER_PERPETUALS_PROGRAM_ID, false), // jupiter_perpetuals_program
        AccountMeta::new(payer.pubkey(), true),                          // owner
        AccountMeta::new(trader_wsol, false),                            // funding_account
        AccountMeta::new(trader_usdc, false),                            // receiving_account
        AccountMeta::new_readonly(TRANSFER_AUTHORITY, false),            // transfer_authority
        AccountMeta::new_readonly(PERPETUALS, false),                    // perpetuals
        AccountMeta::new(POOL, false),                                   // pool
        AccountMeta::new(SOL_CUSTODY, false),                            // receiving_custody
        AccountMeta::new_readonly(SOL_AG_PRICE_FEED, false), // receiving_custody_doves_price_account
        AccountMeta::new_readonly(SOL_AG_PRICE_FEED, false), // receiving_custody_pythnet_price_account
        AccountMeta::new(SOL_VAULT, false),                  // receiving_custody_token_account
        AccountMeta::new(USDC_CUSTODY, false),               // dispensing_custody
        AccountMeta::new_readonly(USDC_AG_PRICE_FEED, false), // dispensing_custody_doves_price_account
        AccountMeta::new_readonly(USDC_AG_PRICE_FEED, false), // dispensing_custody_pythnet_price_account
        AccountMeta::new(USDC_VAULT, false),                  // dispensing_custody_token_account
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),   // token_program
        AccountMeta::new_readonly(EVENT_AUTHORITY, false),    // event_authority
        AccountMeta::new_readonly(JUPITER_PERPETUALS_PROGRAM_ID, false), // program
    ];

    // Jupiter Perpetuals swap2 has no extra data
    let extra_data: &[u8] = &[];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::JupiterPerpetuals,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_wsol = get_token_balance(&svm, &trader_wsol);
            let final_usdc = get_token_balance(&svm, &trader_usdc);

            assert!(
                final_wsol < initial_wsol,
                "WSOL should have decreased: {} -> {}",
                initial_wsol,
                final_wsol
            );
            assert!(
                final_usdc > initial_usdc,
                "USDC should have increased: {} -> {}",
                initial_usdc,
                final_usdc
            );

            println!(
                "Jupiter Perpetuals swap2 successful! WSOL: {} -> {}, USDC: {} -> {}",
                initial_wsol, final_wsol, initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("Jupiter Perpetuals swap2 CPI failed: {}", e);
        }
    }
}

#[test]
fn test_jupiter_perpetuals_swap_cpi_add_liquidity_2() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_programs_and_fixtures(&mut svm);

    // Create trader token accounts with initial balances
    // Selling SOL (input=WSOL) for JLP (output)
    let initial_wsol = 1_000_000_000u64; // 1 SOL
    let initial_jlp = 0u64;
    let trader_wsol =
        create_token_account(&mut svm, &payer.pubkey(), &WSOL_MINT, initial_wsol, false);
    let trader_jlp = create_token_account(&mut svm, &payer.pubkey(), &JLP_MINT, initial_jlp, false);

    // Build swap instruction: sell 0.001 SOL for JLP
    let in_amount = 1_000_000u64; // 0.001 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    // Jupiter Perpetuals liquidity2 accounts layout (24 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(JUPITER_PERPETUALS_PROGRAM_ID, false), // jupiter_perpetuals_program
        AccountMeta::new(payer.pubkey(), true),                          // owner
        AccountMeta::new(trader_wsol, false), // funding_or_receiving_account
        AccountMeta::new(trader_jlp, false),  // lp_token_account
        AccountMeta::new_readonly(TRANSFER_AUTHORITY, false), // transfer_authority
        AccountMeta::new_readonly(PERPETUALS, false), // perpetuals
        AccountMeta::new(POOL, false),        // pool
        AccountMeta::new(SOL_CUSTODY, false), // custody
        AccountMeta::new_readonly(SOL_AG_PRICE_FEED, false), // custody_doves_price_account
        AccountMeta::new_readonly(SOL_AG_PRICE_FEED, false), // custody_pythnet_price_account
        AccountMeta::new(SOL_VAULT, false),   // custody_token_account
        AccountMeta::new(JLP_MINT, false),    // lp_token_mint
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false), // token_program
        AccountMeta::new_readonly(EVENT_AUTHORITY, false), // event_authority
        AccountMeta::new_readonly(JUPITER_PERPETUALS_PROGRAM_ID, false), // program
        AccountMeta::new(SOL_CUSTODY, false), // sol_cutody
        AccountMeta::new_readonly(WETH_CUSTODY, false), // weth_custody
        AccountMeta::new_readonly(WBTC_CUSTODY, false), // wbtc_custody
        AccountMeta::new_readonly(USDC_CUSTODY, false), // usdc_custody
        AccountMeta::new_readonly(USDT_CUSTODY, false), // usdt_custody
        AccountMeta::new_readonly(SOL_AG_PRICE_FEED, false), // sol_ag_price_feed
        AccountMeta::new_readonly(WETH_AG_PRICE_FEED, false), // weth_ag_price_feed
        AccountMeta::new_readonly(WBTC_AG_PRICE_FEED, false), // wbtc_ag_price_feed
        AccountMeta::new_readonly(USDC_AG_PRICE_FEED, false), // usdc_ag_price_feed
        AccountMeta::new_readonly(USDT_AG_PRICE_FEED, false), // usdt_ag_price_feed
    ];

    // is_add_liquidity = true
    let extra_data: &[u8] = &[1u8];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::JupiterPerpetuals,
        extra_data,
    );

    // Execute via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_wsol = get_token_balance(&svm, &trader_wsol);
            let final_jlp = get_token_balance(&svm, &trader_jlp);

            assert!(
                final_wsol < initial_wsol,
                "WSOL should have decreased on add_liquidity_2: {} -> {}",
                initial_wsol,
                final_wsol
            );
            assert!(
                final_jlp > initial_jlp,
                "JLP should have increased on add_liquidity_2: {} -> {}",
                initial_jlp,
                final_jlp
            );

            println!(
                "Jupiter Perpetuals add_liquidity_2 successful! WSOL: {} -> {}, JLP: {} -> {}",
                initial_wsol, final_wsol, initial_jlp, final_jlp
            );
        }
        Err(e) => {
            panic!("Jupiter Perpetuals add_liquidity_2 CPI failed: {}", e);
        }
    }
}

#[test]
fn test_jupiter_perpetuals_swap_cpi_remove_liquidity_2() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_programs_and_fixtures(&mut svm);

    // Create trader token accounts with initial balances
    // Selling JLP (input) for WSOL (output)
    let initial_jlp = 1_000_000_000u64; // 1000 JLP
    let initial_wsol = 1_000_000_000u64; // 1 SOL
    let trader_jlp = create_token_account(&mut svm, &payer.pubkey(), &JLP_MINT, initial_jlp, false);
    let trader_wsol =
        create_token_account(&mut svm, &payer.pubkey(), &WSOL_MINT, initial_wsol, false);

    // Build swap instruction: sell 1 JLP for SOL
    let in_amount = 1_000_000u64; // 1 JLP
    let min_out_amount = 1u64; // Very loose slippage for test

    // Jupiter Perpetuals liquidity2 accounts layout (24 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(JUPITER_PERPETUALS_PROGRAM_ID, false), // jupiter_perpetuals_program
        AccountMeta::new(payer.pubkey(), true),                          // owner
        AccountMeta::new(trader_wsol, false), // funding_or_receiving_account
        AccountMeta::new(trader_jlp, false),  // lp_token_account
        AccountMeta::new_readonly(TRANSFER_AUTHORITY, false), // transfer_authority
        AccountMeta::new_readonly(PERPETUALS, false), // perpetuals
        AccountMeta::new(POOL, false),        // pool
        AccountMeta::new(SOL_CUSTODY, false), // custody
        AccountMeta::new_readonly(SOL_AG_PRICE_FEED, false), // custody_doves_price_account
        AccountMeta::new_readonly(SOL_AG_PRICE_FEED, false), // custody_pythnet_price_account
        AccountMeta::new(SOL_VAULT, false),   // custody_token_account
        AccountMeta::new(JLP_MINT, false),    // lp_token_mint
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false), // token_program
        AccountMeta::new_readonly(EVENT_AUTHORITY, false), // event_authority
        AccountMeta::new_readonly(JUPITER_PERPETUALS_PROGRAM_ID, false), // program
        AccountMeta::new(SOL_CUSTODY, false), // sol_cutody
        AccountMeta::new_readonly(WETH_CUSTODY, false), // weth_custody
        AccountMeta::new_readonly(WBTC_CUSTODY, false), // wbtc_custody
        AccountMeta::new_readonly(USDC_CUSTODY, false), // usdc_custody
        AccountMeta::new_readonly(USDT_CUSTODY, false), // usdt_custody
        AccountMeta::new_readonly(SOL_AG_PRICE_FEED, false), // sol_ag_price_feed
        AccountMeta::new_readonly(WETH_AG_PRICE_FEED, false), // weth_ag_price_feed
        AccountMeta::new_readonly(WBTC_AG_PRICE_FEED, false), // wbtc_ag_price_feed
        AccountMeta::new_readonly(USDC_AG_PRICE_FEED, false), // usdc_ag_price_feed
        AccountMeta::new_readonly(USDT_AG_PRICE_FEED, false), // usdt_ag_price_feed
    ];

    // is_add_liquidity = false
    let extra_data: &[u8] = &[0u8];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::JupiterPerpetuals,
        extra_data,
    );

    // Execute via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_jlp = get_token_balance(&svm, &trader_jlp);
            let final_wsol = get_token_balance(&svm, &trader_wsol);

            assert!(
                final_jlp < initial_jlp,
                "JLP should have decreased on remove_liquidity_2: {} -> {}",
                initial_jlp,
                final_jlp
            );
            assert!(
                final_wsol > initial_wsol,
                "WSOL should have increased on remove_liquidity_2: {} -> {}",
                initial_wsol,
                final_wsol
            );

            println!(
                "Jupiter Perpetuals remove_liquidity_2 successful! WSOL: {} -> {}, JLP: {} -> {}",
                initial_wsol, final_wsol, initial_jlp, final_jlp
            );
        }
        Err(e) => {
            panic!("Jupiter Perpetuals remove_liquidity_2 CPI failed: {}", e);
        }
    }
}
