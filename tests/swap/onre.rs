use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir,
        create_token_account_at, get_token_balance, load_and_set_json_fixture, load_program,
        onre_fixtures_dir, send_transaction, setup_svm, ONRE_PROGRAM_ID, SYSTEM_PROGRAM_ID,
        TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    beethoven_client::{
        get_associated_token_address, ASSOCIATED_TOKEN_PROGRAM_ID, SYSVAR_INSTRUCTIONS_ID,
    },
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const OFFER: Address = address!("E88zkA9Pxb1i8EfSHrEW5ZUe6hiQbo8DHWQ3WhDFw7p6");
const STATE: Address = address!("EL5qwcpKyc2FuQxjWmVLEwpcb4LXXwwWWjMYjf1yi3to");
const BOSS: Address = address!("45YnzauhsBM8CpUz96Djf8UG5vqq2Dua62wuW9H3jaJ5");
const VAULT_AUTHORITY: Address = address!("Ce3R5ZxvW3cnsGS63ikR8KCdA22nkoXW3PnY83yaLJ78");
const VAULT_TOKEN_IN_ACCOUNT: Address = address!("BMP8pEkMWHoDYiB2N4VyVUm4Fpv6JYNuSFhpMnzanuHi");
const VAULT_TOKEN_OUT_ACCOUNT: Address = address!("6zqQk9iDWzCx4NUyKNyfNVyxp8e3od8Br7jwkSDeRz8K");
const PERMISSIONLESS_AUTHORITY: Address = address!("6MvXFNjBDb7arkEHS68Es6MN2giH7SehdHUvYRPFgbyC");
const PERMISSIONLESS_TOKEN_IN_ACCOUNT: Address =
    address!("4iEX62oBnfY9foNH1HjnTHzfbzexHP4xY23h5R7jNppU");
const PERMISSIONLESS_TOKEN_OUT_ACCOUNT: Address =
    address!("3vaMSBXYcwEjUGtVExcAxLpUuFQMgDSCxghNgTP1uZ7K");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const ONRE_MINT: Address = address!("5Y8NV33Vv7WbnLfq3zBcKSdYPrk7g2KoiQoe7M2tcxp5");
const BOSS_TOKEN_IN_ACCOUNT: Address = address!("ASJT6kECWY9U5Qeac8ZyTM8W5zTcoRaKKisRBsBeFxk6");
const MINT_AUTHORITY: Address = address!("AbpE5YLpdpxj2jRczG9P341Jicf67NvZsaZYrATbMnNX");

#[test]
fn test_onre_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Onre program
    load_program(
        &mut svm,
        ONRE_PROGRAM_ID,
        &format!("{}/onre.so", onre_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(&mut svm, &format!("{}/onre_mint.json", onre_fixtures_dir()));
    load_and_set_json_fixture(&mut svm, &format!("{}/offer.json", onre_fixtures_dir()));
    load_and_set_json_fixture(&mut svm, &format!("{}/state.json", onre_fixtures_dir()));
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_token_in_account.json", onre_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_token_out_account.json", onre_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/permissionless_token_in_account.json",
            onre_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/permissionless_token_out_account.json",
            onre_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/permissionless_authority.json", onre_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/boss_token_in_account.json", onre_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/mint_authority.json", onre_fixtures_dir()),
    );

    // Jump ahead to start_time of third OfferVector in offer (1_773_878_400) + 1 sec
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_773_878_400 + 1;
    svm.set_sysvar::<Clock>(&clock);

    // Create trader token accounts with initial balances
    // Selling USDC for ONRE
    let initial_usdc = 100_000_000u64; // 100 USDC
    let initial_onre = 0u64;
    let trader_input = get_associated_token_address(&payer.pubkey(), &USDC_MINT, &TOKEN_PROGRAM_ID);
    let trader_output =
        get_associated_token_address(&payer.pubkey(), &ONRE_MINT, &TOKEN_PROGRAM_ID);
    create_token_account_at(
        &mut svm,
        trader_input,
        &payer.pubkey(),
        &USDC_MINT,
        initial_usdc,
        false,
    );
    create_token_account_at(
        &mut svm,
        trader_output,
        &payer.pubkey(),
        &ONRE_MINT,
        initial_onre,
        false,
    );

    // Build swap instruction: sell 10 USDC for ONRE
    let in_amount = 10_000_000u64; // 10 USDC
    let min_out_amount = 1u64; // Very loose slippage for test

    // Onre account layout (22 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(ONRE_PROGRAM_ID, false), // onre_program
        AccountMeta::new(OFFER, false),                    // offer
        AccountMeta::new_readonly(STATE, false),           // state
        AccountMeta::new_readonly(BOSS, false),            // boss
        AccountMeta::new_readonly(VAULT_AUTHORITY, false), // vault_authority
        AccountMeta::new(VAULT_TOKEN_IN_ACCOUNT, false),   // vault_token_in_account
        AccountMeta::new(VAULT_TOKEN_OUT_ACCOUNT, false),  // vault_token_out_account
        AccountMeta::new_readonly(PERMISSIONLESS_AUTHORITY, false), // permissionless_authority
        AccountMeta::new(PERMISSIONLESS_TOKEN_IN_ACCOUNT, false), // permissionless_token_in_account
        AccountMeta::new(PERMISSIONLESS_TOKEN_OUT_ACCOUNT, false), // permissionless_token_out_account
        AccountMeta::new(USDC_MINT, false),                        // token_in_mint
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),        // token_in_program
        AccountMeta::new(ONRE_MINT, false),                        // token_out_mint
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),        // token_out_program
        AccountMeta::new(trader_input, false),                     // user_token_in_account
        AccountMeta::new(trader_output, false),                    // user_token_out_account
        AccountMeta::new(BOSS_TOKEN_IN_ACCOUNT, false),            // boss_token_in_account
        AccountMeta::new_readonly(MINT_AUTHORITY, false),          // mint_authority
        AccountMeta::new_readonly(SYSVAR_INSTRUCTIONS_ID, false),  // instructions_sysvar
        AccountMeta::new(payer.pubkey(), true),                    // user
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false), // associated_token_program
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),       // system_program
    ];

    // Onre swap has no extra data
    let extra_data: &[u8] = &[];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::Onre,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_usdc = get_token_balance(&svm, &trader_input);
            let final_onre = get_token_balance(&svm, &trader_output);

            assert!(
                final_usdc < initial_usdc,
                "USDC should have decreased: {} -> {}",
                initial_usdc,
                final_usdc
            );
            assert!(
                final_onre > initial_onre,
                "ONRE should have increased: {} -> {}",
                initial_onre,
                final_onre
            );

            println!(
                "Onre swap successful! USDC: {} -> {}, ONRE: {} -> {}",
                initial_usdc, final_usdc, initial_onre, final_onre
            );
        }
        Err(e) => {
            panic!("Onre swap CPI failed: {}", e);
        }
    }
}
