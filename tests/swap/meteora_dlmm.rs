use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, meteora_dlmm_fixtures_dir,
        send_transaction, setup_svm, METEORA_DLMM_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    beethoven_client::MEMO_PROGRAM_ID,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const LB_PAIR: Address = address!("BGm1tav58oGcsQJehL9WXBFXF7D27vZsKefj4xJKD5Y");
const BIN_ARRAY_BITMAP_EXTENSION: Address =
    address!("BzQsUBAbd21nrNDgc7D55EwnABC16uZJ41mgxxqYydHJ");
const RESERVE_X: Address = address!("DwZz4S1Z1LBXomzmncQRVKCYhjCqSAMQ6RPKbUAadr7H");
const RESERVE_Y: Address = address!("4N22J4vW2juHocTntJNmXywSonYjkndCwahjZ2cYLDgb");
const ORACLE: Address = address!("ETc6tqgLrr7wXsH8u2QBK1CyXHX3kvV6WQjBz4cf3sCj");
const EVENT_AUTHORITY: Address = address!("D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6");
const BIN_ARRAY_INDEX_MINUS_36: Address = address!("D6ervPBg2dK8U77vdj5ptpQx1Ti9MDLjkyTgxKFCH6pm");
const BIN_ARRAY_INDEX_MINUS_37: Address = address!("9tQvQWsuFkd3AqqoEbkG3YF3aSG1Kt1rdPDrTCaxzuhn");
const BIN_ARRAY_INDEX_MINUS_38: Address = address!("DKmQ4WQJm5Xkxwo9fcNmWknn18qUWyhLB5UDW4Vwmocv");
const BIN_ARRAY_INDEX_MINUS_39: Address = address!("7mDb6YRqghMiTXU9J8xoHvgDzvvMLj6Q6X1aTumt6RFr");

#[test]
fn test_meteora_dlmm_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Meteora DLMM program
    load_program(
        &mut svm,
        METEORA_DLMM_PROGRAM_ID,
        &format!("{}/meteora_dlmm.so", meteora_dlmm_fixtures_dir()),
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
        &format!("{}/lb_pair.json", meteora_dlmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/bin_array_bitmap_extension.json",
            meteora_dlmm_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/reserve_x.json", meteora_dlmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/reserve_y.json", meteora_dlmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/oracle.json", meteora_dlmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/bin_array_index_minus_36.json",
            meteora_dlmm_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/bin_array_index_minus_37.json",
            meteora_dlmm_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/bin_array_index_minus_38.json",
            meteora_dlmm_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/bin_array_index_minus_39.json",
            meteora_dlmm_fixtures_dir()
        ),
    );

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

    // Meteora DLMM accounts layout (17 accounts + bin array accounts)
    let accounts = vec![
        AccountMeta::new_readonly(METEORA_DLMM_PROGRAM_ID, false), // meteora_dlmm_program
        AccountMeta::new(LB_PAIR, false),                          // lb_pair
        AccountMeta::new_readonly(BIN_ARRAY_BITMAP_EXTENSION, false), // bin_array_bitmap_extension
        AccountMeta::new(RESERVE_X, false),                        // reserve_x
        AccountMeta::new(RESERVE_Y, false),                        // reserve_y
        AccountMeta::new(trader_input, false),                     // user_token_in
        AccountMeta::new(trader_output, false),                    // user_token_out
        AccountMeta::new_readonly(WSOL_MINT, false),               // token_x_mint
        AccountMeta::new_readonly(USDC_MINT, false),               // token_y_mint
        AccountMeta::new(ORACLE, false),                           // oracle
        AccountMeta::new_readonly(METEORA_DLMM_PROGRAM_ID, false), // host_fee_in
        AccountMeta::new(payer.pubkey(), true),                    // user
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),        // token_x_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),        // token_y_program
        AccountMeta::new_readonly(MEMO_PROGRAM_ID, false),         // memo_program
        AccountMeta::new_readonly(EVENT_AUTHORITY, false),         // event_authority
        AccountMeta::new_readonly(METEORA_DLMM_PROGRAM_ID, false), // program
        AccountMeta::new(BIN_ARRAY_INDEX_MINUS_36, false),         // bin_array
        AccountMeta::new(BIN_ARRAY_INDEX_MINUS_37, false),         // bin_array
        AccountMeta::new(BIN_ARRAY_INDEX_MINUS_38, false),         // bin_array
        AccountMeta::new(BIN_ARRAY_INDEX_MINUS_39, false),         // bin_array
    ];

    // no remaining_accounts_slices, vec header of 0 len
    let extra_data: &[u8] = &[0u8; 4];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::MeteoraDlmm,
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
                "Meteora DLMM swap successful! WSOL: {} -> {}, USDC: {} -> {}",
                initial_wsol, final_wsol, initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("Meteora DLMM swap CPI failed: {}", e);
        }
    }
}
