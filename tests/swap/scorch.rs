use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, scorch_fixtures_dir,
        send_transaction, setup_svm, MEMO_PROGRAM_ID, ORACLE_PROGRAM_ID, SCORCH_PROGRAM_ID,
        TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    beethoven_client::SYSVAR_INSTRUCTIONS_ID,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const MARKET: Address = address!("EHcege7dok1iYs7SxL2XzDPvhg6XzMVcx2V5SkMUurJP");
const MARKET_TA_A: Address = address!("44GW6aFire4Fd72h4QqjenNKrQLjfnApHhWYXo3S1gvp");
const MARKET_TA_B: Address = address!("34WpFzQ1WE2nDLLEKJuvCXxapL7ak6CigJS8Ks5NDo5K");
const ACC_1: Address = address!("HLixVmXdBqzP1sXT9au4BHcvUjDgx5ev16cEJdd9tUSM");
const STATE_A: Address = address!("85Etk23kFtyt265MQjyUgzJYZ7u5o2EVNdjDmuNySbGi");
const STATE_B: Address = address!("DmocjvFXp75asezCDKVNH2qaXbjrV7VeQk4aLZaPF88E");
const STATE_C: Address = address!("FnhxUP3dcQbypCUmGWw55ijxPBxifPT558UQSCYDfcCU");

#[test]
#[ignore = "would throw with UnsupportedProgramId error, likely due to instruction not called from a whitelisted router"]
fn test_scorch_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Scorch program
    load_program(
        &mut svm,
        SCORCH_PROGRAM_ID,
        &format!("{}/scorch.so", scorch_fixtures_dir()),
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
    load_and_set_json_fixture(&mut svm, &format!("{}/market.json", scorch_fixtures_dir()));
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/market_ta_a.json", scorch_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/market_ta_b.json", scorch_fixtures_dir()),
    );
    load_and_set_json_fixture(&mut svm, &format!("{}/acc1.json", scorch_fixtures_dir()));
    load_and_set_json_fixture(&mut svm, &format!("{}/state_a.json", scorch_fixtures_dir()));
    load_and_set_json_fixture(&mut svm, &format!("{}/state_b.json", scorch_fixtures_dir()));
    load_and_set_json_fixture(&mut svm, &format!("{}/state_c.json", scorch_fixtures_dir()));

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

    // Scorch accounts layout (18 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(SCORCH_PROGRAM_ID, false),
        AccountMeta::new(MARKET, false),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(trader_input, false),
        AccountMeta::new(trader_output, false),
        AccountMeta::new(MARKET_TA_A, false),
        AccountMeta::new(MARKET_TA_B, false),
        AccountMeta::new_readonly(WSOL_MINT, false),
        AccountMeta::new_readonly(USDC_MINT, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(MEMO_PROGRAM_ID, false),
        AccountMeta::new_readonly(ORACLE_PROGRAM_ID, false),
        AccountMeta::new_readonly(ACC_1, false),
        AccountMeta::new(STATE_A, false),
        AccountMeta::new(STATE_B, false),
        AccountMeta::new(STATE_C, false),
        AccountMeta::new_readonly(SYSVAR_INSTRUCTIONS_ID, false),
    ];

    // 17 byte param
    let extra_data: &[u8] = &[
        0xe0, 0xbe, 0x8c, 0xae, 0x67, 0xc2, 0xbc, 0x97, 0x89, 0x0a, 0x00, 0x00, 0x0c, 0x00, 0x00,
        0xf9, 0x00,
    ];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::Scorch,
        extra_data,
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
                "Scorch swap successful! WSOL: {} -> {}, USDC: {} -> {}",
                initial_wsol, final_wsol, initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("Scorch swap CPI failed: {}", e);
        }
    }
}
