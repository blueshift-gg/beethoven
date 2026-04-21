use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir,
        create_token_account_at, get_token_balance, load_and_set_json_fixture, load_program,
        send_transaction, setup_svm, voltr_fixtures_dir, SYSTEM_PROGRAM_ID, TEST_PROGRAM_ID,
        TOKEN_PROGRAM_ID, VOLTR_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    beethoven_client::get_associated_token_address,
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const PROTOCOL: Address = address!("4sycXz9Xwevedo6eiXR8QEhY8yrQrkNS4G1deY9tAD2Y");
const RAUSDC_MINT: Address = address!("53fZaJGDMHcfku8pzZak5obVFUUjVxwqRTF63M3SQiSS");
const VAULT: Address = address!("3maCuTJVPteZ2dFA8dADxz2EbpJHfoAG5txYhXDs6gNQ");
const VAULT_ASSET_IDLE_ATA: Address = address!("3iKiu9CYBqNSPJ9GdNd46BGMFtwQ27N1qJpXSocpo5wm");
const VAULT_ASSET_IDLE_AUTH: Address = address!("F5FT74NET1Y6JTJyNCioGYpyWXqEYTnvNfb6gh7aM8Yn");
const VAULT_LP_MINT_AUTH: Address = address!("FFh6frp7DsAyCkP1275yVndhpaWfMtevk9sk6BrZB7V8");

#[test]
fn test_voltr_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Voltr program
    load_program(
        &mut svm,
        VOLTR_PROGRAM_ID,
        &format!("{}/voltr.so", voltr_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/rausdc_mint.json", voltr_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/hubra_vault.json", voltr_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/hubra_vault_idle_ata.json", voltr_fixtures_dir()),
    );
    load_and_set_json_fixture(&mut svm, &format!("{}/protocol.json", voltr_fixtures_dir()));

    // Jump ahead of vault last_updated_ts (1_776_795_584) + 1
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_776_795_584 + 1;
    svm.set_sysvar::<Clock>(&clock);

    // deposit_vault

    // Create trader token accounts with initial balances
    // Selling USDC (input) for raUSDC (output)
    let initial_usdc = 100_000_000u64; // 100 USDC
    let initial_rausdc = 0u64;
    let trader_usdc = get_associated_token_address(&payer.pubkey(), &USDC_MINT, &TOKEN_PROGRAM_ID);
    let trader_rausdc =
        get_associated_token_address(&payer.pubkey(), &RAUSDC_MINT, &TOKEN_PROGRAM_ID);
    create_token_account_at(
        &mut svm,
        trader_usdc,
        &payer.pubkey(),
        &USDC_MINT,
        initial_usdc,
        false,
    );
    create_token_account_at(
        &mut svm,
        trader_rausdc,
        &payer.pubkey(),
        &RAUSDC_MINT,
        initial_rausdc,
        false,
    );

    // Build swap instruction: sell 10 USDC for raUSDC
    let in_amount = 10_000_000u64; // 10 USDC
    let min_out_amount = 1u64; // Very loose slippage for test

    // Voltr deposit_vault accounts layout (14 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(VOLTR_PROGRAM_ID, false), // voltr_program
        AccountMeta::new(payer.pubkey(), true),             // user_transfer_authority
        AccountMeta::new_readonly(PROTOCOL, false),         // protocol
        AccountMeta::new(VAULT, false),                     // vault
        AccountMeta::new_readonly(USDC_MINT, false),        // vault_asset_mint
        AccountMeta::new(RAUSDC_MINT, false),               // vault_lp_mint
        AccountMeta::new(trader_usdc, false),               // user_asset_ata
        AccountMeta::new(VAULT_ASSET_IDLE_ATA, false),      // vault_asset_idle_ata
        AccountMeta::new_readonly(VAULT_ASSET_IDLE_AUTH, false), // vault_asset_idle_auth
        AccountMeta::new(trader_rausdc, false),             // user_lp_ata
        AccountMeta::new_readonly(VAULT_LP_MINT_AUTH, false), // vault_lp_mint_auth
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false), // asset_token_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false), // lp_token_program
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false), // system_program
    ];

    // Voltr deposit_vault swap has no extra data
    let extra_data: &[u8] = &[];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::Voltr,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_usdc = get_token_balance(&svm, &trader_usdc);
            let final_rausdc = get_token_balance(&svm, &trader_rausdc);

            assert!(
                final_usdc < initial_usdc,
                "USDC should have decreased: {} -> {}",
                initial_usdc,
                final_usdc
            );
            assert!(
                final_rausdc > initial_rausdc,
                "raUSDC should have increased: {} -> {}",
                initial_rausdc,
                final_rausdc
            );

            println!(
                "Voltr deposit_vault swap successful! USDC: {} -> {}, raUSDC: {} -> {}",
                initial_usdc, final_usdc, initial_rausdc, final_rausdc
            );
        }
        Err(e) => {
            panic!("Voltr deposit_vault swap CPI failed: {}", e);
        }
    }

    // instant_withdraw_vault

    // Create trader token accounts with initial balances
    // Selling raUSDC (input) for USDC (output)
    let initial_usdc = get_token_balance(&svm, &trader_usdc);
    let initial_rausdc = get_token_balance(&svm, &trader_rausdc);

    // Build swap instruction: sell all raUSC for USDC
    let in_amount = 1u64; // doesn't mattter, full amount is set in extra data
    let min_out_amount = 1u64; // Very loose slippage for test

    // Voltr instant_withdraw_vault accounts layout (13 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(VOLTR_PROGRAM_ID, false), // voltr_program
        AccountMeta::new(payer.pubkey(), true),             // user_transfer_authority
        AccountMeta::new_readonly(PROTOCOL, false),         // protocol
        AccountMeta::new(VAULT, false),                     // vault
        AccountMeta::new_readonly(USDC_MINT, false),        // vault_asset_mint
        AccountMeta::new(RAUSDC_MINT, false),               // vault_lp_mint
        AccountMeta::new(trader_rausdc, false),             // user_lp_ata
        AccountMeta::new(VAULT_ASSET_IDLE_ATA, false),      // vault_asset_idle_ata
        AccountMeta::new_readonly(VAULT_ASSET_IDLE_AUTH, false), // vault_asset_idle_auth
        AccountMeta::new(trader_usdc, false),               // user_asset_ata
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false), // asset_token_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false), // lp_token_program
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false), // system_program
    ];

    // is_amount_in_lp = false, is_withdraw_all = true
    let extra_data: &[u8] = &[0_u8, 1_u8];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::Voltr,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_rausdc = get_token_balance(&svm, &trader_rausdc);
            let final_usdc = get_token_balance(&svm, &trader_usdc);

            assert!(
                final_rausdc < initial_rausdc,
                "raUSDC should have decreased: {} -> {}",
                initial_rausdc,
                final_rausdc
            );
            assert!(
                final_usdc > initial_usdc,
                "USDC should have increased: {} -> {}",
                initial_usdc,
                final_usdc
            );

            println!(
                "Voltr instant_withdraw_vault swap successful! raUSDC: {} -> {}, USDC: {} -> {}",
                initial_usdc, final_usdc, initial_rausdc, final_rausdc
            );
        }
        Err(e) => {
            panic!("Voltr instant_withdraw_vault swap CPI failed: {}", e);
        }
    }
}
