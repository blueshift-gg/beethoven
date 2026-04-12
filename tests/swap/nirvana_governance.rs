use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program,
        nirvana_governance_fixtures_dir, send_transaction, setup_svm,
        NIRVANA_GOVERNANCE_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

pub const ANA_MINT: Address = address!("5DkzT65YJvCsZcot9L6qwkJnsBCPmKHjJz3QU7t7QeRW");
pub const NIRV_MINT: Address = address!("3eamaYJ7yicyRd3mYz4YeNyNPGVo6zMmKUp5UP25AxRM");
pub const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
pub const TENANT: Address = address!("BcAoCEdkzV2J21gAjCCEokBw5iMnAe96SbYo9F6QmKWV");
pub const PRICE_CURVE: Address = address!("Fx5u5BCTwpckbB6jBbs13nDsRabHb5bq2t2hBDszhSbd");
pub const BACKING_VAULT_MAIN: Address = address!("FhTJEGXVwj4M6NQ1tPu9jgDZUXWQ9w2hP89ebZHwrJPS");
pub const BACKING_VAULT_NIRV: Address = address!("EkwPHXXZNAguNoxeftVRXThCQJfD6EaG852pDsYLs2eB");
pub const ESCROW_REV_ANA: Address = address!("42rJYSmYHqbn5mk992xAoKZnWEiuMzr6u6ydj9m8fAjP");

#[test]
fn test_nirvana_governance_swap_cpi_buy() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Nirvana Governance program
    load_program(
        &mut svm,
        NIRVANA_GOVERNANCE_PROGRAM_ID,
        &format!(
            "{}/nirvana_governance.so",
            nirvana_governance_fixtures_dir()
        ),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/ana_mint.json", nirvana_governance_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/nirv_mint.json", nirvana_governance_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/tenant.json", nirvana_governance_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/price_curve.json", nirvana_governance_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/backing_vault_main.json",
            nirvana_governance_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/backing_vault_nirv.json",
            nirvana_governance_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/escrow_rev_ana.json", nirvana_governance_fixtures_dir()),
    );

    // Create trader token accounts with initial balances
    // Selling USDC for ANA
    let initial_usdc = 100_000_000u64; // 100 USDC
    let initial_ana = 0u64;
    let trader_input =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);
    let trader_output =
        create_token_account(&mut svm, &payer.pubkey(), &ANA_MINT, initial_ana, false);

    // Build swap instruction: sell 10 USDC for ANA
    let in_amount = 10_000_000u64; // 10 USDC
    let min_out_amount = 1u64; // Very loose slippage for test

    // Nirvana Governance accounts layout (14 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(NIRVANA_GOVERNANCE_PROGRAM_ID, false), // nirvana_governance_program
        AccountMeta::new(payer.pubkey(), true),                          // payer
        AccountMeta::new(TENANT, false),                                 // tenant
        AccountMeta::new_readonly(PRICE_CURVE, false),                   // price_curve
        AccountMeta::new(ANA_MINT, false),                               // mint_ana
        AccountMeta::new_readonly(NIRV_MINT, false),                     // mint_nirv
        AccountMeta::new_readonly(USDC_MINT, false),                     // mint_main
        AccountMeta::new(BACKING_VAULT_MAIN, false),                     // backing_vault_main
        AccountMeta::new(BACKING_VAULT_NIRV, false),                     // backing_vault_nirv
        AccountMeta::new(ESCROW_REV_ANA, false),                         // escrow_rev_ana
        AccountMeta::new(trader_input, false),                           // backing_src
        AccountMeta::new(trader_output, false),                          // ana_dst
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),              // token_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),              // token_program_main
    ];

    // Nirvana Governance swap buy has no extra data
    let extra_data: &[u8] = &[];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::NirvanaGovernance,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_usdc = get_token_balance(&svm, &trader_input);
            let final_ana = get_token_balance(&svm, &trader_output);

            assert!(
                final_usdc < initial_usdc,
                "USDC should have decreased: {} -> {}",
                initial_usdc,
                final_usdc
            );
            assert!(
                final_ana > initial_ana,
                "ANA should have increased: {} -> {}",
                initial_ana,
                final_ana
            );

            println!(
                "Nirvana Governance swap successful! USDC: {} -> {}, ANA: {} -> {}",
                initial_usdc, final_usdc, initial_ana, final_ana
            );
        }
        Err(e) => {
            panic!("Nirvana Governance swap CPI failed: {}", e);
        }
    }
}

#[test]
fn test_nirvana_governance_swap_cpi_sell() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Nirvana Governance program
    load_program(
        &mut svm,
        NIRVANA_GOVERNANCE_PROGRAM_ID,
        &format!(
            "{}/nirvana_governance.so",
            nirvana_governance_fixtures_dir()
        ),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/ana_mint.json", nirvana_governance_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/nirv_mint.json", nirvana_governance_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/tenant.json", nirvana_governance_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/price_curve.json", nirvana_governance_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/backing_vault_main.json",
            nirvana_governance_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/backing_vault_nirv.json",
            nirvana_governance_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/escrow_rev_ana.json", nirvana_governance_fixtures_dir()),
    );

    // Create trader token accounts with initial balances
    // Selling ANA for USDC
    let initial_ana = 100_000_000u64; // 100 ANA
    let initial_usdc = 0u64;
    let trader_input =
        create_token_account(&mut svm, &payer.pubkey(), &ANA_MINT, initial_ana, false);
    let trader_output =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);

    // Build swap instruction: sell 10 ANA for USDC
    let in_amount = 10_000_000u64; // 10 ANA
    let min_out_amount = 1u64; // Very loose slippage for test

    // Nirvana Governance accounts layout (14 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(NIRVANA_GOVERNANCE_PROGRAM_ID, false), // nirvana_governance_program
        AccountMeta::new(payer.pubkey(), true),                          // payer
        AccountMeta::new(TENANT, false),                                 // tenant
        AccountMeta::new(PRICE_CURVE, false),                            // price_curve
        AccountMeta::new(ANA_MINT, false),                               // mint_ana
        AccountMeta::new(trader_output, false),                          // backing_dst
        AccountMeta::new(ESCROW_REV_ANA, false),                         // escrow_rev_ana
        AccountMeta::new(BACKING_VAULT_MAIN, false),                     // backing_vault_main
        AccountMeta::new(BACKING_VAULT_NIRV, false),                     // backing_vault_nirv
        AccountMeta::new(trader_input, false),                           // ana_src
        AccountMeta::new_readonly(NIRV_MINT, false),                     // mint_nirv
        AccountMeta::new_readonly(USDC_MINT, false),                     // mint_main
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),              // token_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),              // token_program_main
    ];

    // Nirvana Governance swap sell has no extra data
    let extra_data: &[u8] = &[];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::NirvanaGovernance,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_ana = get_token_balance(&svm, &trader_input);
            let final_usdc = get_token_balance(&svm, &trader_output);

            assert!(
                final_ana < initial_ana,
                "ANA should have decreased: {} -> {}",
                initial_ana,
                final_ana
            );
            assert!(
                final_usdc > initial_usdc,
                "USDC should have increased: {} -> {}",
                initial_usdc,
                final_usdc
            );

            println!(
                "Nirvana Governance swap successful! ANA: {} -> {}, USDC: {} -> {}",
                initial_ana, final_ana, initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("Nirvana Governance swap CPI failed: {}", e);
        }
    }
}
