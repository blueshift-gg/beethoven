use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, perena_fixtures_dir,
        send_transaction, setup_svm, PERENA_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_2022_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDG_MINT: Address = address!("2u1tszSeqZ3qBWF3uNGPFc8TzMk2tdiwknnRMWGWjGWH");
const USDC_USDG_STABLE_POOL: Address = address!("5M7McNWX7yBBGrZGB6XhmHYhFwWwwB2ckrA1HEpkf3SA");
const POOL_USDC_VAULT: Address = address!("8XTxpDy7BjJkaoZxTiEzCwdwMad6RGBN6oXyfH2yRL7n");
const POOL_USDG_VAULT: Address = address!("BcjVG5To1pi3fHMpFoFdurcFwAoYJFzEtKP9ZTfqdjzT");
const NUMERAIRE_CONFIG: Address = address!("FS159v4b2jo3fjGBaUFmDzgx7k616XhpKhMwX2Q3HeeD");

#[test]
fn test_perena_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Perena program
    load_program(
        &mut svm,
        PERENA_PROGRAM_ID,
        &format!("{}/numeraire.so", perena_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdg_mint.json", perena_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_usdg_stable_pool.json", perena_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/numeraire_config.json", perena_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/stable_pool_usdc_vault.json", perena_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/stable_pool_usdg_vault.json", perena_fixtures_dir()),
    );

    // Create trader token accounts with initial balances
    // Selling USDC for USDG (output)
    let initial_usdc = 1_000_000_000u64;
    let initial_usdg = 0u64;
    let trader_input =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);
    let trader_output =
        create_token_account(&mut svm, &payer.pubkey(), &USDG_MINT, initial_usdg, true);

    // Build swap instruction: sell 10 USDC for USDG
    let in_amount = 10_000_000u64;
    let min_out_amount = 1u64; // Very loose slippage for test

    // Perena accounts layout (12 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(PERENA_PROGRAM_ID, false), // perena program
        AccountMeta::new(USDC_USDG_STABLE_POOL, false),      // pool
        AccountMeta::new(USDC_MINT, false),                  // in mint
        AccountMeta::new(USDG_MINT, false),                  // out mint
        AccountMeta::new(trader_input, false),               // in trader
        AccountMeta::new(trader_output, false),              // out trader
        AccountMeta::new(POOL_USDC_VAULT, false),            // in vault
        AccountMeta::new(POOL_USDG_VAULT, false),            // out vault
        AccountMeta::new_readonly(NUMERAIRE_CONFIG, false),  // numeraire config
        AccountMeta::new(payer.pubkey(), true),              // payer
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),  // token program
        AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false), // token 2022 program
    ];

    // in index = 0, out index = 1
    let extra_data: &[u8] = &[0u8, 1u8];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::Perena,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_usdc = get_token_balance(&svm, &trader_input);
            let final_usdg = get_token_balance(&svm, &trader_output);

            assert!(
                final_usdc < initial_usdc,
                "USDC should have decreased: {} -> {}",
                initial_usdc,
                final_usdc
            );
            assert!(
                final_usdg > initial_usdg,
                "USDG should have increased: {} -> {}",
                initial_usdg,
                final_usdg
            );
        }
        Err(e) => panic!("Perena swap CPI failed: {}", e),
    }
}
