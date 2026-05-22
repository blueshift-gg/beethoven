#[cfg(feature = "resolve")]
use {crate::get_associated_token_address, solana_rpc_client::nonblocking::rpc_client::RpcClient};
use {
    crate::TOKEN_PROGRAM_ID,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const XORCA_PROGRAM_ID: Address = address!("StaKE6XNKVVhG8Qu9hDJBqCW3eRe7MDGLz17nJZetLT");
pub const ORCA_VAULT: Address = address!("Ce5j11WAsSzM3nkzrw4Kw6v6ic3nbyqpv5eywjYKeKc5");
pub const XORCA_MINT: Address = address!("xorcaYqbXUNz3474ubUMJAdu2xgPsew3rUCe5ughT3N");
pub const STATE_ACCOUNT: Address = address!("CSqKhyW1cpdyjheAx5HXx4ibcnYrzpL5JywEMAkZixBK");
pub const ORCA_MINT: Address = address!("orcaEKTdK7LKz57vaAYr9QeNsVEPfiu6QeMU1kektZE");

/// Pre-resolved addresses for building an xORCA stake instruction offline.
pub struct XorcaSwapInput {
    pub staker: Address,
    pub staker_orca_ata: Address,
    pub staker_xorca_ata: Address,
}

/// Build xORCA stake AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &XorcaSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(XORCA_PROGRAM_ID, false),
        AccountMeta::new(input.staker, true),
        AccountMeta::new(ORCA_VAULT, false),
        AccountMeta::new(input.staker_orca_ata, false),
        AccountMeta::new(input.staker_xorca_ata, false),
        AccountMeta::new(XORCA_MINT, false),
        AccountMeta::new_readonly(STATE_ACCOUNT, false),
        AccountMeta::new_readonly(ORCA_MINT, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
    ]
}

/// Resolve accounts and data for an xORCA swap via RPC.
#[cfg(feature = "resolve")]
pub async fn resolve(
    _rpc: &RpcClient,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), crate::error::ClientError> {
    let staker_orca_ata = get_associated_token_address(user, &ORCA_MINT, &TOKEN_PROGRAM_ID);
    let staker_xorca_ata = get_associated_token_address(user, &XORCA_MINT, &TOKEN_PROGRAM_ID);

    let input = XorcaSwapInput {
        staker: *user,
        staker_orca_ata,
        staker_xorca_ata,
    };

    Ok((build_accounts(&input), vec![]))
}
