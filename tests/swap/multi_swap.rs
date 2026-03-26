use {
    crate::helper::*,
    beethoven::SwapProtocolTag,
    beethoven_client::swap::{gamma as gamma_client, manifest as manifest_client},
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_keypair::Keypair,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_signer::Signer,
};

const MANIFEST_MARKET: Address = address!("ENhU8LsaR7vDD2G1CsWcsuSGNrih9Cv5WZEk7q9kPapQ");
const GAMMA_POOL: Address = address!("Hjm1F98vgVdN7Y9L46KLqcZZWyTKS9tj9ybYKJcXnSng");
const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

#[tokio::test]
async fn test_multi_swap_manifest_and_gamma() {
    let rpc = RpcClient::new(get_rpc_url());
    let payer = Keypair::new();

    // Resolve accounts from mainnet RPC — the client reads on-chain state
    // to derive vaults, ATAs, PDAs, and extra_data for each protocol.
    let (manifest_accounts, manifest_extra_data) = manifest_client::resolve(
        &rpc,
        Some(&MANIFEST_MARKET),
        true, // is_exact_in
        &WSOL_MINT,
        &USDC_MINT,
        &payer.pubkey(),
    )
    .await
    .expect("Manifest resolve failed");

    let (gamma_accounts, gamma_extra_data) = gamma_client::resolve(
        &rpc,
        Some(&GAMMA_POOL),
        &WSOL_MINT,
        &USDC_MINT,
        &payer.pubkey(),
    )
    .await
    .expect("Gamma resolve failed");

    // Extract the user's ATA addresses from resolved accounts so we can
    // fund them in LiteSVM. Layout: Manifest [5]=trader_base, [6]=trader_quote;
    // Gamma [5]=user_input_ata, [6]=user_output_ata.
    let manifest_trader_base = manifest_accounts[5].pubkey;
    let manifest_trader_quote = manifest_accounts[6].pubkey;
    let gamma_trader_input = gamma_accounts[5].pubkey;
    let gamma_trader_output = gamma_accounts[6].pubkey;

    // Set up LiteSVM with the same mainnet state (dumped fixtures)
    let mut svm = setup_svm();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_740_000_000;
    svm.set_sysvar::<Clock>(&clock);

    // Load programs
    load_program(&mut svm, TEST_PROGRAM_ID, &beethoven_program_path());
    load_program(
        &mut svm,
        MANIFEST_PROGRAM_ID,
        &format!("{}/manifest_program.so", manifest_fixtures_dir()),
    );
    load_program(
        &mut svm,
        GAMMA_PROGRAM_ID,
        &format!("{}/gamma_program.so", gamma_fixtures_dir()),
    );

    // Load Manifest fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/manifest_usdc_sol_market.json", manifest_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/manifest_sol_usdc_base_vault.json",
            manifest_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!(
            "{}/manifest_sol_usdc_quote_vault.json",
            manifest_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/manifest_global.json", manifest_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/manifest_global_vault.json", manifest_fixtures_dir()),
    );

    // Load Gamma fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/pool_state.json", gamma_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/amm_config.json", gamma_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/observation_key.json", gamma_fixtures_dir()),
    );
    load_and_set_json_fixture(&mut svm, &format!("{}/vault_0.json", gamma_fixtures_dir()));
    load_and_set_json_fixture(&mut svm, &format!("{}/vault_1.json", gamma_fixtures_dir()));

    // Load common mint fixtures
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/wsol_mint.json", common_fixtures_dir()),
    );
    load_and_set_json_fixture(
        &mut svm,
        &format!("{}/usdc_mint.json", common_fixtures_dir()),
    );

    // Create user token accounts at the ATA addresses the client resolved
    create_token_account_at(
        &mut svm,
        manifest_trader_base,
        &payer.pubkey(),
        &WSOL_MINT,
        50_000_000,
    );
    create_token_account_at(
        &mut svm,
        manifest_trader_quote,
        &payer.pubkey(),
        &USDC_MINT,
        0,
    );
    create_token_account_at(
        &mut svm,
        gamma_trader_input,
        &payer.pubkey(),
        &WSOL_MINT,
        50_000_000,
    );
    create_token_account_at(
        &mut svm,
        gamma_trader_output,
        &payer.pubkey(),
        &USDC_MINT,
        0,
    );

    let swap_amount = 1_000_000u64; // 0.001 SOL per swap

    let manifest_leg = SwapLeg {
        accounts: manifest_accounts,
        in_amount: swap_amount,
        min_out_amount: 1,
        protocol_tag: SwapProtocolTag::Manifest,
        extra_data: manifest_extra_data,
    };

    let gamma_leg = SwapLeg {
        accounts: gamma_accounts,
        in_amount: swap_amount,
        min_out_amount: 1,
        protocol_tag: SwapProtocolTag::Gamma,
        extra_data: gamma_extra_data,
    };

    let instruction = build_multi_swap_instruction(vec![manifest_leg, gamma_leg]);

    let result = send_transaction(&mut svm, &payer, instruction);

    match result {
        Ok(compute_units) => {
            let manifest_wsol_after = get_token_balance(&svm, &manifest_trader_base);
            let manifest_usdc_after = get_token_balance(&svm, &manifest_trader_quote);
            assert!(
                manifest_wsol_after < 50_000_000,
                "Manifest: WSOL should have decreased"
            );
            assert!(
                manifest_usdc_after > 0,
                "Manifest: USDC should have increased"
            );

            let gamma_wsol_after = get_token_balance(&svm, &gamma_trader_input);
            let gamma_usdc_after = get_token_balance(&svm, &gamma_trader_output);
            assert!(
                gamma_wsol_after < 50_000_000,
                "Gamma: WSOL should have decreased"
            );
            assert!(gamma_usdc_after > 0, "Gamma: USDC should have increased");

            println!("Multi-swap successful! CU: {}", compute_units);
            println!(
                "  Manifest: WSOL {} -> {}, USDC {} -> {}",
                50_000_000, manifest_wsol_after, 0, manifest_usdc_after
            );
            println!(
                "  Gamma:    WSOL {} -> {}, USDC {} -> {}",
                50_000_000, gamma_wsol_after, 0, gamma_usdc_after
            );
        }
        Err(e) => {
            panic!("Multi-swap CPI failed: {}", e);
        }
    }
}
