use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, scale_amm_fixtures_dir,
        scale_vmm_fixtures_dir, send_transaction, setup_svm, SCALE_AMM_PROGRAM_ID,
        SCALE_VMM_PROGRAM_ID, SYSTEM_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_account::Account,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const PAIR: Address = address!("BWnowWbMBTfsLzTKgZM7vnh8SxbutJA2HB4z7Labswkb");
const MINT_B: Address = address!("3CYBUFXzQ7GJiYoxMfrFjsPNVaSkp2vVFXXLefb57chr");
const VAULT_A: Address = address!("Tf8aks7NB82QXob8NAoorzrPFHwkuPoZ9GVLYNLjYYZ");
const VAULT_B: Address = address!("AaP7zPXf22rpmX27n4t6ohRDPLx7mcXDeodFN9rmbhQh");
const PLATFORM_FEE_TA_A: Address = address!("5otzrfbppNE1j6m7ptWkAcu5gs1nwj5ZQKQVwFFHQAHv");
const PLATFORM_CONFIG: Address = address!("8DxXv6ikV38rCepX3esVCHMb2wMnnnXpp7xYasGSc6bo");
const AMM_POOL: Address = address!("2Lt3pqPLCDzxizyHM8cx1auySxNYxFwmf1JwnEfJtHbw");
const AMM_VAULT_A: Address = address!("8drJbd2DZxcMqDcv8AZf1C1Y7wKBHS2JWXfEgUkjKwtz");
const AMM_VAULT_B: Address = address!("6K5ekuXPF7iAyWpMkTijenLQxtQYZJozY7PJjfFCZSp4");
const AMM_CONFIG: Address = address!("232KbYciAe6ma2VCB6gQyofix8qQwyZd2WYVhpNx8SyR");

#[test]
fn test_scale_vmm_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Scale VMM program
    load_program(
        &mut svm,
        SCALE_VMM_PROGRAM_ID,
        &format!("{}/scale_vmm.so", scale_vmm_fixtures_dir()),
    );
    // Scale VMM swap CPIs into Scale AMM
    load_program(
        &mut svm,
        SCALE_AMM_PROGRAM_ID,
        &format!("{}/scale_amm.so", scale_amm_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/wsol_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/mint_b.json", scale_vmm_fixtures_dir()),
    );
    load_and_set_json_fixture(&mut svm, &format!("{}/pair.json", scale_vmm_fixtures_dir()));
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/platform_config.json", scale_vmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/platform_fee_ta_a.json", scale_vmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_a.json", scale_vmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_b.json", scale_vmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/amm_pool.json", scale_vmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/amm_vault_a.json", scale_vmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/amm_vault_b.json", scale_vmm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/amm_config.json", scale_vmm_fixtures_dir()),
    );

    let pair_account = svm.get_account(&PAIR).unwrap();
    let mut pair_account_data = pair_account.data;

    // override pair enabled bool to true
    pair_account_data[8] = 1;
    // override pair graduated bool to false
    pair_account_data[9] = 0;
    // override pair token a reserves to 362_316_180
    pair_account_data[74..90].copy_from_slice(&362_316_180_u128.to_le_bytes());
    // override pair token b reserves to 724_722_561_545_732
    pair_account_data[90..106].copy_from_slice(&724_722_561_545_732_u128.to_le_bytes());

    svm.set_account(
        PAIR,
        Account {
            data: pair_account_data,
            executable: pair_account.executable,
            lamports: pair_account.lamports,
            owner: pair_account.owner,
            rent_epoch: pair_account.rent_epoch,
        },
    )
    .unwrap();

    let vault_b_account = svm.get_account(&VAULT_B).unwrap();
    let mut vault_b_account_data = vault_b_account.data;

    // override vault_b amount to 724_722_561_545_732
    vault_b_account_data[64..72].copy_from_slice(&724_722_561_545_732_u64.to_le_bytes());

    svm.set_account(
        VAULT_B,
        Account {
            data: vault_b_account_data,
            executable: vault_b_account.executable,
            lamports: vault_b_account.lamports,
            owner: vault_b_account.owner,
            rent_epoch: vault_b_account.rent_epoch,
        },
    )
    .unwrap();

    // Create trader token accounts with initial balances
    // Selling SOL (input=WSOL) for USDC (output)
    let initial_wsol = 1_000_000_000u64; // 1 SOL
    let initial_mint_b = 0u64;
    let trader_input =
        create_token_account(&mut svm, &payer.pubkey(), &WSOL_MINT, initial_wsol, false);
    let trader_output =
        create_token_account(&mut svm, &payer.pubkey(), &MINT_B, initial_mint_b, false);

    // Build swap instruction: sell 0.001 SOL for USDC
    let in_amount = 1_000_000u64; // 0.001 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    // Scale VMM: 19 fixed accounts (through amm_config) + optional fee beneficiary ATAs
    let accounts = vec![
        AccountMeta::new_readonly(SCALE_VMM_PROGRAM_ID, false), // scale_vmm program
        AccountMeta::new(PAIR, false),                          // pair
        AccountMeta::new(payer.pubkey(), true),                 // user
        AccountMeta::new_readonly(WSOL_MINT, false),            // mint a
        AccountMeta::new_readonly(MINT_B, false),               // mint b
        AccountMeta::new(trader_input, false),                  // user ta a
        AccountMeta::new(trader_output, false),                 // user ta b
        AccountMeta::new(VAULT_A, false),                       // vault a
        AccountMeta::new(VAULT_B, false),                       // vault b
        AccountMeta::new(PLATFORM_FEE_TA_A, false),             // platform fee ta a
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),     // token program a
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),     // token program b
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),    // system program
        AccountMeta::new_readonly(PLATFORM_CONFIG, false),      // config
        AccountMeta::new_readonly(SCALE_AMM_PROGRAM_ID, false), // amm program
        AccountMeta::new(AMM_POOL, false),                      // amm pool
        AccountMeta::new(AMM_VAULT_A, false),                   // amm vault a
        AccountMeta::new(AMM_VAULT_B, false),                   // amm vault b
        AccountMeta::new_readonly(AMM_CONFIG, false),           // amm config
    ];

    // side = buy
    let extra_data: &[u8] = &[0u8];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::ScaleVmm,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_wsol = get_token_balance(&svm, &trader_input);
            let final_mint_b = get_token_balance(&svm, &trader_output);

            assert!(
                final_wsol < initial_wsol,
                "WSOL should have decreased: {} -> {}",
                initial_wsol,
                final_wsol
            );
            assert!(
                final_mint_b > initial_mint_b,
                "MINT_B should have increased: {} -> {}",
                initial_mint_b,
                final_mint_b
            );
        }
        Err(e) => panic!("Scale VMM swap CPI failed: {}", e),
    }
}
