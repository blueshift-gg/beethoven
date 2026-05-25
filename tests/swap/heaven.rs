use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, heaven_fixtures_dir, load_and_set_json_fixture, load_program,
        send_transaction, send_transaction_with_signers, setup_svm, ASSOCIATED_TOKEN_PROGRAM_ID,
        HEAVEN_PROGRAM_ID, INSTRUCTIONS_SYSVAR_ID, SYSTEM_PROGRAM_ID, TEST_PROGRAM_ID,
        TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
    spl_associated_token_account_interface::{
        address::get_associated_token_address_with_program_id,
        instruction::create_associated_token_account,
    },
    spl_token_2022_interface::{extension::StateWithExtensions, state::Account},
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const LIGHT_MINT: Address = address!("LiGHtkg3uTa9836RaNkKLLriqTNRcMdRAhqjGWNv777");
const SOL_LIGHT_LIQUIDITY_POOL_STATE: Address =
    address!("EkU9zGSkUnVVK6nhmPSqnxqcKPzt1PicrCjdxSbWo9uA");
const PROTOCOL_CONFIG: Address = address!("KpXrCt3pjJYFind2kgk7nQ3dS6bqjC2Ze3zzE5MQ78v");
const WSOL_VAULT: Address = address!("HBw4rhjiJ1cXDNQz7395QJ51DskLknwHRAjxYzgBsYnK");
const LIGHT_VAULT: Address = address!("FjCZrwymiMvdufnrPZLP6NvgZDY8j9KGnLakRic3vQi7");
const CHAINLINK_SOL_USD_FEED: Address = address!("CH31Xns5z3M1cTAbKW34jcxPPciazARpijcHj9rxtemt");
const CHAINLINK_STORE_PROGRAM_ID: Address =
    address!("HEvSKofvBgfaexv23kMabbYqxasxU3mQ4ibBMEmJWHny");

#[test]
fn test_heaven_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load programs
    load_program(
        &mut svm,
        HEAVEN_PROGRAM_ID,
        &format!("{}/heaven.so", heaven_fixtures_dir()),
    );
    load_program(
        &mut svm,
        CHAINLINK_STORE_PROGRAM_ID,
        &format!("{}/chainlink_store_program.so", heaven_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/wsol_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/light_mint.json", heaven_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/sol_light_liquidity_pool_state.json",
            heaven_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/protocol_config.json", heaven_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/heaven_wsol_vault.json", heaven_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/heaven_light_vault.json", heaven_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/chainlink_transmissions_sol_usd.json",
            heaven_fixtures_dir()
        ),
    );

    // Jump ahead to pool info open_at timestamp (1_754_944_645) + 1
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_754_944_645 + 1;
    // Jump ahead to pool reserve snapshot_slot (411_351_016) + 1
    clock.slot = 411_351_016 + 1;
    svm.set_sysvar::<Clock>(&clock);

    // Create trader token accounts with initial balances
    // Selling WSOL for LIGHT (output)
    let initial_wsol = 1_000_000_000u64;
    let initial_light = 0u64;
    let trader_input =
        create_token_account(&mut svm, &payer.pubkey(), &WSOL_MINT, initial_wsol, false);

    // Build and send custom initialize token account instruction to include immutable owner and transfer fee amount extensions
    let trader_output = get_associated_token_address_with_program_id(
        &payer.pubkey(),
        &LIGHT_MINT,
        &TOKEN_2022_PROGRAM_ID,
    );
    let create_trader_output_ata_ix = create_associated_token_account(
        &payer.pubkey(),
        &payer.pubkey(),
        &LIGHT_MINT,
        &TOKEN_2022_PROGRAM_ID,
    );
    send_transaction_with_signers(&mut svm, &payer, &[&payer], create_trader_output_ata_ix)
        .unwrap();

    // Build swap instruction: sell 0.001 SOL for LIGHT
    let in_amount = 1_000_000u64; // 0.001 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    // Heaven accounts layout (17 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(HEAVEN_PROGRAM_ID, false), // heaven_program
        AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false), // token_a_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),  // token_b_program
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false), // associated_token_program
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false), // system_program
        AccountMeta::new(SOL_LIGHT_LIQUIDITY_POOL_STATE, false), // liquidity_pool_state
        AccountMeta::new(payer.pubkey(), true),              // user
        AccountMeta::new_readonly(LIGHT_MINT, false),        // token_a_mint
        AccountMeta::new_readonly(WSOL_MINT, false),         // token_b_mint
        AccountMeta::new(trader_output, false),              // user_token_a_vault
        AccountMeta::new(trader_input, false),               // user_token_b_vault
        AccountMeta::new(LIGHT_VAULT, false),                // token_a_vault
        AccountMeta::new(WSOL_VAULT, false),                 // token_b_vault
        AccountMeta::new(PROTOCOL_CONFIG, false),            // protocol_config
        AccountMeta::new_readonly(INSTRUCTIONS_SYSVAR_ID, false), // instruction_sysvar_account_info
        AccountMeta::new_readonly(CHAINLINK_STORE_PROGRAM_ID, false), // chainlink_program
        AccountMeta::new_readonly(CHAINLINK_SOL_USD_FEED, false), // chainlink_sol_usd_price_feed
    ];

    // direction = 0, encoded user defined event data = ""
    let extra_data: &[u8] = &[0u8];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::Heaven,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_wsol = get_token_balance(&svm, &trader_input);
            let final_light = {
                let account = svm
                    .get_account(&trader_output)
                    .expect("Token account not found");
                StateWithExtensions::<Account>::unpack(&account.data)
                    .expect("Failed to unpack token-2022 account")
                    .base
                    .amount
            };

            assert!(
                final_wsol < initial_wsol,
                "WSOL should have decreased: {} -> {}",
                initial_wsol,
                final_wsol
            );
            assert!(
                final_light > initial_light,
                "LIGHT should have increased: {} -> {}",
                initial_light,
                final_light
            );
        }
        Err(e) => panic!("Heaven swap CPI failed: {}", e),
    }
}
