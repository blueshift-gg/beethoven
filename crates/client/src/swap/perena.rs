#[cfg(feature = "resolve")]
use {
    crate::{
        discover_pool, get_associated_token_address, get_token_program_for_mint, read_pubkey,
        ClientError,
    },
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    crate::{TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID},
    solana_address::Address,
    solana_instruction::AccountMeta,
};

pub const PERENA_PROGRAM_ID: Address =
    Address::from_str_const("NUMERUNsFCP3kuNmWZuXtm1AaQCPj9uw6Guv2Ekoi5P");

// StablePool account layout offsets
// Layout: [8-byte discriminator] [32 pool_seed] [32 lp_mint] [32 whitelisted_adder] [32 owner] [8 inv_T] [8 inv_T_max]
#[cfg(feature = "resolve")]
const OFFSET_VIRTUAL_STABLE_PAIR: usize = 152;
// VirtualStablePair struct layout offsets
// Layout: [32 pair_authority] [8 x_reserve_amount] [8 y_reserve] [16 curve_Amp] [16 curve_a] [16 curve_b] [16 inv_L] [32 owner] [32 x_mint] [32 x_vault] [8 curve_alpha] [8 curve_beta] [4 newest_rate_num] [4 newest_rate_denom] [1 decimals] [1 pair_index] [1 x_is_2022] [5 _padding] [128 padding]
#[cfg(feature = "resolve")]
const VIRTUAL_STABLE_PAIR_SIZE: usize = 368;
#[cfg(feature = "resolve")]
const OFFSET_X_MINT_IN_VIRTUAL_STABLE_PAIR: usize = 144;
#[cfg(feature = "resolve")]
const OFFSET_X_VAULT_IN_VIRTUAL_STABLE_PAIR: usize = 176;
#[cfg(feature = "resolve")]
const OFFSET_X_IS_2022_IN_VIRTUAL_STABLE_PAIR: usize = 234;

#[cfg(feature = "resolve")]
fn virtual_stable_pair_x_mint_offset(pair_index: u8) -> usize {
    OFFSET_VIRTUAL_STABLE_PAIR
        + (pair_index as usize) * VIRTUAL_STABLE_PAIR_SIZE
        + OFFSET_X_MINT_IN_VIRTUAL_STABLE_PAIR
}

#[cfg(feature = "resolve")]
fn virtual_stable_pair_x_vault_offset(pair_index: u8) -> usize {
    OFFSET_VIRTUAL_STABLE_PAIR
        + (pair_index as usize) * VIRTUAL_STABLE_PAIR_SIZE
        + OFFSET_X_VAULT_IN_VIRTUAL_STABLE_PAIR
}

#[cfg(feature = "resolve")]
fn virtual_stable_pair_x_is_2022_offset(pair_index: u8) -> usize {
    OFFSET_VIRTUAL_STABLE_PAIR
        + (pair_index as usize) * VIRTUAL_STABLE_PAIR_SIZE
        + OFFSET_X_IS_2022_IN_VIRTUAL_STABLE_PAIR
}

/// Pre-resolved addresses for building a Perena Numeraire swap instruction offline.
pub struct PerenaSwapInput {
    pub user: Address,
    pub pool: Address,
    pub in_mint: Address,
    pub out_mint: Address,
    pub in_trader: Address,
    pub out_trader: Address,
    pub in_vault: Address,
    pub out_vault: Address,
    pub numeraire_config: Address,
}

/// Build Perena Numeraire swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &PerenaSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(PERENA_PROGRAM_ID, false),
        AccountMeta::new(input.pool, false),
        AccountMeta::new_readonly(input.in_mint, false),
        AccountMeta::new_readonly(input.out_mint, false),
        AccountMeta::new(input.in_trader, false),
        AccountMeta::new(input.out_trader, false),
        AccountMeta::new(input.in_vault, false),
        AccountMeta::new(input.out_vault, false),
        AccountMeta::new_readonly(input.numeraire_config, false),
        AccountMeta::new(input.user, true),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
    ]
}

/// Build Perena Numeraire extra data: [in_index, out_index].
pub fn build_extra_data(in_index: u8, out_index: u8) -> Vec<u8> {
    vec![in_index, out_index]
}

/// Resolve accounts and data for a Perena Numeraire swap via RPC.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    pool: Option<&Address>,
    in_index: u8,
    out_index: u8,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    if in_index == out_index {
        return Err(ClientError::InvalidAccountData(
            "Perena in_index and out_index must be different".to_string(),
        ));
    }

    let (pool_pubkey, pool_data) = match pool {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = discover_pool(
                rpc,
                &PERENA_PROGRAM_ID,
                &[
                    (virtual_stable_pair_x_mint_offset(in_index), mint_a),
                    (virtual_stable_pair_x_mint_offset(out_index), mint_b),
                ],
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let in_mint = read_pubkey(&pool_data, virtual_stable_pair_x_mint_offset(in_index))?;
    let out_mint = read_pubkey(&pool_data, virtual_stable_pair_x_mint_offset(out_index))?;

    if *mint_a != in_mint {
        return Err(ClientError::MintMismatch {
            expected: in_mint.to_string(),
            got: mint_a.to_string(),
        });
    }
    if *mint_b != out_mint {
        return Err(ClientError::MintMismatch {
            expected: out_mint.to_string(),
            got: mint_b.to_string(),
        });
    }

    let in_vault = read_pubkey(&pool_data, virtual_stable_pair_x_vault_offset(in_index))?;
    let out_vault = read_pubkey(&pool_data, virtual_stable_pair_x_vault_offset(out_index))?;

    let in_token_program = get_token_program_for_mint(rpc, &in_mint).await?;
    let out_token_program = get_token_program_for_mint(rpc, &out_mint).await?;

    let in_is_2022_offset = virtual_stable_pair_x_is_2022_offset(in_index);
    let out_is_2022_offset = virtual_stable_pair_x_is_2022_offset(out_index);
    if pool_data.len() <= in_is_2022_offset || pool_data.len() <= out_is_2022_offset {
        return Err(ClientError::InvalidAccountData(
            "StablePool account data too short for x_is_2022 fields".to_string(),
        ));
    }
    let in_is_2022 = pool_data[in_is_2022_offset] != 0;
    let out_is_2022 = pool_data[out_is_2022_offset] != 0;

    let in_program_matches_flag = (in_is_2022 && in_token_program == TOKEN_2022_PROGRAM_ID)
        || (!in_is_2022 && in_token_program == TOKEN_PROGRAM_ID);
    if !in_program_matches_flag {
        return Err(ClientError::InvalidAccountData(format!(
            "Perena input mint/token program mismatch: x_is_2022={}, owner={}",
            in_is_2022, in_token_program
        )));
    }
    let out_program_matches_flag = (out_is_2022 && out_token_program == TOKEN_2022_PROGRAM_ID)
        || (!out_is_2022 && out_token_program == TOKEN_PROGRAM_ID);
    if !out_program_matches_flag {
        return Err(ClientError::InvalidAccountData(format!(
            "Perena output mint/token program mismatch: x_is_2022={}, owner={}",
            out_is_2022, out_token_program
        )));
    }

    let in_trader = get_associated_token_address(user, &in_mint, &in_token_program);
    let out_trader = get_associated_token_address(user, &out_mint, &out_token_program);

    let (numeraire_config, _) = Address::find_program_address(&[b"config"], &PERENA_PROGRAM_ID);

    let input = PerenaSwapInput {
        user: *user,
        pool: pool_pubkey,
        in_mint,
        out_mint,
        in_trader,
        out_trader,
        in_vault,
        out_vault,
        numeraire_config,
    };

    Ok((
        build_accounts(&input),
        build_extra_data(in_index, out_index),
    ))
}
