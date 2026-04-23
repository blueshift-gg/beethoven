use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, create_token_account, get_token_balance,
        load_and_set_json_fixture, load_program, send_transaction, setup_svm,
        solv_finance_fixtures_dir, SOLV_FINANCE_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    beethoven_client::ASSOCIATED_TOKEN_PROGRAM_ID,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const SOLVBTC_MINT: Address = address!("SoLvHDFVstC74Jr9eNLTDoG4goSUsn1RENmjNtFKZvW");
const BTCPLUS_MINT: Address = address!("soLvpPEDkN8D1Wgjezrb1oj4WjGtj17vynGm6t3jah6");
const TREASURER_TOKEN_TA: Address = address!("4JjcZvMzgcDxxd9YZbdbFQciWcSakwAVe3zT1kNoJqav");
const MULTISIG: Address = address!("msigaXYhoZ6qELubRGf6N6Uj3kJ14WF82PDbV5HL172");
const VAULT: Address = address!("B3ct2h3iCWKZmErPQ8PtZ51qBU98Zfci2TSnjvXNUbUa");

#[test]
fn test_solv_finance_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Solv Finance program
    load_program(
        &mut svm,
        SOLV_FINANCE_PROGRAM_ID,
        &format!("{}/solv_finance.so", solv_finance_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/solvbtc_mint.json", solv_finance_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/btcplus_mint.json", solv_finance_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault.json", solv_finance_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/multisig.json", solv_finance_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/treasurer_token_ta.json", solv_finance_fixtures_dir()),
    );

    // Create trader token accounts with initial balances
    // Selling SolvBTC for BTC+
    let initial_solvbtc = 1_000_000_000u64; // 10 SolvBTC
    let initial_btcplus = 0u64;
    let trader_solvbtc = create_token_account(
        &mut svm,
        &payer.pubkey(),
        &SOLVBTC_MINT,
        initial_solvbtc,
        false,
    );
    let trader_btcplus = create_token_account(
        &mut svm,
        &payer.pubkey(),
        &BTCPLUS_MINT,
        initial_btcplus,
        false,
    );

    // Build swap instruction: sell 1 SolvBtc for BTC+
    let in_amount = 100_000_000u64; // 1 SolvBtc
    let min_out_amount = 1u64; // Very loose slippage for test

    // Solv Finance accounts layout (11 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(SOLV_FINANCE_PROGRAM_ID, false), // solv_finance_program
        AccountMeta::new(payer.pubkey(), true),                    // user
        AccountMeta::new(trader_solvbtc, false),                   // user_token_ta
        AccountMeta::new(trader_btcplus, false),                   // user_target_ta
        AccountMeta::new(TREASURER_TOKEN_TA, false),               // treasurer_token_ta
        AccountMeta::new_readonly(MULTISIG, false),                // multisig
        AccountMeta::new_readonly(SOLVBTC_MINT, false),            // mint_token
        AccountMeta::new(BTCPLUS_MINT, false),                     // mint_target
        AccountMeta::new(VAULT, false),                            // vault
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),        // token_program
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false), // associated_token_program
    ];

    // Solv Finance has no extra data
    let extra_data: &[u8] = &[];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::SolvFinance,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_solvbtc = get_token_balance(&svm, &trader_solvbtc);
            let final_btcplus = get_token_balance(&svm, &trader_btcplus);

            assert!(
                final_solvbtc < initial_solvbtc,
                "SolvBTC should have decreased: {} -> {}",
                initial_solvbtc,
                final_solvbtc
            );
            assert!(
                final_btcplus > initial_btcplus,
                "BTC+ should have increased: {} -> {}",
                initial_btcplus,
                final_btcplus
            );
        }
        Err(e) => {
            panic!("Solv Finance swap CPI failed: {}", e);
        }
    }
}
