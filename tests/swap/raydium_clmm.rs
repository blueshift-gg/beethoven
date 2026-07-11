use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, raydium_clmm_fixtures_dir,
        send_transaction, setup_svm, MEMO_PROGRAM_ID, RAYDIUM_CLMM_PROGRAM_ID, TEST_PROGRAM_ID,
        TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

const AMM_CONFIG: Address = address!("EdPxg8QaeFSrTYqdWJn6Kezwy9McWncTYueD9eMGCuzR");
const SOL_USDC_POOL: Address = address!("CYbD9RaToYMtWKA7QZyoLahnHdWq553Vm62Lh6qWtuxq");
const SOL_VAULT: Address = address!("GviiXg2Xc1xCpyNY36r7h1EAy7uvse5UMkiiyHjRDU6Z");
const USDC_VAULT: Address = address!("3bWPj5eepJm8CxUzk5MMFMN2CFJkntxKvbmy4zwwtpJd");
const OBSERVATION_STATE: Address = address!("AA5RaVvyGyZgtmAsJJHT5ZVBxVPtAXuYaMwfgeFJW4Mk");
const TICK_ARRAY_BITMAP_EXTENSION: Address =
    address!("72jQFwjd14BEhyDfdQsH7D2hS5dN1H6bzsikjkyHyx2D");
const TICK_ARRAY_STATE_1: Address = address!("HYwVPNow6n3ZfsT66xKVzBcm6S42b1bXKDs4oxgnDg4o");
const TICK_ARRAY_STATE_2: Address = address!("Azbhc8wj1N2VkXKHXER8ykaVt7t7nfu9iCwUocfvyJAh");
const TICK_ARRAY_STATE_3: Address = address!("BJbJcDejFdssSR3wDQx23YHqTFkXYnbiYYkrshdj3YP8");
const TICK_ARRAY_STATE_4: Address = address!("ERdkKZZ8Z3TefUJ1sZsA2UEcwaLVSvWBXru3H2anqihN");

#[test]
fn test_raydium_clmm_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Raydium CLMM program
    load_program(
        &mut svm,
        RAYDIUM_CLMM_PROGRAM_ID,
        &format!("{}/raydium_clmm.so", raydium_clmm_fixtures_dir()),
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
        &format!("{}/amm_config.json", raydium_clmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/sol_usdc_pool_state.json", raydium_clmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/sol_vault.json", raydium_clmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_vault.json", raydium_clmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/sol_usdc_observation_state.json",
            raydium_clmm_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/tick_array_bitmap_extension.json",
            raydium_clmm_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/tick_array_state_1.json", raydium_clmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/tick_array_state_2.json", raydium_clmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/tick_array_state_3.json", raydium_clmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/tick_array_state_4.json", raydium_clmm_fixtures_dir()),
    );

    // Jump ahead by pool open time (1_722_694_155) + 1 second
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_722_694_155 + 1;
    svm.set_sysvar::<Clock>(&clock);

    // Create trader token accounts with initial balances
    // Selling SOL (input=WSOL) for USDC (output)
    let initial_wsol = 1_000_000_000u64; // 1 SOL
    let initial_usdc = 0u64;
    let trader_input =
        create_token_account(&mut svm, &payer.pubkey(), &WSOL_MINT, initial_wsol, false);
    let trader_output =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);

    // Build swap instruction: sell 0.001 SOL for USDC
    let in_amount = 1_000_000u64; // 0.001 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    // Raydium CLMM accounts layout (14 accounts + remaining accounts)
    let accounts = vec![
        AccountMeta::new_readonly(RAYDIUM_CLMM_PROGRAM_ID, false), // raydium_clmm_program
        AccountMeta::new_readonly(payer.pubkey(), true),           // payer
        AccountMeta::new_readonly(AMM_CONFIG, false),              // amm_config
        AccountMeta::new(SOL_USDC_POOL, false),                    // pool_state
        AccountMeta::new(trader_input, false),                     // input_token_account
        AccountMeta::new(trader_output, false),                    // output_token_account
        AccountMeta::new(SOL_VAULT, false),                        // input_vault
        AccountMeta::new(USDC_VAULT, false),                       // output_vault
        AccountMeta::new(OBSERVATION_STATE, false),                // observation_state
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),        // token_program
        AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),   // token_2022_program
        AccountMeta::new_readonly(MEMO_PROGRAM_ID, false),         // memo_program
        AccountMeta::new_readonly(WSOL_MINT, false),               // input_vault_mint
        AccountMeta::new_readonly(USDC_MINT, false),               // output_vault_mint
        AccountMeta::new(TICK_ARRAY_STATE_1, false),               // tick_array_state_1
        AccountMeta::new(TICK_ARRAY_STATE_2, false),               // tick_array_state_2
        AccountMeta::new(TICK_ARRAY_STATE_3, false),               // tick_array_state_3
        AccountMeta::new(TICK_ARRAY_STATE_4, false),               // tick_array_state_4
        AccountMeta::new_readonly(TICK_ARRAY_BITMAP_EXTENSION, false), // tick_array_bitmap_extension
    ];

    let sqrt_price_limit_x64 = 0u128;
    let is_base_input = true;
    let mut extra_data = Vec::with_capacity(17);
    extra_data.extend_from_slice(&sqrt_price_limit_x64.to_le_bytes());
    extra_data.push(u8::from(is_base_input));

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::RaydiumClmm,
        &extra_data,
    );

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
                "Raydium CLMM swap successful! WSOL: {} -> {}, USDC: {} -> {}",
                initial_wsol, final_wsol, initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("Raydium CLMM swap CPI failed: {}", e);
        }
    }
}
