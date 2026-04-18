use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program,
        meteora_dynamic_bonding_curve_fixtures_dir, send_transaction, setup_svm,
        METEORA_DYNAMIC_BONDING_CURVE_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const BASE_MINT: Address = address!("GSg4JktbLkn3k2rWrPwfKdFyt4v8PBt6L6MRoFLymoon");
const POOL_AUTHORITY: Address = address!("FhVo3mqL8PW5pH5U2CN4XE33DokiyZnUwuGpH2hmHLuM");
const CONFIG: Address = address!("FbKf76ucsQssF7XZBuzScdJfugtsSKwZFYztKsMEhWZM");
const POOL: Address = address!("Buazd488xG6HofYP2T9ZJLerBMghJftymfYYvu1FP3ck");
const BASE_VAULT: Address = address!("P8xdtARQT7GZCxYYtGfaPRxQDVrYejEETNEKcbgpW7U");
const QUOTE_VAULT: Address = address!("GChqf6Ehx9iufcjHJj1kvnEQn2n4QYX5HozV3vioUmb7");
const EVENT_AUTHORITY: Address = address!("8Ks12pbrD6PXxfty1hVQiE9sc289zgU1zHkvXhrSdriF");

#[test]
fn test_meteora_dynamic_bonding_curve_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Meteora Dynamic Bonding Curve program
    load_program(
        &mut svm,
        METEORA_DYNAMIC_BONDING_CURVE_PROGRAM_ID,
        &format!(
            "{}/meteora_dynamic_bonding_curve.so",
            meteora_dynamic_bonding_curve_fixtures_dir()
        ),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/base_mint.json",
            meteora_dynamic_bonding_curve_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/config.json",
            meteora_dynamic_bonding_curve_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/pool.json", meteora_dynamic_bonding_curve_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/base_vault.json",
            meteora_dynamic_bonding_curve_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/quote_vault.json",
            meteora_dynamic_bonding_curve_fixtures_dir()
        ),
    );

    // Create trader token accounts with initial balances
    // Selling USDC (input=USDC) for base mint (output)
    let initial_usdc = 100_000_000u64; // 100 USDC
    let initial_base_mint = 0u64;
    let trader_input =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);
    let trader_output = create_token_account(
        &mut svm,
        &payer.pubkey(),
        &BASE_MINT,
        initial_base_mint,
        false,
    );

    // Build swap instruction: sell 10 USDC for base mint
    let in_amount = 10_000_000u64; // 10 USDC
    let min_out_amount = 1u64; // Very loose slippage for test

    // Meteora Dynamic Bonding Curve accounts layout (16 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(METEORA_DYNAMIC_BONDING_CURVE_PROGRAM_ID, false), // meteora_dynamic_bonding_curve_program
        AccountMeta::new_readonly(POOL_AUTHORITY, false), // pool_authority
        AccountMeta::new_readonly(CONFIG, false),         // config
        AccountMeta::new(POOL, false),                    // pool
        AccountMeta::new(trader_input, false),            // input_token_account
        AccountMeta::new(trader_output, false),           // output_token_account
        AccountMeta::new(BASE_VAULT, false),              // base_vault
        AccountMeta::new(QUOTE_VAULT, false),             // quote_vault
        AccountMeta::new_readonly(BASE_MINT, false),      // base_mint
        AccountMeta::new_readonly(USDC_MINT, false),      // quote_mint
        AccountMeta::new_readonly(payer.pubkey(), true),  // payer
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false), // token_base_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false), // token_quote_program)
        AccountMeta::new_readonly(METEORA_DYNAMIC_BONDING_CURVE_PROGRAM_ID, false), // referral_token_account
        AccountMeta::new_readonly(EVENT_AUTHORITY, false), // event_authority
        AccountMeta::new_readonly(METEORA_DYNAMIC_BONDING_CURVE_PROGRAM_ID, false), // program
    ];

    // swap_mode = ExactIn
    let extra_data = vec![0];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::MeteoraDynamicBondingCurve,
        &extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_usdc = get_token_balance(&svm, &trader_input);
            let final_base_mint = get_token_balance(&svm, &trader_output);

            assert!(
                final_usdc < initial_usdc,
                "USDC should have decreased: {} -> {}",
                initial_usdc,
                final_usdc
            );
            assert!(
                final_base_mint > initial_base_mint,
                "base mint should have increased: {} -> {}",
                initial_base_mint,
                final_base_mint
            );

            println!(
                "Meteora Dynamic Bonding Curve swap successful! USDC: {} -> {}, base mint: {} -> {}",
                initial_usdc, final_usdc, initial_base_mint, final_base_mint
            );
        }
        Err(e) => {
            panic!("Meteora Dynamic Bonding Curve swap CPI failed: {}", e);
        }
    }
}
