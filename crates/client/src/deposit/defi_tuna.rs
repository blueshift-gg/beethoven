use {
    crate::MEMO_PROGRAM_ID,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};
#[cfg(feature = "resolve")]
use {
    crate::{get_associated_token_address, get_token_program_for_mint, read_pubkey, ClientError},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

pub const DEFI_TUNA_PROGRAM_ID: Address = address!("tuna4uSQZncNeeiAMKbstuxA9CUkHH6HmC64wgmnogD");

pub const TUNA_CONFIG: Address = address!("H1utnsgjEupueAKZckqXbeyi3DSokgauuYUFCGv8mRZ4");

// Vault account layout offsets
// Layout: [8 discriminator] [2 version] [1 bump] [32 mint] ...
#[cfg(feature = "resolve")]
const OFFSET_VAULT_MINT: usize = 11;

/// Pre-resolved addresses for building an DefiTuna deposit instruction offline.
pub struct DefiTunaDepositInput {
    pub authority: Address,
    pub mint: Address,
    pub lending_position: Address,
    pub vault: Address,
    pub vault_ata: Address,
    pub authority_ata: Address,
    pub token_program: Address,
}

/// Build DefiTuna deposit AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &DefiTunaDepositInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(DEFI_TUNA_PROGRAM_ID, false),
        AccountMeta::new(input.authority, true),
        AccountMeta::new_readonly(input.mint, false),
        AccountMeta::new_readonly(TUNA_CONFIG, false),
        AccountMeta::new(input.lending_position, false),
        AccountMeta::new(input.vault, false),
        AccountMeta::new(input.vault_ata, false),
        AccountMeta::new(input.authority_ata, false),
        AccountMeta::new_readonly(input.token_program, false),
        AccountMeta::new_readonly(MEMO_PROGRAM_ID, false),
    ]
}

/// Resolve accounts from a known vault; checks mint pair and PDAs against on-chain data.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    vault: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let vault_data = rpc.get_account(vault).await?.data;

    let mint = read_pubkey(&vault_data, OFFSET_VAULT_MINT)?;
    let lending_position = Address::find_program_address(
        &[b"lending_position", user.as_ref(), vault.as_ref()],
        &DEFI_TUNA_PROGRAM_ID,
    )
    .0;
    let mint_token_program = get_token_program_for_mint(rpc, &mint).await?;
    let vault_ata = get_associated_token_address(vault, &mint, &mint_token_program);
    let authority_ata = get_associated_token_address(user, &mint, &mint_token_program);

    let input = DefiTunaDepositInput {
        authority: *user,
        mint,
        lending_position,
        vault: *vault,
        vault_ata,
        authority_ata,
        token_program: mint_token_program,
    };

    Ok((build_accounts(&input), vec![]))
}
