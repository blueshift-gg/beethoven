#[cfg(feature = "resolve")]
use crate::{get_associated_token_address, get_token_program_for_mint, read_pubkey, ClientError};
#[cfg(feature = "resolve")]
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use {
    crate::SYSVAR_INSTRUCTIONS_ID,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const TESSERA_V_PROGRAM_ID: Address = address!("TessVdML9pBGgG9yGks7o4HewRaXVAMuoVj4x83GLQH");
pub const GLOBAL_STATE: Address = address!("8ekCy2jHHUbW2yeNGFWYJT9Hm9FW7SvZcZK66dSZCDiF");

// Market account layout offset:
// Layout: ... [32-byte mint_a] @ 24 [32-byte mint_b] ...
#[cfg(feature = "resolve")]
const OFFSET_MINT_A: usize = 24;
#[cfg(feature = "resolve")]
const OFFSET_MINT_B: usize = 56;

/// Pre-resolved addresses for building a Tessera V swap instruction offline.
pub struct TesseraVSwapInput {
    pub market: Address,
    pub vault_a: Address,
    pub vault_b: Address,
    pub user_ata_a: Address,
    pub user_ata_b: Address,
    pub mint_a: Address,
    pub mint_b: Address,
    pub token_program_a: Address,
    pub token_program_b: Address,
    pub user: Address,
}

/// Build Tessera V swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &TesseraVSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(TESSERA_V_PROGRAM_ID, false),
        AccountMeta::new_readonly(GLOBAL_STATE, false),
        AccountMeta::new(input.market, false),
        AccountMeta::new(input.user, true),
        AccountMeta::new(input.vault_a, false),
        AccountMeta::new(input.vault_b, false),
        AccountMeta::new(input.user_ata_a, false),
        AccountMeta::new(input.user_ata_b, false),
        AccountMeta::new_readonly(input.mint_a, false),
        AccountMeta::new_readonly(input.mint_b, false),
        AccountMeta::new_readonly(input.token_program_a, false),
        AccountMeta::new_readonly(input.token_program_b, false),
        AccountMeta::new_readonly(SYSVAR_INSTRUCTIONS_ID, false),
    ]
}

/// Build Tessera V extra data: [is_a_to_b].
pub fn build_extra_data(is_a_to_b: bool) -> Vec<u8> {
    vec![is_a_to_b as u8]
}

/// Resolve accounts and extra data for Tessera V `swap` via RPC.
///
/// `mint_a` is the user's input mint (token sold), `mint_b` is the output mint.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    market: Option<&Address>,
    vault_a: &Address,
    vault_b: &Address,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    // let market_account = rpc.get_account(market).await?;
    let (market_pubkey, market_data) = match market {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = crate::discover_pool_with_flip(
                rpc,
                &TESSERA_V_PROGRAM_ID,
                OFFSET_MINT_A,
                OFFSET_MINT_B,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let market_mint_a = read_pubkey(&market_data, OFFSET_MINT_A)?;
    let market_mint_b = read_pubkey(&market_data, OFFSET_MINT_B)?;

    let is_a_to_b = if *mint_a == market_mint_a && *mint_b == market_mint_b {
        true
    } else if *mint_a == market_mint_b && *mint_b == market_mint_a {
        false
    } else {
        return Err(ClientError::MintMismatch {
            expected: format!("{} and {} (or flipped)", market_mint_a, market_mint_b),
            got: format!("{} -> {}", mint_a, mint_b),
        });
    };

    let token_program_a = get_token_program_for_mint(rpc, &market_mint_a).await?;
    let token_program_b = get_token_program_for_mint(rpc, &market_mint_b).await?;

    let user_ata_a = get_associated_token_address(user, &market_mint_a, &token_program_a);
    let user_ata_b = get_associated_token_address(user, &market_mint_b, &token_program_b);

    let vault_account_a = rpc.get_account(vault_a).await?;
    let vault_account_b = rpc.get_account(vault_b).await?;
    let vault_mint_a = read_pubkey(&vault_account_a.data, 0)?;
    let vault_mint_b = read_pubkey(&vault_account_b.data, 0)?;

    if vault_mint_a != market_mint_a || vault_mint_b != market_mint_b {
        return Err(ClientError::InvalidAccountData(
            "vault_a/vault_b mints do not match market mint_a/mint_b".to_string(),
        ));
    }

    let input = TesseraVSwapInput {
        market: market_pubkey,
        vault_a: *vault_a,
        vault_b: *vault_b,
        user_ata_a,
        user_ata_b,
        mint_a: market_mint_a,
        mint_b: market_mint_b,
        token_program_a,
        token_program_b,
        user: *user,
    };

    Ok((build_accounts(&input), build_extra_data(is_a_to_b)))
}
