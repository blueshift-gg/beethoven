use {
    crate::helper::*,
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

// Derived from pool_state.json (CPMM PoolState offsets)
const POOL_STATE: Address = address!("Hjm1F98vgVdN7Y9L46KLqcZZWyTKS9tj9ybYKJcXnSng");
const AMM_CONFIG: Address = address!("68yDnv1sDzU3L2cek5kNEszKFPaK9yUJaC4ghV5LAXW6");
const TOKEN_0_VAULT: Address = address!("61Xc2EKCL6SnqyMjWujTmcsFvBbRh5717MwrD3EMwaaw");
const TOKEN_1_VAULT: Address = address!("7Aihr5kSURKgUtvnAEAkQyZzfJ7vq5WiYLeCd4o78xLW");
const OBSERVATION_KEY: Address = address!("6qFaCY5Ws9bcagcvJoZnUpH9qLv8MkKWmUszvhX9QW3V");

// vault_and_lp_mint_auth_seed PDA — owns the vault token accounts
const AUTHORITY: Address = address!("ALfS4oPB5684XwTvCjWw7XddFfmyTNdcY7xHxbh2Ui8s");

#[test]
fn test_gamma_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    // Advance clock past pool's open_time (1727715087 = Sep 30, 2024)
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_740_000_000; // Feb 2025
    svm.set_sysvar::<Clock>(&clock);

    // Load beethoven-test program (our program that does CPI)
    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Gamma program
    load_program(
        &mut svm,
        GAMMA_PROGRAM_ID,
        &format!("{}/gamma_program.so", gamma_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/pool_state.json", gamma_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/amm_config.json", gamma_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/observation_key.json", gamma_fixtures_dir()),
    );
    load_and_set_json_fixture(&mut svm, &format!("{}/vault_0.json", gamma_fixtures_dir()));
    load_and_set_json_fixture(&mut svm, &format!("{}/vault_1.json", gamma_fixtures_dir()));
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/wsol_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );

    // Create trader token accounts with initial balances
    // Selling SOL (input=WSOL) for USDC (output)
    let initial_wsol = 1_000_000_000u64; // 1 SOL
    let initial_usdc = 0u64;
    let trader_input = create_token_account(&mut svm, &payer.pubkey(), &WSOL_MINT, initial_wsol);
    let trader_output = create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc);

    // Verify initial balances
    assert_eq!(get_token_balance(&svm, &trader_input), initial_wsol);
    assert_eq!(get_token_balance(&svm, &trader_output), initial_usdc);

    // Build swap instruction: sell 0.001 SOL for USDC
    // Using small amount to stay within pool liquidity
    let in_amount = 1_000_000u64; // 0.001 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    // Gamma accounts layout (14 accounts):
    // [0] gamma_program, [1] payer, [2] authority, [3] amm_config,
    // [4] pool_state, [5] input_token_account, [6] output_token_account,
    // [7] input_vault, [8] output_vault, [9] input_token_program,
    // [10] output_token_program, [11] input_token_mint, [12] output_token_mint,
    // [13] observation_state
    let accounts = vec![
        AccountMeta::new_readonly(GAMMA_PROGRAM_ID, false), // gamma_program (for detection)
        AccountMeta::new_readonly(payer.pubkey(), true),    // payer
        AccountMeta::new_readonly(AUTHORITY, false),        // authority PDA
        AccountMeta::new_readonly(AMM_CONFIG, false),       // amm_config
        AccountMeta::new(POOL_STATE, false),                // pool_state
        AccountMeta::new(trader_input, false),              // input_token_account
        AccountMeta::new(trader_output, false),             // output_token_account
        AccountMeta::new(TOKEN_0_VAULT, false),             // input_vault (SOL vault)
        AccountMeta::new(TOKEN_1_VAULT, false),             // output_vault (USDC vault)
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false), // input_token_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false), // output_token_program
        AccountMeta::new_readonly(WSOL_MINT, false),        // input_token_mint
        AccountMeta::new_readonly(USDC_MINT, false),        // output_token_mint
        AccountMeta::new(OBSERVATION_KEY, false),           // observation_state
    ];

    // Gamma has no extra data
    let extra_data: &[u8] = &[];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::Gamma,
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
                "Gamma swap successful! WSOL: {} -> {}, USDC: {} -> {}",
                initial_wsol, final_wsol, initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("Gamma swap CPI failed: {}", e);
        }
    }
}
