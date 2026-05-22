#[cfg(feature = "resolve")]
use {
    crate::{get_associated_token_address, read_mint_authority, read_pubkey, ClientError},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    crate::{ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID},
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const SOLV_FINANCE_PROGRAM_ID: Address =
    address!("soLv1S6GsAEVEnXmVY3oz6GtrNJteQ28iTyRQrHXvkz");

// Vault account layout offsets
// Layout: [1 discriminator] [32 admin] [32 mint] [32 fee_receiver] [32 treasurer] ...
#[cfg(feature = "resolve")]
const OFFSET_TREASURER: usize = 97;

/// Pre-resolved addresses for building a Solv Finance `vault_deposit` offline.
pub struct SolvFinanceSwapInput {
    pub user: Address,
    pub user_token_ta: Address,
    pub user_target_ta: Address,
    pub treasurer_token_ta: Address,
    pub multisig: Address,
    pub mint_token: Address,
    pub mint_target: Address,
    pub vault: Address,
}

/// Build Solv Finance swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &SolvFinanceSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(SOLV_FINANCE_PROGRAM_ID, false),
        AccountMeta::new(input.user, true),
        AccountMeta::new(input.user_token_ta, false),
        AccountMeta::new(input.user_target_ta, false),
        AccountMeta::new(input.treasurer_token_ta, false),
        AccountMeta::new_readonly(input.multisig, false),
        AccountMeta::new_readonly(input.mint_token, false),
        AccountMeta::new(input.mint_target, false),
        AccountMeta::new(input.vault, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
    ]
}

/// Resolve accounts and data for a Solv Finance `vault_deposit` via RPC.
///
/// `mint_a` is the input/deposit token mint and `mint_b` is the target token mint.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    vault: Option<&Address>,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let mint_token = *mint_a;
    let mint_target = *mint_b;

    let vault_pubkey = match vault {
        Some(addr) => *addr,
        None => {
            Address::find_program_address(
                &[b"vault", mint_target.as_ref()],
                &SOLV_FINANCE_PROGRAM_ID,
            )
            .0
        }
    };

    let vault_account = rpc.get_account(&vault_pubkey).await?;
    let treasurer = read_pubkey(&vault_account.data, OFFSET_TREASURER)?;

    let mint_target_account = rpc.get_account(&mint_target).await?;
    let multisig = read_mint_authority(&mint_target_account.data)?;

    let user_token_ta = get_associated_token_address(user, &mint_token, &TOKEN_PROGRAM_ID);
    let user_target_ta = get_associated_token_address(user, &mint_target, &TOKEN_PROGRAM_ID);
    let treasurer_token_ta =
        get_associated_token_address(&treasurer, &mint_token, &TOKEN_PROGRAM_ID);

    let input = SolvFinanceSwapInput {
        user: *user,
        user_token_ta,
        user_target_ta,
        treasurer_token_ta,
        multisig,
        mint_token,
        mint_target,
        vault: vault_pubkey,
    };

    Ok((build_accounts(&input), vec![]))
}
