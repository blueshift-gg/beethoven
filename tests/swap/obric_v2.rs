use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, obric_v2_fixtures_dir,
        send_transaction, setup_svm, OBRIC_V2_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDT_MINT: Address = address!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
const MARKET: Address = address!("BWBHrYqfcjAh5dSiRwzPnY4656cApXVXmkeDmAfwBKQG");
const SECOND_REF_ORACLE: Address = address!("GZsNmWKbqhMYtdSkkvMdEyQF9k5mLmP7tTKYWZjcHVPE");
const THIRD_REF_ORACLE: Address = address!("6YawcNeZ74tRyCv4UfGydYMr7eho7vbUR6ScVffxKAb3");
const RESERVE_X: Address = address!("C3tPQ8TRcHybnPpR8KMASUVD3PukQRRHEsLwxorJMhgm");
const RESERVE_Y: Address = address!("AAamGhyPfpQJWfZHTq944NM1cFvoVLDrQxt7HGjeRQUS");
const REF_ORACLE: Address = address!("J4HJYz4p7TRP96WVFky3vh7XryxoFehHjoRySUTeSeXw");
const X_PRICE_FEED: Address = address!("J4HJYz4p7TRP96WVFky3vh7XryxoFehHjoRySUTeSeXw");
const Y_PRICE_FEED: Address = address!("J4HJYz4p7TRP96WVFky3vh7XryxoFehHjoRySUTeSeXw");

#[test]
fn test_obric_v2_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Obric V2 program
    load_program(
        &mut svm,
        OBRIC_V2_PROGRAM_ID,
        &format!("{}/obric_v2.so", obric_v2_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdt_mint.json", obric_v2_fixtures_dir()),
    );

    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/market.json", obric_v2_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/second_ref_oracle.json", obric_v2_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/third_ref_oracle.json", obric_v2_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/reserve_x.json", obric_v2_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/reserve_y.json", obric_v2_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/ref_oracle.json", obric_v2_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/x_price_feed.json", obric_v2_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/y_price_feed.json", obric_v2_fixtures_dir()),
    );

    // Create trader token accounts with initial balances
    // Selling USDC (input) for USDT (output)
    let initial_usdc = 100_000_000u64; // 100 USDC
    let initial_usdt = 0u64;
    let trader_input =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);
    let trader_output =
        create_token_account(&mut svm, &payer.pubkey(), &USDT_MINT, initial_usdt, false);

    // Build swap instruction: sell 10 USDC for USDT
    let in_amount = 10_000_000u64; // 10 USDC
    let min_out_amount = 1u64; // Very loose slippage for test

    // Obric V2 accounts layout (13 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(OBRIC_V2_PROGRAM_ID, false),
        AccountMeta::new(MARKET, false),
        AccountMeta::new_readonly(SECOND_REF_ORACLE, false),
        AccountMeta::new_readonly(THIRD_REF_ORACLE, false),
        AccountMeta::new(RESERVE_X, false),
        AccountMeta::new(RESERVE_Y, false),
        AccountMeta::new(trader_input, false),
        AccountMeta::new(trader_output, false),
        AccountMeta::new(REF_ORACLE, false),
        AccountMeta::new_readonly(X_PRICE_FEED, false),
        AccountMeta::new_readonly(Y_PRICE_FEED, false),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
    ];

    // x_to_y = true
    let extra_data: &[u8] = &[1_u8];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::ObricV2,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_usdc = get_token_balance(&svm, &trader_input);
            let final_usdt = get_token_balance(&svm, &trader_output);

            assert!(
                final_usdc < initial_usdc,
                "USDC should have decreased: {} -> {}",
                initial_usdc,
                final_usdc
            );
            assert!(
                final_usdt > initial_usdt,
                "USDT should have increased: {} -> {}",
                initial_usdt,
                final_usdt
            );

            println!(
                "Obric V2 swap successful! USDC: {} -> {}, USDT: {} -> {}",
                initial_usdc, final_usdc, initial_usdt, final_usdt
            );
        }
        Err(e) => {
            panic!("Obric V2 swap CPI failed: {}", e);
        }
    }
}
