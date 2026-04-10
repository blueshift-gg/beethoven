use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, meteora_damm_fixtures_dir,
        send_transaction, setup_svm, METEORA_DAMM_PROGRAM_ID, METEORA_DYNAMIC_VAULT_PROGRAM_ID,
        TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDT_MINT: Address = address!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
const POOL: Address = address!("32D4zRxNc1EssbJieVHfPhZM3rH6CzfUPrWUuWxD9prG");
const A_VAULT: Address = address!("3ESUFCnRNgZ7Mn2mPPUMmXYaKU8jpnV9VtA17M7t2mHQ");
const B_VAULT: Address = address!("5XCP3oD3JAuQyDpfBFFVUxsBxNjPQojpKuL4aVhHsDok");
const A_TOKEN_VAULT: Address = address!("C2QoQ111jGHEy5918XkNXQro7gGwC9PKLXd1LqBiYNwA");
const B_TOKEN_VAULT: Address = address!("DQjGWHN9ERn1zSMpWLNvSpTFUSfnxbanBt9A7xyU2bVE");
const A_VAULT_LP_MINT: Address = address!("3RpEekjLE5cdcG15YcXJUpxSepemvq2FpmMcgo342BwC");
const B_VAULT_LP_MINT: Address = address!("EZun6G5514FeqYtUv26cBHWLqXjAEdjGuoX6ThBpBtKj");
const A_VAULT_LP: Address = address!("24NYE3hHQyUTrHUT4n1CcVrMP9Xy3ULuT1Uurw1HDeck");
const B_VAULT_LP: Address = address!("Hv5ogVb2BZCF3ET2KnaEYj2seKHN5ffGDazm6BGt5DD9");
const PROTOCOL_TOKEN_FEE: Address = address!("4Qjrnzp5jXPSBhyv495ApB1SdDbXdZ5Pc9ZSiabf9NmJ");

#[test]
fn test_meteora_damm_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Meteora DAMM v2 program
    load_program(
        &mut svm,
        METEORA_DAMM_PROGRAM_ID,
        &format!("{}/meteora_damm.so", meteora_damm_fixtures_dir()),
    );
    load_program(
        &mut svm,
        METEORA_DYNAMIC_VAULT_PROGRAM_ID,
        &format!("{}/meteora_vault.so", meteora_damm_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdt_mint.json", meteora_damm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_usdt_pool.json", meteora_damm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/a_vault.json", meteora_damm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/b_vault.json", meteora_damm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/a_token_vault.json", meteora_damm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/b_token_vault.json", meteora_damm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/a_vault_lp_mint.json", meteora_damm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/b_vault_lp_mint.json", meteora_damm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/a_vault_lp.json", meteora_damm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/b_vault_lp.json", meteora_damm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/protocol_token_fee.json", meteora_damm_fixtures_dir()),
    );

    // Jump ahead by pool curve_type last_amp_updated_timestamp (1_775_831_595) + 1
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_775_831_595 + 1;
    svm.set_sysvar::<Clock>(&clock);

    // Create payer token accounts with initial balances
    // Selling USDC for USDT
    let initial_usdc = 100_000_000u64;
    let initial_usdt = 0u64;
    let trader_input =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);
    let trader_output =
        create_token_account(&mut svm, &payer.pubkey(), &USDT_MINT, initial_usdt, false);

    // Build swap instruction: sell 10 USDC for USDT
    let in_amount = 10_000_000u64; // 10 USDC
    let min_out_amount = 1u64; // Very loose slippage for test

    // Meteora DAMM accounts layout (16 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(METEORA_DAMM_PROGRAM_ID, false), // damm_program
        AccountMeta::new(POOL, false),                             // pool
        AccountMeta::new(trader_input, false),                     // input_token_account (USDC)
        AccountMeta::new(trader_output, false),                    // output_token_account (USDT)
        AccountMeta::new(A_VAULT, false),                          // token_a_vault
        AccountMeta::new(B_VAULT, false),                          // token_b_vault
        AccountMeta::new(A_TOKEN_VAULT, false),                    // a_token_vault
        AccountMeta::new(B_TOKEN_VAULT, false),                    // b_token_vault
        AccountMeta::new(A_VAULT_LP_MINT, false),                  // a_vault_lp_mint
        AccountMeta::new(B_VAULT_LP_MINT, false),                  // b_vault_lp_mint
        AccountMeta::new(A_VAULT_LP, false),                       // a_vault_lp
        AccountMeta::new(B_VAULT_LP, false),                       // b_vault_lp
        AccountMeta::new(PROTOCOL_TOKEN_FEE, false),               // protocol_token_fee
        AccountMeta::new(payer.pubkey(), true),                    // payer
        AccountMeta::new_readonly(METEORA_DYNAMIC_VAULT_PROGRAM_ID, false), // vault_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),        // token_program
    ];

    // Meteora DAMM has no extra data
    let extra_data: &[u8] = &[];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::MeteoraDamm,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_usdc = get_token_balance(&svm, &trader_input);
            let final_usdt = get_token_balance(&svm, &trader_output);

            assert!(
                final_usdc < initial_usdc,
                "USDC should have decreased: {} -> {}",
                initial_usdc,
                final_usdc
            );
            assert!(
                final_usdt > initial_usdt,
                "USDT should have increased: {} -> {}",
                initial_usdt,
                final_usdt
            );
        }
        Err(e) => panic!("Meteora DAMM swap CPI failed: {}", e),
    }
}
