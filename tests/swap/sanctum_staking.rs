use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, create_token_account, get_token_balance,
        load_and_set_json_fixture, load_program, sanctum_staking_fixtures_dir, send_transaction,
        setup_svm, SANCTUM_STAKING_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

pub const CLOUD_MINT: Address = address!("CLoUDKc4Ane7HeQcPpE3YHnznRxhMimJ4MyaUqyHFzAu");
pub const SCLOUD_MINT: Address = address!("sc1dNAxRBj5CNWaGC26AR7PEW75R36Umzt1V8vuP8kZ");
pub const VAULT: Address = address!("5jbzpJeGZFpPFrwXAdeWn25UJiParK8rayQYJY3r14cv");
pub const BOND_MINT_AUTHORITY: Address = address!("3vLkpgiPPupTLfQ3WHw6zPVrjKVsB18Aiaz9sCqfhE3n");
pub const BOND_POOL: Address = address!("8DFDU25Rzgx9bp4VUirZPykADdnVDLehi5s9enMqmXpq");

#[test]
fn test_sanctum_staking_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Sanctum Staking program
    load_program(
        &mut svm,
        SANCTUM_STAKING_PROGRAM_ID,
        &format!("{}/sanctum_staking.so", sanctum_staking_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/cloud_mint.json", sanctum_staking_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/scloud_mint.json", sanctum_staking_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault.json", sanctum_staking_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/bond_pool.json", sanctum_staking_fixtures_dir()),
    );

    // Create trader token accounts with initial balances
    // Selling CLOUD for sCLOUD
    let initial_cloud = 1_000_000_000u64; // 1 CLOUD
    let initial_scloud = 0u64;
    let trader_cloud =
        create_token_account(&mut svm, &payer.pubkey(), &CLOUD_MINT, initial_cloud, false);
    let trader_scloud = create_token_account(
        &mut svm,
        &payer.pubkey(),
        &SCLOUD_MINT,
        initial_scloud,
        false,
    );

    // Build swap instruction: sell 0.1 CLOUD for sCLOUD
    let in_amount = 100_000_000u64; // 0.1 CLOUD
    let min_out_amount = 1u64; // Very loose slippage for test

    // Sanctum Staking accounts layout (9 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(SANCTUM_STAKING_PROGRAM_ID, false),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(trader_cloud, false),
        AccountMeta::new(trader_scloud, false),
        AccountMeta::new(VAULT, false),
        AccountMeta::new(SCLOUD_MINT, false),
        AccountMeta::new_readonly(BOND_MINT_AUTHORITY, false),
        AccountMeta::new_readonly(BOND_POOL, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
    ];

    // Sanctum Staking swap has no extra data
    let extra_data: &[u8] = &[];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::SanctumStaking,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_cloud = get_token_balance(&svm, &trader_cloud);
            let final_scloud = get_token_balance(&svm, &trader_scloud);

            assert!(
                final_cloud < initial_cloud,
                "CLOUD should have decreased: {} -> {}",
                initial_cloud,
                final_cloud
            );
            assert!(
                final_scloud > initial_scloud,
                "sCLOUD should have increased: {} -> {}",
                initial_scloud,
                final_scloud
            );

            println!(
                "Sanctum Staking swap successful! CLOUD: {} -> {}, sCLOUD: {} -> {}",
                initial_cloud, final_cloud, initial_scloud, final_scloud
            );
        }
        Err(e) => {
            panic!("Sanctum Staking swap CPI failed: {}", e);
        }
    }
}
