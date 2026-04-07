use {
    crate::helper::*,
    solana_account::Account,
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const CLEND_GROUP: Address = address!("9PWf4kEwa3E4WCMnPp4SQoUGWNaA8Zn427g33n6jcmMb");
const CLEND_ACCOUNT: Address = address!("HwEujdhizP5gpHC63a6xF9qWjo2NvKvdTJdNEhHY9hhK");
const USDC_BANK: Address = address!("4a74Z8rY6JuuTUeVv7i8kB7LQRANb72jMtweFTUoQM81");
const USDC_VAULT: Address = address!("4ZU6vJULZNxP9BQzRgc5UFtzrSJhs77An9iA6W9ceUEq");
const USDC_PRICE_UPDATE_V2: Address = address!("Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX");

#[test]
fn test_carrot_boost_deposit_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Carrot Boost program
    load_program(
        &mut svm,
        CARROT_BOOST_PROGRAM_ID,
        &format!("{}/carrot_boost.so", carrot_boost_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/clend_account.json", carrot_boost_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/clend_group.json", carrot_boost_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_bank.json", carrot_boost_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_vault.json", carrot_boost_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_price_update_v2.json", carrot_boost_fixtures_dir()),
    );

    let clend_account = svm.get_account(&CLEND_ACCOUNT).unwrap();
    let mut clend_account_data = clend_account.data;

    // override ClendAccount authority to be the payer
    clend_account_data[40..72].copy_from_slice(&payer.pubkey().to_bytes());

    svm.set_account(
        CLEND_ACCOUNT,
        Account {
            data: clend_account_data,
            executable: clend_account.executable,
            lamports: clend_account.lamports,
            owner: clend_account.owner,
            rent_epoch: clend_account.rent_epoch,
        },
    )
    .unwrap();

    let mut clock = svm.get_sysvar::<Clock>();
    // Jump ahead by PriceUpdateV2 posted_slot (411_694_972)
    clock.slot = 411_694_972;
    // // Jump ahead by price_message published_time (1_775_590_044) + 1
    clock.unix_timestamp = 1_775_590_044 + 1;
    svm.set_sysvar::<Clock>(&clock);

    // Create trader token accounts with initial balances
    // Depositing USDC
    let initial_usdc = 100_000_000u64; // 100 USDC
    let trader_input = create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc);

    // Build deposit instruction: deposit 10 USDC
    let in_amount = 10_000_000u64; // 10 USDC

    let accounts = vec![
        AccountMeta::new_readonly(CARROT_BOOST_PROGRAM_ID, false), // carrot_boost_program
        AccountMeta::new_readonly(CLEND_GROUP, false),             // clend_group
        AccountMeta::new(CLEND_ACCOUNT, false),                    // clend_account
        AccountMeta::new(payer.pubkey(), true),                    // signer
        AccountMeta::new(USDC_BANK, false),                        // bank
        AccountMeta::new(trader_input, false),                     // signer_token_account
        AccountMeta::new(USDC_VAULT, false),                       // bank_liquidity_vault
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),        // token_program
        AccountMeta::new_readonly(USDC_PRICE_UPDATE_V2, false),    // oracle account
    ];

    // deposit_up_to_amount = 1
    let extra_data = vec![1];

    let instruction = build_deposit_instruction(accounts, in_amount, &extra_data);

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
                "Carrot Boost deposit successful! USDC: {} -> {}",
                initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("Carrot Boost deposit CPI failed: {}", e);
        }
    }
}
