use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, create_token_account, get_token_balance,
        hylo_exchange_fixtures_dir, load_and_set_json_fixture, load_program, send_transaction,
        setup_svm, ASSOCIATED_TOKEN_PROGRAM_ID, HYLO_EXCHANGE_PROGRAM_ID, SYSTEM_PROGRAM_ID,
        TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    litesvm::LiteSVM,
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const JITOSOL_MINT: Address = address!("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn");
const JITOSOL_FEE_AUTH: Address = address!("FpLaqELxKRm6S3bjfNSknwZu43TL89VYkwuMDwsRMj59");
const JITOSOL_VAULT_AUTH: Address = address!("82MNhUCha26wY4kohTUEC965b4ypEe7RPa4itp9UMrKK");
const HYUSD_STABLECOIN_AUTH: Address = address!("CfuSViqf6wvUKEprLhtuCsSanvfAsMbDmkAW92FP95qe");
const XSOL_LEVERCOIN_AUTH: Address = address!("J8rGkrzsvqinX9kfwD8SkP3mRzXAk3uiDRUaiZKXM4as");
const HYUSD_FEE_AUTH: Address = address!("3HT6dD6APJh89XJs9rkn3BmsvkXE9jPG9dWJmUjWu6TS");
const HYUSD_FEE_VAULT: Address = address!("Hh8N3Fdauxgq1jjcKdzGBR3D8cdkpLZrFEVumL1tYQLp");
const JITOSOL_FEE_VAULT: Address = address!("3JENUTyYnMMtZUSg5ErSHEvowjQteYD7wr7RDNw12bei");
const JITOSOL_VAULT: Address = address!("2Y3TLkdGoJwbdizxqrZmQwNLYJyGKTgzC4tbetbkvQ43");
const JITOSOL_LST_HEADER: Address = address!("8Ri52tZXZehgAHKbx1MQiXhWXXkVsvAL9op6C5HytDKF");
const HYLO: Address = address!("9cd2sAfbBvKs4SX9YKo4dcjwP3TgTVQ8dT5koshGcDND");
const HYUSD_MINT: Address = address!("5YMkXAYccHSGnHn9nob9xEvv6Pvka9DZWH7nTbotTu9E");
const XSOL_MINT: Address = address!("4sWNB8zGWHkh6UnmwiEtzNxL4XrN7uK9tosbESbJFfVs");
const SOL_PRICE_UPDATE_V2: Address = address!("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");
const EVENT_AUTHORITY: Address = address!("4VzpNE51Be5vD5Yg8MC3z6TVHq5gGbLJptjv18QbD6WP");

fn load_program_and_fixtures(svm: &mut LiteSVM) {
    load_program(svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Hylo Exchange program
    load_program(
        svm,
        HYLO_EXCHANGE_PROGRAM_ID,
        &format!("{}/hylo_exchange.so", hylo_exchange_fixtures_dir()),
    );

    // Load fixtures (union of stablecoin + levercoin + leverage)
    load_and_set_json_fixture(
        svm,
        &format!("{}/hyusd_mint.json", hylo_exchange_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/jitosol_mint.json", hylo_exchange_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/xsol_mint.json", hylo_exchange_fixtures_dir()),
    );
    load_and_set_json_fixture(svm, &format!("{}/hylo.json", hylo_exchange_fixtures_dir()));
    load_and_set_json_fixture(
        svm,
        &format!("{}/jitosol_fee_vault.json", hylo_exchange_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/jitosol_vault.json", hylo_exchange_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/jitosol_lst_header.json", hylo_exchange_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/hyusd_fee_vault.json", hylo_exchange_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/sol_price_update_v2.json", hylo_exchange_fixtures_dir()),
    );
}

fn override_state(svm: &mut LiteSVM) {
    let mut clock = svm.get_sysvar::<Clock>();
    // Jump ahead by Hylo total_sol_cache current_update_epoch (953)
    clock.epoch = 953;
    // Jump ahead by PriceUpdateV2 posted_slot (411_819_630)
    clock.slot = 411_819_630;
    // Jump ahead by PriceUpdateV2 price_message published_time (1_775_639_108) + 1
    clock.unix_timestamp = 1_775_639_108 + 1;
    svm.set_sysvar::<Clock>(&clock);
}

#[test]
fn test_hylo_exchange_mint_stablecoin_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program_and_fixtures(&mut svm);
    override_state(&mut svm);

    // Selling JITOSOL (input) for HYUSD (output)
    let initial_jitosol = 1_000_000_000u64; // 1 SOL
    let initial_hyusd = 0u64;
    let trader_input =
        create_token_account(&mut svm, &payer.pubkey(), &JITOSOL_MINT, initial_jitosol);
    let trader_output = create_token_account(&mut svm, &payer.pubkey(), &HYUSD_MINT, initial_hyusd);

    let in_amount = 1_000_000u64; // 0.001 JITOSOL
    let min_out_amount = 1u64;

    // mint_stablecoin accounts layout (19 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(HYLO_EXCHANGE_PROGRAM_ID, false), // hylo_exchange_program
        AccountMeta::new(payer.pubkey(), true),                     // user
        AccountMeta::new(HYLO, false),                              // hylo
        AccountMeta::new_readonly(JITOSOL_FEE_AUTH, false),         // fee_auth
        AccountMeta::new_readonly(JITOSOL_VAULT_AUTH, false),       // vault_auth
        AccountMeta::new(HYUSD_STABLECOIN_AUTH, false),             // stablecoin_auth
        AccountMeta::new(JITOSOL_FEE_VAULT, false),                 // fee_vault
        AccountMeta::new(JITOSOL_VAULT, false),                     // lst_vault
        AccountMeta::new_readonly(JITOSOL_LST_HEADER, false),       // lst_header
        AccountMeta::new(trader_input, false),                      // user_lst_ta
        AccountMeta::new(trader_output, false),                     // user_stablecoin_ta
        AccountMeta::new_readonly(JITOSOL_MINT, false),             // lst_mint
        AccountMeta::new(HYUSD_MINT, false),                        // stablecoin_mint
        AccountMeta::new_readonly(SOL_PRICE_UPDATE_V2, false),      // sol_usd_pyth_feed
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),         // token_program
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false), // associated_token_program
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),        // system_program
        AccountMeta::new_readonly(EVENT_AUTHORITY, false),          // event_authority
        AccountMeta::new_readonly(HYLO_EXCHANGE_PROGRAM_ID, false), // program
    ];

    // swap_type = 0 (MintStablecoin)
    let extra_data: &[u8] = &[0_u8];

    let instruction = build_swap_instruction(accounts, in_amount, min_out_amount, extra_data);

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_jitosol = get_token_balance(&svm, &trader_input);
            let final_hyusd = get_token_balance(&svm, &trader_output);

            assert!(
                final_jitosol < initial_jitosol,
                "JITOSOL should have decreased: {} -> {}",
                initial_jitosol,
                final_jitosol
            );
            assert!(
                final_hyusd > initial_hyusd,
                "HYUSD should have increased: {} -> {}",
                initial_hyusd,
                final_hyusd
            );

            println!(
                "Hylo Exchange swap successful! JITOSOL: {} -> {}, HYUSD: {} -> {}",
                initial_jitosol, final_jitosol, initial_hyusd, final_hyusd
            );
        }
        Err(e) => panic!("Hylo Exchange swap CPI failed: {}", e),
    }
}

#[test]
fn test_hylo_exchange_redeem_stablecoin_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program_and_fixtures(&mut svm);
    override_state(&mut svm);

    // Selling HYUSD (input) for JITOSOL (output)
    let initial_hyusd = 1_000_000u64; // 1 hyUSD (6 decimals)
    let initial_jitosol = 0u64;
    let trader_input = create_token_account(&mut svm, &payer.pubkey(), &HYUSD_MINT, initial_hyusd);
    let trader_output =
        create_token_account(&mut svm, &payer.pubkey(), &JITOSOL_MINT, initial_jitosol);

    let in_amount = 100_000u64; // 0.10 hyUSD
    let min_out_amount = 1u64;

    // redeem_stablecoin accounts layout (18 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(HYLO_EXCHANGE_PROGRAM_ID, false), // hylo_exchange_program
        AccountMeta::new(payer.pubkey(), true),                     // user
        AccountMeta::new(HYLO, false),                              // hylo
        AccountMeta::new_readonly(JITOSOL_FEE_AUTH, false),         // fee_auth
        AccountMeta::new_readonly(JITOSOL_VAULT_AUTH, false),       // vault_auth
        AccountMeta::new(JITOSOL_FEE_VAULT, false),                 // fee_vault
        AccountMeta::new(JITOSOL_VAULT, false),                     // lst_vault
        AccountMeta::new_readonly(JITOSOL_LST_HEADER, false),       // lst_header
        AccountMeta::new(trader_input, false),                      // user_stablecoin_ta
        AccountMeta::new(trader_output, false),                     // user_lst_ta
        AccountMeta::new(HYUSD_MINT, false),                        // stablecoin_mint
        AccountMeta::new_readonly(JITOSOL_MINT, false),             // lst_mint
        AccountMeta::new_readonly(SOL_PRICE_UPDATE_V2, false),      // sol_usd pyth feed
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),        // system_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),         // token_program
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false), // associated_token_program
        AccountMeta::new_readonly(EVENT_AUTHORITY, false),          // event_authority
        AccountMeta::new_readonly(HYLO_EXCHANGE_PROGRAM_ID, false), // program
    ];

    // swap_type = 1 (RedeemStablecoin)
    let extra_data: &[u8] = &[1_u8];

    let instruction = build_swap_instruction(accounts, in_amount, min_out_amount, extra_data);

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_hyusd = get_token_balance(&svm, &trader_input);
            let final_jitosol = get_token_balance(&svm, &trader_output);

            assert!(
                final_hyusd < initial_hyusd,
                "HYUSD should have decreased: {} -> {}",
                initial_hyusd,
                final_hyusd
            );
            assert!(
                final_jitosol > initial_jitosol,
                "JITOSOL should have increased: {} -> {}",
                initial_jitosol,
                final_jitosol
            );

            println!(
                "Hylo Exchange swap successful! HYUSD: {} -> {}, JITOSOL: {} -> {}",
                initial_hyusd, final_hyusd, initial_jitosol, final_jitosol
            );
        }
        Err(e) => panic!("Hylo Exchange swap CPI failed: {}", e),
    }
}

#[test]
fn test_hylo_exchange_mint_levercoin_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program_and_fixtures(&mut svm);
    override_state(&mut svm);

    // Selling JITOSOL (input) for XSOL (output)
    let initial_jitosol = 1_000_000_000u64; // 1 SOL
    let initial_xsol = 0u64;
    let trader_input =
        create_token_account(&mut svm, &payer.pubkey(), &JITOSOL_MINT, initial_jitosol);
    let trader_output = create_token_account(&mut svm, &payer.pubkey(), &XSOL_MINT, initial_xsol);

    let in_amount = 1_000_000u64; // 0.001 JITOSOL
    let min_out_amount = 1u64;

    // mint_levercoin accounts layout (20 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(HYLO_EXCHANGE_PROGRAM_ID, false), // hylo_exchange_program
        AccountMeta::new(payer.pubkey(), true),                     // user
        AccountMeta::new(HYLO, false),                              // hylo
        AccountMeta::new_readonly(JITOSOL_FEE_AUTH, false),         // fee_auth
        AccountMeta::new_readonly(JITOSOL_VAULT_AUTH, false),       // vault_auth
        AccountMeta::new_readonly(XSOL_LEVERCOIN_AUTH, false),      // levercoin_auth
        AccountMeta::new(JITOSOL_FEE_VAULT, false),                 // fee_vault
        AccountMeta::new(JITOSOL_VAULT, false),                     // lst_vault
        AccountMeta::new_readonly(JITOSOL_LST_HEADER, false),       // lst_header
        AccountMeta::new(trader_input, false),                      // user_lst_ta
        AccountMeta::new(trader_output, false),                     // user_levercoin_ta
        AccountMeta::new_readonly(JITOSOL_MINT, false),             // lst_mint
        AccountMeta::new(XSOL_MINT, false),                         // levercoin_mint
        AccountMeta::new_readonly(HYUSD_MINT, false),               // stablecoin_mint
        AccountMeta::new_readonly(SOL_PRICE_UPDATE_V2, false),      // sol_usd_pyth_feed
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),         // token_program
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false), // associated_token_program
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),        // system_program
        AccountMeta::new_readonly(EVENT_AUTHORITY, false),          // event_authority
        AccountMeta::new_readonly(HYLO_EXCHANGE_PROGRAM_ID, false), // program
    ];

    // swap_type = 2 (MintLevercoin)
    let extra_data: &[u8] = &[2_u8];

    let instruction = build_swap_instruction(accounts, in_amount, min_out_amount, extra_data);

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_jitosol = get_token_balance(&svm, &trader_input);
            let final_xsol = get_token_balance(&svm, &trader_output);

            assert!(
                final_jitosol < initial_jitosol,
                "JITOSOL should have decreased: {} -> {}",
                initial_jitosol,
                final_jitosol
            );
            assert!(
                final_xsol > initial_xsol,
                "XSOL should have increased: {} -> {}",
                initial_xsol,
                final_xsol
            );

            println!(
                "Hylo Exchange swap successful! JITOSOL: {} -> {}, XSOL: {} -> {}",
                initial_jitosol, final_jitosol, initial_xsol, final_xsol
            );
        }
        Err(e) => panic!("Hylo Exchange swap CPI failed: {}", e),
    }
}

#[test]
fn test_hylo_exchange_redeem_levercoin_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program_and_fixtures(&mut svm);
    override_state(&mut svm);

    // Selling XSOL (input) for JITOSOL (output)
    let initial_xsol = 1_000_000u64;
    let initial_jitosol = 0u64;
    let trader_input = create_token_account(&mut svm, &payer.pubkey(), &XSOL_MINT, initial_xsol);
    let trader_output =
        create_token_account(&mut svm, &payer.pubkey(), &JITOSOL_MINT, initial_jitosol);

    let in_amount = 100_000u64; // 0.10 xSOL
    let min_out_amount = 1u64;

    // redeem_levercoin accounts layout (19 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(HYLO_EXCHANGE_PROGRAM_ID, false), // hylo_exchange_program
        AccountMeta::new(payer.pubkey(), true),                     // user
        AccountMeta::new(HYLO, false),                              // hylo
        AccountMeta::new_readonly(JITOSOL_FEE_AUTH, false),         // fee_auth
        AccountMeta::new_readonly(JITOSOL_VAULT_AUTH, false),       // vault_auth
        AccountMeta::new(JITOSOL_FEE_VAULT, false),                 // fee_vault
        AccountMeta::new(JITOSOL_VAULT, false),                     // lst_vault
        AccountMeta::new_readonly(JITOSOL_LST_HEADER, false),       // lst_header
        AccountMeta::new(trader_input, false),                      // user_levercoin_ta
        AccountMeta::new(trader_output, false),                     // user_lst_ta
        AccountMeta::new(XSOL_MINT, false),                         // levercoin_mint
        AccountMeta::new_readonly(HYUSD_MINT, false),               // stablecoin_mint
        AccountMeta::new_readonly(JITOSOL_MINT, false),             // lst_mint
        AccountMeta::new_readonly(SOL_PRICE_UPDATE_V2, false),      // sol_usd_pyth_feed
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),        // system_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),         // token_program
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false), // associated_token_program
        AccountMeta::new_readonly(EVENT_AUTHORITY, false),          // event_authority
        AccountMeta::new_readonly(HYLO_EXCHANGE_PROGRAM_ID, false), // program
    ];

    // swap_type = 3 (RedeemLevercoin)
    let extra_data: &[u8] = &[3_u8];

    let instruction = build_swap_instruction(accounts, in_amount, min_out_amount, extra_data);

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_xsol = get_token_balance(&svm, &trader_input);
            let final_jitosol = get_token_balance(&svm, &trader_output);

            assert!(
                final_xsol < initial_xsol,
                "XSOL should have decreased: {} -> {}",
                initial_xsol,
                final_xsol
            );
            assert!(
                final_jitosol > initial_jitosol,
                "JITOSOL should have increased: {} -> {}",
                initial_jitosol,
                final_jitosol
            );

            println!(
                "Hylo Exchange swap successful! XSOL: {} -> {}, JITOSOL: {} -> {}",
                initial_xsol, final_xsol, initial_jitosol, final_jitosol
            );
        }
        Err(e) => panic!("Hylo Exchange swap CPI failed: {}", e),
    }
}

#[test]
fn test_hylo_exchange_stable_to_lever_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program_and_fixtures(&mut svm);
    override_state(&mut svm);

    // Selling hyUSD (input) for xSOL (output)
    let initial_hyusd = 10_000_000u64; // 10 hyUSD
    let initial_xsol = 0u64;
    let trader_input = create_token_account(&mut svm, &payer.pubkey(), &HYUSD_MINT, initial_hyusd);
    let trader_output = create_token_account(&mut svm, &payer.pubkey(), &XSOL_MINT, initial_xsol);

    let in_amount = 1_000_000u64; // 1 hyUSD
    let min_out_amount = 1u64;

    // stable_to_lever accounts layout (15 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(HYLO_EXCHANGE_PROGRAM_ID, false), // hylo_exchange_program
        AccountMeta::new(payer.pubkey(), true),                     // user
        AccountMeta::new(HYLO, false),                              // hylo
        AccountMeta::new_readonly(SOL_PRICE_UPDATE_V2, false),      // sol_usd_pyth_feed
        AccountMeta::new(HYUSD_MINT, false),                        // stablecoin_mint
        AccountMeta::new_readonly(HYUSD_STABLECOIN_AUTH, false),    // stablecoin_auth
        AccountMeta::new_readonly(HYUSD_FEE_AUTH, false),           // fee_auth
        AccountMeta::new(HYUSD_FEE_VAULT, false),                   // fee_vault
        AccountMeta::new(trader_input, false),                      // user_stablecoin_ta
        AccountMeta::new(XSOL_MINT, false),                         // levercoin_mint
        AccountMeta::new_readonly(XSOL_LEVERCOIN_AUTH, false),      // levercoin_auth
        AccountMeta::new(trader_output, false),                     // user_levercoin_ta
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),         // token_program
        AccountMeta::new_readonly(EVENT_AUTHORITY, false),          // event_authority
        AccountMeta::new_readonly(HYLO_EXCHANGE_PROGRAM_ID, false), // program
    ];

    // swap_type = 4 (SwapStableToLever)
    let extra_data: &[u8] = &[4_u8];

    let instruction = build_swap_instruction(accounts, in_amount, min_out_amount, extra_data);

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_hyusd = get_token_balance(&svm, &trader_input);
            let final_xsol = get_token_balance(&svm, &trader_output);

            assert!(
                final_hyusd < initial_hyusd,
                "hyUSD should have decreased: {} -> {}",
                initial_hyusd,
                final_hyusd
            );
            assert!(
                final_xsol > initial_xsol,
                "XSOL should have increased: {} -> {}",
                initial_xsol,
                final_xsol
            );

            println!(
                "Hylo Exchange swap successful! hyUSD: {} -> {}, XSOL: {} -> {}",
                initial_hyusd, final_hyusd, initial_xsol, final_xsol
            );
        }
        Err(e) => panic!("Hylo Exchange swap CPI failed: {}", e),
    }
}

#[test]
fn test_hylo_exchange_lever_to_stable_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program_and_fixtures(&mut svm);
    override_state(&mut svm);

    // Selling xSOL (input) for hyUSD (output)
    let initial_xsol = 10_000_000u64; // 10 xSOL
    let initial_hyusd = 0u64;
    let trader_input = create_token_account(&mut svm, &payer.pubkey(), &XSOL_MINT, initial_xsol);
    let trader_output = create_token_account(&mut svm, &payer.pubkey(), &HYUSD_MINT, initial_hyusd);

    let in_amount = 1_000_000u64; // 1 xSOL
    let min_out_amount = 1u64;

    // lever_to_stable accounts layout (15 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(HYLO_EXCHANGE_PROGRAM_ID, false), // hylo_exchange_program
        AccountMeta::new(payer.pubkey(), true),                     // user
        AccountMeta::new(HYLO, false),                              // hylo
        AccountMeta::new_readonly(SOL_PRICE_UPDATE_V2, false),      // sol_usd_pyth_feed
        AccountMeta::new(HYUSD_MINT, false),                        // stablecoin_mint
        AccountMeta::new_readonly(HYUSD_STABLECOIN_AUTH, false),    // stablecoin_auth
        AccountMeta::new_readonly(HYUSD_FEE_AUTH, false),           // fee_auth
        AccountMeta::new(HYUSD_FEE_VAULT, false),                   // fee_vault
        AccountMeta::new(trader_output, false),                     // user_stablecoin_ta
        AccountMeta::new(XSOL_MINT, false),                         // levercoin_mint
        AccountMeta::new_readonly(XSOL_LEVERCOIN_AUTH, false),      // levercoin_auth
        AccountMeta::new(trader_input, false),                      // user_levercoin_ta
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),         // token_program
        AccountMeta::new_readonly(EVENT_AUTHORITY, false),          // event_authority
        AccountMeta::new_readonly(HYLO_EXCHANGE_PROGRAM_ID, false), // program
    ];

    // swap_type = 5 (SwapLeverToStable)
    let extra_data: &[u8] = &[5_u8];

    let instruction = build_swap_instruction(accounts, in_amount, min_out_amount, extra_data);

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_xsol = get_token_balance(&svm, &trader_input);
            let final_hyusd = get_token_balance(&svm, &trader_output);

            assert!(
                final_xsol < initial_xsol,
                "xSOL should have decreased: {} -> {}",
                initial_xsol,
                final_xsol
            );
            assert!(
                final_hyusd > initial_hyusd,
                "hyUSD should have increased: {} -> {}",
                initial_hyusd,
                final_hyusd
            );

            println!(
                "Hylo Exchange swap successful! xSOL: {} -> {}, hyUSD: {} -> {}",
                initial_xsol, final_xsol, initial_hyusd, final_hyusd
            );
        }
        Err(e) => panic!("Hylo Exchange swap CPI failed: {}", e),
    }
}
