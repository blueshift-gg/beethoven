use {
    crate::helper::*,
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const POOL_CONFIG: Address = address!("2jk7miWrsTbt5hUSaCXPkEQPvuUMgbFLpgMzMQw3Z6ar");
const HYLO: Address = address!("9cd2sAfbBvKs4SX9YKo4dcjwP3TgTVQ8dT5koshGcDND");
const HYUSD_MINT: Address = address!("5YMkXAYccHSGnHn9nob9xEvv6Pvka9DZWH7nTbotTu9E");
const XSOL_MINT: Address = address!("4sWNB8zGWHkh6UnmwiEtzNxL4XrN7uK9tosbESbJFfVs");
const POOL_AUTH: Address = address!("5YrRAQag9BbJkauDtJkd1vsTquXT6N46oU8rJ66GDxHd");
const STABLECOIN_POOL: Address = address!("EqozKyMj7FVnLHc2cJj3VC25aBr4AhVh1cGM2WDajGe9");
const LEVERCOIN_POOL: Address = address!("4GPXVXuzk8ABAUkoXeBJg8r9kccEXQjoi5vqSxE9rhk1");
const LP_TOKEN_AUTH: Address = address!("5YWerkcqAXTSCzKC1X52BXtfv2aoNCB6wzv7wEXuGWpq");
const SHYUSD_MINT: Address = address!("HnnGv3HrSqjRpgdFmx7vQGjntNEoex1SU4e9Lxcxuihz");
const SOL_PRICE_UPDATE_V2: Address = address!("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");
const EVENT_AUTHORITY: Address = address!("8fjUWoZTb8ox8JFRJTb7WznL1V8oJT9o21kQKHJzbTS8");

#[test]
fn test_hylo_stability_pool_deposit_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Hylo Stability Pool program
    load_program(
        &mut svm,
        HYLO_STABILITY_PROGRAM_ID,
        &format!(
            "{}/hylo_stability_pool.so",
            hylo_stability_pool_fixtures_dir()
        ),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/hylo.json", hylo_stability_pool_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/hyusd_mint.json", hylo_stability_pool_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/hyusd_vault.json", hylo_stability_pool_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/pool_config.json", hylo_stability_pool_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/shyusd_mint.json", hylo_stability_pool_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/sol_price_update_v2.json",
            hylo_stability_pool_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/xsol_mint.json", hylo_stability_pool_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/xsol_vault.json", hylo_stability_pool_fixtures_dir()),
    );

    let mut clock = svm.get_sysvar::<Clock>();
    // Jump ahead by Hylo total_sol_cache current_update_epoch (952)
    clock.epoch = 952;
    // Jump ahead by PriceUpdateV2 posted_slot (411_594_464)
    clock.slot = 411_594_464;
    // Jump ahead by PriceUpdateV2 price_message published_time (1_775_550_567) + 1
    clock.unix_timestamp = 1_775_550_567 + 1;
    svm.set_sysvar::<Clock>(&clock);

    // Create trader token accounts with initial balances
    // Depositing hyUSD for sHYUSD
    let initial_hyusd = 100_000_000u64; // 100 hyUSD
    let initial_shyusd = 0u64;
    let trader_input =
        create_token_account(&mut svm, &payer.pubkey(), &HYUSD_MINT, initial_hyusd, false);
    let trader_output = create_token_account(
        &mut svm,
        &payer.pubkey(),
        &SHYUSD_MINT,
        initial_shyusd,
        false,
    );

    // Build deposit instruction: deposit 10 hyUSD for sHYUSD
    let in_amount = 10_000_000u64; // 10 hyUSD

    // Hylo Stability Pool deposit accounts layout (19 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(HYLO_STABILITY_PROGRAM_ID, false), // hylo_stability_program
        AccountMeta::new(payer.pubkey(), true),                      // user
        AccountMeta::new(POOL_CONFIG, false),                        // pool_config
        AccountMeta::new(HYLO, false),                               // hylo
        AccountMeta::new(HYUSD_MINT, false),                         // stablecoin_mint
        AccountMeta::new(XSOL_MINT, false),                          // levercoin_mint
        AccountMeta::new(trader_input, false),                       // user_stablecoin_ta
        AccountMeta::new(trader_output, false),                      // user_lp_token_ta
        AccountMeta::new(POOL_AUTH, false),                          // pool_auth
        AccountMeta::new(STABLECOIN_POOL, false),                    // stablecoin_pool
        AccountMeta::new(LEVERCOIN_POOL, false),                     // levercoin_pool
        AccountMeta::new(LP_TOKEN_AUTH, false),                      // lp_token_auth
        AccountMeta::new(SHYUSD_MINT, false),                        // lp_token_mint
        AccountMeta::new(SOL_PRICE_UPDATE_V2, false),                // sol_price_update_v2
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),         // system_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),          // token_program
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false), // associated_token_program
        AccountMeta::new_readonly(EVENT_AUTHORITY, false),           // event_authority
        AccountMeta::new_readonly(HYLO_STABILITY_PROGRAM_ID, false), // program
    ];

    // Hylo Stability Pool deposit has no extra data
    let extra_data = vec![];

    let instruction = build_deposit_instruction(accounts, in_amount, &extra_data);

    // Execute the deposit via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_hyusd = get_token_balance(&svm, &trader_input);
            let final_shyusd = get_token_balance(&svm, &trader_output);

            assert!(
                final_hyusd < initial_hyusd,
                "hyUSD should have decreased: {} -> {}",
                initial_hyusd,
                final_hyusd
            );
            assert!(
                final_shyusd > initial_shyusd,
                "sHYUSD should have increased: {} -> {}",
                initial_shyusd,
                final_shyusd
            );

            println!(
                "Hylo Stability Pool deposit successful! hyUSD: {} -> {}, sHYUSD: {} -> {}",
                initial_hyusd, final_hyusd, initial_shyusd, final_shyusd
            );
        }
        Err(e) => {
            panic!("Hylo Stability Pool deposit CPI failed: {}", e);
        }
    }
}
