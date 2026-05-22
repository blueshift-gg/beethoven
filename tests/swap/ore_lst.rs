use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, create_token_account_at, get_token_balance,
        load_and_set_json_fixture, load_program, ore_lst_fixtures_dir, send_transaction, setup_svm,
        ORE_LST_PROGRAM_ID, ORE_STAKE_PROGRAM_ID, SYSTEM_PROGRAM_ID, TEST_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    beethoven_client::{get_associated_token_address, ASSOCIATED_TOKEN_PROGRAM_ID},
    litesvm::LiteSVM,
    solana_address::{address, Address},
    solana_compute_budget::compute_budget::ComputeBudget,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const ORE_MINT: Address = address!("oreoU2P8bN6jkk3jbaiVxYnG1dCXcYxwhwyK9jSybcp");
const STORE_MINT: Address = address!("sTorERYB6xAZ1SSbwpK3zoK2EEwbBrc7TZAzg1uCGiH");
const STAKE: Address = address!("DfdZYzgLuqRickq57fyb4dX88VgPkhoEs1uuBKdxzaaJ");
const STAKE_TOKENS: Address = address!("6uEvYBcpb8KdhKxrRzffce9S7n8u9hiP2CXihJuUDihX");
const TREASURY: Address = address!("ANX3pRkcGipsZjcWVBvRaHFasBMw8FDPBvJHoubpWym6");
const TREASURY_TOKENS: Address = address!("FVynQtSNrWMa5Ueh1QNedca2YHSNtqH5LFjK3Sa9si2u");
const VAULT: Address = address!("7taXpXz6eqYzscXEi1d1fgwATQMqAR6Nku9pJCjb8gQN");
const VAULT_TOKENS: Address = address!("C1ZiFq8DocfTFxVUe75pqhbmaR8a7sKPsT9A48jmtzzr");

fn load_program_and_fixtures(svm: &mut LiteSVM) {
    load_program(svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Ore programs
    load_program(
        svm,
        ORE_LST_PROGRAM_ID,
        &format!("{}/ore_lst.so", ore_lst_fixtures_dir()),
    );
    load_program(
        svm,
        ORE_STAKE_PROGRAM_ID,
        &format!("{}/ore_stake.so", ore_lst_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(svm, &format!("{}/ore_mint.json", ore_lst_fixtures_dir()));
    load_and_set_json_fixture(svm, &format!("{}/store_mint.json", ore_lst_fixtures_dir()));
    load_and_set_json_fixture(svm, &format!("{}/stake.json", ore_lst_fixtures_dir()));
    load_and_set_json_fixture(
        svm,
        &format!("{}/stake_tokens.json", ore_lst_fixtures_dir()),
    );
    load_and_set_json_fixture(svm, &format!("{}/treasury.json", ore_lst_fixtures_dir()));
    load_and_set_json_fixture(
        svm,
        &format!("{}/treasury_tokens.json", ore_lst_fixtures_dir()),
    );
    load_and_set_json_fixture(svm, &format!("{}/vault.json", ore_lst_fixtures_dir()));
    load_and_set_json_fixture(
        svm,
        &format!("{}/vault_tokens.json", ore_lst_fixtures_dir()),
    );
}

#[test]
fn test_ore_lst_swap_cpi() {
    let mut svm = setup_svm();
    let mut compute_budget = ComputeBudget::new_with_defaults(true, true);
    compute_budget.compute_unit_limit = 800_000;
    svm = svm.with_compute_budget(compute_budget);

    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program_and_fixtures(&mut svm);

    // wrap

    // Create trader token accounts with initial balances
    // Selling ORE for stORE
    let initial_ore = 100_000_000_000u64; // 1 ORE
    let initial_store = 0u64;
    let sender_ore = get_associated_token_address(&payer.pubkey(), &ORE_MINT, &TOKEN_PROGRAM_ID);
    let sender_store =
        get_associated_token_address(&payer.pubkey(), &STORE_MINT, &TOKEN_PROGRAM_ID);
    create_token_account_at(
        &mut svm,
        sender_ore,
        &payer.pubkey(),
        &ORE_MINT,
        initial_ore,
        false,
    );
    create_token_account_at(
        &mut svm,
        sender_store,
        &payer.pubkey(),
        &STORE_MINT,
        initial_store,
        false,
    );

    // Build swap instruction: sell 0.1 ORE for stORE
    let in_amount = 10_000_000_000u64; // 0.1 ORE
    let min_out_amount = 1u64; // Very loose slippage for test

    // ORE LST accounts layout (17 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(ORE_LST_PROGRAM_ID, false), // ore_lst_program
        AccountMeta::new(payer.pubkey(), true),               // signer
        AccountMeta::new(payer.pubkey(), true),               // payer
        AccountMeta::new(sender_ore, false),                  // sender_ore
        AccountMeta::new(sender_store, false),                // sender_store
        AccountMeta::new(ORE_MINT, false),                    // ore_mint
        AccountMeta::new(STORE_MINT, false),                  // store_mint
        AccountMeta::new(STAKE, false),                       // stake
        AccountMeta::new(STAKE_TOKENS, false),                // stake_tokens
        AccountMeta::new(TREASURY, false),                    // treasury
        AccountMeta::new(TREASURY_TOKENS, false),             // treasury_tokens
        AccountMeta::new(VAULT, false),                       // vault
        AccountMeta::new(VAULT_TOKENS, false),                // vault_tokens
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),  // system_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),   // token_program
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false), // associated_token_program
        AccountMeta::new_readonly(ORE_STAKE_PROGRAM_ID, false), // ore_stake_program
    ];

    // is_wrap = true
    let extra_data: &[u8] = &[1_u8];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::OreLst,
        extra_data,
    );

    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_ore = get_token_balance(&svm, &sender_ore);
            let final_store = get_token_balance(&svm, &sender_store);

            assert!(
                final_ore < initial_ore,
                "ORE should have decreased: {} -> {}",
                initial_ore,
                final_ore
            );
            assert!(
                final_store > 0,
                "stORE should have been minted: {} -> {}",
                0,
                final_store
            );

            println!(
                "ORE LST wrap successful! ORE: {} -> {}, stORE: {} -> {}",
                initial_ore, final_ore, 0, final_store
            );
        }
        Err(e) => {
            panic!("ORE LST wrap CPI failed: {}", e);
        }
    }

    // unwrap

    // Selling stORE for ORE
    let initial_store = get_token_balance(&svm, &sender_store);
    let initial_ore = get_token_balance(&svm, &sender_ore);

    // Build swap instruction: sell 0.1 stORE for ORE
    let in_amount = initial_store;
    let min_out_amount = 1u64; // Very loose slippage for test

    // ORE LST accounts layout (17 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(ORE_LST_PROGRAM_ID, false), // ore_lst_program
        AccountMeta::new(payer.pubkey(), true),               // signer
        AccountMeta::new(payer.pubkey(), true),               // payer
        AccountMeta::new(sender_ore, false),                  // sender_ore
        AccountMeta::new(sender_store, false),                // sender_store
        AccountMeta::new(ORE_MINT, false),                    // ore_mint
        AccountMeta::new(STORE_MINT, false),                  // store_mint
        AccountMeta::new(STAKE, false),                       // stake
        AccountMeta::new(STAKE_TOKENS, false),                // stake_tokens
        AccountMeta::new(TREASURY, false),                    // treasury
        AccountMeta::new(TREASURY_TOKENS, false),             // treasury_tokens
        AccountMeta::new(VAULT, false),                       // vault
        AccountMeta::new(VAULT_TOKENS, false),                // vault_tokens
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),  // system_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),   // token_program
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false), // associated_token_program
        AccountMeta::new_readonly(ORE_STAKE_PROGRAM_ID, false), // ore_stake_program
    ];

    // is_wrap = false
    let extra_data: &[u8] = &[0_u8];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::OreLst,
        extra_data,
    );

    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_store = get_token_balance(&svm, &sender_store);
            let final_ore = get_token_balance(&svm, &sender_ore);

            assert!(
                final_store < initial_store,
                "stORE should have decreased: {} -> {}",
                initial_store,
                final_store
            );
            assert!(
                final_ore > 0,
                "ORE should have been minted: {} -> {}",
                initial_ore,
                final_ore
            );

            println!(
                "ORE LST unwrap successful! stORE: {} -> {}, ORE: {} -> {}",
                initial_store, final_store, initial_ore, final_ore
            );
        }
        Err(e) => {
            panic!("ORE LST unwrap CPI failed: {}", e);
        }
    }
}
