use {
    crate::helper::{
        aldrin_fixtures_dir, beethoven_program_path, build_swap_instruction, common_fixtures_dir,
        create_token_account, get_token_balance, load_and_set_json_fixture, load_program,
        send_transaction, setup_svm, ALDRIN_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

const POOL: Address = address!("4GUniSDrCAZR3sKtLa1AWC8oyYubZeKJQ8KraQmy3Wt5");
const POOL_SIGNER: Address = address!("7Zi96LCCjSEEd5yyFik8XvAhfJsdUGzLPMprjKKrdaCA");
const POOL_MINT: Address = address!("3sbMDzGtyHAzJqzxE7DPdLMhrsxQASYoKLkHMYJPuWkp");
const SOL_VAULT: Address = address!("CLt1DtCioiByTizqLhxLAXweXr2g9D4ZEAStibACBg4L");
const USDC_VAULT: Address = address!("2M1JTZsc71V6FhRNjCDSttcs17HewC4KNNNkkc81L3gB");
const FEE_POOL_TOKEN_ACCOUNT: Address = address!("DuoYmMoZBy2MyGP8xa3LiWyURjmpfbZbfwRmoPvYKmr6");

#[test]
fn test_aldrin_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Aldrin program
    load_program(
        &mut svm,
        ALDRIN_PROGRAM_ID,
        &format!("{}/aldrin.so", aldrin_fixtures_dir()),
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
        &format!("{}/sol_usdc_pool.json", aldrin_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/sol_usdc_pool_mint.json", aldrin_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/sol_vault.json", aldrin_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_vault.json", aldrin_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/fee_pool_token_account.json", aldrin_fixtures_dir()),
    );

    // Create trader token accounts with initial balances
    // Selling SOL (input=WSOL) for USDC (output)
    let initial_wsol = 1_000_000_000u64;
    let initial_usdc = 0u64;
    let trader_input =
        create_token_account(&mut svm, &payer.pubkey(), &WSOL_MINT, initial_wsol, false);
    let trader_output =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);

    // Build swap instruction: sell 0.001 SOL for USDC
    let in_amount = 1_000_000u64; // 0.001 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    // Aldrin accounts layout (11 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(ALDRIN_PROGRAM_ID, false), // aldrin_program
        AccountMeta::new_readonly(POOL, false),              // pool
        AccountMeta::new_readonly(POOL_SIGNER, false),       // pool_signer
        AccountMeta::new(POOL_MINT, false),                  // pool_mint
        AccountMeta::new(SOL_VAULT, false),                  // base_token_vault
        AccountMeta::new(USDC_VAULT, false),                 // quote_token_vault
        AccountMeta::new(FEE_POOL_TOKEN_ACCOUNT, false),     // fee_pool_token_account
        AccountMeta::new(payer.pubkey(), true),              // wallet_authority
        AccountMeta::new(trader_input, false),               // user_base_token_account
        AccountMeta::new(trader_output, false),              // user_quote_token_account
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),  // token_program
    ];

    // Ask = 1 (sell base for quote)
    let extra_data: &[u8] = &[1u8];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::Aldrin,
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
                "Aldrin swap successful! WSOL: {} -> {}, USDC: {} -> {}",
                initial_wsol, final_wsol, initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("Aldrin swap CPI failed: {}", e);
        }
    }
}
