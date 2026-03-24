use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir,
        create_token_account_at, get_token_balance, load_and_set_json_fixture, load_program,
        oxedium_fixtures_dir, send_transaction, setup_svm, OXEDIUM_PROGRAM_ID, TEST_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const PYTH_PRICE_WSOL: Address = address!("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");
const PYTH_PRICE_USDC: Address = address!("Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX");
const VAULT_PDA_WSOL: Address = address!("22b9qiBiN5JRx4heTmMyF4qnr6RVWGR7AUVZkoam5mBL");
const VAULT_PDA_USDC: Address = address!("A2ucxVH898hNfhaYDrXTPQvQiQ9QMCoLrjxEdvxt6tm9");
const VAULT_ATA_WSOL: Address = address!("EqGthvLqeBWhCdDk7iH12ru9s6yxnDdeg4QC4tibVqwt");
const VAULT_ATA_USDC: Address = address!("55aahNVFWrcg6p6PAkoDt9kZhfJvMftEKb8TQT3Lr6m1");
const OXE_GLOBAL_PDA: Address = address!("AyPnt35W2f9UB3EoQW8impxvA7NqAPBK2iUj3xiyuzH");
const ASSOCIATED_TOKEN_PROGRAM: Address = address!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
const SYSTEM_PROGRAM: Address = address!("11111111111111111111111111111111");

#[test]
fn test_oxedium_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    load_program(
        &mut svm,
        OXEDIUM_PROGRAM_ID,
        &format!("{}/oxedium.so", oxedium_fixtures_dir()),
    );

    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/wsol_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/pyth_price_wsol.json", oxedium_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/pyth_price_usdc.json", oxedium_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_pda_wsol.json", oxedium_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_pda_usdc.json", oxedium_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_ata_wsol.json", oxedium_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/vault_ata_usdc.json", oxedium_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/oxe_global_pda.json", oxedium_fixtures_dir()),
    );

    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1774382899;
    svm.set_sysvar::<Clock>(&clock);

    let initial_wsol = 1_000_000_000u64;
    let initial_usdc = 0u64;
    let (trader_input, _) = Address::find_program_address(
        &[payer.pubkey().as_ref(), TOKEN_PROGRAM_ID.as_ref(), WSOL_MINT.as_ref()],
        &ASSOCIATED_TOKEN_PROGRAM,
    );
    let (trader_output, _) = Address::find_program_address(
        &[payer.pubkey().as_ref(), TOKEN_PROGRAM_ID.as_ref(), USDC_MINT.as_ref()],
        &ASSOCIATED_TOKEN_PROGRAM,
    );
    create_token_account_at(&mut svm, trader_input, &payer.pubkey(), &WSOL_MINT, initial_wsol);
    create_token_account_at(&mut svm, trader_output, &payer.pubkey(), &USDC_MINT, initial_usdc);

    let in_amount = 1_000_000u64;
    let min_out_amount = 1u64;

    let accounts = vec![
        AccountMeta::new_readonly(OXEDIUM_PROGRAM_ID, false),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new_readonly(WSOL_MINT, false),
        AccountMeta::new_readonly(USDC_MINT, false),
        AccountMeta::new_readonly(PYTH_PRICE_WSOL, false),
        AccountMeta::new_readonly(PYTH_PRICE_USDC, false),
        AccountMeta::new(trader_input, false),
        AccountMeta::new(trader_output, false),
        AccountMeta::new(VAULT_PDA_WSOL, false),
        AccountMeta::new(VAULT_PDA_USDC, false),
        AccountMeta::new(VAULT_ATA_WSOL, false),
        AccountMeta::new(VAULT_ATA_USDC, false),
        AccountMeta::new_readonly(OXE_GLOBAL_PDA, false),
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM, false),
    ];

    let extra_data: &[u8] = &[];

    let instruction = build_swap_instruction(accounts, in_amount, min_out_amount, extra_data);

    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_wsol = get_token_balance(&svm, &trader_input);
            let final_usdc = get_token_balance(&svm, &trader_output);

            assert!(
                final_wsol < initial_wsol,
                "WSOL should have decreased: {} -> {}",
                initial_wsol,
                final_wsol
            );
            assert!(
                final_usdc > initial_usdc,
                "USDC should have increased: {} -> {}",
                initial_usdc,
                final_usdc
            );

            println!(
                "Oxedium swap successful! WSOL: {} -> {}, USDC: {} -> {}",
                initial_wsol, final_wsol, initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("Oxedium swap CPI failed: {}", e);
        }
    }
}
