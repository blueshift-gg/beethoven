use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, send_transaction, setup_svm,
        zerofi_fixtures_dir, TEST_PROGRAM_ID, TOKEN_PROGRAM_ID, ZEROFI_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    beethoven_client::SYSVAR_INSTRUCTIONS_ID,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const MARKET: Address = address!("AWguet57BQuPftMuiV6vY89TQCQXxyTvQQ4QmoS7K2Mt");
const CFG_SOL: Address = address!("7RHJ2WfexqUxy7SXfbNZRZDgZi3D9jtMAQp9VhfzpU8T");
const SOL_VAULT: Address = address!("ERP5RTV6cWmoGrv7r9W2V5pbgDFSepc4j97qNnx1Jris");
const CFG_USDC: Address = address!("Ef7zPqj4NuZHwaTczUTY9oRbxXrfZseUcKcqPaidCZ5W");
const USDC_VAULT: Address = address!("7wYJVD8iXmMQjND1fwi1hPr68QwruVVtirbotyJZXaVH");

#[test]
#[ignore = "would throw with MissingRequiredSignature error, likely due to instruction not called from a whitelisted router"]
fn test_zerofi_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load ZeroFi program
    load_program(
        &mut svm,
        ZEROFI_PROGRAM_ID,
        &format!("{}/zerofi.so", zerofi_fixtures_dir()),
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
    load_and_set_json_fixture(&mut svm, &format!("{}/market.json", zerofi_fixtures_dir()));
    load_and_set_json_fixture(&mut svm, &format!("{}/cfg_sol.json", zerofi_fixtures_dir()));
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/sol_vault.json", zerofi_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/cfg_usdc.json", zerofi_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_vault.json", zerofi_fixtures_dir()),
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

    // ZeroFi accounts layout (10 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(ZEROFI_PROGRAM_ID, false), // zerofi_program
        AccountMeta::new(MARKET, false),                     // market
        AccountMeta::new(CFG_SOL, false),                    // cfg_in
        AccountMeta::new(SOL_VAULT, false),                  // ta_in
        AccountMeta::new(CFG_USDC, false),                   // cfg_out
        AccountMeta::new(USDC_VAULT, false),                 // ta_out
        AccountMeta::new(trader_input, false),               // usr_ta_in
        AccountMeta::new(trader_output, false),              // usr_ta_out
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),  // token_program
        AccountMeta::new_readonly(SYSVAR_INSTRUCTIONS_ID, false), // sysvar_instructions
    ];

    // ZeroFi swap has no extra data
    let extra_data: &[u8] = &[];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::Zerofi,
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
                "Omnipair swap successful! WSOL: {} -> {}, USDC: {} -> {}",
                initial_wsol, final_wsol, initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("Omnipair swap CPI failed: {}", e);
        }
    }
}
