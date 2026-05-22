use {
    crate::helper::{
        alphaq_fixtures_dir, beethoven_program_path, build_swap_instruction, common_fixtures_dir,
        create_token_account, get_token_balance, load_and_set_json_fixture, load_program,
        send_transaction, setup_svm, ALPHAQ_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    beethoven_client::SYSVAR_INSTRUCTIONS_ID,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const USDT_MINT: Address = address!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const MARKET: Address = address!("Pi9nzTjPxD8DsRfRBGfKYzmefJoJM8TcXu2jyaQjSHm");
const MARKET_STATE: Address = address!("445fd6ffBZqWYsryCgs6wcE8exaLkRsMrefAQ5UHvt8v");
const VAULT_TA_A: Address = address!("GF8SKKobum6UJnhX2mLHePU38htg5vdr9zcY4jH8Pqs2");
const VAULT_TA_B: Address = address!("F2KCaXcp7AoQtxTDvNEDCyMyWjSCAMWNzcyN9dsPfPs5");

#[test]
#[ignore = "would throw with InvalidAccountOwner error, likely due to instruction not called from a whitelisted router"]
fn test_alphaq_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load AlphaQ program
    load_program(
        &mut svm,
        ALPHAQ_PROGRAM_ID,
        &format!("{}/alphaq.so", alphaq_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdt_mint.json", alphaq_fixtures_dir()),
    );
    load_and_set_json_fixture(&mut svm, &format!("{}/market.json", alphaq_fixtures_dir()));
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/market_state.json", alphaq_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_ta_a.json", alphaq_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_ta_b.json", alphaq_fixtures_dir()),
    );

    // Create trader token accounts with initial balances
    // Selling USDT for USDC
    let initial_usdt = 100_000_000u64; // 100 USDT
    let initial_usdc = 0u64;
    let trader_usdt =
        create_token_account(&mut svm, &payer.pubkey(), &USDT_MINT, initial_usdt, false);
    let trader_usdc =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);

    // Build swap instruction: sell 1 USDT for USDC
    let in_amount = 1_000_000u64; // 1 USDT
    let min_out_amount = 1u64; // Very loose slippage for test

    // AlphaQ accounts layout (13 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(ALPHAQ_PROGRAM_ID, false), // alphaq_program
        AccountMeta::new(payer.pubkey(), true),              // user
        AccountMeta::new(MARKET, false),                     // market
        AccountMeta::new(MARKET_STATE, false),               // market_state
        AccountMeta::new(trader_usdt, false),                // user_ata_a
        AccountMeta::new(trader_usdc, false),                // user_ata_b
        AccountMeta::new(VAULT_TA_A, false),                 // vault_ta_a
        AccountMeta::new(VAULT_TA_B, false),                 // vault_ta_b
        AccountMeta::new(VAULT_TA_A, false),                 // token_authority_a
        AccountMeta::new(VAULT_TA_B, false),                 // token_authority_b
        AccountMeta::new(VAULT_TA_B, false),                 // vendor_key
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),  // token_program
        AccountMeta::new_readonly(SYSVAR_INSTRUCTIONS_ID, false), // instructions_sysvar
    ];

    // a_to_b = true
    let extra_data: &[u8] = &[1u8];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::AlphaQ,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_usdt = get_token_balance(&svm, &trader_usdt);
            let final_usdc = get_token_balance(&svm, &trader_usdc);

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
                "AlphaQ swap successful! USDT: {} -> {}, USDC: {} -> {}",
                initial_usdt, final_usdt, initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("AlphaQ swap CPI failed: {}", e);
        }
    }
}
