use {
    crate::helper::*,
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

// JitoSOL as the test LST
const JITOSOL_MINT: Address = address!("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn");
const HYUSD_MINT: Address = address!("5YMkXAYccHSGnHn9nob9xEvv6Pvka9DZWH7nTbotTu9E");
const XSOL_MINT: Address = address!("4sWNB8zGWHkh6UnmwiEtzNxL4XrN7uK9tosbESbJFfVs");
const SOL_USD_PYTH_FEED: Address = address!("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");

fn derive_pda(seeds: &[&[u8]]) -> Address {
    let (addr, _) = Address::find_program_address(seeds, &HYLO_PROGRAM_ID);
    addr
}

fn hylo_ata(owner: &Address, mint: &Address) -> Address {
    // SPL ATA derivation: [owner, TOKEN_PROGRAM_ID, mint] with ASSOCIATED_TOKEN_PROGRAM_ID
    let (addr, _) = Address::find_program_address(
        &[owner.as_ref(), TOKEN_PROGRAM_ID.as_ref(), mint.as_ref()],
        &ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    addr
}

/// Build extra_data for a Hylo deposit: [mint_type(1), expected_token_out(8), slippage_tolerance(8)]
fn build_hylo_extra_data(
    mint_type: u8,
    expected_token_out: u64,
    slippage_tolerance: u64,
) -> Vec<u8> {
    let mut data = Vec::with_capacity(17);
    data.push(mint_type);
    data.extend_from_slice(&expected_token_out.to_le_bytes());
    data.extend_from_slice(&slippage_tolerance.to_le_bytes());
    data
}

fn load_hylo_protocol_fixtures(svm: &mut litesvm::LiteSVM) {
    let dir = hylo_fixtures_dir();

    // Warp to a slot past the Pyth oracle's posted_slot (dumped from mainnet ~411M).
    // Set epoch to match hylo_state (offset 304).
    svm.warp_to_slot(420_000_000);
    let mut clock = svm.get_sysvar::<Clock>();
    clock.epoch = 952;
    clock.unix_timestamp = 1_775_554_200;
    svm.set_sysvar::<Clock>(&clock);
    // Verify the clock is set correctly
    let verify_clock = svm.get_sysvar::<Clock>();
    assert_eq!(
        verify_clock.slot, 420_000_000,
        "Clock slot not set correctly"
    );
    assert_eq!(verify_clock.epoch, 952, "Clock epoch not set correctly");

    // Load Hylo program
    load_program(svm, HYLO_PROGRAM_ID, &format!("{dir}/hylo_program.so"));

    // Load protocol state accounts (dumped from mainnet)
    load_and_set_json_fixture(svm, &format!("{dir}/hylo_state.json"));
    load_and_set_json_fixture(svm, &format!("{dir}/lst_header.json"));

    // Authority PDAs — these are signing-only addresses with no on-chain data.
    // The Hylo program derives them internally; they just need to exist in the SVM.
    let fee_auth = derive_pda(&[b"fee_auth", JITOSOL_MINT.as_ref()]);
    let vault_auth = derive_pda(&[b"vault_auth", JITOSOL_MINT.as_ref()]);
    let stablecoin_auth = derive_pda(&[b"mint_auth", HYUSD_MINT.as_ref()]);
    let levercoin_auth = derive_pda(&[b"mint_auth", XSOL_MINT.as_ref()]);
    let event_authority = derive_pda(&[b"__event_authority"]);
    create_mock_account_at(svm, fee_auth, &HYLO_PROGRAM_ID, vec![]);
    create_mock_account_at(svm, vault_auth, &HYLO_PROGRAM_ID, vec![]);
    create_mock_account_at(svm, stablecoin_auth, &HYLO_PROGRAM_ID, vec![]);
    create_mock_account_at(svm, levercoin_auth, &HYLO_PROGRAM_ID, vec![]);
    create_mock_account_at(svm, event_authority, &HYLO_PROGRAM_ID, vec![]);

    // Load vault ATAs (dumped from mainnet)
    load_and_set_json_fixture(svm, &format!("{dir}/fee_vault.json"));
    load_and_set_json_fixture(svm, &format!("{dir}/lst_vault.json"));

    // Load mints (dumped from mainnet)
    load_and_set_json_fixture(svm, &format!("{dir}/jitosol_mint.json"));
    load_and_set_json_fixture(svm, &format!("{dir}/hyusd_mint.json"));
    load_and_set_json_fixture(svm, &format!("{dir}/xsol_mint.json"));

    // Load oracle (dumped from mainnet)
    load_and_set_json_fixture(svm, &format!("{dir}/sol_usd_pyth_feed.json"));
}

fn build_stablecoin_accounts(
    payer: &Address,
    user_lst_ta: &Address,
    user_output_ta: &Address,
) -> Vec<AccountMeta> {
    let hylo_state = derive_pda(&[b"hylo"]);
    let fee_auth = derive_pda(&[b"fee_auth", JITOSOL_MINT.as_ref()]);
    let vault_auth = derive_pda(&[b"vault_auth", JITOSOL_MINT.as_ref()]);
    let stablecoin_auth = derive_pda(&[b"mint_auth", HYUSD_MINT.as_ref()]);
    let lst_header = derive_pda(&[b"lst_header", JITOSOL_MINT.as_ref()]);
    let event_authority = derive_pda(&[b"__event_authority"]);
    let fee_vault = hylo_ata(&fee_auth, &JITOSOL_MINT);
    let lst_vault = hylo_ata(&vault_auth, &JITOSOL_MINT);

    vec![
        AccountMeta::new_readonly(HYLO_PROGRAM_ID, false), // detection prefix
        AccountMeta::new(*payer, true),                    // user (signer)
        AccountMeta::new(hylo_state, false),               // hylo state
        AccountMeta::new_readonly(fee_auth, false),        // fee_auth
        AccountMeta::new_readonly(vault_auth, false),      // vault_auth
        AccountMeta::new_readonly(stablecoin_auth, false), // coin_auth
        AccountMeta::new(fee_vault, false),                // fee_vault
        AccountMeta::new(lst_vault, false),                // lst_vault
        AccountMeta::new_readonly(lst_header, false),      // lst_header
        AccountMeta::new(*user_lst_ta, false),             // user_lst_ta
        AccountMeta::new(*user_output_ta, false),          // user_output_ta
        AccountMeta::new_readonly(JITOSOL_MINT, false),    // lst_mint
        AccountMeta::new(HYUSD_MINT, false),               // output_mint (hyUSD)
        AccountMeta::new_readonly(SOL_USD_PYTH_FEED, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new_readonly(event_authority, false),
        AccountMeta::new_readonly(HYLO_PROGRAM_ID, false), // program (Anchor CPI)
    ]
}

fn build_levercoin_accounts(
    payer: &Address,
    user_lst_ta: &Address,
    user_output_ta: &Address,
) -> Vec<AccountMeta> {
    let hylo_state = derive_pda(&[b"hylo"]);
    let fee_auth = derive_pda(&[b"fee_auth", JITOSOL_MINT.as_ref()]);
    let vault_auth = derive_pda(&[b"vault_auth", JITOSOL_MINT.as_ref()]);
    let levercoin_auth = derive_pda(&[b"mint_auth", XSOL_MINT.as_ref()]);
    let lst_header = derive_pda(&[b"lst_header", JITOSOL_MINT.as_ref()]);
    let event_authority = derive_pda(&[b"__event_authority"]);
    let fee_vault = hylo_ata(&fee_auth, &JITOSOL_MINT);
    let lst_vault = hylo_ata(&vault_auth, &JITOSOL_MINT);

    vec![
        AccountMeta::new_readonly(HYLO_PROGRAM_ID, false), // detection prefix
        AccountMeta::new(*payer, true),                    // user (signer)
        AccountMeta::new(hylo_state, false),               // hylo state
        AccountMeta::new_readonly(fee_auth, false),        // fee_auth
        AccountMeta::new_readonly(vault_auth, false),      // vault_auth
        AccountMeta::new_readonly(levercoin_auth, false),  // coin_auth (levercoin)
        AccountMeta::new(fee_vault, false),                // fee_vault
        AccountMeta::new(lst_vault, false),                // lst_vault
        AccountMeta::new_readonly(lst_header, false),      // lst_header
        AccountMeta::new(*user_lst_ta, false),             // user_lst_ta
        AccountMeta::new(*user_output_ta, false),          // user_output_ta
        AccountMeta::new_readonly(JITOSOL_MINT, false),    // lst_mint
        AccountMeta::new(XSOL_MINT, false),                // output_mint (xSOL)
        AccountMeta::new_readonly(HYUSD_MINT, false),      // stablecoin_mint (extra for levercoin)
        AccountMeta::new_readonly(SOL_USD_PYTH_FEED, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new_readonly(event_authority, false),
        AccountMeta::new_readonly(HYLO_PROGRAM_ID, false), // program (Anchor CPI)
    ]
}

// Ignored: Hylo's Pyth oracle slot check requires a real validator slot context
// that LiteSVM cannot provide (Clock::slot is not exposed to CPI programs).
// The CPI wiring and account deserialization are verified (reaches MintStablecoin).
#[test]
#[ignore]
fn test_hylo_mint_stablecoin_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    // Load beethoven-test program (our CPI wrapper)
    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load all Hylo protocol fixtures
    load_hylo_protocol_fixtures(&mut svm);

    // Create user token accounts
    let initial_lst = 1_000_000_000u64; // 1 JitoSOL (9 decimals)
    let initial_hyusd = 0u64;
    let user_lst_ta = create_token_account(&mut svm, &payer.pubkey(), &JITOSOL_MINT, initial_lst);
    let user_output_ta =
        create_token_account(&mut svm, &payer.pubkey(), &HYUSD_MINT, initial_hyusd);

    // Verify initial balances
    assert_eq!(get_token_balance(&svm, &user_lst_ta), initial_lst);
    assert_eq!(get_token_balance(&svm, &user_output_ta), initial_hyusd);

    // Build deposit instruction: mint_type=0 (stablecoin/hyUSD), small amount
    let deposit_amount = 1_000_000u64; // 0.001 JitoSOL
    let extra_data = build_hylo_extra_data(0, 0, 0); // no slippage config

    let accounts = build_stablecoin_accounts(&payer.pubkey(), &user_lst_ta, &user_output_ta);
    let instruction = build_deposit_instruction(accounts, deposit_amount, &extra_data);

    // Execute the deposit via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_lst = get_token_balance(&svm, &user_lst_ta);
            let final_hyusd = get_token_balance(&svm, &user_output_ta);

            assert!(
                final_lst < initial_lst,
                "JitoSOL should have decreased: {} -> {}",
                initial_lst,
                final_lst
            );
            assert!(
                final_hyusd > initial_hyusd,
                "hyUSD should have increased: {} -> {}",
                initial_hyusd,
                final_hyusd
            );

            println!(
                "Hylo stablecoin deposit successful! JitoSOL: {} -> {}, hyUSD: {} -> {}",
                initial_lst, final_lst, initial_hyusd, final_hyusd
            );
        }
        Err(e) => {
            panic!("Hylo stablecoin deposit CPI failed: {}", e);
        }
    }
}

#[test]
#[ignore]
fn test_hylo_mint_levercoin_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    // Load beethoven-test program (our CPI wrapper)
    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load all Hylo protocol fixtures
    load_hylo_protocol_fixtures(&mut svm);

    // Create user token accounts
    let initial_lst = 1_000_000_000u64; // 1 JitoSOL (9 decimals)
    let initial_xsol = 0u64;
    let user_lst_ta = create_token_account(&mut svm, &payer.pubkey(), &JITOSOL_MINT, initial_lst);
    let user_output_ta = create_token_account(&mut svm, &payer.pubkey(), &XSOL_MINT, initial_xsol);

    // Verify initial balances
    assert_eq!(get_token_balance(&svm, &user_lst_ta), initial_lst);
    assert_eq!(get_token_balance(&svm, &user_output_ta), initial_xsol);

    // Build deposit instruction: mint_type=1 (levercoin/xSOL), small amount
    let deposit_amount = 1_000_000u64; // 0.001 JitoSOL
    let extra_data = build_hylo_extra_data(1, 0, 0); // no slippage config

    let accounts = build_levercoin_accounts(&payer.pubkey(), &user_lst_ta, &user_output_ta);
    let instruction = build_deposit_instruction(accounts, deposit_amount, &extra_data);

    // Execute the deposit via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_lst = get_token_balance(&svm, &user_lst_ta);
            let final_xsol = get_token_balance(&svm, &user_output_ta);

            assert!(
                final_lst < initial_lst,
                "JitoSOL should have decreased: {} -> {}",
                initial_lst,
                final_lst
            );
            assert!(
                final_xsol > initial_xsol,
                "xSOL should have increased: {} -> {}",
                initial_xsol,
                final_xsol
            );

            println!(
                "Hylo levercoin deposit successful! JitoSOL: {} -> {}, xSOL: {} -> {}",
                initial_lst, final_lst, initial_xsol, final_xsol
            );
        }
        Err(e) => {
            panic!("Hylo levercoin deposit CPI failed: {}", e);
        }
    }
}
