use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, scale_amm_fixtures_dir,
        send_transaction, setup_svm, SCALE_AMM_PROGRAM_ID, SYSTEM_PROGRAM_ID, TEST_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const POOL: Address = address!("AZDqVz1TiKYGcMhaYKBMoCnRH6bXXqoxuZNR4dLL8B8K");
const OWNER: Address = address!("BXfXDZh5HfyyPPHT5xYUVXWve5oJ2cY2P2Y6VyKwoqGg");
const MINT_B: Address = address!("7j5Zo8vzDTN8qJhWSFY9RWPE76rVRXMkqvGLeaWqcyz9");
const VAULT_A: Address = address!("5Lsuh97Dnzsj9wp2DspyodwmUTX2ABiFYqCRtH7Ym65o");
const VAULT_B: Address = address!("ENpu9WqhnEzUSQzhEqVx6LtpXoexmRqjWYAGYYfhDGnt");
const PLATFORM_FEE_TA_A: Address = address!("5otzrfbppNE1j6m7ptWkAcu5gs1nwj5ZQKQVwFFHQAHv");
const PLATFORM_CONFIG: Address = address!("232KbYciAe6ma2VCB6gQyofix8qQwyZd2WYVhpNx8SyR");
const FEE_BENEFICIARY_ATA: Address = address!("7XRb5qdYdCh1QUp6WZtHGtyGgwVnuu8BPS3fr2FvXboD");

#[test]
fn test_scale_amm_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Scale AMM program
    load_program(
        &mut svm,
        SCALE_AMM_PROGRAM_ID,
        &format!("{}/scale_amm.so", scale_amm_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/wsol_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/mint_b.json", scale_amm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/owner.json", scale_amm_fixtures_dir()),
    );
    load_and_set_json_fixture(&mut svm, &format!("{}/pool.json", scale_amm_fixtures_dir()));
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/platform_config.json", scale_amm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/platform_fee_ta_a.json", scale_amm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_a.json", scale_amm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_b.json", scale_amm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/fee_beneficiary_ata.json", scale_amm_fixtures_dir()),
    );

    // Create trader token accounts with initial balances
    // Selling SOL (input=WSOL) for USDC (output)
    let initial_wsol = 1_000_000_000u64; // 1 SOL
    let initial_mint_b = 0u64;
    let trader_input =
        create_token_account(&mut svm, &payer.pubkey(), &WSOL_MINT, initial_wsol, false);
    let trader_output =
        create_token_account(&mut svm, &payer.pubkey(), &MINT_B, initial_mint_b, false);

    // Build swap instruction: sell 0.001 SOL for USDC
    let in_amount = 1_000_000u64; // 0.001 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    // Scale AMM accounts layout (15 accounts + 1 remaining account for every fee beneficiary)
    let accounts = vec![
        AccountMeta::new_readonly(SCALE_AMM_PROGRAM_ID, false), // scale_amm program
        AccountMeta::new(POOL, false),                          // pool
        AccountMeta::new(payer.pubkey(), true),                 // user
        AccountMeta::new_readonly(OWNER, false),                // owner
        AccountMeta::new_readonly(WSOL_MINT, false),            // mint a
        AccountMeta::new_readonly(MINT_B, false),               // mint b
        AccountMeta::new(trader_input, false),                  // user ta a
        AccountMeta::new(trader_output, false),                 // user ta b
        AccountMeta::new(VAULT_A, false),                       // vault a
        AccountMeta::new(VAULT_B, false),                       // vault b
        AccountMeta::new(PLATFORM_FEE_TA_A, false),             // platform fee ta a
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),     // token program a
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),     // token program b
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),    // system program
        AccountMeta::new_readonly(PLATFORM_CONFIG, false),      // config
        AccountMeta::new(FEE_BENEFICIARY_ATA, false),           // fee beneficiary ata
    ];

    // side = buy
    let extra_data: &[u8] = &[0u8];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::ScaleAmm,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_wsol = get_token_balance(&svm, &trader_input);
            let final_mint_b = get_token_balance(&svm, &trader_output);

            assert!(
                final_wsol < initial_wsol,
                "WSOL should have decreased: {} -> {}",
                initial_wsol,
                final_wsol
            );
            assert!(
                final_mint_b > initial_mint_b,
                "MINT_B should have increased: {} -> {}",
                initial_mint_b,
                final_mint_b
            );
        }
        Err(e) => panic!("Scale AMM swap CPI failed: {}", e),
    }
}
