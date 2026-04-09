use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, send_transaction, setup_svm,
        solfi_v2_fixtures_dir, INSTRUCTIONS_SYSVAR_ID, SOLFI_V2_PROGRAM_ID, TEST_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const MARKET: Address = address!("65ZHSArs5XxPseKQbB1B4r16vDxMWnCxHMzogDAqiDUc");
const ORACLE: Address = address!("2ny7eGyZCoeEVTkNLf5HcnJFBKkyA4p4gcrtb3b8y8ou");
const CONFIG: Address = address!("FmxXDSR9WvpJTCh738D1LEDuhMoA8geCtZgHb3isy7Dp");
const BASE_VAULT: Address = address!("CRo8DBwrmd97DJfAnvCv96tZPL5Mktf2NZy2ZnhDer1A");
const QUOTE_VAULT: Address = address!("GhFfLFSprPpfoRaWakPMmJTMJBHuz6C694jYwxy2dAic");

#[test]
fn test_solfi_v2_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load SolFi v2 program
    load_program(
        &mut svm,
        SOLFI_V2_PROGRAM_ID,
        &format!("{}/solfi_v2.so", solfi_v2_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/wsol_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/market.json", solfi_v2_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/oracle.json", solfi_v2_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/config.json", solfi_v2_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/base_vault.json", solfi_v2_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/quote_vault.json", solfi_v2_fixtures_dir()),
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

    // SolFi v2 accounts layout (14 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(SOLFI_V2_PROGRAM_ID, false), // solfi v2 program
        AccountMeta::new(payer.pubkey(), true),                // user
        AccountMeta::new(MARKET, false),                       // market
        AccountMeta::new_readonly(ORACLE, false),              // oracle
        AccountMeta::new_readonly(CONFIG, false),              // config
        AccountMeta::new(BASE_VAULT, false),                   // base vault
        AccountMeta::new(QUOTE_VAULT, false),                  // quote vault
        AccountMeta::new(trader_input, false),                 // user base ata
        AccountMeta::new(trader_output, false),                // user quote ata
        AccountMeta::new_readonly(WSOL_MINT, false),           // base mint
        AccountMeta::new_readonly(USDC_MINT, false),           // quote mint
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),    // base token program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),    // quote token program
        AccountMeta::new_readonly(INSTRUCTIONS_SYSVAR_ID, false), // instructions sysvar
    ];

    // is_quote_to_base = false
    let extra_data: &[u8] = &[0_u8];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::SolFiV2,
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
        }
        Err(e) => panic!("SolFi v2 swap CPI failed: {}", e),
    }
}
