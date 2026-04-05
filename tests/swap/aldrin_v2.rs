use {
    crate::helper::{
        aldrin_v2_fixtures_dir, beethoven_program_path, build_swap_instruction,
        common_fixtures_dir, create_token_account, get_token_balance, load_and_set_json_fixture,
        load_program, send_transaction, setup_svm, ALDRIN_V2_PROGRAM_ID, TEST_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const BONK_MINT: Address = address!("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263");

const POOL: Address = address!("BMFQxq1x9GBsSDpHVf8iMVmmoB8rR9sJq3JSSE2kDQwG");
const POOL_SIGNER: Address = address!("6yioeAnhko4Vzw9CZsncwaHDz5S12YjTms3eGjhrGJUX");
const POOL_MINT: Address = address!("3SbhGXhZMSoLJKMpGEvb29ZEsXcgmyTizMsjFYJxWPgL");
const USDC_VAULT: Address = address!("J94S454qunF5xH4UdyXbv6j8kTneLajFYzjTpfHJ62cU");
const BONK_VAULT: Address = address!("3hWGKXEtKbAyArLb4y4hpMyhqqXXp4HM1jPrz4mPjxcJ");
const FEE_POOL_TOKEN_ACCOUNT: Address = address!("4eRYmUET6EtaVHwYmgxJeHTXVdYJG5pXmpixQ175RAiQ");
const CURVE: Address = address!("2LnTcdBH6zytWoDkTgUakgHmYf9eP51MgkBdTGow4jQL");

#[test]
fn test_aldrin_v2_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Aldrin v2 program
    load_program(
        &mut svm,
        ALDRIN_V2_PROGRAM_ID,
        &format!("{}/aldrin_v2.so", aldrin_v2_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/bonk_mint.json", aldrin_v2_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_bonk_pool.json", aldrin_v2_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_bonk_pool_mint.json", aldrin_v2_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_vault.json", aldrin_v2_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/bonk_vault.json", aldrin_v2_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/fee_pool_token_account.json", aldrin_v2_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/product_curve.json", aldrin_v2_fixtures_dir()),
    );

    // Create trader token accounts with initial balances
    // Selling USDC (input) for BONK (output)
    let initial_usdc = 1_000_000_000u64;
    let initial_bonk = 0u64;
    let trader_input =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);
    let trader_output =
        create_token_account(&mut svm, &payer.pubkey(), &BONK_MINT, initial_bonk, false);

    // Build swap instruction: sell 1 USDC for BONK
    let in_amount = 1_000_000u64; // 1 USDC (6 decimals)
    let min_out_amount = 1u64; // Very loose slippage for test

    // Aldrin v2 accounts layout (12 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(ALDRIN_V2_PROGRAM_ID, false), // aldrin_v2_program
        AccountMeta::new_readonly(POOL, false),                 // pool
        AccountMeta::new_readonly(POOL_SIGNER, false),          // pool_signer
        AccountMeta::new(POOL_MINT, false),                     // pool_mint
        AccountMeta::new(USDC_VAULT, false),                    // base_token_vault
        AccountMeta::new(BONK_VAULT, false),                    // quote_token_vault
        AccountMeta::new(FEE_POOL_TOKEN_ACCOUNT, false),        // fee_pool_token_account
        AccountMeta::new(payer.pubkey(), true),                 // wallet_authority
        AccountMeta::new(trader_input, false),                  // user_base_token_account
        AccountMeta::new(trader_output, false),                 // user_quote_token_account
        AccountMeta::new_readonly(CURVE, false),                // curve
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),     // token_program
    ];

    // Ask = 1 (sell base for quote)
    let extra_data: &[u8] = &[1u8];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::AldrinV2,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_usdc = get_token_balance(&svm, &trader_input);
            let final_bonk = get_token_balance(&svm, &trader_output);

            assert!(
                final_usdc < initial_usdc,
                "USDC should have decreased: {} -> {}",
                initial_usdc,
                final_usdc
            );
            assert!(
                final_bonk > initial_bonk,
                "BONK should have increased: {} -> {}",
                initial_bonk,
                final_bonk
            );

            println!(
                "Aldrin v2 swap successful! USDC: {} -> {}, BONK: {} -> {}",
                initial_usdc, final_usdc, initial_bonk, final_bonk
            );
        }
        Err(e) => {
            panic!("Aldrin v2 swap CPI failed: {}", e);
        }
    }
}
