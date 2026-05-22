#[cfg(feature = "resolve")]
use {
    crate::{get_associated_token_address, ClientError},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    crate::{ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_2022_PROGRAM_ID},
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const UNITAS_VAULT_PROGRAM_ID: Address =
    address!("VALT7AM76ZWfRhjVeYQRrLvNRLvqBzNs8dTsAcLW3jj");
pub const SUSDU_PROGRAM_ID: Address = address!("SUSD2TSk8DJodCkPviKb2okzbeQ597kCBjLVjq3G7pp");
pub const ACCESS_REGISTRY: Address = address!("8maav1g7bK1vRamXzADLUu3DQ7VmXxjVTJt9PbBuWcpd");
pub const VAULT_STAKE_POOL_USDU_TOKEN_ACCOUNT: Address =
    address!("CFgrWjb9DYKVqf7QyQfmwjboDDkXpFHQ6292rnYxrjsa");
pub const SUSDU_MINTER: Address = address!("6ZY9KMGD9UjTX4tWGcw4Y4UHh14nzNmiEr92wTieYub5");
pub const USDU_MINT: Address = address!("9ckR7pPPvyPadACDTzLwK2ZAEeUJ3qGSnzPs8bVaHrSy");
pub const SUSDU_MINT: Address = address!("9iq5Q33RSiz1WcupHAQKbHBZkpn92UxBG2HfPWAZhMCa");
pub const VAULT_STATE: Address = address!("4x4h6NxSBsVJr5iQ4M7NfTqv8gUQrP4nE2psM6JQ2xn8");
pub const VAULT_CONFIG: Address = address!("DyiptL8AUJjxqphpkWAcVbFrA53EawpyaJ1VzDi8YoLc");
pub const SUSDU_CONFIG: Address = address!("6fMbMU14Q5sfiVHr8QY8wYbL7dDrV7N5vZrN8nQ3M2vN");

/// Pre-resolved addresses for building a Unitas vault stake_usdu_mint_susdu instruction offline.
pub struct UnitasVaultSwapInput {
    pub caller: Address,
    pub receiver: Address,
    pub receiver_susdu_token_account: Address,
    pub caller_usdu_token_account: Address,
}

/// Build Unitas vault stake_usdu_mint_susdu AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &UnitasVaultSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(UNITAS_VAULT_PROGRAM_ID, false),
        AccountMeta::new(input.caller, true),
        AccountMeta::new(input.receiver, true),
        AccountMeta::new(input.receiver_susdu_token_account, false),
        AccountMeta::new(input.caller_usdu_token_account, false),
        AccountMeta::new_readonly(ACCESS_REGISTRY, false),
        AccountMeta::new(VAULT_STAKE_POOL_USDU_TOKEN_ACCOUNT, false),
        AccountMeta::new_readonly(SUSDU_MINTER, false),
        AccountMeta::new(USDU_MINT, false),
        AccountMeta::new(SUSDU_MINT, false),
        AccountMeta::new_readonly(VAULT_STATE, false),
        AccountMeta::new(VAULT_CONFIG, false),
        AccountMeta::new(SUSDU_CONFIG, false),
        AccountMeta::new_readonly(SUSDU_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
    ]
}

/// Resolve accounts and data for a Unitas vault stake_usdu_mint_susdu swap.
#[cfg(feature = "resolve")]
pub async fn resolve(
    _rpc: &RpcClient,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let input = UnitasVaultSwapInput {
        caller: *user,
        receiver: *user,
        receiver_susdu_token_account: get_associated_token_address(
            user,
            &SUSDU_MINT,
            &TOKEN_2022_PROGRAM_ID,
        ),
        caller_usdu_token_account: get_associated_token_address(
            user,
            &USDU_MINT,
            &TOKEN_2022_PROGRAM_ID,
        ),
    };

    Ok((build_accounts(&input), vec![]))
}
