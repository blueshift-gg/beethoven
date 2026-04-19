use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, send_transaction, setup_svm,
        tessera_v_fixtures_dir, TESSERA_V_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
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
const GLOBAL_STATE: Address = address!("8ekCy2jHHUbW2yeNGFWYJT9Hm9FW7SvZcZK66dSZCDiF");
const MARKET: Address = address!("FLckHLGMJy5gEoXWwcE68Nprde1D4araK4TGLw4pQq2n");
const VAULT_A: Address = address!("5pVN5XZB8cYBjNLFrsBCPWkCQBan5K5Mq2dWGzwPgGJV");
const VAULT_B: Address = address!("9t4P5wMwfFkyn92Z7hf463qYKEZf8ERVZsGBEPNp8uJx");

#[test]
#[ignore = "would throw with custom error 0x2, likely due to instruction not called from a whitelisted router"]
fn test_tessera_v_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Tessera V program
    load_program(
        &mut svm,
        TESSERA_V_PROGRAM_ID,
        &format!("{}/tessera_v.so", tessera_v_fixtures_dir()),
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
        &format!("{}/global_state.json", tessera_v_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/market.json", tessera_v_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_a.json", tessera_v_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_b.json", tessera_v_fixtures_dir()),
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

    // Tessera V accounts layout (13 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(TESSERA_V_PROGRAM_ID, false),
        AccountMeta::new_readonly(GLOBAL_STATE, false),
        AccountMeta::new(MARKET, false),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(VAULT_A, false),
        AccountMeta::new(VAULT_B, false),
        AccountMeta::new(trader_input, false),
        AccountMeta::new(trader_output, false),
        AccountMeta::new_readonly(WSOL_MINT, false),
        AccountMeta::new_readonly(USDC_MINT, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(SYSVAR_INSTRUCTIONS_ID, false),
    ];

    // is_a_to_b = true
    let extra_data: &[u8] = &[1_u8];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::TesseraV,
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
                "Tessera V swap successful! WSOL: {} -> {}, USDC: {} -> {}",
                initial_wsol, final_wsol, initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("Tessera V swap CPI failed: {}", e);
        }
    }
}
