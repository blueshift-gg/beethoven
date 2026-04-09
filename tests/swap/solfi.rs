use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, send_transaction, setup_svm,
        solfi_fixtures_dir, INSTRUCTIONS_SYSVAR_ID, SOLFI_PROGRAM_ID, TEST_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const WETH_MINT: Address = address!("7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs");
const MARKET: Address = address!("7NbPAgjn6W9xH6G1opqYAXVVuJBAtn67DDbD5PTCbA3o");
const BASE_VAULT: Address = address!("9PsHds1eaSLgTBmSefJ2KhjGRcVCbwP9SkaYiWFr31yP");
const QUOTE_VAULT: Address = address!("E5D142djU2atMNuLq8nPr4X2bgskqW32nhYWqPPzU1gS");

#[test]
fn test_solfi_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load SolFi program
    load_program(
        &mut svm,
        SOLFI_PROGRAM_ID,
        &format!("{}/solfi.so", solfi_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/weth_mint.json", solfi_fixtures_dir()),
    );
    load_and_set_json_fixture(&mut svm, &format!("{}/market.json", solfi_fixtures_dir()));
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/base_vault.json", solfi_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/quote_vault.json", solfi_fixtures_dir()),
    );

    // Create trader token accounts with initial balances
    // Selling USDC (input) for WETH (output)
    let initial_usdc = 100_000_000u64; // 100 USDC
    let initial_weth = 0u64;
    let trader_input =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);
    let trader_output =
        create_token_account(&mut svm, &payer.pubkey(), &WETH_MINT, initial_weth, false);

    // Build swap instruction: sell 10 USDC for WETH
    let in_amount = 10_000_000u64; // 10 USDC
    let min_out_amount = 1u64; // Very loose slippage for test

    // SolFi accounts layout (9 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(SOLFI_PROGRAM_ID, false), // solfi program
        AccountMeta::new(payer.pubkey(), true),             // user
        AccountMeta::new(MARKET, false),                    // market
        AccountMeta::new(BASE_VAULT, false),                // base vault
        AccountMeta::new(QUOTE_VAULT, false),               // quote vault
        AccountMeta::new(trader_output, false),             // user base ata
        AccountMeta::new(trader_input, false),              // user quote ata
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false), // token program
        AccountMeta::new_readonly(INSTRUCTIONS_SYSVAR_ID, false), // instructions sysvar
    ];

    // is_quote_to_base = true
    let extra_data: &[u8] = &[1_u8];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::SolFi,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_usdc = get_token_balance(&svm, &trader_input);
            let final_weth = get_token_balance(&svm, &trader_output);

            assert!(
                final_usdc < initial_usdc,
                "USDC should have decreased: {} -> {}",
                initial_usdc,
                final_usdc
            );
            assert!(
                final_weth > initial_weth,
                "WETH should have increased: {} -> {}",
                initial_weth,
                final_weth
            );
        }
        Err(e) => panic!("SolFi swap CPI failed: {}", e),
    }
}
