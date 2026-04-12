#[cfg(feature = "resolve")]
use crate::get_associated_token_address;
#[cfg(feature = "resolve")]
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use {
    crate::{error::ClientError, TOKEN_PROGRAM_ID},
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const NIRVANA_GOVERNANCE_PROGRAM_ID: Address =
    address!("NirvHuZvrm2zSxjkBvSbaF2tHfP5j7cvMj9QmdoHVwb");
pub const ANA_MINT: Address = address!("5DkzT65YJvCsZcot9L6qwkJnsBCPmKHjJz3QU7t7QeRW");
pub const NIRV_MINT: Address = address!("3eamaYJ7yicyRd3mYz4YeNyNPGVo6zMmKUp5UP25AxRM");
pub const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
pub const TENANT: Address = address!("BcAoCEdkzV2J21gAjCCEokBw5iMnAe96SbYo9F6QmKWV");
pub const PRICE_CURVE: Address = address!("Fx5u5BCTwpckbB6jBbs13nDsRabHb5bq2t2hBDszhSbd");
pub const BACKING_VAULT_MAIN: Address = address!("FhTJEGXVwj4M6NQ1tPu9jgDZUXWQ9w2hP89ebZHwrJPS");
pub const BACKING_VAULT_NIRV: Address = address!("EkwPHXXZNAguNoxeftVRXThCQJfD6EaG852pDsYLs2eB");
pub const ESCROW_REV_ANA: Address = address!("42rJYSmYHqbn5mk992xAoKZnWEiuMzr6u6ydj9m8fAjP");

/// Pre-resolved addresses for building a Nirvana Governance swap instruction offline.
#[derive(Clone, Copy)]
pub struct NirvanaGovernanceSwapInput {
    pub payer: Address,
    pub backing_token_account: Address,
    pub ana_token_account: Address,
}

/// Build Nirvana Governance swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &NirvanaGovernanceSwapInput, is_buy: bool) -> Vec<AccountMeta> {
    let mut accounts = vec![
        AccountMeta::new_readonly(NIRVANA_GOVERNANCE_PROGRAM_ID, false),
        AccountMeta::new(input.payer, true),
        AccountMeta::new(TENANT, false),
        match is_buy {
            true => AccountMeta::new_readonly(PRICE_CURVE, false),
            false => AccountMeta::new(PRICE_CURVE, false),
        },
        AccountMeta::new(ANA_MINT, false),
    ];

    match is_buy {
        true => {
            accounts.extend_from_slice(&[
                AccountMeta::new_readonly(NIRV_MINT, false),
                AccountMeta::new_readonly(USDC_MINT, false),
                AccountMeta::new(BACKING_VAULT_MAIN, false),
                AccountMeta::new(BACKING_VAULT_NIRV, false),
                AccountMeta::new(ESCROW_REV_ANA, false),
                AccountMeta::new(input.backing_token_account, false),
                AccountMeta::new(input.ana_token_account, false),
            ]);
        }
        false => {
            accounts.extend_from_slice(&[
                AccountMeta::new(input.backing_token_account, false),
                AccountMeta::new(ESCROW_REV_ANA, false),
                AccountMeta::new(BACKING_VAULT_MAIN, false),
                AccountMeta::new(BACKING_VAULT_NIRV, false),
                AccountMeta::new(input.ana_token_account, false),
                AccountMeta::new_readonly(NIRV_MINT, false),
                AccountMeta::new_readonly(USDC_MINT, false),
            ]);
        }
    }

    accounts.extend_from_slice(&[
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
    ]);

    accounts
}

/// Resolve accounts and data for a Nirvana Governance swap via RPC.
#[cfg(feature = "resolve")]
pub async fn resolve(
    _rpc: &RpcClient,
    is_buy: bool,
    _mint_nirv: &Address,
    _mint_main: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    // mint_main hardcoded to USDC
    let backing_token_account = get_associated_token_address(user, &USDC_MINT, &TOKEN_PROGRAM_ID);
    let ana_token_account = get_associated_token_address(user, &ANA_MINT, &TOKEN_PROGRAM_ID);

    let input = NirvanaGovernanceSwapInput {
        payer: *user,
        backing_token_account,
        ana_token_account,
    };

    Ok((build_accounts(&input, is_buy), vec![]))
}
