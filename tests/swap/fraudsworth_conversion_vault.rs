use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, fraudsworth_conversion_vault_fixtures_dir,
        get_token_balance, load_and_set_json_fixture, load_program, send_transaction,
        send_transaction_with_signers, set_token_balance, setup_svm,
        FRAUDSWORTH_CONVERSION_VAULT_PROGRAM_ID, FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
        TEST_PROGRAM_ID, TOKEN_2022_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    beethoven_client::get_associated_token_address,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
    spl_associated_token_account::instruction::create_associated_token_account,
};

const VAULT_CONFIG: Address = address!("8vFpSBnCVt8dfX57FKrsGwy39TEo1TjVzrj9QYGxCkcD");
const VAULT_FRAUD: Address = address!("DLciB9t3qEuRcndGyjRmu1Z34NCwTPvNwbv7eUsFxTZG");
const VAULT_PROFIT: Address = address!("DBMaWgfUW8WBb8VVvqDFkrMpEkPkCPTcLpSpyzHAiwp3");

const EXTRA_ACCOUNT_META_LIST_FRAUD: Address =
    address!("7QGodnZAYGgastQMXcitcQjraYCMMNDgbp2uL73qjGkd");
const EXTRA_ACCOUNT_META_LIST_PROFIT: Address =
    address!("J4dubfKw7vnZLhpPfMHqz8PcYWaChugnnSGUgGDzQ9AB");

const FRAUD_MINT: Address = address!("FraUdp6YhtVJYPxC2w255yAbpTsPqd8Bfhy9rC56jau5");
const PROFIT_MINT: Address = address!("pRoFiTj36haRD5sG2Neqib9KoSrtdYMGrM7SEkZetfR");

fn load_program_and_fixtures(svm: &mut litesvm::LiteSVM) {
    load_program(svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Fraudsworth programs
    load_program(
        svm,
        FRAUDSWORTH_CONVERSION_VAULT_PROGRAM_ID,
        &format!(
            "{}/fraudsworth_conversion_vault.so",
            fraudsworth_conversion_vault_fixtures_dir()
        ),
    );
    load_program(
        svm,
        FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
        &format!(
            "{}/fraudsworth_transfer_hook.so",
            fraudsworth_conversion_vault_fixtures_dir()
        ),
    );

    // Load fixtures
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/fraud_mint.json",
            fraudsworth_conversion_vault_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/profit_mint.json",
            fraudsworth_conversion_vault_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/vault_config.json",
            fraudsworth_conversion_vault_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/vault_fraud.json",
            fraudsworth_conversion_vault_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/vault_profit.json",
            fraudsworth_conversion_vault_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/extra_account_meta_list_fraud.json",
            fraudsworth_conversion_vault_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/extra_account_meta_list_profit.json",
            fraudsworth_conversion_vault_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/whitelist_fraud_vault.json",
            fraudsworth_conversion_vault_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/whitelist_profit_vault.json",
            fraudsworth_conversion_vault_fixtures_dir()
        ),
    );
}

#[test]
fn test_fraudsworth_conversion_vault_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program_and_fixtures(&mut svm);

    let user_fraud =
        get_associated_token_address(&payer.pubkey(), &FRAUD_MINT, &TOKEN_2022_PROGRAM_ID);
    let create_user_crime_ata_ix = create_associated_token_account(
        &payer.pubkey(),
        &payer.pubkey(),
        &FRAUD_MINT,
        &TOKEN_2022_PROGRAM_ID,
    );
    send_transaction_with_signers(&mut svm, &payer, &[&payer], create_user_crime_ata_ix).unwrap();

    let user_profit =
        get_associated_token_address(&payer.pubkey(), &PROFIT_MINT, &TOKEN_2022_PROGRAM_ID);
    let create_user_profit_ata_ix = create_associated_token_account(
        &payer.pubkey(),
        &payer.pubkey(),
        &PROFIT_MINT,
        &TOKEN_2022_PROGRAM_ID,
    );
    send_transaction_with_signers(&mut svm, &payer, &[&payer], create_user_profit_ata_ix).unwrap();

    // Create trader token accounts with initial balances
    // Selling FRAUD for PROFIT
    let initial_fraud = 1_000_000u64;
    let initial_profit = 0u64;

    set_token_balance(&mut svm, &user_fraud, initial_fraud);

    let whitelist_user_fraud = Address::find_program_address(
        &[b"whitelist", user_fraud.as_ref()],
        &FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
    )
    .0;
    let whitelist_vault_fraud = Address::find_program_address(
        &[b"whitelist", VAULT_FRAUD.as_ref()],
        &FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
    )
    .0;

    let whitelist_vault_profit = Address::find_program_address(
        &[b"whitelist", VAULT_PROFIT.as_ref()],
        &FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
    )
    .0;
    let whitelist_user_profit = Address::find_program_address(
        &[b"whitelist", user_profit.as_ref()],
        &FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
    )
    .0;
    // Build swap instruction: sell 1 FRAUD for PROFIT
    let in_amount = 1_000_000u64;
    let min_out_amount = 1u64; // Very loose slippage for test

    // Fraudsworth Conversion Vault accounts layout (18 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(FRAUDSWORTH_CONVERSION_VAULT_PROGRAM_ID, false), // fraudsworth_conversion_vault_program
        AccountMeta::new(payer.pubkey(), true),                                    // user
        AccountMeta::new_readonly(VAULT_CONFIG, false),                            // vault_config
        AccountMeta::new(user_fraud, false), // user_input_account
        AccountMeta::new(user_profit, false), // user_output_account
        AccountMeta::new_readonly(FRAUD_MINT, false), // input_mint
        AccountMeta::new_readonly(PROFIT_MINT, false), // output_mint
        AccountMeta::new(VAULT_FRAUD, false), // vault_input
        AccountMeta::new(VAULT_PROFIT, false), // vault_output
        AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false), // token_program
        AccountMeta::new_readonly(EXTRA_ACCOUNT_META_LIST_FRAUD, false), // input_mint_extra_account_meta_list
        AccountMeta::new_readonly(whitelist_user_fraud, false), // input_mint_whitelist_source
        AccountMeta::new_readonly(whitelist_vault_fraud, false), // input_mint_whitelist_destination
        AccountMeta::new_readonly(FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID, false), // input_mint_transfer_hook_program
        AccountMeta::new_readonly(EXTRA_ACCOUNT_META_LIST_PROFIT, false), // output_mint_extra_account_meta_list
        AccountMeta::new_readonly(whitelist_vault_profit, false), // output_mint_whitelist_source
        AccountMeta::new_readonly(whitelist_user_profit, false), // output_mint_whitelist_destination
        AccountMeta::new_readonly(FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID, false), // output_mint_transfer_hook_program
    ];

    // pre_balance
    let extra_data = 0u64.to_le_bytes().to_vec();

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::FraudsworthConversionVault,
        &extra_data,
    );
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_fraud = get_token_balance(&svm, &user_fraud);
            let final_profit = get_token_balance(&svm, &user_profit);

            assert!(
                final_fraud < initial_fraud,
                "FRAUD should have decreased: {} -> {}",
                initial_fraud,
                final_fraud
            );
            assert!(
                final_profit > initial_profit,
                "PROFIT should have increased: {} -> {}",
                initial_profit,
                final_profit
            );
        }
        Err(e) => {
            panic!("Fraudsworth Conversion Vault convert_v2 CPI failed: {}", e);
        }
    }
}
