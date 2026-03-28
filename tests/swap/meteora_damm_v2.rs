use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, meteora_damm_v2_fixtures_dir,
        send_transaction, setup_svm, METEORA_DAMM_V2_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const SOL_USDC_POOL: Address = address!("CGPxT5d1uf9a8cKVJuZaJAU76t2EfLGbTmRbfvLLZp5j");
const POOL_AUTHORITY: Address = address!("HLnpSz9h2S4hiLQ43rnSD9XkcUThA7B8hQMKmDaiTLcC");
const EVENT_AUTHORITY: Address = address!("3rmHSu74h1ZcmAisVcWerTCiRDQbUrBKmcwptYGjHfet");
const SOL_USDC_POOL_TOKEN_A_VAULT: Address =
    address!("E3r3rs6C9bZbokaPiMEwmvPUtcd6CE2nuK8RSMQdE64E");
const SOL_USDC_POOL_TOKEN_B_VAULT: Address =
    address!("HK2HggD4Eg1tAyr3gnRvNG32Z8v7s1NQGjH77b14qvsx");

#[test]
fn test_meteora_damm_v2_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Meteora DAMM v2 program
    load_program(
        &mut svm,
        METEORA_DAMM_V2_PROGRAM_ID,
        &format!("{}/meteora_damm_v2.so", meteora_damm_v2_fixtures_dir()),
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
        &format!("{}/pool_authority.json", meteora_damm_v2_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/sol_usdc_pool.json", meteora_damm_v2_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/sol_usdc_pool_token_a_vault.json",
            meteora_damm_v2_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/sol_usdc_pool_token_b_vault.json",
            meteora_damm_v2_fixtures_dir()
        ),
    );

    // Jump ahead by pool activation time (1_747_446_361) + 1 hour (3600)
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_747_446_361 + 3600;
    svm.set_sysvar::<Clock>(&clock);

    // Create payer token accounts with initial balances
    // Selling WSOL (input=WSOL) for USDC (output)
    let initial_wsol = 1_000_000_000u64;
    let initial_usdc = 0u64;
    let trader_input = create_token_account(&mut svm, &payer.pubkey(), &WSOL_MINT, initial_wsol);
    let trader_output = create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc);

    // Build swap instruction: sell 0.001 SOL for USDC
    let in_amount = 1_000_000u64; // 0.001 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    // Meteora DAMM v2 accounts layout (15 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(METEORA_DAMM_V2_PROGRAM_ID, false), // cp_amm_program
        AccountMeta::new_readonly(POOL_AUTHORITY, false),             // pool_authority
        AccountMeta::new(SOL_USDC_POOL, false),                       // pool
        AccountMeta::new(trader_input, false),                        // input_token_account (WSOL)
        AccountMeta::new(trader_output, false),                       // output_token_account (USDC)
        AccountMeta::new(SOL_USDC_POOL_TOKEN_A_VAULT, false),         // token_a_vault
        AccountMeta::new(SOL_USDC_POOL_TOKEN_B_VAULT, false),         // token_b_vault
        AccountMeta::new_readonly(WSOL_MINT, false),                  // token_a_mint
        AccountMeta::new_readonly(USDC_MINT, false),                  // token_b_mint
        AccountMeta::new(payer.pubkey(), true),                       // payer
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),           // token_a_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),           // token_b_program
        AccountMeta::new_readonly(METEORA_DAMM_V2_PROGRAM_ID, false), // referral_token_account (defaults to program)
        AccountMeta::new_readonly(EVENT_AUTHORITY, false),            // event_authority
        AccountMeta::new_readonly(METEORA_DAMM_V2_PROGRAM_ID, false), // program
    ];

    // SwapType::ExactIn
    let extra_data: &[u8] = &[0u8];

    let instruction = build_swap_instruction(accounts, in_amount, min_out_amount, extra_data);

    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_cu) => {
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
        }
        Err(e) => panic!("Meteora DAMM v2 swap CPI failed: {}", e),
    }
}
