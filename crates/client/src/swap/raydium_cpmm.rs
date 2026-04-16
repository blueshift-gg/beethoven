#[cfg(feature = "resolve")]
use {
    crate::{discover_pool_with_flip, get_associated_token_address, read_pubkey},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {solana_address::Address, solana_instruction::AccountMeta};

pub const RAYDIUM_CPMM_PROGRAM_ID: Address =
    Address::from_str_const("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C");

// Pool state account layout offsets
// Layout: [8 discriminator] [32 amm_config] [32 pool_creator]
//         [32 token_0_vault] [32 token_1_vault] [32 lp_mint]
//         [32 token_0_mint] [32 token_1_mint] [32 token_0_program]
//         [32 token_1_program] [32 observation_key] ...
#[cfg(feature = "resolve")]
const OFFSET_AMM_CONFIG: usize = 8;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_0_VAULT: usize = 72;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_1_VAULT: usize = 104;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_0_MINT: usize = 168;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_1_MINT: usize = 200;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_0_PROGRAM: usize = 232;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_1_PROGRAM: usize = 264;
#[cfg(feature = "resolve")]
const OFFSET_OBSERVATION_KEY: usize = 296;

/// Pre-resolved addresses for building a Raydium CPMM `swap_base_input` offline.
pub struct RaydiumCpmmSwapInput {
    pub user: Address,
    pub authority: Address,
    pub amm_config: Address,
    pub pool: Address,
    pub user_input_ata: Address,
    pub user_output_ata: Address,
    pub input_vault: Address,
    pub output_vault: Address,
    pub input_token_program: Address,
    pub output_token_program: Address,
    pub input_mint: Address,
    pub output_mint: Address,
    pub observation_key: Address,
}

pub fn build_accounts(input: &RaydiumCpmmSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(RAYDIUM_CPMM_PROGRAM_ID, false),
        AccountMeta::new_readonly(input.user, true),
        AccountMeta::new_readonly(input.authority, false),
        AccountMeta::new_readonly(input.amm_config, false),
        AccountMeta::new(input.pool, false),
        AccountMeta::new(input.user_input_ata, false),
        AccountMeta::new(input.user_output_ata, false),
        AccountMeta::new(input.input_vault, false),
        AccountMeta::new(input.output_vault, false),
        AccountMeta::new_readonly(input.input_token_program, false),
        AccountMeta::new_readonly(input.output_token_program, false),
        AccountMeta::new_readonly(input.input_mint, false),
        AccountMeta::new_readonly(input.output_mint, false),
        AccountMeta::new(input.observation_key, false),
    ]
}

#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    pool: Option<&Address>,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), crate::error::ClientError> {
    let (pool_pubkey, pool_data) = match pool {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = discover_pool_with_flip(
                rpc,
                &RAYDIUM_CPMM_PROGRAM_ID,
                OFFSET_TOKEN_0_MINT,
                OFFSET_TOKEN_1_MINT,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let amm_config = read_pubkey(&pool_data, OFFSET_AMM_CONFIG)?;
    let token_0_mint = read_pubkey(&pool_data, OFFSET_TOKEN_0_MINT)?;
    let token_1_mint = read_pubkey(&pool_data, OFFSET_TOKEN_1_MINT)?;
    let token_0_vault = read_pubkey(&pool_data, OFFSET_TOKEN_0_VAULT)?;
    let token_1_vault = read_pubkey(&pool_data, OFFSET_TOKEN_1_VAULT)?;
    let observation_key = read_pubkey(&pool_data, OFFSET_OBSERVATION_KEY)?;

    let (input_vault, output_vault, input_mint, output_mint) =
        if *mint_a == token_0_mint && *mint_b == token_1_mint {
            (token_0_vault, token_1_vault, token_0_mint, token_1_mint)
        } else if *mint_a == token_1_mint && *mint_b == token_0_mint {
            (token_1_vault, token_0_vault, token_1_mint, token_0_mint)
        } else {
            return Err(crate::error::ClientError::MintMismatch {
                expected: format!(
                    "({}, {}) or ({}, {})",
                    token_0_mint, token_1_mint, token_1_mint, token_0_mint
                ),
                got: format!("({}, {})", mint_a, mint_b),
            });
        };

    let input_token_program = if input_mint == token_0_mint {
        read_pubkey(&pool_data, OFFSET_TOKEN_0_PROGRAM)?
    } else {
        read_pubkey(&pool_data, OFFSET_TOKEN_1_PROGRAM)?
    };
    let output_token_program = if output_mint == token_0_mint {
        read_pubkey(&pool_data, OFFSET_TOKEN_0_PROGRAM)?
    } else {
        read_pubkey(&pool_data, OFFSET_TOKEN_1_PROGRAM)?
    };

    let (authority, _) =
        Address::find_program_address(&[b"vault_and_lp_mint_auth_seed"], &RAYDIUM_CPMM_PROGRAM_ID);

    let user_input_ata = get_associated_token_address(user, &input_mint, &input_token_program);
    let user_output_ata = get_associated_token_address(user, &output_mint, &output_token_program);

    let input = RaydiumCpmmSwapInput {
        user: *user,
        authority,
        amm_config,
        pool: pool_pubkey,
        user_input_ata,
        user_output_ata,
        input_vault,
        output_vault,
        input_token_program,
        output_token_program,
        input_mint,
        output_mint,
        observation_key,
    };

    Ok((build_accounts(&input), vec![]))
}
