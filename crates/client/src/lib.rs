use solana_address::Address;

pub mod error;
pub mod swap;

#[cfg(feature = "resolve")]
pub use swap::{resolve_swap, resolve_swaps};
pub use {
    error::ClientError,
    swap::{SwapProtocol, SwapStep},
};

/// Helper to get the associated token account address.
pub fn get_associated_token_address(
    wallet: &Address,
    mint: &Address,
    token_program: &Address,
) -> Address {
    let seeds = &[wallet.as_ref(), token_program.as_ref(), mint.as_ref()];
    let (address, _bump) = Address::find_program_address(seeds, &spl_associated_token_address());
    address
}

fn spl_associated_token_address() -> Address {
    Address::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL")
}

/// Token Program ID
pub const TOKEN_PROGRAM_ID: Address =
    Address::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

/// Token 2022 Program ID
pub const TOKEN_2022_PROGRAM_ID: Address =
    Address::from_str_const("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

/// System Program ID
pub const SYSTEM_PROGRAM_ID: Address = Address::from_str_const("11111111111111111111111111111111");

/// Sysvar Instructions ID
pub const SYSVAR_INSTRUCTIONS_ID: Address =
    Address::from_str_const("Sysvar1nstructions1111111111111111111111111");

/// Sysvar Clock ID
pub const SYSVAR_CLOCK_ID: Address =
    Address::from_str_const("SysvarC1ock11111111111111111111111111111111");

/// Associated Token Program ID
pub const ASSOCIATED_TOKEN_PROGRAM_ID: Address =
    Address::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

/// Determine which token program owns a mint by checking the account owner.
#[cfg(feature = "resolve")]
pub async fn get_token_program_for_mint(
    rpc: &solana_rpc_client::nonblocking::rpc_client::RpcClient,
    mint: &Address,
) -> Result<Address, ClientError> {
    let account = rpc.get_account(mint).await.map_err(ClientError::from)?;
    let owner = account.owner;
    if owner == TOKEN_PROGRAM_ID || owner == TOKEN_2022_PROGRAM_ID {
        Ok(owner)
    } else {
        Err(ClientError::InvalidAccountData(format!(
            "Mint {} is not owned by a token program (owner: {})",
            mint, owner
        )))
    }
}

/// Discover a program account using `getProgramAccounts` with memcmp filters.
///
/// `filters` is a list of (offset, pubkey) pairs — the returned account must
/// contain each pubkey at its respective byte offset.
///
/// Returns `(account_pubkey, account_data)` for the first match.
#[cfg(feature = "resolve")]
pub async fn discover_pool(
    rpc: &solana_rpc_client::nonblocking::rpc_client::RpcClient,
    program_id: &Address,
    filters: &[(usize, &Address)],
) -> Result<(Address, solana_account::Account), ClientError> {
    use {
        solana_account_decoder_client_types::UiAccountEncoding,
        solana_rpc_client_api::{
            config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
            filter::{Memcmp, RpcFilterType},
        },
    };

    let rpc_filters: Vec<RpcFilterType> = filters
        .iter()
        .map(|(offset, pubkey)| {
            RpcFilterType::Memcmp(Memcmp::new_raw_bytes(*offset, pubkey.to_bytes().to_vec()))
        })
        .collect();

    let config = RpcProgramAccountsConfig {
        filters: Some(rpc_filters),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            ..RpcAccountInfoConfig::default()
        },
        ..RpcProgramAccountsConfig::default()
    };

    let accounts = rpc
        .get_program_ui_accounts_with_config(program_id, config)
        .await?;

    let (pubkey, ui_account) = accounts
        .into_iter()
        .next()
        .ok_or(ClientError::PoolNotFound)?;

    let account: solana_account::Account = ui_account
        .decode()
        .ok_or_else(|| ClientError::InvalidAccountData("Failed to decode UiAccount".to_string()))?;

    Ok((pubkey, account))
}

/// Discover a pool account, trying the given mint ordering first and then flipped.
///
/// `offset_a` / `offset_b` are the byte offsets in the pool account data where
/// the two mint pubkeys are stored. Tries `(mint_a @ offset_a, mint_b @ offset_b)`
/// first, then `(mint_b @ offset_a, mint_a @ offset_b)` if no match.
#[cfg(feature = "resolve")]
pub async fn discover_pool_with_flip(
    rpc: &solana_rpc_client::nonblocking::rpc_client::RpcClient,
    program_id: &Address,
    offset_a: usize,
    offset_b: usize,
    mint_a: &Address,
    mint_b: &Address,
) -> Result<(Address, solana_account::Account), ClientError> {
    match discover_pool(rpc, program_id, &[(offset_a, mint_a), (offset_b, mint_b)]).await {
        Ok(r) => Ok(r),
        Err(ClientError::PoolNotFound) => {
            discover_pool(rpc, program_id, &[(offset_a, mint_b), (offset_b, mint_a)]).await
        }
        Err(e) => Err(e),
    }
}

/// Read a 32-byte pubkey from a byte slice at a given offset.
pub fn read_pubkey(data: &[u8], offset: usize) -> Result<Address, ClientError> {
    if data.len() < offset + 32 {
        return Err(ClientError::InvalidAccountData(format!(
            "Account data too short: {} bytes, need at least {}",
            data.len(),
            offset + 32
        )));
    }
    Ok(Address::from(
        <[u8; 32]>::try_from(&data[offset..offset + 32]).unwrap(),
    ))
}
