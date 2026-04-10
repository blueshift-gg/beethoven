use {
    crate::helper::*,
    beethoven_client::get_associated_token_address,
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::{AccountMeta, Instruction},
    solana_keypair::Keypair,
    solana_signer::Signer,
    solana_transaction::Transaction,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const TUNA_CONFIG: Address = address!("H1utnsgjEupueAKZckqXbeyi3DSokgauuYUFCGv8mRZ4");
const VAULT: Address = address!("D76dDcSU5HnAGqVEZCDLyGgLpTp4xZuqeZyVDtUdDv55");
const VAULT_USDC_ATA: Address = address!("4iTbtBmr4fXpkUD4kTW9pujvXbCT3AkWya6h3dbNP7a6");

const OPEN_LENDING_POSITION_V2_DISCRIMINATOR: [u8; 8] = [227, 222, 46, 156, 56, 44, 48, 55];

#[test]
fn test_defi_tuna_deposit_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Bankineco program
    load_program(
        &mut svm,
        DEFI_TUNA_PROGRAM_ID,
        &format!("{}/defi_tuna.so", defi_tuna_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault.json", defi_tuna_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/tuna_config.json", defi_tuna_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_vault_ata.json", defi_tuna_fixtures_dir()),
    );

    // Jump ahead by Vault last_update_timestamp (1_775_807_355) + 1
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_775_807_355 + 1;
    svm.set_sysvar::<Clock>(&clock);

    // Create trader token accounts with initial balances
    // Depositing USDC
    let initial_usdc = 100_000_000u64; // 100 USDC
    let trader_input = get_associated_token_address(&payer.pubkey(), &USDC_MINT, &TOKEN_PROGRAM_ID);
    create_token_account_at(
        &mut svm,
        trader_input,
        &payer.pubkey(),
        &USDC_MINT,
        initial_usdc,
        false,
    );

    // Build deposit instruction: deposit 10 USDC
    let in_amount = 10_000_000u64; // 10 USDC

    let lending_position = Address::find_program_address(
        &[b"lending_position", payer.pubkey().as_ref(), VAULT.as_ref()],
        &DEFI_TUNA_PROGRAM_ID,
    )
    .0;

    // Prerequisite: open lending position v2
    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),      // authority
        AccountMeta::new_readonly(USDC_MINT, false), // mint
        AccountMeta::new(VAULT, false),              // vault
        AccountMeta::new(lending_position, false),   // lending_position
        AccountMeta::new(SYSTEM_PROGRAM_ID, false),  // system program
    ];

    let instruction_data: &[u8] = &OPEN_LENDING_POSITION_V2_DISCRIMINATOR;
    let instruction = Instruction {
        accounts,
        data: instruction_data.to_vec(),
        program_id: DEFI_TUNA_PROGRAM_ID,
    };
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[&payer],
        svm.latest_blockhash(),
    );
    svm.send_transaction(transaction).unwrap();

    // DefiTuna deposit accounts layout (10 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(DEFI_TUNA_PROGRAM_ID, false), // defi_tuna_program
        AccountMeta::new(payer.pubkey(), true),                 // authority
        AccountMeta::new_readonly(USDC_MINT, false),            // mint
        AccountMeta::new_readonly(TUNA_CONFIG, false),          // tuna_config
        AccountMeta::new(lending_position, false),              // lending_position
        AccountMeta::new(VAULT, false),                         // vault
        AccountMeta::new(VAULT_USDC_ATA, false),                // vault_ata
        AccountMeta::new(trader_input, false),                  // authority_ata
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),     // token_program
        AccountMeta::new_readonly(MEMO_PROGRAM_ID, false),      // memo_program
    ];

    // DefiTuna deposit has no extra data
    let extra_data: &[u8] = &[];

    let instruction = build_deposit_instruction(accounts, in_amount, extra_data);

    // Execute the deposit via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_usdc = get_token_balance(&svm, &trader_input);

            assert!(
                final_usdc < initial_usdc,
                "USDC should have decreased: {} -> {}",
                initial_usdc,
                final_usdc
            );

            println!(
                "DefiTuna deposit successful! USDC: {} -> {}",
                initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("DefiTuna deposit CPI failed: {}", e);
        }
    }
}
