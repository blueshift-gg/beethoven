use {
    crate::helper::{
        beethoven_program_path, build_swap_instruction, common_fixtures_dir, create_token_account,
        get_token_balance, load_and_set_json_fixture, load_program, phoenix_legacy_fixtures_dir,
        send_transaction, setup_svm, PHOENIX_LEGACY_PROGRAM_ID, TEST_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    beethoven::SwapProtocolTag,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
    solana_keypair::Keypair,
    solana_signer::Signer,
};

const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const LOG_AUTHORITY: Address = address!("7aDTsspkQNGKmrexAN7FLx9oxU3iPczSSvHNggyuqYkR");
const MARKET: Address = address!("4DoNfFBfF7UokCC2FQzriy7yHK6DY6NVdYpuekQ5pRgg");
const BASE_VAULT: Address = address!("8g4Z9d6PqGkgH31tMW6FwxGhwYJrXpxZHQrkikpLJKrG");
const QUOTE_VAULT: Address = address!("3HSYXeGc3LjEPCuzoNDjQN37F1ebsSiR4CqXVqQCdekZ");

/// Append Beethoven `Option<u64>` layout: `0` + 8 ignored bytes, or `1` + value LE.
fn push_opt_u64(buf: &mut Vec<u8>, v: Option<u64>) {
    match v {
        None => {
            buf.push(0);
            buf.extend_from_slice(&[0u8; 8]);
        }
        Some(x) => {
            buf.push(1);
            buf.extend_from_slice(&x.to_le_bytes());
        }
    }
}

/// 63-byte Phoenix Legacy swap extra payload (`PhoenixLegacySwapData`), field order matching
/// the on-chain `try_from` parser.
///
/// `side`: `0` = Bid, `1` = Ask. `self_trade_behavior`: Phoenix `SelfTradeBehavior` discriminant
/// (`0` = Abort, `1` = CancelProvide, `2` = DecrementTake).
#[allow(clippy::too_many_arguments)]
fn build_extra_data(
    side: u8,
    price_in_ticks: Option<u64>,
    max_counterpart_lots: u64,
    self_trade_behavior: u8,
    match_limit: Option<u64>,
    client_order_id: u128,
    use_only_deposited_funds: bool,
    last_valid_slot: Option<u64>,
    last_valid_unix_timestamp_in_seconds: Option<u64>,
) -> Vec<u8> {
    debug_assert!((..=1).contains(&side));

    let mut b = Vec::new();

    // side
    b.push(side);

    // price_in_ticks: Option<u64>
    push_opt_u64(&mut b, price_in_ticks);

    // max_counterpart_lots: u64
    b.extend_from_slice(&max_counterpart_lots.to_le_bytes());

    // self_trade_behavior
    b.push(self_trade_behavior);

    // match_limit: Option<u64>
    push_opt_u64(&mut b, match_limit);

    // client_order_id: u128
    b.extend_from_slice(&client_order_id.to_le_bytes());

    // use_only_deposited_funds: bool as u8
    b.push(u8::from(use_only_deposited_funds));

    // last_valid_slot: Option<u64>
    push_opt_u64(&mut b, last_valid_slot);

    // last_valid_unix_timestamp_in_seconds: Option<u64>
    push_opt_u64(&mut b, last_valid_unix_timestamp_in_seconds);

    b
}

#[test]
fn test_phoenix_legacy_swap_cpi() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());

    // Load Phoenix Legacy program
    load_program(
        &mut svm,
        PHOENIX_LEGACY_PROGRAM_ID,
        &format!("{}/phoenix_legacy.so", phoenix_legacy_fixtures_dir()),
    );

    // Load fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/wsol_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/market.json", phoenix_legacy_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/base_vault.json", phoenix_legacy_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/quote_vault.json", phoenix_legacy_fixtures_dir()),
    );

    // Create trader token accounts with initial balances
    // Selling SOL (input=WSOL) for USDC (output)
    let initial_wsol = 1_000_000_000u64; // 1 SOL
    let initial_usdc = 0u64;
    let trader_input =
        create_token_account(&mut svm, &payer.pubkey(), &WSOL_MINT, initial_wsol, false);
    let trader_output =
        create_token_account(&mut svm, &payer.pubkey(), &USDC_MINT, initial_usdc, false);

    // Build swap instruction: 0.0000001 SOL for USDC
    let in_amount = 1_000u64;
    let min_out_amount = 1u64; // Very loose slippage for test

    // Phoenix Legacy accounts layout (9 accounts)
    let accounts = vec![
        AccountMeta::new_readonly(PHOENIX_LEGACY_PROGRAM_ID, false), // phoenix legacy program
        AccountMeta::new_readonly(LOG_AUTHORITY, false),             // log authority
        AccountMeta::new(MARKET, false),                             // market
        AccountMeta::new(payer.pubkey(), true),                      // trader
        AccountMeta::new(trader_input, false),                       // base account
        AccountMeta::new(trader_output, false),                      // quote account
        AccountMeta::new(BASE_VAULT, false),                         // base vault
        AccountMeta::new(QUOTE_VAULT, false),                        // quote vault
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),          // token program
    ];

    let extra_data = build_extra_data(1, None, 0, 1, None, 0, false, None, None);

    let instruction = build_swap_instruction(
        accounts,
        in_amount,
        min_out_amount,
        SwapProtocolTag::PhoenixLegacy,
        &extra_data,
    );

    // Execute the swap via CPI through beethoven-test program
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
        }
        Err(e) => panic!("Phoenix Legacy swap CPI failed: {}", e),
    }
}
