use {
    crate::helper::*, solana_address::Address, solana_instruction::AccountMeta,
    solana_keypair::Keypair, solana_program_pack::Pack, solana_signer::Signer,
    spl_token_interface::state::Account as TokenAccount, std::str::FromStr,
};

// Known addresses from dumped fixtures
const WSOL_MINT: &str = "So11111111111111111111111111111111111111112";
const USDC_MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const MARKET: &str = "ENhU8LsaR7vDD2G1CsWcsuSGNrih9Cv5WZEk7q9kPapQ";
const BASE_VAULT: &str = "AKjfJDv4ywdpCDrj7AURuNkGA3696GTVFgrMwk4TjkKs";
const QUOTE_VAULT: &str = "FN9K6rTdWtRDUPmLTN2FnGvLZpHVNRN2MeRghKknSGDs";
const GLOBAL: &str = "7mR36vj6pvg1U1cRatvUbLG57yqsd1ojLbrgxb6azaQ1";
const GLOBAL_VAULT: &str = "E1mBVQyt7BHK8SaBSfME7usYxx94T4DtHEjbUpEBhZx";

fn common_fixtures_dir() -> String {
    format!("{}/fixtures/common", env!("CARGO_MANIFEST_DIR"))
}

fn manifest_fixtures_dir() -> String {
    format!("{}/fixtures/swap/manifest", env!("CARGO_MANIFEST_DIR"))
}

fn beethoven_program_path() -> String {
    format!(
        "{}/target/deploy/beethoven_test.so",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn get_token_balance(svm: &litesvm::LiteSVM, token_account: &Address) -> u64 {
    let account = svm
        .get_account(token_account)
        .expect("Token account not found");
    let token_data = TokenAccount::unpack(&account.data).expect("Failed to unpack token account");
    token_data.amount
}

#[test]
fn test_manifest_swap_loads_fixtures() {
    let mut svm = setup_svm();

    // Load Manifest program
    load_program(
        &mut svm,
        MANIFEST_PROGRAM_ID,
        &format!("{}/manifest_program.so", manifest_fixtures_dir()),
    );

    // Load market and vaults from manifest fixtures
    let market = load_and_set_json_fixture(
        &mut svm,
        &format!("{}/manifest_usdc_sol_market.json", manifest_fixtures_dir()),
    );
    assert_eq!(market, Address::from_str(MARKET).unwrap());

    // Load mints from common fixtures
    let wsol_mint = load_and_set_json_fixture(
        &mut svm,
        &format!("{}/wsol_mint.json", common_fixtures_dir()),
    );
    assert_eq!(wsol_mint, Address::from_str(WSOL_MINT).unwrap());

    let usdc_mint = load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    assert_eq!(usdc_mint, Address::from_str(USDC_MINT).unwrap());

    // Load vaults from manifest fixtures
    let base_vault = load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/manifest_sol_usdc_base_vault.json",
            manifest_fixtures_dir()
        ),
    );
    assert_eq!(base_vault, Address::from_str(BASE_VAULT).unwrap());

    let quote_vault = load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/manifest_sol_usdc_quote_vault.json",
            manifest_fixtures_dir()
        ),
    );
    assert_eq!(quote_vault, Address::from_str(QUOTE_VAULT).unwrap());

    // Verify accounts are loaded
    assert!(svm.get_account(&market).is_some());
    assert!(svm.get_account(&wsol_mint).is_some());
    assert!(svm.get_account(&usdc_mint).is_some());
    assert!(svm.get_account(&base_vault).is_some());
    assert!(svm.get_account(&quote_vault).is_some());
}

#[test]
fn test_manifest_swap_account_structure() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    // Load Manifest program and fixtures
    load_program(
        &mut svm,
        MANIFEST_PROGRAM_ID,
        &format!("{}/manifest_program.so", manifest_fixtures_dir()),
    );
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

    let wsol_mint = Address::from_str(WSOL_MINT).unwrap();
    let usdc_mint = Address::from_str(USDC_MINT).unwrap();
    let market = Address::from_str(MARKET).unwrap();
    let base_vault = Address::from_str(BASE_VAULT).unwrap();
    let quote_vault = Address::from_str(QUOTE_VAULT).unwrap();

    // Create trader token accounts
    let trader_base = create_token_account(&mut svm, &payer.pubkey(), &wsol_mint, 1_000_000_000); // 1 SOL
    let trader_quote = create_token_account(&mut svm, &payer.pubkey(), &usdc_mint, 100_000_000); // 100 USDC

    // Build account metas for Manifest SwapV2
    // Account order from beethoven implementation:
    // 0. manifest_program - for protocol detection
    // 1. payer - writable, signer
    // 2. owner - signer (same as payer for simple case)
    // 3. market - writable
    // 4. system_program
    // 5. trader_base - writable
    // 6. trader_quote - writable
    // 7. base_vault - writable
    // 8. quote_vault - writable
    // 9. token_program_base
    // 10. base_mint (optional, set to program ID for non-Token22)
    // 11. token_program_quote (optional, same as token_program_base)
    // 12. quote_mint (optional, set to program ID for non-Token22)
    // 13. global (optional, set to program ID)
    // 14. global_vault (optional, set to program ID)
    let accounts = vec![
        AccountMeta::new_readonly(MANIFEST_PROGRAM_ID, false), // manifest_program (for detection)
        AccountMeta::new(payer.pubkey(), true),                // payer
        AccountMeta::new_readonly(payer.pubkey(), true),       // owner (same as payer)
        AccountMeta::new(market, false),                       // market
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),   // system_program
        AccountMeta::new(trader_base, false),                  // trader_base
        AccountMeta::new(trader_quote, false),                 // trader_quote
        AccountMeta::new(base_vault, false),                   // base_vault
        AccountMeta::new(quote_vault, false),                  // quote_vault
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),    // token_program_base
        AccountMeta::new_readonly(MANIFEST_PROGRAM_ID, false), // base_mint (optional -> program ID)
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),    // token_program_quote
        AccountMeta::new_readonly(MANIFEST_PROGRAM_ID, false), // quote_mint (optional -> program ID)
        AccountMeta::new(MANIFEST_PROGRAM_ID, false),          // global (optional -> program ID)
        AccountMeta::new(MANIFEST_PROGRAM_ID, false), // global_vault (optional -> program ID)
    ];

    // ManifestSwapData: is_base_in (sell SOL), is_exact_in (exact input amount)
    let extra_data = [1u8, 1u8]; // is_base_in=true, is_exact_in=true

    // Just verify instruction builds correctly (actual swap needs beethoven-test program)
    let instruction = build_swap_instruction(
        accounts,
        100_000_000, // in_amount: 0.1 SOL
        1_000_000,   // min_out_amount: 1 USDC (very loose slippage)
        &extra_data,
    );

    assert_eq!(instruction.program_id, TEST_PROGRAM_ID);
    assert_eq!(instruction.accounts.len(), 15);
    // Data: discriminator(1) + in_amount(8) + min_out_amount(8) + extra_data(2) = 19 bytes
    assert_eq!(instruction.data.len(), 19);
}

#[test]
fn test_manifest_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    // Load beethoven-test program (our program that does CPI)
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

    let wsol_mint = Address::from_str(WSOL_MINT).unwrap();
    let usdc_mint = Address::from_str(USDC_MINT).unwrap();
    let market = Address::from_str(MARKET).unwrap();
    let base_vault = Address::from_str(BASE_VAULT).unwrap();
    let quote_vault = Address::from_str(QUOTE_VAULT).unwrap();
    let global = Address::from_str(GLOBAL).unwrap();
    let global_vault = Address::from_str(GLOBAL_VAULT).unwrap();

    // Debug: verify account owners
    let market_account = svm.get_account(&market).expect("Market not found");
    println!("Market {} owner: {}", market, market_account.owner);

    let base_vault_account = svm.get_account(&base_vault).expect("Base vault not found");
    println!(
        "Base vault {} owner: {}",
        base_vault, base_vault_account.owner
    );

    let quote_vault_account = svm
        .get_account(&quote_vault)
        .expect("Quote vault not found");
    println!(
        "Quote vault {} owner: {}",
        quote_vault, quote_vault_account.owner
    );

    assert_eq!(
        market_account.owner, MANIFEST_PROGRAM_ID,
        "Market should be owned by Manifest program"
    );

    // Create trader token accounts with initial balances
    let initial_wsol = 1_000_000_000u64; // 1 SOL in lamports
    let initial_usdc = 0u64;
    let trader_base = create_token_account(&mut svm, &payer.pubkey(), &wsol_mint, initial_wsol);
    let trader_quote = create_token_account(&mut svm, &payer.pubkey(), &usdc_mint, initial_usdc);

    // Verify initial balances
    assert_eq!(get_token_balance(&svm, &trader_base), initial_wsol);
    assert_eq!(get_token_balance(&svm, &trader_quote), initial_usdc);

    // Build swap instruction: sell 0.1 SOL for USDC
    let in_amount = 100_000_000u64; // 0.1 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    let accounts = vec![
        AccountMeta::new_readonly(MANIFEST_PROGRAM_ID, false), // manifest_program (for detection)
        AccountMeta::new(payer.pubkey(), true),                // payer
        AccountMeta::new_readonly(payer.pubkey(), true),       // owner
        AccountMeta::new(market, false),                       // market
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),   // system_program
        AccountMeta::new(trader_base, false),                  // trader_base (SOL)
        AccountMeta::new(trader_quote, false),                 // trader_quote (USDC)
        AccountMeta::new(base_vault, false),                   // base_vault
        AccountMeta::new(quote_vault, false),                  // quote_vault
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),    // token_program_base
        AccountMeta::new_readonly(wsol_mint, false),           // base_mint
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),    // token_program_quote
        AccountMeta::new_readonly(usdc_mint, false),           // quote_mint
        AccountMeta::new(global, false),                       // global
        AccountMeta::new(global_vault, false),                 // global_vault
    ];

    // is_base_in=true (selling base/SOL), is_exact_in=true (exact input amount)
    let extra_data = [1u8, 1u8];

    let instruction = build_swap_instruction(accounts, in_amount, min_out_amount, &extra_data);

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(()) => {
            // Verify balances changed
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

            println!(
                "Swap successful! WSOL: {} -> {}, USDC: {} -> {}",
                initial_wsol, final_wsol, initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("Swap CPI failed: {}", e);
        }
    }
}
