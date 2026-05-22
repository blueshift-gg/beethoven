use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, saros_amm_fixtures_dir,
        send_transaction, setup_svm, SAROS_AMM_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const USDT_MINT: Address = address!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const SWAP_INFO: Address = address!("DtLM35DLrZTCPsh3WYRDi36528aZiNsTwXk71EJTjRFG");
const AUTHORITY_INFO: Address = address!("DSyCtcQDxy6y2iebzVdxpRXiiN6fumQozS3eVPnQgAmT");
const USDT_SWAP_SOURCE: Address = address!("gziKuBRMtdcHSzAQRJLFbkLQHB7DQKx1Hz83APnEQYT");
const USDC_SWAP_SOURCE: Address = address!("2PNC93VZsyd38QD23NApjhesUCMZSNcArsj4xzB74rQ3");
const POOL_MINT: Address = address!("9LHtzoDpKgqS7jMr4RHruTvxHDZKPcKnQvcbm4LUfpwN");
const POOL_FEE_ACCOUNT: Address = address!("CRRfsi4W5ZgyC2M79yWhma7CBZ9qgg8GHFiqU7poyy2f");

#[test]
fn test_saros_amm_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Saros AMM program
    load_program(
        &mut svm,
        SAROS_AMM_PROGRAM_ID,
        &format!("{}/saros_amm.so", saros_amm_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdt_mint.json", saros_amm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_usdt_swap_info.json", saros_amm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/pool_mint_info.json", saros_amm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/pool_fee_account_info.json", saros_amm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdt_swap_source_info.json", saros_amm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_swap_source_info.json", saros_amm_fixtures_dir()),
    );

    // Create trader token accounts with initial balances
    // Selling USDT (input=USDT) for USDC (output)
    let initial_usdt = 10_000_000u64;
    let initial_usdc = 0u64;
    let trader_input =
        create_token_account(&mut svm, &payer.pubkey(), &USDT_MINT, initial_usdt, false);
    let trader_output =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);

    // Build swap instruction: sell 1 USDT for USDC
    let in_amount = 1_000_000u64; // 1
    let min_out_amount = 1u64; // Very loose slippage for test

    // Saros AMM accounts layout (11 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(SAROS_AMM_PROGRAM_ID, false),
        AccountMeta::new_readonly(SWAP_INFO, false),
        AccountMeta::new_readonly(AUTHORITY_INFO, false),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(trader_input, false),
        AccountMeta::new(USDT_SWAP_SOURCE, false),
        AccountMeta::new(USDC_SWAP_SOURCE, false),
        AccountMeta::new(trader_output, false),
        AccountMeta::new(POOL_MINT, false),
        AccountMeta::new(POOL_FEE_ACCOUNT, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
    ];

    // Saros AMM swap has no extra data
    let extra_data: &[u8] = &[];
    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::SarosAmm,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_usdt = get_token_balance(&svm, &trader_input);
            let final_usdc = get_token_balance(&svm, &trader_output);

            assert!(
                final_usdt < initial_usdt,
                "USDT should have decreased: {} -> {}",
                initial_usdt,
                final_usdt
            );
            assert!(
                final_usdc > initial_usdc,
                "USDC should have increased: {} -> {}",
                initial_usdc,
                final_usdc
            );

            println!(
                "Saros AMM swap successful! USDT: {} -> {}, USDC: {} -> {}",
                initial_usdt, final_usdt, initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("Saros AMM swap CPI failed: {}", e);
        }
    }
}
