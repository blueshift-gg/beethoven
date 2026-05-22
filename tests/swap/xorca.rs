use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, create_token_account, get_token_balance,
        load_and_set_json_fixture, load_program, send_transaction, setup_svm, xorca_fixtures_dir,
        TEST_PROGRAM_ID, TOKEN_PROGRAM_ID, XORCA_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const ORCA_VAULT: Address = address!("Ce5j11WAsSzM3nkzrw4Kw6v6ic3nbyqpv5eywjYKeKc5");
const XORCA_MINT: Address = address!("xorcaYqbXUNz3474ubUMJAdu2xgPsew3rUCe5ughT3N");
const STATE_ACCOUNT: Address = address!("CSqKhyW1cpdyjheAx5HXx4ibcnYrzpL5JywEMAkZixBK");
const ORCA_MINT: Address = address!("orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE");

#[test]
fn test_xorca_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load xORCA program
    load_program(
        &mut svm,
        XORCA_PROGRAM_ID,
        &format!("{}/xorca.so", xorca_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/orca_mint.json", xorca_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/xorca_mint.json", xorca_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/xorca_vault.json", xorca_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/state_account.json", xorca_fixtures_dir()),
    );

    // Create trader token accounts with initial balances
    // Selling ORCA for xORCA
    let initial_orca = 1_000_000_000u64; // 1000 ORCA
    let initial_xorca = 0u64;
    let trader_orca =
        create_token_account(&mut svm, &payer.pubkey(), &ORCA_MINT, initial_orca, false);
    let trader_xorca =
        create_token_account(&mut svm, &payer.pubkey(), &XORCA_MINT, initial_xorca, false);

    // Build swap instruction: sell 10 ORCA for xORCA
    let in_amount = 10_000_000u64; // 10 ORCA
    let min_out_amount = 1u64; // Very loose slippage for test

    // xORCA accounts layout (9 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(XORCA_PROGRAM_ID, false),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(ORCA_VAULT, false),
        AccountMeta::new(trader_orca, false),
        AccountMeta::new(trader_xorca, false),
        AccountMeta::new(XORCA_MINT, false),
        AccountMeta::new_readonly(STATE_ACCOUNT, false),
        AccountMeta::new_readonly(ORCA_MINT, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
    ];

    // xORCA swap has no extra data
    let extra_data: &[u8] = &[];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::Xorca,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_orca = get_token_balance(&svm, &trader_orca);
            let final_xorca = get_token_balance(&svm, &trader_xorca);

            assert!(
                final_orca < initial_orca,
                "ORCA should have decreased: {} -> {}",
                initial_orca,
                final_orca
            );
            assert!(
                final_xorca > initial_xorca,
                "xORCA should have increased: {} -> {}",
                initial_xorca,
                final_xorca
            );

            println!(
                "xORCA swap successful! ORCA: {} -> {}, xORCA: {} -> {}",
                initial_orca, final_orca, initial_xorca, final_xorca
            );
        }
        Err(e) => {
            panic!("xORCA swap CPI failed: {}", e);
        }
    }
}
