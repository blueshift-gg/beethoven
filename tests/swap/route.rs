use {
    crate::helper::*,
    beethoven::SwapProtocolTag,
    beethoven_client::swap::{gamma as gamma_client, manifest as manifest_client},
    solana_address::{address, Address},
    solana_clock::Clock,
    solana_instruction::{AccountMeta, Instruction},
    solana_keypair::Keypair,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_sdk_ids::compute_budget,
    solana_signer::Signer,
};

const MANIFEST_MARKET: Address = address!("ENhU8LsaR7vDD2G1CsWcsuSGNrih9Cv5WZEk7q9kPapQ");
const GAMMA_POOL: Address = address!("Hjm1F98vgVdN7Y9L46KLqcZZWyTKS9tj9ybYKJcXnSng");
const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
const USDT_MINT: Address = address!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

fn set_compute_unit_limit_instruction(units: u32) -> Instruction {
    let mut data = Vec::with_capacity(5);
    data.push(2);
    data.extend_from_slice(&units.to_le_bytes());

    Instruction {
        program_id: compute_budget::ID,
        accounts: vec![],
        data,
    }
}

fn load_gamma_route_programs(svm: &mut litesvm::LiteSVM) {
    load_program(svm, TEST_PROGRAM_ID, &beethoven_program_path());
    load_program(
        svm,
        GAMMA_PROGRAM_ID,
        &format!("{}/gamma_program.so", gamma_fixtures_dir()),
    );
}

async fn load_accounts_from_rpc(
    svm: &mut litesvm::LiteSVM,
    rpc: &RpcClient,
    accounts: &[AccountMeta],
    excluded_accounts: &[Address],
) {
    let mut unique_accounts = Vec::new();

    for account in accounts {
        if excluded_accounts.contains(&account.pubkey) || unique_accounts.contains(&account.pubkey)
        {
            continue;
        }

        unique_accounts.push(account.pubkey);
    }

    let fetched_accounts = rpc
        .get_multiple_accounts(&unique_accounts)
        .await
        .expect("failed to fetch route accounts from RPC");

    for (pubkey, account) in unique_accounts.into_iter().zip(fetched_accounts) {
        if let Some(account) = account {
            svm.set_account(pubkey, account).unwrap();
        }
    }
}

fn load_route_fixtures(svm: &mut litesvm::LiteSVM) {
    load_program(svm, TEST_PROGRAM_ID, &beethoven_program_path());
    load_program(
        svm,
        MANIFEST_PROGRAM_ID,
        &format!("{}/manifest_program.so", manifest_fixtures_dir()),
    );
    load_program(
        svm,
        GAMMA_PROGRAM_ID,
        &format!("{}/gamma_program.so", gamma_fixtures_dir()),
    );

    load_and_set_json_fixture(
        svm,
        &format!("{}/manifest_usdc_sol_market.json", manifest_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/manifest_sol_usdc_base_vault.json",
            manifest_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!(
            "{}/manifest_sol_usdc_quote_vault.json",
            manifest_fixtures_dir()
        ),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/manifest_global.json", manifest_fixtures_dir()),
    );
    load_and_set_json_fixture(
        svm,
        &format!("{}/manifest_global_vault.json", manifest_fixtures_dir()),
    );

    load_and_set_json_fixture(svm, &format!("{}/pool_state.json", gamma_fixtures_dir()));
    load_and_set_json_fixture(svm, &format!("{}/amm_config.json", gamma_fixtures_dir()));
    load_and_set_json_fixture(
        svm,
        &format!("{}/observation_key.json", gamma_fixtures_dir()),
    );
    load_and_set_json_fixture(svm, &format!("{}/vault_0.json", gamma_fixtures_dir()));
    load_and_set_json_fixture(svm, &format!("{}/vault_1.json", gamma_fixtures_dir()));

    load_and_set_json_fixture(svm, &format!("{}/wsol_mint.json", common_fixtures_dir()));
    load_and_set_json_fixture(svm, &format!("{}/usdc_mint.json", common_fixtures_dir()));
}

#[tokio::test]
async fn test_route_gamma_wsol_to_usdt_to_usdc() {
    let rpc = RpcClient::new(get_rpc_url());
    let payer = Keypair::new();

    let (first_leg_accounts, first_leg_extra_data) =
        gamma_client::resolve(&rpc, None, &WSOL_MINT, &USDT_MINT, &payer.pubkey())
            .await
            .expect("Gamma WSOL/USDT resolve failed");

    let (second_leg_accounts, second_leg_extra_data) =
        gamma_client::resolve(&rpc, None, &USDT_MINT, &USDC_MINT, &payer.pubkey())
            .await
            .expect("Gamma USDT/USDC resolve failed");

    let shared_wsol_account = first_leg_accounts[5].pubkey;
    let shared_usdt_account = first_leg_accounts[6].pubkey;
    let final_usdc_account = second_leg_accounts[6].pubkey;

    assert_eq!(shared_usdt_account, second_leg_accounts[5].pubkey);

    let mut svm = setup_svm();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_740_000_000;
    svm.set_sysvar::<Clock>(&clock);

    load_gamma_route_programs(&mut svm);
    load_accounts_from_rpc(
        &mut svm,
        &rpc,
        &first_leg_accounts,
        &[
            TEST_PROGRAM_ID,
            GAMMA_PROGRAM_ID,
            TOKEN_PROGRAM_ID,
            TOKEN_2022_PROGRAM_ID,
            payer.pubkey(),
            shared_wsol_account,
            shared_usdt_account,
            final_usdc_account,
        ],
    )
    .await;
    load_accounts_from_rpc(
        &mut svm,
        &rpc,
        &second_leg_accounts,
        &[
            TEST_PROGRAM_ID,
            GAMMA_PROGRAM_ID,
            TOKEN_PROGRAM_ID,
            TOKEN_2022_PROGRAM_ID,
            payer.pubkey(),
            shared_wsol_account,
            shared_usdt_account,
            final_usdc_account,
        ],
    )
    .await;

    let initial_wsol = 50_000_000u64;
    create_token_account_at(
        &mut svm,
        shared_wsol_account,
        &payer.pubkey(),
        &WSOL_MINT,
        initial_wsol,
    );
    create_token_account_at(
        &mut svm,
        shared_usdt_account,
        &payer.pubkey(),
        &USDT_MINT,
        0,
    );
    create_token_account_at(&mut svm, final_usdc_account, &payer.pubkey(), &USDC_MINT, 0);

    let initial_in_amount = 1_000_000u64;
    let instruction = build_route_instruction(
        initial_in_amount,
        1,
        vec![
            RouteLeg {
                accounts: first_leg_accounts,
                protocol_tag: SwapProtocolTag::Gamma,
                extra_data: first_leg_extra_data,
            },
            RouteLeg {
                accounts: second_leg_accounts,
                protocol_tag: SwapProtocolTag::Gamma,
                extra_data: second_leg_extra_data,
            },
        ],
    );

    let result = send_transaction_with_instructions(
        &mut svm,
        &payer,
        &[set_compute_unit_limit_instruction(400_000), instruction],
    );
    if let Err(err) = result {
        panic!("Route CPI failed: {}", err);
    }

    let final_wsol = get_token_balance(&svm, &shared_wsol_account);
    let final_usdt = get_token_balance(&svm, &shared_usdt_account);
    let final_usdc = get_token_balance(&svm, &final_usdc_account);

    assert_eq!(final_wsol, initial_wsol - initial_in_amount);
    assert_eq!(final_usdt, 0, "intermediate USDT should be fully consumed");
    assert!(final_usdc > 0, "final USDC output should be credited");
}

#[tokio::test]
async fn test_route_rejects_mismatched_adjacent_token_accounts() {
    let rpc = RpcClient::new(get_rpc_url());
    let payer = Keypair::new();

    let (manifest_accounts, manifest_extra_data) = manifest_client::resolve(
        &rpc,
        Some(&MANIFEST_MARKET),
        true,
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

    let shared_wsol_account = manifest_accounts[5].pubkey;
    let shared_usdc_account = manifest_accounts[6].pubkey;

    assert_ne!(shared_usdc_account, gamma_accounts[5].pubkey);

    let mut svm = setup_svm();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    let mut clock = svm.get_sysvar::<Clock>();
    clock.unix_timestamp = 1_740_000_000;
    svm.set_sysvar::<Clock>(&clock);

    load_route_fixtures(&mut svm);

    create_token_account_at(
        &mut svm,
        shared_wsol_account,
        &payer.pubkey(),
        &WSOL_MINT,
        50_000_000,
    );
    create_token_account_at(
        &mut svm,
        shared_usdc_account,
        &payer.pubkey(),
        &USDC_MINT,
        0,
    );

    let instruction = build_route_instruction(
        1_000_000,
        1,
        vec![
            RouteLeg {
                accounts: manifest_accounts,
                protocol_tag: SwapProtocolTag::Manifest,
                extra_data: manifest_extra_data,
            },
            RouteLeg {
                accounts: gamma_accounts,
                protocol_tag: SwapProtocolTag::Gamma,
                extra_data: gamma_extra_data,
            },
        ],
    );

    let err = send_transaction(&mut svm, &payer, instruction)
        .expect_err("route should reject discontinuous token-account flow");

    assert!(
        err.contains("InvalidAccountData"),
        "unexpected route failure: {}",
        err
    );
}
