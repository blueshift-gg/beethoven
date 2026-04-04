use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        futarchy_fixtures_dir, get_token_balance, load_and_set_json_fixture, load_program,
        send_transaction, setup_svm, FUTARCHY_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const METADAO_MINT: Address = address!("METAwkXcqyXKy1AtsSgJ8JiUHwGCafnZL38n3vYmeta");
const METADAO_DAO: Address = address!("CUPoiqkK4hxyCiJcLC4yE9AtJP1MoV1vFV2vx3jqwWeS");
const METADAO_DAO_AMM_VAULT_META: Address =
    address!("JDx4m2VpQQLqCdd3URUThhAY2wJePpAn49f2BcNUU3WX");
const METADAO_DAO_AMM_VAULT_USDC: Address =
    address!("HgtoZVvdVjrCm7jmKmiySjHmvbXRahqmRP8WSxGPC5VM");
const EVENT_AUTHORITY: Address = address!("DGEympSS4qLvdr9r3uGHTfACdN8snShk4iGdJtZPxuBC");

#[test]
fn test_futarchy_swap() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Futarchy program
    load_program(
        &mut svm,
        FUTARCHY_PROGRAM_ID,
        &format!("{}/futarchy.so", futarchy_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/metadao_mint.json", futarchy_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/metadao_dao.json", futarchy_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/metadao_dao_amm_vault_meta.json",
            futarchy_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/metadao_dao_amm_vault_usdc.json",
            futarchy_fixtures_dir()
        ),
    );

    // Create trader token accounts with initial balances
    // Selling USDC for META (output)
    let initial_usdc = 1_000_000_000u64; // 1000 USDC
    let initial_meta = 0u64;
    let trader_input = create_token_account(
        &mut svm,
        &payer.pubkey(),
        &METADAO_MINT,
        initial_meta,
        false,
    );
    let trader_output =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);

    // Build swap instruction: sell 10 USDC for META
    let in_amount = 10_000_000u64; // 10 USDC
    let min_out_amount = 1u64; // Very loose slippage for test

    // Futarchy accounts layout (10 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(FUTARCHY_PROGRAM_ID, false), // futarchy_program
        AccountMeta::new(METADAO_DAO, false),                  // dao
        AccountMeta::new(trader_input, false),                 // user_base_account
        AccountMeta::new(trader_output, false),                // user_quote_account
        AccountMeta::new(METADAO_DAO_AMM_VAULT_META, false),   // amm_base_vault
        AccountMeta::new(METADAO_DAO_AMM_VAULT_USDC, false),   // amm_quote_vault
        AccountMeta::new(payer.pubkey(), true),                // user
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),    // token_program
        AccountMeta::new_readonly(EVENT_AUTHORITY, false),     // event_authority
        AccountMeta::new_readonly(FUTARCHY_PROGRAM_ID, false), // program
    ];

    // swap type = buy
    let extra_data: &[u8] = &[0u8];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::Futarchy,
        extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_meta = get_token_balance(&svm, &trader_input);
            let final_usdc = get_token_balance(&svm, &trader_output);

            assert!(
                final_usdc < initial_usdc,
                "USDC should have decreased: {} -> {}",
                initial_usdc,
                final_usdc
            );
            assert!(
                final_meta > initial_meta,
                "META should have increased: {} -> {}",
                initial_meta,
                final_meta
            );

            println!(
                "Futarchy swap successful! USDC: {} -> {}, META: {} -> {}",
                initial_usdc, final_usdc, initial_meta, final_meta
            );
        }
        Err(e) => {
            panic!("Futarchy swap CPI failed: {}", e);
        }
    }
}
