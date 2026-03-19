use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, omnipair_fixtures_dir,
        send_transaction, setup_svm, OMNIPAIR_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_2022_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const SOL_USDC_PAIR: Address = address!("3cPJTS5kfD7414aTRPcyBrA55aSx8csCUPWsrS4mnFWV");
const RATE_MODEL: Address = address!("GEbFhfNcpu1gnKbyuzGZ4wfP4kXzuyKj4J8xRm2yTKeG");
const FUTARCHY_AUTHORITY: Address = address!("2SMS1Y4EAyL2dQLpXD6VJCrNbQJ2eQ2pN3qYcX1vim3E");
const EVENT_AUTHORITY: Address = address!("FWdP9yTogKbuXvEqQNNHYw2TYm38MbinAZ2iTHeZWX8H");
const SOL_USDC_PAIR_SOL_RESERVE_VAULT: Address =
    address!("2PXu1RN3zW5PDjAZoNBaijaGs3rEZ3bG9omRihb5C8Bi");
const SOL_USDC_PAIR_USDC_RESERVE_VAULT: Address =
    address!("F5c9GM9rZXPk99z6sahgSnZcyp67ck4Q694uve1RUU2Z");

#[test]
fn test_omnipair_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Omnipair program
    load_program(
        &mut svm,
        OMNIPAIR_PROGRAM_ID,
        &format!("{}/omnipair.so", omnipair_fixtures_dir()),
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
        &format!("{}/sol_usdc_pair.json", omnipair_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/rate_model.json", omnipair_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/futarchy_authority.json", omnipair_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/sol_usdc_pair_sol_reserve_vault.json",
            omnipair_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/sol_usdc_pair_usdc_reserve_vault.json",
            omnipair_fixtures_dir()
        ),
    );

    // Create trader token accounts with initial balances
    // Selling SOL (input=WSOL) for USDC (output)
    let initial_wsol = 1_000_000_000u64; // 1 SOL
    let initial_usdc = 0u64;
    let trader_input = create_token_account(&mut svm, &payer.pubkey(), &WSOL_MINT, initial_wsol);
    let trader_output = create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc);

    // Build swap instruction: sell 0.001 SOL for USDC
    let in_amount = 1_000_000u64; // 0.001 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    // Omnipair accounts layout (15 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(OMNIPAIR_PROGRAM_ID, false), // omnipair_program
        AccountMeta::new(SOL_USDC_PAIR, false),                // pair
        AccountMeta::new(RATE_MODEL, false),                   // rate_model
        AccountMeta::new_readonly(FUTARCHY_AUTHORITY, false),  // futarchy_authority
        AccountMeta::new(SOL_USDC_PAIR_SOL_RESERVE_VAULT, false), // token_in_vault
        AccountMeta::new(SOL_USDC_PAIR_USDC_RESERVE_VAULT, false), // token_out_vault
        AccountMeta::new(trader_input, false),                 // user_token_in_account
        AccountMeta::new(trader_output, false),                // user_token_out_account
        AccountMeta::new_readonly(WSOL_MINT, false),           // token_in_mint
        AccountMeta::new_readonly(USDC_MINT, false),           // token_out_mint
        AccountMeta::new(payer.pubkey(), true),                // user
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),    // token_program
        AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false), // token_2022_program
        AccountMeta::new_readonly(EVENT_AUTHORITY, false),     // event_authority
        AccountMeta::new_readonly(OMNIPAIR_PROGRAM_ID, false), // program
    ];

    // Omnipair swap has no extra data
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
                "Omnipair swap successful! WSOL: {} -> {}, USDC: {} -> {}",
                initial_wsol, final_wsol, initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("Omnipair swap CPI failed: {}", e);
        }
    }
}
