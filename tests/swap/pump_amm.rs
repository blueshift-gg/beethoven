use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, pump_amm_fixtures_dir,
        send_transaction, setup_svm, ASSOCIATED_TOKEN_PROGRAM_ID, FEE_PROGRAM_ID,
        PUMP_AMM_PROGRAM_ID, SYSTEM_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_compute_budget::compute_budget::ComputeBudget,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const SOL_USDC_POOL: Address = address!("Gf7sXMoP8iRw4iiXmJ1nq4vxcRycbGXy5RL8a8LnTd3v");
const GLOBAL_CONFIG: Address = address!("ADyA8hdefvWN2dbGGWFotbzWxrAvLW83WG6QCVXvJKqw");
const POOL_BASE_TOKEN_ACCOUNT: Address = address!("nML7msD1MiJHxFvhv4po1u6C4KpWr64ugKqc75DMuD2");
const POOL_QUOTE_TOKEN_ACCOUNT: Address = address!("EjHirXt2bQd2DDNveagHHCWYzUwtY1iwNbBrV5j84e6j");
const PROTOCOL_FEE_RECIPIENT: Address = address!("7VtfL8fvgNfhz17qKRMjzQEXgbdpnHHHQRh54R9jP2RJ");
const PROTOCOL_FEE_RECIPIENT_TOKEN_ACCOUNT: Address =
    address!("7GFUN3bWzJMKMRZ34JLsvcqdssDbXnp589SiE33KVwcC");
const EVENT_AUTHORITY: Address = address!("GS4CU59F31iL7aR2Q8zVS8DRrcRnXX1yjQ66TqNVQnaR");
const COIN_CREATOR_VAULT_ATA: Address = address!("Ei6iux5MMYG8JxCTr58goADqFTtMroL9TXJityF3fAQc");
const COIN_CREATOR_VAULT_AUTHORITY: Address =
    address!("8N3GDaZ2iwN65oxVatKTLPNooAVUJTbfiVJ1ahyqwjSk");
const GLOBAL_VOLUME_ACCUMULATOR: Address = address!("C2aFPdENg4A2HQsmrd5rTw5TaYBX5Ku887cWjbFKtZpw");
const FEE_CONFIG: Address = address!("5PHirr8joyTMp9JMm6nW7hNDVyEYdkzDqazxPD7RaTjx");

#[test]
fn test_pump_amm_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Pump AMM program
    load_program(
        &mut svm,
        PUMP_AMM_PROGRAM_ID,
        &format!("{}/pump_amm.so", pump_amm_fixtures_dir()),
    );
    load_program(
        &mut svm,
        FEE_PROGRAM_ID,
        &format!("{}/fee_program.so", pump_amm_fixtures_dir()),
    );

    // Load fixtures
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
        &format!("{}/sol_usdc_pool.json", pump_amm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/global_config.json", pump_amm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/sol_usdc_pool_base_token_account.json",
            pump_amm_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/sol_usdc_pool_quote_token_account.json",
            pump_amm_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/global_volume_accumulator.json", pump_amm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/fee_config.json", pump_amm_fixtures_dir()),
    );

    // Set compute budget limit
    svm = svm.with_compute_budget(ComputeBudget::new_with_defaults(true, true));

    // Create trader token accounts with initial balances
    // Selling SOL (input=WSOL) for USDC (output)
    let initial_wsol = 1_000_000_000u64; // 1 SOL
    let initial_usdc = 0u64;
    let trader_input = create_token_account(&mut svm, &payer.pubkey(), &WSOL_MINT, initial_wsol);
    let trader_output = create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc);

    // Build swap instruction: sell 0.001 SOL for USDC
    let in_amount = 1_000_000u64; // 0.001 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    // Derive user volume accumulator from user
    let user_volume_accumulator = Address::find_program_address(
        &[b"user_volume_accumulator", payer.pubkey().as_ref()],
        &PUMP_AMM_PROGRAM_ID,
    )
    .0;

    // Pump AMM accounts layout (24 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(PUMP_AMM_PROGRAM_ID, false), // pump_amm_program
        AccountMeta::new(SOL_USDC_POOL, false),                // pool
        AccountMeta::new(payer.pubkey(), true),                // user
        AccountMeta::new(GLOBAL_CONFIG, false),                // global_config
        AccountMeta::new_readonly(USDC_MINT, false),           // base_mint
        AccountMeta::new_readonly(WSOL_MINT, false),           // quote_mint
        AccountMeta::new(trader_output, false),                // user_base_token_account
        AccountMeta::new(trader_input, false),                 // user_quote_token_account
        AccountMeta::new(POOL_BASE_TOKEN_ACCOUNT, false),      // pool_base_token_account
        AccountMeta::new(POOL_QUOTE_TOKEN_ACCOUNT, false),     // pool_quote_token_account
        AccountMeta::new_readonly(PROTOCOL_FEE_RECIPIENT, false), // protocol_fee_recipient
        AccountMeta::new(PROTOCOL_FEE_RECIPIENT_TOKEN_ACCOUNT, false), // protocol_fee_recipient_token_account
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),            // base_token_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),            // quote_token_program
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),           // system_program
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false), // associated_token_program
        AccountMeta::new_readonly(EVENT_AUTHORITY, false),             // event_authority
        AccountMeta::new_readonly(PUMP_AMM_PROGRAM_ID, false),         // program
        AccountMeta::new(COIN_CREATOR_VAULT_ATA, false),               // coin_creator_vault_ata
        AccountMeta::new_readonly(COIN_CREATOR_VAULT_AUTHORITY, false), // coin_creator_vault_authority
        AccountMeta::new(GLOBAL_VOLUME_ACCUMULATOR, false),             // global_volume_accumulator
        AccountMeta::new(user_volume_accumulator, false),               // user_volume_accumulator
        AccountMeta::new(FEE_CONFIG, false),                            // fee_config
        AccountMeta::new_readonly(FEE_PROGRAM_ID, false),               // fee_program
    ];

    // track volume = Some(true)
    let extra_data: &[u8] = &[1, 1];

    let instruction = build_swap_instruction(accounts, in_amount, min_out_amount, extra_data);

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_wsol = get_token_balance(&svm, &trader_input);
            let final_usdc = get_token_balance(&svm, &trader_output);

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
                "Pump AMM swap successful! WSOL: {} -> {}, USDC: {} -> {}",
                initial_wsol, final_wsol, initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("Pump AMM swap CPI failed: {}", e);
        }
    }
}
