use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, hadron_fixtures_dir, load_and_set_json_fixture, load_program,
        send_transaction, setup_svm, HADRON_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven_client::{SYSVAR_CLOCK_ID, SYSVAR_INSTRUCTIONS_ID},
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const SOL_USDC_POOL_CONFIG: Address = address!("3BKXKtD8oRgfbfYU997oHYjqcUcwZng8GUcEBPuqrM52");
const SOL_USDC_POOL_CURVE_META: Address = address!("7QiPDHa2KdgFxtMt8trPKoGL9LKgGf8Amj8s5RSqRgrY");
const SOL_USDC_POOL_CURVE_PREFABS: Address =
    address!("B5mqchgv1dMLGAdZuPPRGrZHH8hKkSpM1Sdq9452chUQ");
const SOL_USDC_POOL_CURVE_UPDATES: Address =
    address!("AXV636YSCBadzqbC12DzHF6gMvEswHXsZQzw8aGM3vwo");
const SOL_USDC_POOL_SPREAD_CONFIG: Address =
    address!("Ee3nrioTwmfiAqkGpZUQQod6xNTfSfRxbGDuFMC7JSWY");
const SOL_USDC_POOL_MIDPRICE_ORACLE: Address =
    address!("Es1rSVcMYxARBWBFwwuzaxYbHT9XvDviCyUdetHpJBvd");
const SOL_USDC_POOL_VAULT_DEST: Address = address!("69TeReLEXBAUKPokrwiU9Xw6CBDnKkqbRFGLmvJdWz6L");
const SOL_USDC_POOL_VAULT_SOURCE: Address =
    address!("BAWQ8oUSHYpaA836gPJWanXipKU4tb7bAMRHjbzapa7p");
const FEE_RECIPIENT_SOL_ATA: Address = address!("5X6eBNjcfQYEZRQWE954ud3d5cR8ez4Ax3asyCuZgC7r");
const FEE_CONFIG: Address = address!("FoDswiGbeRMEm2dEpLfvJTZaQjjZXFcJzCYyhzxiWcRd");

#[test]
fn test_hadron_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Hadron program
    load_program(
        &mut svm,
        HADRON_PROGRAM_ID,
        &format!("{}/hadron.so", hadron_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/wsol_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/fee_config.json", hadron_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/fee_recipient_sol_ata.json", hadron_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/sol_usdc_pool_config.json", hadron_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/sol_usdc_pool_curve_meta.json", hadron_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/sol_usdc_pool_curve_prefabs.json", hadron_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/sol_usdc_pool_curve_updates.json", hadron_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/sol_usdc_pool_midprice_oracle.json",
            hadron_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/sol_usdc_vault_dest.json", hadron_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/sol_usdc_vault_source.json", hadron_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/sol_usdc_pool_spread_config.json", hadron_fixtures_dir()),
    );

    // Set timestamp and slot, specific to when fixtures are fetched
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_775_035_931;
    clock.slot = 410_283_799;
    svm.set_sysvar::<Clock>(&clock);

    // Create trader token accounts with initial balances
    // Selling SOL (input=WSOL) for USDC (output)
    let initial_wsol = 1_000_000_000u64; // 1 SOL
    let initial_usdc = 0u64;
    let trader_input = create_token_account(&mut svm, &payer.pubkey(), &WSOL_MINT, initial_wsol);
    let trader_output = create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc);

    // Build swap instruction: sell 0.001 SOL for USDC
    let in_amount = 1_000_000u64; // 0.001 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    // Hadron accounts layout (18 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(HADRON_PROGRAM_ID, false), // hadron_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),  // token_program_x
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),  // token_program_y
        AccountMeta::new_readonly(SOL_USDC_POOL_CONFIG, false), // config
        AccountMeta::new_readonly(SOL_USDC_POOL_MIDPRICE_ORACLE, false), // midprice_oracle
        AccountMeta::new_readonly(SOL_USDC_POOL_CURVE_META, false), // curve_meta
        AccountMeta::new(SOL_USDC_POOL_CURVE_PREFABS, false), // curve_prefabs
        AccountMeta::new_readonly(payer.pubkey(), true),     // user
        AccountMeta::new(trader_input, false),               // user_source
        AccountMeta::new(SOL_USDC_POOL_VAULT_SOURCE, false), // vault_source
        AccountMeta::new(SOL_USDC_POOL_VAULT_DEST, false),   // vault_dest
        AccountMeta::new(trader_output, false),              // user_dest
        AccountMeta::new_readonly(FEE_CONFIG, false),        // fee_config_pda
        AccountMeta::new(FEE_RECIPIENT_SOL_ATA, false),      // fee_recipient_ata
        AccountMeta::new_readonly(SYSVAR_CLOCK_ID, false),   // clock
        AccountMeta::new(SOL_USDC_POOL_CURVE_UPDATES, false), // curve_updates
        AccountMeta::new_readonly(SOL_USDC_POOL_SPREAD_CONFIG, false), // spread_config
        AccountMeta::new_readonly(SYSVAR_INSTRUCTIONS_ID, false), // sysvar_instructions
    ];

    // is_x, timestamp expiration (default is current timestamp + 3600)
    let mut extra_data = vec![1];
    let timestamp = svm.get_sysvar::<Clock>().unix_timestamp;
    let expiration = timestamp + 3600;
    extra_data.extend_from_slice(&expiration.to_le_bytes());

    let instruction = build_swap_instruction(accounts, in_amount, min_out_amount, &extra_data);

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_wsol = get_token_balance(&svm, &trader_input);
            let final_usdc = get_token_balance(&svm, &trader_output);

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
                "Hadron swap successful! WSOL: {} -> {}, USDC: {} -> {}",
                initial_wsol, final_wsol, initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("Hadron swap CPI failed: {}", e);
        }
    }
}
