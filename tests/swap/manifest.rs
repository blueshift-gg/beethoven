use {
    crate::helper::*,
    beethoven::SwapProtocolTag,
    solana_account::Account,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_program_option::COption,
    solana_program_pack::Pack,
    solana_signer::Signer,
    spl_token_interface::state::{Account as TokenAccount, AccountState},
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const MARKET: Address = address!("ENhU8LsaR7vDD2G1CsWcsuSGNrih9Cv5WZEk7q9kPapQ");
const BASE_VAULT: Address = address!("AKjfJDv4ywdpCDrj7AURuNkGA3696GTVFgrMwk4TjkKs");
const QUOTE_VAULT: Address = address!("FN9K6rTdWtRDUPmLTN2FnGvLZpHVNRN2MeRghKknSGDs");
const GLOBAL: Address = address!("7mR36vj6pvg1U1cRatvUbLG57yqsd1ojLbrgxb6azaQ1");
const GLOBAL_VAULT: Address = address!("E1mBVQyt7BHK8SaBSfME7usYxx94T4DtHEjbUpEBhZx");

#[test]
fn test_manifest_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Manifest program
    load_program(
        &mut svm,
        MANIFEST_PROGRAM_ID,
        &format!("{}/manifest_program.so", manifest_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/manifest_usdc_sol_market.json", manifest_fixtures_dir()),
    );
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
        &format!(
            "{}/manifest_sol_usdc_base_vault.json",
            manifest_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/manifest_sol_usdc_quote_vault.json",
            manifest_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/manifest_global.json", manifest_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/manifest_global_vault.json", manifest_fixtures_dir()),
    );

    // Create trader token accounts with initial balances
    // Selling SOL (input=WSOL) for USDC (output)
    let initial_wsol = 1_000_000_000u64; // 1 SOL
    let initial_usdc = 0u64;
    let trader_base =
        create_token_account(&mut svm, &payer.pubkey(), &WSOL_MINT, initial_wsol, false);
    let trader_quote =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);

    // Build swap instruction: sell 0.001 SOL for USDC
    let in_amount = 1_000_000u64; // 0.001 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    // Manifest accounts layout (14 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(MANIFEST_PROGRAM_ID, false), // manifest_program
        AccountMeta::new(payer.pubkey(), true),                // payer
        AccountMeta::new(payer.pubkey(), true),                // owner
        AccountMeta::new(MARKET, false),                       // market
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),   // system_program
        AccountMeta::new(trader_base, false),                  // trader_base
        AccountMeta::new(trader_quote, false),                 // trader_quote
        AccountMeta::new(BASE_VAULT, false),                   // base_vault
        AccountMeta::new(QUOTE_VAULT, false),                  // quote_vault
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),    // token_program_base
        AccountMeta::new_readonly(WSOL_MINT, false),           // base_mint
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),    // token_program_quote
        AccountMeta::new_readonly(USDC_MINT, false),           // quote_mint
        AccountMeta::new(GLOBAL, false),                       // global
        AccountMeta::new(GLOBAL_VAULT, false),                 // global_vault
    ];

    // is_base_in = true (selling base/SOL), is_exact_in = true (exact input amount)
    let extra_data = [1u8, 1u8];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::Manifest,
        &extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_wsol = get_token_balance(&svm, &trader_base);
            let final_usdc = get_token_balance(&svm, &trader_quote);

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
        Err(e) => {
            panic!("Swap CPI failed: {}", e);
        }
    }
}

#[test]
fn test_manifest_swap_cpi_mollusk() {
    let beethoven_bytes = load_fixture_bytes(&beethoven_program_path());
    let manifest_bytes =
        load_fixture_bytes(&format!("{}/manifest_program.so", manifest_fixtures_dir()));

    let mollusk =
        setup_mollusk_with_programs(&beethoven_bytes, &[(MANIFEST_PROGRAM_ID, &manifest_bytes)]);

    let (market_addr, market_account) = load_json_fixture(&format!(
        "{}/manifest_usdc_sol_market.json",
        manifest_fixtures_dir()
    ));
    let (wsol_mint_addr, wsol_mint_account) =
        load_json_fixture(&format!("{}/wsol_mint.json", common_fixtures_dir()));
    let (usdc_mint_addr, usdc_mint_account) =
        load_json_fixture(&format!("{}/usdc_mint.json", common_fixtures_dir()));
    let (base_vault_addr, base_vault_account) = load_json_fixture(&format!(
        "{}/manifest_sol_usdc_base_vault.json",
        manifest_fixtures_dir()
    ));
    let (quote_vault_addr, quote_vault_account) = load_json_fixture(&format!(
        "{}/manifest_sol_usdc_quote_vault.json",
        manifest_fixtures_dir()
    ));
    let (global_addr, global_account) =
        load_json_fixture(&format!("{}/manifest_global.json", manifest_fixtures_dir()));
    let (global_vault_addr, global_vault_account) = load_json_fixture(&format!(
        "{}/manifest_global_vault.json",
        manifest_fixtures_dir()
    ));

    assert_eq!(market_addr, MARKET);
    assert_eq!(wsol_mint_addr, WSOL_MINT);
    assert_eq!(usdc_mint_addr, USDC_MINT);
    assert_eq!(base_vault_addr, BASE_VAULT);
    assert_eq!(quote_vault_addr, QUOTE_VAULT);
    assert_eq!(global_addr, GLOBAL);
    assert_eq!(global_vault_addr, GLOBAL_VAULT);

    let payer = Address::new_from_array([0x02; 32]);
    let payer_account = Account::new(10_000_000_000u64, 0, &Address::default());

    let trader_base_addr = Address::new_from_array([0x03; 32]);
    let initial_wsol = 1_000_000_000u64;
    let trader_base_account = create_account_for_token_account(
        TokenAccount {
            mint: wsol_mint_addr,
            owner: payer,
            amount: initial_wsol,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        false,
    );

    let trader_quote_addr = Address::new_from_array([0x04; 32]);
    let initial_usdc = 0u64;
    let trader_quote_account = create_account_for_token_account(
        TokenAccount {
            mint: usdc_mint_addr,
            owner: payer,
            amount: initial_usdc,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        },
        false,
    );

    let in_amount = 100_000_000u64;
    let min_out_amount = 1u64;

    let account_metas = vec![
        AccountMeta::new_readonly(MANIFEST_PROGRAM_ID, false),
        AccountMeta::new(payer, true),
        AccountMeta::new_readonly(payer, true),
        AccountMeta::new(market_addr, false),
        AccountMeta::new_readonly(solana_sdk_ids::system_program::ID, false),
        AccountMeta::new(trader_base_addr, false),
        AccountMeta::new(trader_quote_addr, false),
        AccountMeta::new(base_vault_addr, false),
        AccountMeta::new(quote_vault_addr, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(wsol_mint_addr, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(usdc_mint_addr, false),
        AccountMeta::new(global_addr, false),
        AccountMeta::new(global_vault_addr, false),
    ];

    let extra_data = [1u8, 1u8];
    let instruction = build_swap_instruction(
        account_metas,
        in_amount,
        min_out_amount,
        SwapProtocolTag::Manifest,
        &extra_data,
    );

    let (system_program_id, system_program_account) = get_mollusk_system_program();
    let (token_program_id, token_program_account) = get_mollusk_token_program();

    let manifest_program_account = create_mollusk_program_account(&manifest_bytes);

    let accounts = vec![
        (payer, payer_account),
        (market_addr, market_account),
        (wsol_mint_addr, wsol_mint_account),
        (usdc_mint_addr, usdc_mint_account),
        (trader_base_addr, trader_base_account),
        (trader_quote_addr, trader_quote_account),
        (base_vault_addr, base_vault_account),
        (quote_vault_addr, quote_vault_account),
        (global_addr, global_account),
        (global_vault_addr, global_vault_account),
        (system_program_id, system_program_account),
        (token_program_id, token_program_account),
        (MANIFEST_PROGRAM_ID, manifest_program_account),
    ];

    let result = mollusk.process_instruction(&instruction, &accounts);

    assert_mollusk_success(&result);

    for (pubkey, account) in &result.resulting_accounts {
        if *pubkey == trader_base_addr {
            let token_data =
                TokenAccount::unpack(&account.data).expect("Failed to unpack trader_base");
            assert!(
                token_data.amount < initial_wsol,
                "WSOL should have decreased"
            );
        }
        if *pubkey == trader_quote_addr {
            let token_data =
                TokenAccount::unpack(&account.data).expect("Failed to unpack trader_quote");
            assert!(
                token_data.amount > initial_usdc,
                "USDC should have increased"
            );
        }
    }
}
