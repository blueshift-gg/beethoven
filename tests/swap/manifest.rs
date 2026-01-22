use {
    crate::helper::*,
    solana_account::Account,
    solana_address::Address,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_program_option::COption,
    solana_program_pack::Pack,
    solana_signer::Signer,
    spl_token_interface::state::{Account as TokenAccount, AccountState},
    std::str::FromStr,
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

#[cfg(feature = "upstream-bpf")]
fn beethoven_program_path() -> String {
    format!(
        "{}/target/bpfel-unknown-none/release/libbeethoven_test.so",
        env!("CARGO_MANIFEST_DIR")
    )
}

#[cfg(not(feature = "upstream-bpf"))]
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
        Ok(_compute_units) => {
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

#[test]
fn test_manifest_swap_cpi_mollusk() {
    // Load program bytes
    let beethoven_bytes = load_fixture_bytes(&beethoven_program_path());
    let manifest_bytes =
        load_fixture_bytes(&format!("{}/manifest_program.so", manifest_fixtures_dir()));

    // Set up mollusk with both programs
    let mollusk =
        setup_mollusk_with_programs(&beethoven_bytes, &[(MANIFEST_PROGRAM_ID, &manifest_bytes)]);

    // Load fixtures
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

    // Verify addresses match expected
    assert_eq!(market_addr, Address::from_str(MARKET).unwrap());
    assert_eq!(wsol_mint_addr, Address::from_str(WSOL_MINT).unwrap());
    assert_eq!(usdc_mint_addr, Address::from_str(USDC_MINT).unwrap());
    assert_eq!(base_vault_addr, Address::from_str(BASE_VAULT).unwrap());
    assert_eq!(quote_vault_addr, Address::from_str(QUOTE_VAULT).unwrap());
    assert_eq!(global_addr, Address::from_str(GLOBAL).unwrap());
    assert_eq!(global_vault_addr, Address::from_str(GLOBAL_VAULT).unwrap());

    // Create payer/owner address
    let payer = Address::new_from_array([0x02; 32]);
    let payer_account = Account::new(10_000_000_000u64, 0, &Address::default());

    // Create trader token accounts
    let trader_base_addr = Address::new_from_array([0x03; 32]);
    let initial_wsol = 1_000_000_000u64; // 1 SOL
    let trader_base_account = create_account_for_token_account(TokenAccount {
        mint: wsol_mint_addr,
        owner: payer,
        amount: initial_wsol,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    });

    let trader_quote_addr = Address::new_from_array([0x04; 32]);
    let initial_usdc = 0u64;
    let trader_quote_account = create_account_for_token_account(TokenAccount {
        mint: usdc_mint_addr,
        owner: payer,
        amount: initial_usdc,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    });

    // Build swap instruction: sell 0.1 SOL for USDC
    let in_amount = 100_000_000u64; // 0.1 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    let account_metas = vec![
        AccountMeta::new_readonly(MANIFEST_PROGRAM_ID, false), // manifest_program (for detection)
        AccountMeta::new(payer, true),                         // payer
        AccountMeta::new_readonly(payer, true),                // owner
        AccountMeta::new(market_addr, false),                  // market
        AccountMeta::new_readonly(solana_sdk_ids::system_program::ID, false), // system_program
        AccountMeta::new(trader_base_addr, false),             // trader_base (SOL)
        AccountMeta::new(trader_quote_addr, false),            // trader_quote (USDC)
        AccountMeta::new(base_vault_addr, false),              // base_vault
        AccountMeta::new(quote_vault_addr, false),             // quote_vault
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),    // token_program_base
        AccountMeta::new_readonly(wsol_mint_addr, false),      // base_mint
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),    // token_program_quote
        AccountMeta::new_readonly(usdc_mint_addr, false),      // quote_mint
        AccountMeta::new(global_addr, false),                  // global
        AccountMeta::new(global_vault_addr, false),            // global_vault
    ];

    // is_base_in=true (selling base/SOL), is_exact_in=true (exact input amount)
    let extra_data = [1u8, 1u8];
    let instruction = build_swap_instruction(account_metas, in_amount, min_out_amount, &extra_data);

    // Get system program and token program keyed accounts
    let (system_program_id, system_program_account) = get_mollusk_system_program();
    let (token_program_id, token_program_account) = get_mollusk_token_program();

    // Manifest program account (needed for instruction account reference)
    let manifest_program_account = create_mollusk_program_account(&manifest_bytes);

    // Build accounts list for mollusk
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

    // Execute the instruction
    let result = mollusk.process_instruction(&instruction, &accounts);

    // Verify success
    assert_mollusk_success(&result);

    // Check resulting account data
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

    println!(
        "Mollusk manifest swap CPI succeeded! Compute units: {}",
        result.compute_units_consumed
    );
}
