use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir,
        create_token_account_at, fraudsworth_tax_fixtures_dir, get_token_balance,
        load_and_set_json_fixture, load_program, send_transaction, send_transaction_with_signers,
        set_token_balance, setup_svm, SYSTEM_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_2022_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    beethoven_client::get_associated_token_address,
    litesvm::LiteSVM,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
    spl_associated_token_account::instruction::create_associated_token_account,
};

pub const FRAUDSWORTH_TAX_PROGRAM_ID: Address =
    address!("43fZGRtmEsP7ExnJE1dbTbNjaP1ncvVmMPusSeksWGEj");
pub const FRAUDSWORTH_AMM_PROGRAM_ID: Address =
    address!("5JsSAL3kJDUWD4ZveYXYZmgm1eVqueesTZVdAvtZg8cR");
pub const FRAUDSWORTH_STAKING_PROGRAM_ID: Address =
    address!("12b3t1cNiAUoYLiWFEnFa4w6qYxVAiqCWU7KZuzLPYtH");
pub const FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID: Address =
    address!("CiQPQrmQh6BPhb9k7dFnsEs5gKPgdrvNKFc5xie5xVGd");
pub const EPOCH_STATE: Address = address!("FjJrLcmDjA8FtavGWdhJq3pdirAH889oWXc2bhEAMbDU");
pub const SWAP_AUTHORITY: Address = address!("CoCdbornGtiZ8tLxF5HD2TdGidfgfwbbiDX79BaZGJ2D");
pub const TAX_AUTHORITY: Address = address!("8zijSBnoiGQzwccQkdNuAwbZCieDZsxdn2GgKDErCemQ");
pub const STAKE_POOL: Address = address!("5BdRPPwEDpHEtRgdp4MfywbwmZnrf6u23bXMnG1w8ViN");
pub const STAKING_ESCROW: Address = address!("E68zPDgzMqnycj23g9T74ioHbDdvq3Npj5tT2yPd1SY");
pub const CARNAGE_VAULT: Address = address!("5988CYMcvJpNtGbtCDnAMxrjrLxRCq3qPME7w2v36aNT");
pub const TREASURY: Address = address!("3ihhwLnEJ2duwPSLYxhLbFrdhhxXLcvcrV9rAHqMgzCv");
pub const WSOL_INTERMEDIARY: Address = address!("2HPNULWVVdTcRiAm2DkghLA6frXxA2Nsu4VRu8a4qQ1s");

pub const POOL_WSOL_CRIME: Address = address!("ZWUZ3PzGk6bg6g3BS3WdXKbdAecUgZxnruKXQkte7wf");
pub const POOL_WSOL_CRIME_VAULT_A: Address =
    address!("14rFLiXzXk7aXLnwAz2kwQUjG9vauS84AQLu6LH9idUM");
pub const POOL_WSOL_CRIME_VAULT_B: Address =
    address!("6s6cprCGxTAYCk9LiwCpCsdHzReW7CLZKqy3ZSCtmV1b");

pub const POOL_WSOL_FRAUD: Address = address!("AngvViTVGd2zxP8KoFUjGU3TyrQjqeM1idRWiKM8p3mq");
pub const POOL_WSOL_FRAUD_VAULT_A: Address =
    address!("3sUDyw1k61NSKgn2EA9CaS3FbSZAApGeCRNwNFQPwg8o");
pub const POOL_WSOL_FRAUD_VAULT_B: Address =
    address!("2nzqXn6FivXjPSgrUGTA58eeVUDjGhvn4QLfhXK1jbjP");

pub const EXTRA_ACCOUNT_META_LIST_CRIME: Address =
    address!("CStTzemevJvk8vnjw57Wjzk5EFwN12Nmniz6R7qXWykr");
pub const EXTRA_ACCOUNT_META_LIST_FRAUD: Address =
    address!("7QGodnZAYGgastQMXcitcQjraYCMMNDgbp2uL73qjGkd");

pub const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
pub const CRIME_MINT: Address = address!("cRiMEhAxoDhcEuh3Yf7Z2QkXUXUMKbakhcVqmDsqPXc");
pub const FRAUD_MINT: Address = address!("FraUdp6YhtVJYPxC2w255yAbpTsPqd8Bfhy9rC56jau5");

fn load_program_and_fixtures(svm: &mut LiteSVM) {
    load_program(svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Fraudsworth programs
    load_program(
        svm,
        FRAUDSWORTH_TAX_PROGRAM_ID,
        &format!("{}/fraudsworth_tax.so", fraudsworth_tax_fixtures_dir()),
    );
    load_program(
        svm,
        FRAUDSWORTH_AMM_PROGRAM_ID,
        &format!("{}/fraudsworth_amm.so", fraudsworth_tax_fixtures_dir()),
    );
    load_program(
        svm,
        FRAUDSWORTH_STAKING_PROGRAM_ID,
        &format!("{}/fraudsworth_staking.so", fraudsworth_tax_fixtures_dir()),
    );
    load_program(
        svm,
        FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
        &format!(
            "{}/fraudsworth_transfer_hook.so",
            fraudsworth_tax_fixtures_dir()
        ),
    );

    // Load fixtures
    load_and_set_json_fixture(svm, &format!("{}/wsol_mint.json", common_fixtures_dir()));
    load_and_set_json_fixture(
        svm,
        &format!("{}/fraud_mint.json", fraudsworth_tax_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/crime_mint.json", fraudsworth_tax_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/epoch_state.json", fraudsworth_tax_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/stake_pool.json", fraudsworth_tax_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/staking_escrow.json", fraudsworth_tax_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/carnage_vault.json", fraudsworth_tax_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/treasury.json", fraudsworth_tax_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/wsol_intermediary.json", fraudsworth_tax_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/pool_wsol_crime.json", fraudsworth_tax_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/pool_wsol_crime_vault_a.json",
            fraudsworth_tax_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/pool_wsol_crime_vault_b.json",
            fraudsworth_tax_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/pool_wsol_fraud.json", fraudsworth_tax_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/pool_wsol_fraud_vault_a.json",
            fraudsworth_tax_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/pool_wsol_fraud_vault_b.json",
            fraudsworth_tax_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/extra_account_meta_list_crime.json",
            fraudsworth_tax_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/extra_account_meta_list_fraud.json",
            fraudsworth_tax_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/whitelist_crime_vault_a.json",
            fraudsworth_tax_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/whitelist_crime_vault_b.json",
            fraudsworth_tax_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/whitelist_fraud_vault_a.json",
            fraudsworth_tax_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/whitelist_fraud_vault_b.json",
            fraudsworth_tax_fixtures_dir()
        ),
    );
}

#[test]
fn test_fraudsworth_tax_swap_cpi_buy() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program_and_fixtures(&mut svm);

    // Create trader token accounts with initial balances
    // Selling SOL for CRIME
    let initial_wsol = 1_000_000_000u64; // 1 SOL
    let initial_crime = 0u64;
    let trader_wsol = get_associated_token_address(&payer.pubkey(), &WSOL_MINT, &TOKEN_PROGRAM_ID);
    create_token_account_at(
        &mut svm,
        trader_wsol,
        &payer.pubkey(),
        &WSOL_MINT,
        initial_wsol,
        false,
    );
    // Build and send custom initialize token account instruction to include extensions
    let trader_crime =
        get_associated_token_address(&payer.pubkey(), &CRIME_MINT, &TOKEN_2022_PROGRAM_ID);
    let create_trader_crime_ata_ix = create_associated_token_account(
        &payer.pubkey(),
        &payer.pubkey(),
        &CRIME_MINT,
        &TOKEN_2022_PROGRAM_ID,
    );
    send_transaction_with_signers(&mut svm, &payer, &[&payer], create_trader_crime_ata_ix).unwrap();

    // Build swap instruction: sell 0.001 SOL for CRIME
    let in_amount = 1_000_000u64; // 0.001 SOL
    let min_out_amount = 100_000_000u64; // Amplified amount to bypass MinimumOutputFloorViolation error

    let whitelist_source = Address::find_program_address(
        &[b"whitelist", POOL_WSOL_CRIME_VAULT_B.as_ref()],
        &FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
    )
    .0;
    let whitelist_destination = Address::find_program_address(
        &[b"whitelist", trader_crime.as_ref()],
        &FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
    )
    .0;

    // Fraudsworth Tax accounts swap_sol_buy layout (21 accounts + 4 transfer hook accounts)
    let accounts = vec![
        AccountMeta::new_readonly(FRAUDSWORTH_TAX_PROGRAM_ID, false), // fraudsworth_tax_program
        AccountMeta::new(payer.pubkey(), true),                       // user
        AccountMeta::new(EPOCH_STATE, false),                         // epoch_state
        AccountMeta::new_readonly(SWAP_AUTHORITY, false),             // swap_authority
        AccountMeta::new_readonly(TAX_AUTHORITY, false),              // tax_authority
        AccountMeta::new(POOL_WSOL_CRIME, false),                     // pool
        AccountMeta::new(POOL_WSOL_CRIME_VAULT_A, false),             // pool_vault_a
        AccountMeta::new(POOL_WSOL_CRIME_VAULT_B, false),             // pool_vault_b
        AccountMeta::new_readonly(WSOL_MINT, false),                  // mint_a
        AccountMeta::new_readonly(CRIME_MINT, false),                 // mint_b
        AccountMeta::new(trader_wsol, false),                         // user_token_a
        AccountMeta::new(trader_crime, false),                        // user_token_b
        AccountMeta::new(STAKE_POOL, false),                          // stake_pool
        AccountMeta::new(STAKING_ESCROW, false),                      // staking_escrow
        AccountMeta::new(CARNAGE_VAULT, false),                       // carnage_vault
        AccountMeta::new(TREASURY, false),                            // treasury
        AccountMeta::new_readonly(FRAUDSWORTH_AMM_PROGRAM_ID, false), // amm_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),           // token_program_a
        AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),      // token_program_b
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),          // system_program
        AccountMeta::new_readonly(FRAUDSWORTH_STAKING_PROGRAM_ID, false), // staking_program
        AccountMeta::new_readonly(EXTRA_ACCOUNT_META_LIST_CRIME, false), // extra_account_meta_list
        AccountMeta::new_readonly(whitelist_source, false),           // whitelist_source
        AccountMeta::new_readonly(whitelist_destination, false),      // whitelist_destination
        AccountMeta::new_readonly(FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID, false), // transfer_hook_program
    ];

    // is_buy = true, is_crime = true
    let extra_data: &[u8] = &[1u8, 1u8];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::FraudsworthTax,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_wsol = get_token_balance(&svm, &trader_wsol);
            let final_crime = get_token_balance(&svm, &trader_crime);

            assert!(
                final_wsol < initial_wsol,
                "WSOL should have decreased: {} -> {}",
                initial_wsol,
                final_wsol
            );
            assert!(
                final_crime > initial_crime,
                "CRIME should have increased: {} -> {}",
                initial_crime,
                final_crime
            );

            println!(
                "Fraudsworth Tax swap_sol_buy successful! WSOL: {} -> {}, CRIME: {} -> {}",
                initial_wsol, final_wsol, initial_crime, final_crime,
            );
        }
        Err(e) => {
            panic!("Fraudsworth Tax swap_sol_buy CPI failed: {}", e);
        }
    }
}

#[test]
fn test_fraudsworth_tax_swap_cpi_sell() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program_and_fixtures(&mut svm);

    // Create trader token accounts with initial balances
    // Selling FRAUD for SOL
    let initial_fraud = 1_000_000_000u64; // 1000 FRAUD
    let initial_wsol = 0u64;

    // Build and send custom initialize token account instruction to include extensions
    let trader_fraud =
        get_associated_token_address(&payer.pubkey(), &FRAUD_MINT, &TOKEN_2022_PROGRAM_ID);
    let create_trader_fraud_ata_ix = create_associated_token_account(
        &payer.pubkey(),
        &payer.pubkey(),
        &FRAUD_MINT,
        &TOKEN_2022_PROGRAM_ID,
    );
    send_transaction_with_signers(&mut svm, &payer, &[&payer], create_trader_fraud_ata_ix).unwrap();

    // override trader_fraud initial balance
    set_token_balance(&mut svm, &trader_fraud, initial_fraud);

    let trader_wsol = get_associated_token_address(&payer.pubkey(), &WSOL_MINT, &TOKEN_PROGRAM_ID);
    create_token_account_at(
        &mut svm,
        trader_wsol,
        &payer.pubkey(),
        &WSOL_MINT,
        initial_fraud,
        false,
    );

    // Build swap instruction: sell 0.001 FRAUD for SOL
    let in_amount = 1_000u64; // 0.001 FRAUD
    let min_out_amount = 3u64; // Very loose slippage for test

    let whitelist_source = Address::find_program_address(
        &[b"whitelist", trader_fraud.as_ref()],
        &FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
    )
    .0;
    let whitelist_destination = Address::find_program_address(
        &[b"whitelist", POOL_WSOL_FRAUD_VAULT_B.as_ref()],
        &FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
    )
    .0;

    // Fraudsworth Tax accounts swap_sol_sell layout (22 accounts + 4 transfer hook accounts)
    let accounts = vec![
        AccountMeta::new_readonly(FRAUDSWORTH_TAX_PROGRAM_ID, false), // fraudsworth_tax_program
        AccountMeta::new(payer.pubkey(), true),                       // user
        AccountMeta::new(EPOCH_STATE, false),                         // epoch_state
        AccountMeta::new(SWAP_AUTHORITY, false),                      // swap_authority
        AccountMeta::new_readonly(TAX_AUTHORITY, false),              // tax_authority
        AccountMeta::new(POOL_WSOL_FRAUD, false),                     // pool
        AccountMeta::new(POOL_WSOL_FRAUD_VAULT_A, false),             // pool_vault_a
        AccountMeta::new(POOL_WSOL_FRAUD_VAULT_B, false),             // pool_vault_b
        AccountMeta::new_readonly(WSOL_MINT, false),                  // mint_a
        AccountMeta::new_readonly(FRAUD_MINT, false),                 // mint_b
        AccountMeta::new(trader_wsol, false),                         // user_token_a
        AccountMeta::new(trader_fraud, false),                        // user_token_b
        AccountMeta::new(STAKE_POOL, false),                          // stake_pool
        AccountMeta::new(STAKING_ESCROW, false),                      // staking_escrow
        AccountMeta::new(CARNAGE_VAULT, false),                       // carnage_vault
        AccountMeta::new(TREASURY, false),                            // treasury
        AccountMeta::new(WSOL_INTERMEDIARY, false),                   // wsol_intermediary
        AccountMeta::new_readonly(FRAUDSWORTH_AMM_PROGRAM_ID, false), // amm_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),           // token_program_a
        AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),      // token_program_b
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),          // system_program
        AccountMeta::new_readonly(FRAUDSWORTH_STAKING_PROGRAM_ID, false), // staking_program
        AccountMeta::new_readonly(EXTRA_ACCOUNT_META_LIST_FRAUD, false), // extra_account_meta_list
        AccountMeta::new_readonly(whitelist_source, false),           // whitelist_source
        AccountMeta::new_readonly(whitelist_destination, false),      // whitelist_destination
        AccountMeta::new_readonly(FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID, false), // transfer_hook_program
    ];

    // is_buy = false, is_crime = false
    let extra_data: &[u8] = &[0u8, 0u8];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::FraudsworthTax,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_fraud = get_token_balance(&svm, &trader_fraud);
            let final_wsol = get_token_balance(&svm, &trader_wsol);

            assert!(
                final_fraud < initial_fraud,
                "FRAUD should have decreased: {} -> {}",
                initial_fraud,
                final_fraud
            );
            assert!(
                final_wsol > initial_wsol,
                "WSOL should have increased: {} -> {}",
                initial_wsol,
                final_wsol
            );

            println!(
                "Fraudsworth Tax swap_sol_sell successful! FRAUD: {} -> {}, WSOL: {} -> {}",
                initial_fraud, final_fraud, initial_wsol, final_wsol,
            );
        }
        Err(e) => {
            panic!("Fraudsworth Tax swap_sol_sell CPI failed: {}", e);
        }
    }
}
