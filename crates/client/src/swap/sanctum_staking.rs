#[cfg(feature = "resolve")]
use {crate::error::ClientError, solana_rpc_client::nonblocking::rpc_client::RpcClient};
use {
    crate::{get_associated_token_address, TOKEN_PROGRAM_ID},
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const SANCTUM_STAKING_PROGRAM_ID: Address =
    address!("bon4Kh3x1uQK16w9b9DKgz3Aw4AP1pZxBJk55Q6Sosb");
pub const CLOUD_MINT: Address = address!("CLoUDKc4Ane7HeQcPpE3YHnznRxhMimJ4MyaUqyHFzAu");
pub const SCLOUD_MINT: Address = address!("sc1dNAxRBj5CNWaGC26AR7PEW75R36Umzt1V8vuP8kZ");
pub const VAULT: Address = address!("5jbzpJeGZFpPFrwXAdeWn25UJiParK8rayQYJY3r14cv");
pub const BOND_MINT_AUTHORITY: Address = address!("3vLkpgiPPupTLfQ3WHw6zPVrjKVsB18Aiaz9sCqfhE3n");
pub const BOND_POOL: Address = address!("8DFDU25Rzgx9bp4VUirZPykADdnVDLehi5s9enMqmXpq");

/// Pre-resolved addresses for building a Sanctum Staking deposit instruction offline.
pub struct SanctumStakingSwapInput {
    pub authority: Address,
    pub deposit_from: Address,
    pub mint_to: Address,
}

/// Build Sanctum Staking deposit AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &SanctumStakingSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(SANCTUM_STAKING_PROGRAM_ID, false),
        AccountMeta::new(input.authority, true),
        AccountMeta::new(input.deposit_from, false),
        AccountMeta::new(input.mint_to, false),
        AccountMeta::new(VAULT, false),
        AccountMeta::new(SCLOUD_MINT, false),
        AccountMeta::new_readonly(BOND_MINT_AUTHORITY, false),
        AccountMeta::new_readonly(BOND_POOL, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
    ]
}

/// Resolve accounts and data for a Sanctum Staking deposit via RPC.
#[cfg(feature = "resolve")]
pub async fn resolve(
    _rpc: &RpcClient,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let deposit_from = get_associated_token_address(user, &CLOUD_MINT, &TOKEN_PROGRAM_ID);
    let mint_to = get_associated_token_address(user, &SCLOUD_MINT, &TOKEN_PROGRAM_ID);

    let input = SanctumStakingSwapInput {
        authority: *user,
        deposit_from,
        mint_to,
    };

    Ok((build_accounts(&input), vec![]))
}
