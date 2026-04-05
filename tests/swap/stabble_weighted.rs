//! CPI integration test for Stabble weighted swap.
//! Expects `stabble_weighted.so`, `vault_program.so`, and account JSON under
//! `fixtures/swap/stabble-weighted`, plus mint JSON under `fixtures/common`.

use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, send_transaction, setup_svm,
        stabble_weighted_fixtures_dir, STABBLE_WEIGHTED_PROGRAM_ID, TEST_PROGRAM_ID,
        TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const POOL: Address = address!("HZzitoVgr9PWWUr2mchGRRp2RkJVUseeVSFgzbuHmMeC");
const SOL_VAULT: Address = address!("HoAHDQss5qzYkoKPXtRJRHCQrUWxcHvs4vmZ8QsN4nSq");
const USDC_VAULT: Address = address!("2PkFYJpyum86qkAM46hZ7bNvUGq157RoaPKFrgTAWLub");
const VAULT_STATE: Address = address!("w8edo9a9TDw52c1rBmVbP6dNakaAuFiPjDd52ZJwwVi");
const VAULT_PROGRAM: Address = address!("vo1tWgqZMjG61Z2T9qUaMYKqZ75CYzMuaZ2LZP1n7HV");
const BENEFICIARY_TOKEN_OUT: Address = address!("ArLSJrSstZ3kjeZDyMAgjfjad1qdRZHHYaCQTQeAcTpa");
const WITHDRAW_AUTHORITY: Address = address!("BXj5a4J5YDByKzd3Y7NU59QDrjy1KcH1dCbftsxJGmna");
const VAULT_AUTHORITY: Address = address!("7HkzG4LYyCJSrD3gopPQv3VVzQQKbHBZcm9fbjj5fuaH");

#[test]
fn test_stabble_weighted_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    load_program(
        &mut svm,
        STABBLE_WEIGHTED_PROGRAM_ID,
        &format!("{}/stabble_weighted.so", stabble_weighted_fixtures_dir()),
    );
    load_program(
        &mut svm,
        VAULT_PROGRAM,
        &format!("{}/vault_program.so", stabble_weighted_fixtures_dir()),
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
        &format!("{}/sol_usdc_pool.json", stabble_weighted_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/sol_vault.json", stabble_weighted_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_vault.json", stabble_weighted_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_fee_account.json", stabble_weighted_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_authority.json", stabble_weighted_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_state.json", stabble_weighted_fixtures_dir()),
    );

    // Create trader token accounts with initial balances
    // Selling SOL (input=WSOL) for USDC (output)
    let initial_wsol = 1_000_000_000u64; // 1 SOL
    let initial_usdc = 0u64;
    let trader_input =
        create_token_account(&mut svm, &payer.pubkey(), &WSOL_MINT, initial_wsol, false);
    let trader_output =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);

    // Build swap instruction: sell 0.001 SOL for USDC
    let in_amount = 1_000_000u64; // 0.001 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    // Stabble Weighted accounts layout (16 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(STABBLE_WEIGHTED_PROGRAM_ID, false), // stabble program
        AccountMeta::new(payer.pubkey(), true),                        // user
        AccountMeta::new_readonly(WSOL_MINT, false),                   // mint in
        AccountMeta::new_readonly(USDC_MINT, false),                   // mint out
        AccountMeta::new(trader_input, false),                         // user token in
        AccountMeta::new(trader_output, false),                        // user token out
        AccountMeta::new(SOL_VAULT, false),                            // vault token in
        AccountMeta::new(USDC_VAULT, false),                           // vault token out
        AccountMeta::new(BENEFICIARY_TOKEN_OUT, false),                // beneficiary token out
        AccountMeta::new(POOL, false),                                 // pool
        AccountMeta::new_readonly(WITHDRAW_AUTHORITY, false),          // withdraw authority
        AccountMeta::new_readonly(VAULT_STATE, false),                 // vault
        AccountMeta::new_readonly(VAULT_AUTHORITY, false),             // vault authority
        AccountMeta::new_readonly(VAULT_PROGRAM, false),               // vault program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),            // token program
        AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),       // token 2022 program
    ];

    // Stabble Weighted swap has no extra data
    let extra_data: &[u8] = &[];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::StabbleWeighted,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_wsol = get_token_balance(&svm, &trader_input);
            let final_usdc = get_token_balance(&svm, &trader_output);

            assert!(
                final_wsol < initial_wsol,
                "SOL should have decreased: {} -> {}",
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
            panic!("Stabble Weighted swap CPI failed: {}", e);
        }
    }
}
