use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, create_token_account_at, get_token_balance,
        load_and_set_json_fixture, load_program, send_transaction, setup_svm,
        unitas_vault_fixtures_dir, ASSOCIATED_TOKEN_PROGRAM_ID, SUSDU_PROGRAM_ID,
        SYSTEM_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_2022_PROGRAM_ID, UNITAS_VAULT_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    beethoven_client::get_associated_token_address,
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const USDU_MINT: Address = address!("9ckR7pPPvyPadACDTzLwK2ZAEeUJ3qGSnzPs8bVaHrSy");
const SUSDU_MINT: Address = address!("9iq5Q33RSiz1WcupHAQKbHBZkpn92UxBG2HfPWAZhMCa");
const ACCESS_REGISTRY: Address = address!("8maav1g7bK1vRamXzADLUu3DQ7VmXxjVTJt9PbBuWcpd");
const VAULT_STAKE_POOL_USDU_TOKEN_ACCOUNT: Address =
    address!("CFgrWjb9DYKVqf7QyQfmwjboDDkXpFHQ6292rnYxrjsa");
const SUSDU_MINTER: Address = address!("6ZY9KMGD9UjTX4tWGcw4Y4UHh14nzNmiEr92wTieYub5");
const VAULT_STATE: Address = address!("Fx6AsfJ5GGzUyYgNAaGehG1PSQTG1e7qNwSCJ8vXJ9SX");
const VAULT_CONFIG: Address = address!("ENcCimzGPU6dNih1qnsSShTYBu9rRERnF4Wwx7BVVt7h");
const SUSDU_CONFIG: Address = address!("DyiptL8AUJjxqphpkWAcVbFrA53EawpyaJ1VzDi8YoLc");

#[test]
fn test_unitas_vault_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Unitas programs
    load_program(
        &mut svm,
        UNITAS_VAULT_PROGRAM_ID,
        &format!("{}/unitas_vault.so", unitas_vault_fixtures_dir()),
    );
    load_program(
        &mut svm,
        SUSDU_PROGRAM_ID,
        &format!("{}/susdu_program.so", unitas_vault_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/access_registry.json", unitas_vault_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_stake_pool_usdu.json", unitas_vault_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/susdu_minter.json", unitas_vault_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdu_mint.json", unitas_vault_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/susdu_mint.json", unitas_vault_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_state.json", unitas_vault_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_config.json", unitas_vault_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/susdu_config.json", unitas_vault_fixtures_dir()),
    );

    // Jump ahead to vault config time_since_last_distribution (1_777_286_485) + 1
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_777_286_485 + 1;
    svm.set_sysvar::<Clock>(&clock);

    // Create trader token accounts with initial balances
    // Selling USDU for sUSDU
    let initial_usdu = 1_000_000_000u64; // 1000 USDU
    let initial_susdu = 0u64;
    let trader_usdu =
        get_associated_token_address(&payer.pubkey(), &USDU_MINT, &TOKEN_2022_PROGRAM_ID);
    let trader_susdu =
        get_associated_token_address(&payer.pubkey(), &SUSDU_MINT, &TOKEN_2022_PROGRAM_ID);
    create_token_account_at(
        &mut svm,
        trader_usdu,
        &payer.pubkey(),
        &USDU_MINT,
        initial_usdu,
        true,
    );
    create_token_account_at(
        &mut svm,
        trader_susdu,
        &payer.pubkey(),
        &SUSDU_MINT,
        initial_susdu,
        true,
    );

    // Build swap instruction: sell 10 USDU for sUSDU
    let in_amount = 10_000_000u64; // 10 USDU
    let min_out_amount = 1u64; // Very loose slippage for test

    // Unitas vault accounts layout (18 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(UNITAS_VAULT_PROGRAM_ID, false),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(trader_susdu, false),
        AccountMeta::new(trader_usdu, false),
        AccountMeta::new_readonly(ACCESS_REGISTRY, false),
        AccountMeta::new(VAULT_STAKE_POOL_USDU_TOKEN_ACCOUNT, false),
        AccountMeta::new_readonly(SUSDU_MINTER, false),
        AccountMeta::new(USDU_MINT, false),
        AccountMeta::new(SUSDU_MINT, false),
        AccountMeta::new_readonly(VAULT_STATE, false),
        AccountMeta::new(VAULT_CONFIG, false),
        AccountMeta::new(SUSDU_CONFIG, false),
        AccountMeta::new_readonly(SUSDU_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
    ];

    // Unitas vault swap has no extra data
    let extra_data: &[u8] = &[];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::UnitasVault,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_usdu = get_token_balance(&svm, &trader_usdu);
            let final_susdu = get_token_balance(&svm, &trader_susdu);

            assert!(
                final_usdu < initial_usdu,
                "USDU should have decreased: {} -> {}",
                initial_usdu,
                final_usdu
            );
            assert!(
                final_susdu > initial_susdu,
                "sUSDU should have increased: {} -> {}",
                initial_susdu,
                final_susdu
            );

            println!(
                "unitas_vault swap successful! USDU: {} -> {}, sUSDU: {} -> {}",
                initial_usdu, final_usdu, initial_susdu, final_susdu
            );
        }
        Err(e) => {
            panic!("unitas_vault swap CPI failed: {}", e);
        }
    }
}
