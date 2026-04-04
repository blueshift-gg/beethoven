use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, orca_whirlpool_fixtures_dir,
        send_transaction, setup_svm, MEMO_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
        WHIRLPOOL_PROGRAM_ID,
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
const WHIRLPOOL: Address = address!("HJPjoWUrhoZzkNfRpHuieeFk9WcZWjwy6PBjZ81ngndJ");
const SOL_VAULT: Address = address!("3YQm7ujtXWJU2e9jhp2QGHpnn1ShXn12QjvzMvDgabpX");
const USDC_VAULT: Address = address!("2JTw1fE2wz1SymWUQ7UqpVtrTuKjcd6mWwYwUJUCh2rq");
const TICK_ARRAY_0: Address = address!("A2W6hiA2nf16iqtbZt9vX8FJbiXjv3DBUG3DgTja61HT");
const TICK_ARRAY_1: Address = address!("2Eh8HEeu45tCWxY6ruLLRN6VcTSD7bfshGj7bZA87Kne");
const TICK_ARRAY_2: Address = address!("EVqGhR2ukNuqZNfvFFAitrX6UqrRm2r8ayKX9LH9xHzK");
const ORACLE: Address = address!("4GkRbcYg1VKsZropgai4dMf2Nj2PkXNLf43knFpavrSi");

#[test]
fn test_orca_whirlpool_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Whirlpool program
    load_program(
        &mut svm,
        WHIRLPOOL_PROGRAM_ID,
        &format!("{}/orca-whirlpool.so", orca_whirlpool_fixtures_dir()),
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
        &format!("{}/sol_usdc_whirlpool.json", orca_whirlpool_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/sol_vault.json", orca_whirlpool_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_vault.json", orca_whirlpool_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/tick_array_0.json", orca_whirlpool_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/tick_array_1.json", orca_whirlpool_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/tick_array_2.json", orca_whirlpool_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/oracle.json", orca_whirlpool_fixtures_dir()),
    );

    // Jump ahead by whirlpool last_updated_timestamp (1_775_303_470) + 1 second
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_775_303_470;
    svm.set_sysvar::<Clock>(&clock);

    // Create trader token accounts with initial balances
    // Selling SOL (input=WSOL) for USDC (output)
    let initial_wsol = 1_000_000_000u64;
    let initial_usdc = 0u64;
    let trader_wsol =
        create_token_account(&mut svm, &payer.pubkey(), &WSOL_MINT, initial_wsol, false);
    let trader_usdc =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);

    // Build swap instruction: sell 0.001 SOL for USDC
    let in_amount = 1_000_000u64; // 0.001 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    // Orca Whirlpool accounts layout (16 accounts + remaining accounts)
    let accounts = vec![
        AccountMeta::new_readonly(WHIRLPOOL_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(MEMO_PROGRAM_ID, false),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(WHIRLPOOL, false),
        AccountMeta::new_readonly(WSOL_MINT, false),
        AccountMeta::new_readonly(USDC_MINT, false),
        AccountMeta::new(trader_wsol, false),
        AccountMeta::new(SOL_VAULT, false),
        AccountMeta::new(trader_usdc, false),
        AccountMeta::new(USDC_VAULT, false),
        AccountMeta::new(TICK_ARRAY_0, false),
        AccountMeta::new(TICK_ARRAY_1, false),
        AccountMeta::new(TICK_ARRAY_2, false),
        AccountMeta::new(ORACLE, false),
    ];

    let sqrt_price_limit: u128 = 0;
    let amount_specified_is_input = true;
    let a_to_b = true;
    let mut extra_data = Vec::with_capacity(19);
    extra_data.extend_from_slice(&sqrt_price_limit.to_le_bytes());
    extra_data.push(amount_specified_is_input as u8);
    extra_data.push(a_to_b as u8);
    // remaining_accounts_info - None
    extra_data.push(0u8);

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::OrcaWhirlpool,
        &extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_wsol = get_token_balance(&svm, &trader_wsol);
            let final_usdc = get_token_balance(&svm, &trader_usdc);

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
                "Orca Whirlpool swap successful! WSOL: {} -> {}, USDC: {} -> {}",
                initial_wsol, final_wsol, initial_usdc, final_usdc
            );
        }
        Err(e) => panic!("Orca Whirlpool swap CPI failed: {}", e),
    }
}
