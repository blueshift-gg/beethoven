use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, saros_dlmm_fixtures_dir,
        send_transaction, setup_svm, MEMO_PROGRAM_ID, SAROS_DLMM_PROGRAM_ID, TEST_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USD1_MINT: Address = address!("USD1ttGY1N17NEEHLmELoaybftRBUSErhqYiQzvEmuB");
const PAIR: Address = address!("8yrUdy1XufCuupHgbpptcer1npNkQDVh95sLnc67CfR2");
const BIN_ARRAY_LOWER: Address = address!("4PvQBrRimmeHiKQs2op4upDm3nEPDbWbUrnoUrkSvoDD");
const BIN_ARRAY_UPPER: Address = address!("AxPaxEzyyk1MFg5Xhk8u2PUvKR5ppYRBA2rjaT8NYbQc");
const TOKEN_VAULT_X: Address = address!("GJDXcwHfdJ1AbRZ1RoR9CLPXkiUwvf7zdWEnMnY1Cibp");
const TOKEN_VAULT_Y: Address = address!("A1rGSThS9uSgkb5SiDa5Fo479Lg1r4vFv3UcWPCMh9hm");
const HOOK: Address = address!("FBsXR7JfRyMsyoSpcGDaoax7XbJZS2Cj3aaoSAm8L7uH");
const SAROS_MDMA_HOOKS_PROGRAM_ID: Address =
    address!("mdmavMvJpF4ZcLJNg6VSjuKVMiBo5uKwERTg1ZB9yUH");
const EVENT_AUTHORITY: Address = address!("AQjz6RZK93SLjxfDGKL9nCYQNSjEbQSdETxwR63jXV8m");
const HOOK_BIN_ARRAY_0: Address = address!("4JZ5GA1xPP5o1FSe7H8kzSK5dZS8LX1yDf2zPVuueTho");
const HOOK_BIN_ARRAY_1: Address = address!("HHxjVEz8KW79C1CAchvUyrbJJaf6ydwBjTz5DGi1HQfM");

#[test]
fn test_saros_dlmm_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Saros DLMM program
    load_program(
        &mut svm,
        SAROS_DLMM_PROGRAM_ID,
        &format!("{}/saros_dlmm.so", saros_dlmm_fixtures_dir()),
    );
    load_program(
        &mut svm,
        SAROS_MDMA_HOOKS_PROGRAM_ID,
        &format!("{}/saros_mdma_hooks.so", saros_dlmm_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usd1_mint.json", saros_dlmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_usd1_pair.json", saros_dlmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/pair_bin_array_lower.json", saros_dlmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/pair_bin_array_upper.json", saros_dlmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usd1_vault.json", saros_dlmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_vault.json", saros_dlmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/hook.json", saros_dlmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/hook_bin_array_0.json", saros_dlmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/hook_bin_array_1.json", saros_dlmm_fixtures_dir()),
    );

    // Jump ahead to pair dynamic_fee_parameters time_last_updated (1,775_394_376) + 1
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_775_394_376 + 1;
    svm.set_sysvar::<Clock>(&clock);

    // Create trader token accounts with initial balances
    // Selling USDC (input) for USD1 (output)
    let initial_usdc = 10_000_000u64;
    let initial_usd1 = 0u64;
    let trader_usdc =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);
    let trader_usd1 =
        create_token_account(&mut svm, &payer.pubkey(), &USD1_MINT, initial_usd1, false);

    // Build swap instruction: sell 0.00001 USDC for USD1
    let in_amount = 100u64; // 0.00001 USDC
    let min_out_amount = 1u64; // Very loose slippage for test

    // Saros DLMM accounts layout (18 + 2 remaining accounts)
    let accounts = vec![
        AccountMeta::new_readonly(SAROS_DLMM_PROGRAM_ID, false),
        AccountMeta::new(PAIR, false),
        AccountMeta::new_readonly(USD1_MINT, false),
        AccountMeta::new_readonly(USDC_MINT, false),
        AccountMeta::new(BIN_ARRAY_LOWER, false),
        AccountMeta::new(BIN_ARRAY_UPPER, false),
        AccountMeta::new(TOKEN_VAULT_X, false),
        AccountMeta::new(TOKEN_VAULT_Y, false),
        AccountMeta::new(trader_usd1, false),
        AccountMeta::new(trader_usdc, false),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(MEMO_PROGRAM_ID, false),
        AccountMeta::new(HOOK, false),
        AccountMeta::new_readonly(SAROS_MDMA_HOOKS_PROGRAM_ID, false),
        AccountMeta::new_readonly(EVENT_AUTHORITY, false),
        AccountMeta::new_readonly(SAROS_DLMM_PROGRAM_ID, false),
        AccountMeta::new(HOOK_BIN_ARRAY_0, false),
        AccountMeta::new(HOOK_BIN_ARRAY_1, false),
    ];

    // swap_for_y = false, swap_type = ExactInput
    let extra_data: &[u8] = &[0, 0];
    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::SarosDlmm,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_usdc_in = get_token_balance(&svm, &trader_usdc);
            let final_usd1_out = get_token_balance(&svm, &trader_usd1);

            assert!(
                final_usdc_in < initial_usdc,
                "USDC should have decreased: {} -> {}",
                initial_usdc,
                final_usdc_in
            );
            assert!(
                final_usd1_out > initial_usd1,
                "USD1 should have increased: {} -> {}",
                initial_usd1,
                final_usd1_out
            );

            println!(
                "Saros DLMM swap OK: USDC {} -> {}, USD1 {} -> {}",
                initial_usdc, final_usdc_in, initial_usd1, final_usd1_out
            );
        }
        Err(e) => {
            panic!("Saros AMM swap CPI failed: {}", e);
        }
    }
}
