use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, raydium_cpmm_fixtures_dir,
        send_transaction, setup_svm, RAYDIUM_CPMM_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const POOL_STATE: Address = address!("7JuwJuNU88gurFnyWeiyGKbFmExMWcmRZntn9imEzdny");
const AMM_CONFIG: Address = address!("D4FPEruKEHrG5TenZ2mpDGEfu1iUvTiqBxvpU8HLBvC2");
const OBSERVATION_STATE: Address = address!("4MYrPgjgFceyhtwhG1ZX8UVb4wn1aQB5wzMimtFqg7U8");
const VAULT_AUTHORITY: Address = address!("GpMZbSM2GgvTKHJirzeGfMFoaZ8UR2X7F4v8vHTvxFbL");
const INPUT_VAULT: Address = address!("7VLUXrnSSDo9BfCa4NWaQs68g7ddDY1sdXBKW6Xswj9Y");
const OUTPUT_VAULT: Address = address!("3rzbbW5Q8MA7sCaowf28hNgACNPecdS2zceWy7Ptzua9");

#[test]
fn test_raydium_cpmm_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    // Past `pool_state.open_time` so Raydium `swap_base_input` passes `NotApproved` gate
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_740_000_000;
    svm.set_sysvar::<Clock>(&clock);

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    load_program(
        &mut svm,
        RAYDIUM_CPMM_PROGRAM_ID,
        &format!("{}/raydium_cpmm.so", raydium_cpmm_fixtures_dir()),
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
        &format!("{}/sol_usdc_pool_state.json", raydium_cpmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/amm_config.json", raydium_cpmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/sol_usdc_observation_state.json",
            raydium_cpmm_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_authority.json", raydium_cpmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/sol_vault.json", raydium_cpmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_vault.json", raydium_cpmm_fixtures_dir()),
    );

    // Create trader token accounts with initial balances
    // Selling SOL (input=WSOL) for USDC (output)
    let initial_wsol = 1_000_000_000u64;
    let initial_usdc = 0u64;
    let trader_input = create_token_account(&mut svm, &payer.pubkey(), &WSOL_MINT, initial_wsol);
    let trader_output = create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc);

    let in_amount = 1_000_000u64;
    let min_out_amount = 1u64;

    // Raydium CPMM accounts layout (14 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(RAYDIUM_CPMM_PROGRAM_ID, false), // raydium_cpmm_program
        AccountMeta::new_readonly(payer.pubkey(), true),           // payer
        AccountMeta::new_readonly(VAULT_AUTHORITY, false),         // authority PDA
        AccountMeta::new_readonly(AMM_CONFIG, false),              // amm_config
        AccountMeta::new(POOL_STATE, false),                       // pool_state
        AccountMeta::new(trader_input, false),                     // input_token_account
        AccountMeta::new(trader_output, false),                    // output_token_account
        AccountMeta::new(INPUT_VAULT, false),                      // input_vault
        AccountMeta::new(OUTPUT_VAULT, false),                     // output_vault
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),        // input_token_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),        // output_token_program
        AccountMeta::new_readonly(WSOL_MINT, false),               // input_token_mint
        AccountMeta::new_readonly(USDC_MINT, false),               // output_token_mint
        AccountMeta::new(OBSERVATION_STATE, false),                // observation_state
    ];

    // Raydium CPMM swap_base_input has no extra data
    let extra_data: &[u8] = &[];

    let instruction = build_swap_instruction(accounts, in_amount, min_out_amount, extra_data);

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
                "Raydium CPMM swap successful! WSOL: {} -> {}, USDC: {} -> {}",
                initial_wsol, final_wsol, initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("Raydium CPMM swap CPI failed: {}", e);
        }
    }
}
