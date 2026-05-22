use {
    crate::helper::*,
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const MARKET: Address = address!("ENhU8LsaR7vDD2G1CsWcsuSGNrih9Cv5WZEk7q9kPapQ");
const BASE_VAULT: Address = address!("AKjfJDv4ywdpCDrj7AURuNkGA3696GTVFgrMwk4TjkKs");
const QUOTE_VAULT: Address = address!("FN9K6rTdWtRDUPmLTN2FnGvLZpHVNRN2MeRghKknSGDs");
const GLOBAL: Address = address!("7mR36vj6pvg1U1cRatvUbLG57yqsd1ojLbrgxb6azaQ1");
const GLOBAL_VAULT: Address = address!("E1mBVQyt7BHK8SaBSfME7usYxx94T4DtHEjbUpEBhZx");

#[test]
fn test_manifest_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Manifest program
    load_program(
        &mut svm,
        MANIFEST_PROGRAM_ID,
        &format!("{}/manifest_program.so", manifest_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/manifest_usdc_sol_market.json", manifest_fixtures_dir()),
    );
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
        &format!(
            "{}/manifest_sol_usdc_base_vault.json",
            manifest_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/manifest_sol_usdc_quote_vault.json",
            manifest_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/manifest_global.json", manifest_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/manifest_global_vault.json", manifest_fixtures_dir()),
    );

    // Create trader token accounts with initial balances
    // Selling SOL (input=WSOL) for USDC (output)
    let initial_wsol = 1_000_000_000u64; // 1 SOL
    let initial_usdc = 0u64;
    let trader_base =
        create_token_account(&mut svm, &payer.pubkey(), &WSOL_MINT, initial_wsol, false);
    let trader_quote =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);

    // Build swap instruction: sell 0.001 SOL for USDC
    let in_amount = 1_000_000u64; // 0.001 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    // Manifest accounts layout (14 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(MANIFEST_PROGRAM_ID, false), // manifest_program
        AccountMeta::new(payer.pubkey(), true),                // payer
        AccountMeta::new(payer.pubkey(), true),                // owner
        AccountMeta::new(MARKET, false),                       // market
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),   // system_program
        AccountMeta::new(trader_base, false),                  // trader_base
        AccountMeta::new(trader_quote, false),                 // trader_quote
        AccountMeta::new(BASE_VAULT, false),                   // base_vault
        AccountMeta::new(QUOTE_VAULT, false),                  // quote_vault
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),    // token_program_base
        AccountMeta::new_readonly(WSOL_MINT, false),           // base_mint
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),    // token_program_quote
        AccountMeta::new_readonly(USDC_MINT, false),           // quote_mint
        AccountMeta::new(GLOBAL, false),                       // global
        AccountMeta::new(GLOBAL_VAULT, false),                 // global_vault
    ];

    // is_base_in = true (selling base/SOL), is_exact_in = true (exact input amount)
    let extra_data = [1u8, 1u8];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::Manifest,
        &extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_wsol = get_token_balance(&svm, &trader_base);
            let final_usdc = get_token_balance(&svm, &trader_quote);

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
        }
        Err(e) => {
            panic!("Swap CPI failed: {}", e);
        }
    }
}
