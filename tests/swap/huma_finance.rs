use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir,
        create_token_account_at, get_token_balance, huma_finance_fixtures_dir,
        load_and_set_json_fixture, load_program, send_transaction, setup_svm,
        HUMA_FINANCE_PROGRAM_ID, SYSTEM_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    beethoven_client::{get_associated_token_address, ASSOCIATED_TOKEN_PROGRAM_ID},
    litesvm::LiteSVM,
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::{AccountMeta, Instruction},
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const PST_MINT: Address = address!("59obFNBzyTBGowrkif5uK7ojS58vsuWz3ZCvg6tfZAGw");
const HUMA_CONFIG: Address = address!("Fh2WKYCJfota6k76gDGnhTELUuhPa7FHQvVza4cE11ja");
const POOL_CONFIG: Address = address!("28hFhD21Nka3stL27a8zZ4nRLgaDVxRYwJgeEVgeakzS");
const POOL_STATE: Address = address!("iFgP2EbzHUZzMjqbjaagJQ8zmn6as3Hw95aVUKm67od");
const MODE_CONFIG: Address = address!("3FhoMDyKzQqxtGxnz9DfysfoGQKvgDnSFjoDGgguDCQN");
const POOL_AUTHORITY: Address = address!("9936VFvgRmW1STvdgeyPQaKHDx5DwBtbhZkT3HcdL3QK");
const POOL_UNDERLYING_TOKEN: Address = address!("6Xh2Jg9sWJE16VQGppJFTHvQ8Vii3ABUvUF8Pwcwy7Vq");
const INSTANT_WITHDRAWAL_LENDER_CONFIG: Address =
    address!("BHyrtUoHpFfrkc3SF4nfunBt3cVHrBP3Fp279sVFTmKd");

fn load_program_and_fixtures(svm: &mut LiteSVM) {
    load_program(svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Huma Finance program
    load_program(
        svm,
        HUMA_FINANCE_PROGRAM_ID,
        &format!("{}/huma_finance.so", huma_finance_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(svm, &format!("{}/usdc_mint.json", common_fixtures_dir()));
    load_and_set_json_fixture(
        svm,
        &format!("{}/pst_mint.json", huma_finance_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/huma_config.json", huma_finance_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/pool_config.json", huma_finance_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/pool_state.json", huma_finance_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/mode_config.json", huma_finance_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/pst_mint.json", huma_finance_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/pool_authority.json", huma_finance_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/pool_underlying_token.json", huma_finance_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/instant_withdrawal_lender_config.json",
            huma_finance_fixtures_dir()
        ),
    );

    // Jump ahead to pool state mode_states[0] assets_refreshed_at (1_777_254_075) + 1
    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_777_254_075 + 1;
    svm.set_sysvar::<Clock>(&clock);
}

#[test]
fn test_huma_finance_swap_cpi_deposit() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program_and_fixtures(&mut svm);

    // Create trader token accounts with initial balances
    // Selling USDC for PST
    let initial_usdc = 100_000_000u64; // 100 USDC
    let initial_pst = 0u64;
    let trader_usdc = get_associated_token_address(&payer.pubkey(), &USDC_MINT, &TOKEN_PROGRAM_ID);
    let trader_pst = get_associated_token_address(&payer.pubkey(), &PST_MINT, &TOKEN_PROGRAM_ID);
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
        trader_pst,
        &payer.pubkey(),
        &PST_MINT,
        initial_pst,
        false,
    );

    // Build swap instruction: sell 100 USDC for PST
    let in_amount = 10_000_000u64; // 10 USDC
    let min_out_amount = 1u64; // Very loose slippage for test

    // Huma Finance deposit accounts layout (14 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(HUMA_FINANCE_PROGRAM_ID, false), // huma_finance_program
        AccountMeta::new_readonly(payer.pubkey(), true),           // depositor
        AccountMeta::new_readonly(HUMA_CONFIG, false),             // huma_config
        AccountMeta::new_readonly(POOL_CONFIG, false),             // pool_config
        AccountMeta::new(POOL_STATE, false),                       // pool_state
        AccountMeta::new_readonly(MODE_CONFIG, false),             // mode_config
        AccountMeta::new(PST_MINT, false),                         // mode_mint
        AccountMeta::new_readonly(POOL_AUTHORITY, false),          // pool_authority
        AccountMeta::new_readonly(USDC_MINT, false),               // underlying_mint
        AccountMeta::new(POOL_UNDERLYING_TOKEN, false),            // pool_underlying_token
        AccountMeta::new(trader_usdc, false),                      // depositor_underlying_token
        AccountMeta::new(trader_pst, false),                       // depositor_mode_token
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),        // underlying_token_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),        // mode_token_program
    ];

    // deposit has no extra data
    let extra_data = vec![];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::HumaFinance,
        &extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_usdc = get_token_balance(&svm, &trader_usdc);
            let final_pst = get_token_balance(&svm, &trader_pst);

            assert!(
                final_usdc < initial_usdc,
                "USDC should have decreased: {} -> {}",
                initial_usdc,
                final_usdc
            );
            assert!(
                final_pst > initial_pst,
                "PST should have increased: {} -> {}",
                initial_pst,
                final_pst
            );

            println!(
                "Huma Finance deposit successful! USDC: {} -> {}, PST: {} -> {}",
                initial_usdc, final_usdc, initial_pst, final_pst
            );
        }
        Err(e) => {
            panic!("Huma Finance deposit CPI failed: {}", e);
        }
    }
}

#[test]
#[ignore]
fn test_huma_finance_swap_cpi_instant_withdraw() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program_and_fixtures(&mut svm);

    // Create trader token accounts with initial balances
    // Selling PST for USDC
    let initial_pst = 100_000_000u64; // 100 PST
    let initial_usdc = 0u64;
    let trader_pst = get_associated_token_address(&payer.pubkey(), &PST_MINT, &TOKEN_PROGRAM_ID);
    let trader_usdc = get_associated_token_address(&payer.pubkey(), &USDC_MINT, &TOKEN_PROGRAM_ID);
    create_token_account_at(
        &mut svm,
        trader_pst,
        &payer.pubkey(),
        &PST_MINT,
        initial_pst,
        false,
    );
    create_token_account_at(
        &mut svm,
        trader_usdc,
        &payer.pubkey(),
        &USDC_MINT,
        initial_usdc,
        false,
    );

    // pre-requisite: invoke create_lender_accounts_v2 to create the lender state account
    let lender_state = Address::find_program_address(
        &[
            b"lender_state",
            MODE_CONFIG.as_ref(),
            payer.pubkey().as_ref(),
        ],
        &HUMA_FINANCE_PROGRAM_ID,
    )
    .0;

    let instruction = Instruction {
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(HUMA_CONFIG, false),
            AccountMeta::new(POOL_CONFIG, false),
            AccountMeta::new(POOL_STATE, false),
            AccountMeta::new(MODE_CONFIG, false),
            AccountMeta::new(PST_MINT, false),
            AccountMeta::new(lender_state, false),
            AccountMeta::new(trader_pst, false),
            AccountMeta::new(TOKEN_PROGRAM_ID, false),
            AccountMeta::new(ASSOCIATED_TOKEN_PROGRAM_ID, false),
            AccountMeta::new(SYSTEM_PROGRAM_ID, false),
        ],
        // discriminator
        data: vec![203, 52, 185, 231, 192, 74, 121, 108],
        program_id: HUMA_FINANCE_PROGRAM_ID,
    };

    send_transaction(&mut svm, &payer, instruction).unwrap();

    // Build swap instruction: sell 100 USDC for PST
    let in_amount = 10_000_000u64; // 10 USDC
    let min_out_amount = 1u64; // Very loose slippage for test

    // Huma Finance instant_withdraw accounts layout (16 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(HUMA_FINANCE_PROGRAM_ID, false), // huma_finance_program
        AccountMeta::new_readonly(payer.pubkey(), true),           // lender
        AccountMeta::new_readonly(HUMA_CONFIG, false),             // huma_config
        AccountMeta::new_readonly(POOL_CONFIG, false),             // pool_config
        AccountMeta::new(POOL_STATE, false),                       // pool_state
        AccountMeta::new_readonly(INSTANT_WITHDRAWAL_LENDER_CONFIG, false), // instant_withdrawal_lender_config
        AccountMeta::new_readonly(MODE_CONFIG, false),                      // mode_config
        AccountMeta::new(PST_MINT, false),                                  // mode_mint
        AccountMeta::new(lender_state, false),                              // lender_state
        AccountMeta::new_readonly(USDC_MINT, false),                        // underlying_mint
        AccountMeta::new_readonly(POOL_AUTHORITY, false),                   // pool_authority
        AccountMeta::new(POOL_UNDERLYING_TOKEN, false),                     // pool_underlying_token
        AccountMeta::new(trader_usdc, false), // lender_underlying_token
        AccountMeta::new(trader_pst, false),  // lender_mode_token
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false), // underlying_token_program
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false), // mode_token_program
    ];

    // instant_withdraw has no extra data
    let extra_data = vec![];

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::HumaFinance,
        &extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(_compute_units) => {
            let final_pst = get_token_balance(&svm, &trader_pst);
            let final_usdc = get_token_balance(&svm, &trader_usdc);

            assert!(
                final_pst < initial_pst,
                "PST should have decreased: {} -> {}",
                initial_pst,
                final_pst
            );
            assert!(
                final_usdc > initial_usdc,
                "USDC should have increased: {} -> {}",
                initial_usdc,
                final_pst
            );

            println!(
                "Huma Finance instant_withdraw successful! PST: {} -> {}, USDC: {} -> {}",
                initial_pst, final_pst, initial_usdc, final_usdc
            );
        }
        Err(e) => {
            panic!("Huma Finance instant_withdraw CPI failed: {}", e);
        }
    }
}
