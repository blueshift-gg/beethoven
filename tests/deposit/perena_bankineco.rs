use {
    crate::helper::*,
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USD_STAR_MINT: Address = address!("star9agSpjiFe3M49B3RniVU4CMBBEK3Qnaqn3RGiFM");
const VAULT_STATE: Address = address!("3bZ1qY6wfzyDH7QMPiRKLr6k8p1asdtyjvJyJsJBdv23");
const BANK_STATE: Address = address!("sM6P4mh53CnG4faN4Fo3seY7wMSAiHdy8o6gKjwQF7A");
const ORACLE_STATE: Address = address!("CmKFP4YJg5QpAryUm9xk5QD611bccYMzZvpvQDJkMwt6");
const YIELDING_VAULT_TA: Address = address!("HvG7HSrNHVAcjzgwt3UVtnY9srkrY7qnMG4zS1SnPQT2");
const TEAM_STATE: Address = address!("6tqLkhbqJSx4KG616VhNCvsaFqcDPok7wdbzU2DmEAub");
const FEE_TEAM_ATA: Address = address!("3msJbxNbSeosztbNEB1eFPitMFnP8ogCszegPUswipdL");

#[test]
fn test_perena_bankineco_deposit_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Bankineco program
    load_program(
        &mut svm,
        BANKINECO_PROGRAM_ID,
        &format!("{}/bankineco.so", bankineco_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usd_star.json", bankineco_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/bank_state.json", bankineco_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_state.json", bankineco_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/oracle_state.json", bankineco_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/fee_team_ata.json", bankineco_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/team_state.json", bankineco_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/yielding_vault.json", bankineco_fixtures_dir()),
    );

    // Jump ahead by OracleGenState result update_ts (1_775_538_086) + 1
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_775_538_086 + 1;
    svm.set_sysvar::<Clock>(&clock);

    // Create trader token accounts with initial balances
    // Depositing USDC for USD*
    let initial_usdc = 100_000_000u64; // 100 USDC
    let initial_usd_star = 0u64;
    let trader_input =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);
    let trader_output = create_token_account(
        &mut svm,
        &payer.pubkey(),
        &USD_STAR_MINT,
        initial_usd_star,
        false,
    );

    // Build deposit instruction: deposit 10 USDC for USD*
    let in_amount = 10_000_000u64; // 10 USDC

    // Bankineco deposit accounts layout (16 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(BANKINECO_PROGRAM_ID, false), // bankineco_program
        AccountMeta::new(payer.pubkey(), true),                 // user
        AccountMeta::new(BANK_STATE, false),                    // bank_state
        AccountMeta::new(VAULT_STATE, false),                   // vault_state
        AccountMeta::new_readonly(ORACLE_STATE, false),         // oracle_state
        AccountMeta::new_readonly(USDC_MINT, false),            // yielding_mint
        AccountMeta::new(USD_STAR_MINT, false),                 // bank_mint
        AccountMeta::new(trader_input, false),                  // yielding_user_ta
        AccountMeta::new(trader_output, false),                 // bank_mint_user_ta
        AccountMeta::new(YIELDING_VAULT_TA, false),             // yielding_vault_ata
        AccountMeta::new(TEAM_STATE, false),                    // team_state
        AccountMeta::new(FEE_TEAM_ATA, false),                  // fee_team_ata
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),    // system_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),     // token_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),     // yielding_mint_program
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false), // associated_token_program
    ];

    // min_bank_mint_minted = 1u64
    let extra_data = 1_u64.to_le_bytes().to_vec();

    let instruction = build_deposit_instruction(accounts, in_amount, &extra_data);

    // Execute the deposit via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_usdc = get_token_balance(&svm, &trader_input);
            let final_usd_star = get_token_balance(&svm, &trader_output);

            assert!(
                final_usdc < initial_usdc,
                "USDC should have decreased: {} -> {}",
                initial_usdc,
                final_usdc
            );
            assert!(
                final_usd_star > initial_usd_star,
                "USD* should have increased: {} -> {}",
                initial_usd_star,
                final_usd_star
            );

            println!(
                "Bankineco deposit successful! USDC: {} -> {}, USD*: {} -> {}",
                initial_usdc, final_usdc, initial_usd_star, final_usd_star
            );
        }
        Err(e) => {
            panic!("Bankineco deposit CPI failed: {}", e);
        }
    }
}
