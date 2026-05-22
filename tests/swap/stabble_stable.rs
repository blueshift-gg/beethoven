use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, send_transaction, setup_svm,
        stabble_stable_fixtures_dir, STABBLE_STABLE_PROGRAM_ID, TEST_PROGRAM_ID,
        TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDT_MINT: Address = address!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
const POOL: Address = address!("CXeH7npzb5UPWfB1TesjwxjXMT31XT4pjeUAQ4z65Wpg");
const USDC_VAULT: Address = address!("AioJRQXvcDLRhHMd6DAkTbbMpgVx63qSGQYmRBS2vHYA");
const USDT_VAULT: Address = address!("95QUtvDkuoDZrNJiuh9MdahkpRNtSVhZRe83oepd8AM7");
const VAULT_STATE: Address = address!("stab1io8dHvK26KoHmTwwHyYmHRbUWbyEJx6CdrGabC");
const VAULT_PROGRAM: Address = address!("vo1tWgqZMjG61Z2T9qUaMYKqZ75CYzMuaZ2LZP1n7HV");
const BENEFICIARY_TOKEN_OUT: Address = address!("4CQUrzq6qaVtMVtWEL2CvaZjqBUxnMJtBgM6M3hHHDsJ");
const WITHDRAW_AUTHORITY: Address = address!("8BSWYgAczR36C7ukr32v7uTepoRhYJYxAVnpBtYniZTm");
const VAULT_AUTHORITY: Address = address!("7imnGYfCovXjMWKdbQvETFVMe72MQDX4S5zW4GFxMJME");

#[test]
fn test_stabble_stable_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Stabble Stable program
    load_program(
        &mut svm,
        STABBLE_STABLE_PROGRAM_ID,
        &format!("{}/stabble_stable.so", stabble_stable_fixtures_dir()),
    );
    load_program(
        &mut svm,
        VAULT_PROGRAM,
        &format!("{}/vault_program.so", stabble_stable_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdt_mint.json", stabble_stable_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/usdc_usdt_usdg_usd1_pool.json",
            stabble_stable_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_vault.json", stabble_stable_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdt_vault.json", stabble_stable_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdt_fee_account.json", stabble_stable_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_authority.json", stabble_stable_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_state.json", stabble_stable_fixtures_dir()),
    );

    // Create trader token accounts with initial balances
    // Selling USDC (input=USDC) for USDT (output)
    let initial_usdc = 1_000_000_000u64; // 1 USDC
    let initial_usdt = 0u64;
    let trader_usdc =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);
    let trader_usdt =
        create_token_account(&mut svm, &payer.pubkey(), &USDT_MINT, initial_usdt, false);

    // Build swap instruction: sell 1 USDC for USDT
    let in_amount = 1_000_000u64; // 1 USDC
    let min_out_amount = 1u64; // Very loose slippage for test

    // Stabble Stable accounts layout (16 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(STABBLE_STABLE_PROGRAM_ID, false), // stabble_program
        AccountMeta::new(payer.pubkey(), true),                      // user
        AccountMeta::new_readonly(USDC_MINT, false),                 // mint_in
        AccountMeta::new_readonly(USDT_MINT, false),                 // mint_out
        AccountMeta::new(trader_usdc, false),                        // user_token_in
        AccountMeta::new(trader_usdt, false),                        // user_token_out
        AccountMeta::new(USDC_VAULT, false),                         // vault_token_in
        AccountMeta::new(USDT_VAULT, false),                         // vault_token_out
        AccountMeta::new(BENEFICIARY_TOKEN_OUT, false),              // beneficiary_token_out
        AccountMeta::new(POOL, false),                               // pool
        AccountMeta::new_readonly(WITHDRAW_AUTHORITY, false),        // withdraw_authority
        AccountMeta::new_readonly(VAULT_STATE, false),               // vault
        AccountMeta::new_readonly(VAULT_AUTHORITY, false),           // vault_authority
        AccountMeta::new_readonly(VAULT_PROGRAM, false),             // vault_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),          // token_program
        AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),     // token_2022_program
    ];

    // Stabble Stable swap has no extra data
    let extra_data: &[u8] = &[];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::StabbleStable,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_usdc = get_token_balance(&svm, &trader_usdc);
            let final_usdt = get_token_balance(&svm, &trader_usdt);

            assert!(
                final_usdc < initial_usdc,
                "USDC should have decreased: {} -> {}",
                initial_usdc,
                final_usdt
            );
            assert!(
                final_usdt > initial_usdt,
                "USDT should have increased: {} -> {}",
                initial_usdt,
                final_usdt
            );

            println!(
                "Stabble Stable swap successful! USDC: {} -> {}, USDT: {} -> {}",
                initial_usdc, final_usdc, initial_usdt, final_usdt
            );
        }
        Err(e) => {
            panic!("Stabble Stable swap CPI failed: {}", e);
        }
    }
}
