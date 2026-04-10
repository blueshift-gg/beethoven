use {
    crate::TOKEN_PROGRAM_ID,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const METEORA_DAMM_PROGRAM_ID: Address =
    address!("Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB");
pub const METEORA_DYNAMIC_VAULT_PROGRAM_ID: Address =
    address!("24Uqj9JCLxUeoC3hGfh5W3s9FM9uCHDS2SG3LYwBpyTi");

// Pool account layout offsets
// Layout: [8 discriminator] [32 lp_mint] [32 token_a_mint] [32 token_b_mint] [32 a_vault] [32 b_vault]
//         [32 a_vault_lp] [32 b_vault_lp] [u8 a_vault_lp_bump] [bool enabled]
//         [32 protocol_token_a_fee] [32 protocol_token_b_fee] ...
#[cfg(feature = "resolve")]
const OFFSET_POOL_TOKEN_A_MINT: usize = 40;
#[cfg(feature = "resolve")]
const OFFSET_POOL_TOKEN_B_MINT: usize = 72;
#[cfg(feature = "resolve")]
const OFFSET_POOL_A_VAULT: usize = 104;
#[cfg(feature = "resolve")]
const OFFSET_POOL_B_VAULT: usize = 136;
#[cfg(feature = "resolve")]
const OFFSET_POOL_A_VAULT_LP: usize = 168;
#[cfg(feature = "resolve")]
const OFFSET_POOL_B_VAULT_LP: usize = 200;
#[cfg(feature = "resolve")]
const OFFSET_POOL_PROTOCOL_TOKEN_A_FEE: usize = 234;
#[cfg(feature = "resolve")]
const OFFSET_POOL_PROTOCOL_TOKEN_B_FEE: usize = 266;

// VaultBumps struct layout offsets
// Layout: [1 vault_bump] [1 total_vault_bump]

// Vault account layout offsets
// Layout: [8 discriminator] [1 enabled] [2 vault_bumps] [8 total_amount] [32 token_vault] [32 fee_vault] [32 token_mint] [32 lp_mint] ...
#[cfg(feature = "resolve")]
const OFFSET_VAULT_TOKEN_VAULT: usize = 19;
#[cfg(feature = "resolve")]
const OFFSET_VAULT_LP_MINT: usize = 115;

/// Pre-resolved addresses for building a Meteora DAMM swap offline.
pub struct MeteoraDammSwapInput {
    pub pool: Address,
    pub user_source_token: Address,
    pub user_destination_token: Address,
    pub a_vault: Address,
    pub b_vault: Address,
    pub a_token_vault: Address,
    pub b_token_vault: Address,
    pub a_vault_lp_mint: Address,
    pub b_vault_lp_mint: Address,
    pub a_vault_lp: Address,
    pub b_vault_lp: Address,
    pub protocol_token_fee: Address,
    pub user: Address,
}

/// Build Meteora DAMM swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &MeteoraDammSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(METEORA_DAMM_PROGRAM_ID, false),
        AccountMeta::new(input.pool, false),
        AccountMeta::new(input.user_source_token, false),
        AccountMeta::new(input.user_destination_token, false),
        AccountMeta::new(input.a_vault, false),
        AccountMeta::new(input.b_vault, false),
        AccountMeta::new(input.a_token_vault, false),
        AccountMeta::new(input.b_token_vault, false),
        AccountMeta::new(input.a_vault_lp_mint, false),
        AccountMeta::new(input.b_vault_lp_mint, false),
        AccountMeta::new(input.a_vault_lp, false),
        AccountMeta::new(input.b_vault_lp, false),
        AccountMeta::new(input.protocol_token_fee, false),
        AccountMeta::new(input.user, true),
        AccountMeta::new_readonly(METEORA_DYNAMIC_VAULT_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
    ]
}

/// Resolve accounts and data for a Meteora DAMM swap via RPC.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &solana_rpc_client::nonblocking::rpc_client::RpcClient,
    pool: Option<&Address>,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), crate::error::ClientError> {
    use crate::{get_associated_token_address, read_pubkey};

    let (pool_pubkey, pool_data) = match pool {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = crate::discover_pool_with_flip(
                rpc,
                &METEORA_DAMM_PROGRAM_ID,
                OFFSET_POOL_TOKEN_A_MINT,
                OFFSET_POOL_TOKEN_B_MINT,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let token_a_mint = read_pubkey(&pool_data, OFFSET_POOL_TOKEN_A_MINT)?;
    let token_b_mint = read_pubkey(&pool_data, OFFSET_POOL_TOKEN_B_MINT)?;

    if token_a_mint != *mint_a {
        return Err(crate::error::ClientError::MintMismatch {
            expected: mint_a.to_string(),
            got: token_a_mint.to_string(),
        });
    }

    if token_b_mint != *mint_b {
        return Err(crate::error::ClientError::MintMismatch {
            expected: mint_b.to_string(),
            got: token_b_mint.to_string(),
        });
    }

    let a_vault = read_pubkey(&pool_data, OFFSET_POOL_A_VAULT)?;
    let b_vault = read_pubkey(&pool_data, OFFSET_POOL_B_VAULT)?;
    let a_vault_lp = read_pubkey(&pool_data, OFFSET_POOL_A_VAULT_LP)?;
    let b_vault_lp = read_pubkey(&pool_data, OFFSET_POOL_B_VAULT_LP)?;
    let protocol_token_a_fee = read_pubkey(&pool_data, OFFSET_POOL_PROTOCOL_TOKEN_A_FEE)?;
    let protocol_token_b_fee = read_pubkey(&pool_data, OFFSET_POOL_PROTOCOL_TOKEN_B_FEE)?;

    let a_vault_acc = rpc.get_account(&a_vault).await?;
    let b_vault_acc = rpc.get_account(&b_vault).await?;
    let a_token_vault = read_pubkey(&a_vault_acc.data, OFFSET_VAULT_TOKEN_VAULT)?;
    let b_token_vault = read_pubkey(&b_vault_acc.data, OFFSET_VAULT_TOKEN_VAULT)?;
    let a_vault_lp_mint = read_pubkey(&a_vault_acc.data, OFFSET_VAULT_LP_MINT)?;
    let b_vault_lp_mint = read_pubkey(&b_vault_acc.data, OFFSET_VAULT_LP_MINT)?;

    let (token_in_mint, token_out_mint, protocol_token_fee): (Address, Address, Address) =
        if *mint_a == token_a_mint {
            (token_a_mint, token_b_mint, protocol_token_a_fee)
        } else if *mint_a == token_b_mint {
            (token_b_mint, token_a_mint, protocol_token_b_fee)
        } else {
            return Err(crate::error::ClientError::MintMismatch {
                expected: format!("{} or {}", token_a_mint, token_b_mint),
                got: mint_a.to_string(),
            });
        };

    let user_source_token = get_associated_token_address(user, &token_in_mint, &TOKEN_PROGRAM_ID);
    let user_destination_token =
        get_associated_token_address(user, &token_out_mint, &TOKEN_PROGRAM_ID);

    let input = MeteoraDammSwapInput {
        pool: pool_pubkey,
        user_source_token,
        user_destination_token,
        a_vault,
        b_vault,
        a_token_vault,
        b_token_vault,
        a_vault_lp_mint,
        b_vault_lp_mint,
        a_vault_lp,
        b_vault_lp,
        protocol_token_fee,
        user: *user,
    };

    Ok((build_accounts(&input), vec![]))
}
