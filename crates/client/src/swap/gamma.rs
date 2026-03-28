use {solana_address::Address, solana_instruction::AccountMeta};

pub const GAMMA_PROGRAM_ID: Address =
    Address::from_str_const("GAMMA7meSFWaBXF25oSUgmGRwaW6sCMFLmBNiMSdbHVT");

// Raydium CPMM PoolState layout offsets (raydium-cp-swap, #[repr(C, packed)])
// Layout: [8-byte discriminator] [32 amm_config] [32 pool_creator]
//         [32 token_0_vault] [32 token_1_vault] [32 lp_mint]
//         [32 token_0_mint] [32 token_1_mint] [32 token_0_program]
//         [32 token_1_program] [32 observation_key] ...
#[cfg(feature = "resolve")]
const OFFSET_AMM_CONFIG: usize = 8;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_VAULT_0: usize = 72;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_VAULT_1: usize = 104;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_MINT_0: usize = 168;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_MINT_1: usize = 200;
#[cfg(feature = "resolve")]
const OFFSET_OBSERVATION_KEY: usize = 296;

/// Pre-resolved addresses for building a Gamma swap instruction offline.
pub struct GammaSwapInput {
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

/// Build Gamma swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &GammaSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(GAMMA_PROGRAM_ID, false),
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

/// Gamma swap has no extra data.
pub fn build_extra_data() -> Vec<u8> {
    vec![]
}

/// Resolve accounts and data for a Gamma swap via RPC.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &solana_rpc_client::nonblocking::rpc_client::RpcClient,
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
            let (pubkey, account) = crate::discover_pool_with_flip(
                rpc,
                &GAMMA_PROGRAM_ID,
                OFFSET_TOKEN_MINT_0,
                OFFSET_TOKEN_MINT_1,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let amm_config = crate::read_pubkey(&pool_data, OFFSET_AMM_CONFIG)?;
    let token_mint_0 = crate::read_pubkey(&pool_data, OFFSET_TOKEN_MINT_0)?;
    let token_mint_1 = crate::read_pubkey(&pool_data, OFFSET_TOKEN_MINT_1)?;
    let token_vault_0 = crate::read_pubkey(&pool_data, OFFSET_TOKEN_VAULT_0)?;
    let token_vault_1 = crate::read_pubkey(&pool_data, OFFSET_TOKEN_VAULT_1)?;
    let observation_key = crate::read_pubkey(&pool_data, OFFSET_OBSERVATION_KEY)?;

    let (input_vault, output_vault, input_mint, output_mint) = if *mint_a == token_mint_0 {
        (token_vault_0, token_vault_1, token_mint_0, token_mint_1)
    } else if *mint_a == token_mint_1 {
        (token_vault_1, token_vault_0, token_mint_1, token_mint_0)
    } else {
        return Err(crate::error::ClientError::MintMismatch {
            expected: format!("{} or {}", token_mint_0, token_mint_1),
            got: mint_a.to_string(),
        });
    };

    let input_token_program = crate::get_token_program_for_mint(rpc, &input_mint).await?;
    let output_token_program = crate::get_token_program_for_mint(rpc, &output_mint).await?;

    let (authority, _) =
        Address::find_program_address(&[b"vault_and_lp_mint_auth_seed"], &GAMMA_PROGRAM_ID);

    let user_input_ata =
        crate::get_associated_token_address(user, &input_mint, &input_token_program);
    let user_output_ata =
        crate::get_associated_token_address(user, &output_mint, &output_token_program);

    let input = GammaSwapInput {
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

    Ok((build_accounts(&input), build_extra_data()))
}
