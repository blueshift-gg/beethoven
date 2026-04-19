use {
    crate::helper::{
        beethoven_program_path, bisonfi_fixtures_dir, build_swap_instruction, common_fixtures_dir,
        create_token_account, get_token_balance, load_and_set_json_fixture, load_program,
        send_transaction, setup_svm, BISONFI_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const MARKET: Address = address!("8FnX3xo2yYw3EUE6w3nQA4GfXGS9wpK6oj3veJpbFzLo");
const MARKET_TA_A: Address = address!("ATRsNGv2nDw7hSMfkUTBoVUDsFDwN7po7KbecyiGWNB4");
const MARKET_TA_B: Address = address!("2Y7HATmn9aJBcxCskE5V2U2epmjvkZmB51zTJBbhj4cU");
const DFLOW_LOGGER: Address = address!("8xeaWCsJYxRoudEZGJWURdfrtFhLYZz9b4iHJnW5tb3d");

#[test]
fn test_bisonfi_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load BisonFi program
    load_program(
        &mut svm,
        BISONFI_PROGRAM_ID,
        &format!("{}/bisonfi.so", bisonfi_fixtures_dir()),
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
    load_and_set_json_fixture(&mut svm, &format!("{}/market.json", bisonfi_fixtures_dir()));
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/market_ta_a.json", bisonfi_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/market_ta_b.json", bisonfi_fixtures_dir()),
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

    let accounts = vec![
        AccountMeta::new_readonly(BISONFI_PROGRAM_ID, false),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(MARKET, false),
        AccountMeta::new(MARKET_TA_A, false),
        AccountMeta::new(MARKET_TA_B, false),
        AccountMeta::new(trader_input, false),
        AccountMeta::new(trader_output, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(DFLOW_LOGGER, false),
    ];

    // b_to_a = 0, exact_out = 0
    let extra_data: &[u8] = &[0, 0];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::Bisonfi,
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
                "BisonFi swap successful! WSOL: {} -> {}, USDC: {} -> {}",
                initial_wsol, final_wsol, initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("BisonFi swap CPI failed: {}", e);
        }
    }
}
