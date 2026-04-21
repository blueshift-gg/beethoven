use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir,
        create_token_account_at, get_token_balance, load_and_set_json_fixture, load_program,
        send_transaction, setup_svm, synatra_fixtures_dir, SYNATRA_PROGRAM_ID, SYSTEM_PROGRAM_ID,
        TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    beethoven_client::{get_associated_token_address, ASSOCIATED_TOKEN_PROGRAM_ID},
    litesvm::LiteSVM,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const YSOL_MINT: Address = address!("yso11zxLbHA3wBJ9HAtVu6wnesqz9A2qxnhxanasZ4N");
const YUSD_MINT: Address = address!("yUSDX7W89jXWn4zzDPLnhykDymSjQSmpaJ8e4fjC1fg");
const YSOL_POOL: Address = address!("2wMDWx7a1PpbrsnNAHGJLPMgRs7H3pcYxqmmkQrzLxHg");
const YUSD_POOL: Address = address!("Fm8E4fEAiRraWP2EhMfycyYzYdvNgzQiKUwhxCCUB4ho");
const YUSD_POOL_ATA: Address = address!("DME9KG2K16wTWvpMrijFHDzsENZAf1YS7DLwzWG2AiHU");

fn load_program_and_fixtures(svm: &mut LiteSVM) {
    load_program(svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Synatra program
    load_program(
        svm,
        SYNATRA_PROGRAM_ID,
        &format!("{}/synatra.so", synatra_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(svm, &format!("{}/wsol_mint.json", common_fixtures_dir()));
    load_and_set_json_fixture(svm, &format!("{}/usdc_mint.json", common_fixtures_dir()));
    load_and_set_json_fixture(svm, &format!("{}/ysol_mint.json", synatra_fixtures_dir()));
    load_and_set_json_fixture(svm, &format!("{}/yusd_mint.json", synatra_fixtures_dir()));
    load_and_set_json_fixture(svm, &format!("{}/ysol_pool.json", synatra_fixtures_dir()));
    load_and_set_json_fixture(svm, &format!("{}/yusd_pool.json", synatra_fixtures_dir()));
    load_and_set_json_fixture(
        svm,
        &format!("{}/yusd_pool_ata.json", synatra_fixtures_dir()),
    );
}

#[test]
fn test_synatra_swap_cpi_stake_sol() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program_and_fixtures(&mut svm);

    // Create trader token accounts with initial balances
    // Selling SOL (input=WSOL) for ySOL (output)
    let initial_wsol = svm.get_balance(&payer.pubkey()).unwrap();
    let initial_ysol = 0u64;
    let trader_output =
        get_associated_token_address(&payer.pubkey(), &YSOL_MINT, &TOKEN_PROGRAM_ID);
    create_token_account_at(
        &mut svm,
        trader_output,
        &payer.pubkey(),
        &YSOL_MINT,
        initial_ysol,
        false,
    );

    // Build swap instruction: sell 0.001 SOL for USDC
    let in_amount = 1_000_000u64; // 0.001 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    // Synatra stake_sol accounts layout (9 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(SYNATRA_PROGRAM_ID, false), // synatra_program
        AccountMeta::new(payer.pubkey(), true),               // signer
        AccountMeta::new(payer.pubkey(), true),               // payer
        AccountMeta::new(YSOL_POOL, false),                   // pool
        AccountMeta::new(YSOL_MINT, false),                   // receipt_token
        AccountMeta::new(trader_output, false),               // user_receipt_ata
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false), // associated_token_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),   // token_program
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),  // system_program
    ];

    // Synatra stake_sol has no extra data
    let extra_data: &[u8] = &[];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::Synatra,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_wsol = svm.get_balance(&payer.pubkey()).unwrap();
            let final_ysol = get_token_balance(&svm, &trader_output);

            assert!(
                final_wsol < initial_wsol,
                "WSOL should have decreased: {} -> {}",
                initial_wsol,
                final_wsol
            );
            assert!(
                final_ysol > initial_ysol,
                "ySOL should have increased: {} -> {}",
                initial_ysol,
                final_ysol
            );

            println!(
                "Synatra stake_sol successful! WSOL: {} -> {}, ySOL: {} -> {}",
                initial_wsol, final_wsol, initial_ysol, final_ysol
            );
        }
        Err(e) => {
            panic!("Synatra stake_sol CPI failed: {}", e);
        }
    }
}

#[test]
fn test_synatra_swap_cpi_stake_token() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program_and_fixtures(&mut svm);

    // Create trader token accounts with initial balances
    // Selling USDC (input=USDC) for yUSD (output)
    let initial_usdc = 100_000_000u64; // 100 USDC
    let initial_yusd = 0u64;
    let trader_input = get_associated_token_address(&payer.pubkey(), &USDC_MINT, &TOKEN_PROGRAM_ID);
    let trader_output =
        get_associated_token_address(&payer.pubkey(), &YUSD_MINT, &TOKEN_PROGRAM_ID);
    create_token_account_at(
        &mut svm,
        trader_input,
        &payer.pubkey(),
        &USDC_MINT,
        initial_usdc,
        false,
    );
    create_token_account_at(
        &mut svm,
        trader_output,
        &payer.pubkey(),
        &YUSD_MINT,
        initial_yusd,
        false,
    );

    // Build swap instruction: sell 0.001 SOL for USDC
    let in_amount = 1_000_000u64; // 0.001 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    // Synatra stake_token accounts layout (12 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(SYNATRA_PROGRAM_ID, false), // synatra_program
        AccountMeta::new(payer.pubkey(), true),               // signer
        AccountMeta::new(payer.pubkey(), true),               // payer
        AccountMeta::new(YUSD_POOL, false),                   // pool
        AccountMeta::new(USDC_MINT, false),                   // stake_token
        AccountMeta::new(YUSD_MINT, false),                   // receipt_token
        AccountMeta::new(trader_input, false),                // user_token_ata
        AccountMeta::new(trader_output, false),               // user_receipt_ata
        AccountMeta::new(YUSD_POOL_ATA, false),               // pool_token_ata
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false), // associated_token_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),   // token_program
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),  // system_program
    ];

    // Synatra stake_token has no extra data
    let extra_data: &[u8] = &[];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::Synatra,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_usdc = get_token_balance(&svm, &trader_input);
            let final_yusd = get_token_balance(&svm, &trader_output);

            assert!(
                final_usdc < initial_usdc,
                "USDC should have decreased: {} -> {}",
                initial_usdc,
                final_usdc
            );
            assert!(
                final_yusd > initial_yusd,
                "yUSD should have increased: {} -> {}",
                initial_yusd,
                final_yusd
            );

            println!(
                "Synatra stake_token successful! USDC: {} -> {}, yUSD: {} -> {}",
                initial_usdc, final_usdc, initial_yusd, final_yusd
            );
        }
        Err(e) => {
            panic!("Synatra stake_token CPI failed: {}", e);
        }
    }
}
