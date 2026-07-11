#[cfg(feature = "resolve")]
use {
    crate::{
        discover_pool_with_flip, get_associated_token_address, get_token_program_for_mint,
        read_pubkey, ClientError,
    },
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    crate::{MEMO_PROGRAM_ID, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID},
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const RAYDIUM_CLMM_PROGRAM_ID: Address =
    address!("CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK");

// Pool state account layout offsets
// Layout: [8 discriminator] [1 bump] [32 amm_config] [32 owner] [32 token_mint_0]
//         [32 token_mint_1] [32 token_vault_0] [32 token_vault_1] [32 observation_key] ...
#[cfg(feature = "resolve")]
const OFFSET_AMM_CONFIG: usize = 9;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_MINT_0: usize = 73;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_MINT_1: usize = 105;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_VAULT_0: usize = 137;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_VAULT_1: usize = 169;
#[cfg(feature = "resolve")]
const OFFSET_OBSERVATION_KEY: usize = 201;

/// Pre-resolved addresses for building a Raydium CLMM `swap_v2` instruction offline.
pub struct RaydiumClmmSwapInput {
    pub payer: Address,
    pub amm_config: Address,
    pub pool: Address,
    pub user_input_ata: Address,
    pub user_output_ata: Address,
    pub input_vault: Address,
    pub output_vault: Address,
    pub observation_state: Address,
    pub input_vault_mint: Address,
    pub output_vault_mint: Address,
}

pub fn build_accounts(input: &RaydiumClmmSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(RAYDIUM_CLMM_PROGRAM_ID, false),
        AccountMeta::new_readonly(input.payer, true),
        AccountMeta::new_readonly(input.amm_config, false),
        AccountMeta::new(input.pool, false),
        AccountMeta::new(input.user_input_ata, false),
        AccountMeta::new(input.user_output_ata, false),
        AccountMeta::new(input.input_vault, false),
        AccountMeta::new(input.output_vault, false),
        AccountMeta::new(input.observation_state, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
        AccountMeta::new_readonly(MEMO_PROGRAM_ID, false),
        AccountMeta::new_readonly(input.input_vault_mint, false),
        AccountMeta::new_readonly(input.output_vault_mint, false),
    ]
}

// Build Raydium CLMM swap_v2 extra data: [sqrt_price_limit_x64, is_base_input].
pub fn build_extra_data(sqrt_price_limit_x64: u128, is_base_input: bool) -> Vec<u8> {
    let mut v = Vec::with_capacity(17);
    v.extend_from_slice(&sqrt_price_limit_x64.to_le_bytes());
    v.push(u8::from(is_base_input));
    v
}

/// Resolve accounts and data for a Raydium CLMM swap via RPC.
///
/// `mint_a` is the input mint (what you're selling). `sqrt_price_limit_x64` is the
/// maximum price limit for the swap. `is_base_input` is whether the input mint is
/// the base token.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    pool: Option<&Address>,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
    sqrt_price_limit_x64: u128,
    is_base_input: bool,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let (pool_pubkey, pool_data) = match pool {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = discover_pool_with_flip(
                rpc,
                &RAYDIUM_CLMM_PROGRAM_ID,
                OFFSET_TOKEN_MINT_0,
                OFFSET_TOKEN_MINT_1,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let amm_config = read_pubkey(&pool_data, OFFSET_AMM_CONFIG)?;
    let token_mint_0 = read_pubkey(&pool_data, OFFSET_TOKEN_MINT_0)?;
    let token_mint_1 = read_pubkey(&pool_data, OFFSET_TOKEN_MINT_1)?;
    let token_vault_0 = read_pubkey(&pool_data, OFFSET_TOKEN_VAULT_0)?;
    let token_vault_1 = read_pubkey(&pool_data, OFFSET_TOKEN_VAULT_1)?;
    let observation_key = read_pubkey(&pool_data, OFFSET_OBSERVATION_KEY)?;

    let (input_vault, output_vault, input_mint, output_mint) = if *mint_a == token_mint_0 {
        (token_vault_0, token_vault_1, token_mint_0, token_mint_1)
    } else if *mint_a == token_mint_1 {
        (token_vault_1, token_vault_0, token_mint_1, token_mint_0)
    } else {
        return Err(ClientError::MintMismatch {
            expected: format!("{} or {}", token_mint_0, token_mint_1),
            got: mint_a.to_string(),
        });
    };

    let input_token_program = get_token_program_for_mint(rpc, &input_mint).await?;
    let output_token_program = get_token_program_for_mint(rpc, &output_mint).await?;

    let user_input_ata = get_associated_token_address(user, &input_mint, &input_token_program);
    let user_output_ata = get_associated_token_address(user, &output_mint, &output_token_program);

    let input = RaydiumClmmSwapInput {
        payer: *user,
        amm_config,
        pool: pool_pubkey,
        user_input_ata,
        user_output_ata,
        input_vault,
        output_vault,
        observation_state: observation_key,
        input_vault_mint: input_mint,
        output_vault_mint: output_mint,
    };

    Ok((
        build_accounts(&input),
        build_extra_data(sqrt_price_limit_x64, is_base_input),
    ))
}
