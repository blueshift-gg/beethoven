use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, pancake_fixtures_dir,
        send_transaction, setup_svm, MEMO_PROGRAM_ID, PANCAKE_PROGRAM_ID, TEST_PROGRAM_ID,
        TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

const AMM_CONFIG: Address = address!("2zmg7259ahZkrSn6M3PEM7eFvEfBU8obgfVHT3AL9Qwu");
const SOL_USDC_POOL: Address = address!("4QU2NpRaqmKMvPSwVKQDeW4V6JFEKJdkzbzdauumD9qN");
const SOL_VAULT: Address = address!("8JL1ZnyMvd48AqF9YTCE4UnEV8dDydU2cYQSa6bzykYP");
const USDC_VAULT: Address = address!("FVDSv2aymcXu5ubgFePqPmp24nrKiwpUFc21b2Kqh6gC");
const OBSERVATION_STATE: Address = address!("2h4rB9TSGFehKrvKzrMM4RrWqBefGV1STdpekc3cTyBy");
const TICK_ARRAY_BITMAP_EXTENSION: Address =
    address!("AP4Jafrw8jzASGrgZHEiNvYxhdapUYPAqxFiMsKLyvXu");
const TICK_ARRAY_STATE_1: Address = address!("Gy4zp3Kz5N5UKhcMyNE1ymgrncTcDd6b3FD9sw9EH1k5");
const TICK_ARRAY_STATE_2: Address = address!("3Z151sJtAMP2KGsj9oYF5UF9cnKSWaGwGaoAgjGt8hUo");
const TICK_ARRAY_STATE_3: Address = address!("EHekGmRXkTVjztDwSCnfaxi6D3QpZLXnj31XCTdat9Z3");

#[test]
fn test_pancake_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Pancake Swap program
    load_program(
        &mut svm,
        PANCAKE_PROGRAM_ID,
        &format!("{}/pancake.so", pancake_fixtures_dir()),
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
        &format!("{}/amm_config.json", pancake_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/sol_usdc_pool_state.json", pancake_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/sol_vault.json", pancake_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_vault.json", pancake_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/sol_usdc_observation_state.json", pancake_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/tick_array_bitmap_extension.json",
            pancake_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/tick_array_state_1.json", pancake_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/tick_array_state_2.json", pancake_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/tick_array_state_3.json", pancake_fixtures_dir()),
    );

    // Jump ahead by pool open time (0) + 1 second
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1;
    svm.set_sysvar::<Clock>(&clock);

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

    // Pancake Swap accounts layout (14 accounts + remaining accounts)
    let accounts = vec![
        AccountMeta::new_readonly(PANCAKE_PROGRAM_ID, false),
        AccountMeta::new_readonly(payer.pubkey(), true),
        AccountMeta::new_readonly(AMM_CONFIG, false),
        AccountMeta::new(SOL_USDC_POOL, false),
        AccountMeta::new(trader_input, false),
        AccountMeta::new(trader_output, false),
        AccountMeta::new(SOL_VAULT, false),
        AccountMeta::new(USDC_VAULT, false),
        AccountMeta::new(OBSERVATION_STATE, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
        AccountMeta::new_readonly(MEMO_PROGRAM_ID, false),
        AccountMeta::new_readonly(WSOL_MINT, false),
        AccountMeta::new_readonly(USDC_MINT, false),
        AccountMeta::new_readonly(TICK_ARRAY_BITMAP_EXTENSION, false),
        AccountMeta::new(TICK_ARRAY_STATE_1, false),
        AccountMeta::new(TICK_ARRAY_STATE_2, false),
        AccountMeta::new(TICK_ARRAY_STATE_3, false),
    ];

    let sqrt_price_limit_x64 = 0u128;
    let is_base_input = true;
    let mut extra_data = Vec::with_capacity(17);
    extra_data.extend_from_slice(&sqrt_price_limit_x64.to_le_bytes());
    extra_data.push(u8::from(is_base_input));

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::Pancake,
        &extra_data,
    );

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
                "Pancake CLMM swap successful! WSOL: {} -> {}, USDC: {} -> {}",
                initial_wsol, final_wsol, initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("Pancake CLMM swap CPI failed: {}", e);
        }
    }
}
