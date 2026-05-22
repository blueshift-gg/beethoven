#[cfg(feature = "resolve")]
use crate::{error::ClientError, get_associated_token_address, TOKEN_PROGRAM_ID};
use {
    crate::{ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID},
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const ORE_LST_PROGRAM_ID: Address = address!("LStwN2E5Uw6MCtuxHRLhy8RY9hxqW2XRpLzettb696y");
pub const ORE_STAKE_PROGRAM_ID: Address = address!("STkEAu2cEyQp5ktgUauRVq8es6mEP2w6ixw4NEd5tDJ");
pub const ORE_MINT: Address = address!("oreoU2P8bN6jkk3jbaiVxYnG1dCXcYxwhwyK9jSybcp");
pub const STORE_MINT: Address = address!("sTorERYB6xAZ1SSbwpK3zoK2EEwbBrc7TZAzg1uCGiH");
pub const STAKE: Address = address!("DfdZYzgLuqRickq57fyb4dX88VgPkhoEs1uuBKdxzaaJ");
pub const STAKE_TOKENS: Address = address!("6uEvYBcpb8KdhKxrRzffce9S7n8u9hiP2CXihJuUDihX");
pub const TREASURY: Address = address!("ANX3pRkcGipsZjcWVBvRaHFasBMw8FDPBvJHoubpWym6");
pub const TREASURY_TOKENS: Address = address!("FVynQtSNrWMa5Ueh1QNedca2YHSNtqH5LFjK3Sa9si2u");
pub const VAULT: Address = address!("7taXpXz6eqYzscXEi1d1fgwATQMqAR6Nku9pJCjb8gQN");
pub const VAULT_TOKENS: Address = address!("C1ZiFq8DocfTFxVUe75pqhbmaR8a7sKPsT9A48jmtzzr");

/// Pre-resolved addresses for building an OreLst swap instruction offline.
pub struct OreLstSwapInput {
    pub user: Address,
    pub sender_ore: Address,
    pub sender_store: Address,
}

/// Build OreLst swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &OreLstSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(ORE_LST_PROGRAM_ID, false),
        AccountMeta::new(input.user, true),
        AccountMeta::new(input.user, true),
        AccountMeta::new(input.sender_ore, false),
        AccountMeta::new(input.sender_store, false),
        AccountMeta::new(ORE_MINT, false),
        AccountMeta::new(STORE_MINT, false),
        AccountMeta::new(STAKE, false),
        AccountMeta::new(STAKE_TOKENS, false),
        AccountMeta::new(TREASURY, false),
        AccountMeta::new(TREASURY_TOKENS, false),
        AccountMeta::new(VAULT, false),
        AccountMeta::new(VAULT_TOKENS, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(ORE_STAKE_PROGRAM_ID, false),
    ]
}

/// Build OreLst extra data: [is_wrap].
pub fn build_extra_data(is_wrap: bool) -> Vec<u8> {
    vec![is_wrap as u8]
}

/// Resolve accounts and data for an OreLst swap via RPC.
#[cfg(feature = "resolve")]
pub async fn resolve(
    is_wrap: bool,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let sender_ore = get_associated_token_address(user, &ORE_MINT, &TOKEN_PROGRAM_ID);
    let sender_store = get_associated_token_address(user, &STORE_MINT, &TOKEN_PROGRAM_ID);

    let input = OreLstSwapInput {
        user: *user,
        sender_ore,
        sender_store,
    };

    Ok((build_accounts(&input), build_extra_data(is_wrap)))
}
