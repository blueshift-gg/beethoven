use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        fusion_amm_fixtures_dir, get_token_balance, load_and_set_json_fixture, load_program,
        send_transaction, setup_svm, FUSION_AMM_PROGRAM_ID, MEMO_PROGRAM_ID, TEST_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const FUSION_POOL: Address = address!("7VuKeevbvbQQcxz6N4SNLmuq6PYy4AcGQRDssoqo4t65");
const SOL_VAULT: Address = address!("CYuiCBEhHLAYcDbFVtJ1KfgeQaQuN2sV18pNmzcDsbM7");
const USDC_VAULT: Address = address!("CjQWTPK84zwBq1PjVXmhmtKqhD9BfnMP7dcUFuN8Ljyd");
const TICK_ARRAY_0: Address = address!("ApZyU4eAZipeozyvcPdMNZrfPUsq7nn86k9L1zwszXbi");
const TICK_ARRAY_1: Address = address!("2oz7RHysyQw8fmW7woFVPM5vRmn1KEewgHE4mS2ZBdbo");
const TICK_ARRAY_2: Address = address!("29gZUXRGxSFkHLJEMK8HgndkFz8Ej7Qjs39qAz571sCM");

#[test]
fn test_fusion_amm_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Fusion AMM program
    load_program(
        &mut svm,
        FUSION_AMM_PROGRAM_ID,
        &format!("{}/fusion_amm.so", fusion_amm_fixtures_dir()),
    );

    // Load fixtures
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
        &format!("{}/fusion_pool.json", fusion_amm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/wsol_vault.json", fusion_amm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_vault.json", fusion_amm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/tick_array_0.json", fusion_amm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/tick_array_1.json", fusion_amm_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/tick_array_2.json", fusion_amm_fixtures_dir()),
    );

    // Advance clock (set past any pool staleness checks if the program enforces them)
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_800_000_000;
    svm.set_sysvar::<Clock>(&clock);

    // Create trader token accounts with initial balances
    // Selling SOL (input=WSOL) for USDC (output)
    let initial_wsol = 1_000_000_000u64;
    let initial_usdc = 0u64;
    let trader_wsol =
        create_token_account(&mut svm, &payer.pubkey(), &WSOL_MINT, initial_wsol, false);
    let trader_usdc =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);

    // Build swap instruction: sell 0.001 SOL for USDC
    let in_amount = 1_000_000u64; // 0.001 SOL
    let min_out_amount = 1u64; // Very loose slippage for test

    // Fusion AMM accounts layout (15 accounts: program id for detection + 14 IDL `swap` accounts; no oracle)
    let accounts = vec![
        AccountMeta::new_readonly(FUSION_AMM_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(MEMO_PROGRAM_ID, false),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(FUSION_POOL, false),
        AccountMeta::new_readonly(WSOL_MINT, false),
        AccountMeta::new_readonly(USDC_MINT, false),
        AccountMeta::new(trader_wsol, false),
        AccountMeta::new(trader_usdc, false),
        AccountMeta::new(SOL_VAULT, false),
        AccountMeta::new(USDC_VAULT, false),
        AccountMeta::new(TICK_ARRAY_0, false),
        AccountMeta::new(TICK_ARRAY_1, false),
        AccountMeta::new(TICK_ARRAY_2, false),
    ];

    let sqrt_price_limit: u128 = 0;
    let amount_specified_is_input = true;
    let a_to_b = true;
    let mut extra_data = Vec::with_capacity(19);
    extra_data.extend_from_slice(&sqrt_price_limit.to_le_bytes());
    extra_data.push(amount_specified_is_input as u8);
    extra_data.push(a_to_b as u8);
    // remaining_accounts_info - None
    extra_data.push(0u8);

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::FusionAmm,
        &extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_wsol = get_token_balance(&svm, &trader_wsol);
            let final_usdc = get_token_balance(&svm, &trader_usdc);

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
                "Fusion AMM swap successful! WSOL: {} -> {}, USDC: {} -> {}",
                initial_wsol, final_wsol, initial_usdc, final_usdc
            );
        }
        Err(e) => panic!("Fusion AMM swap CPI failed: {}", e),
    }
}
