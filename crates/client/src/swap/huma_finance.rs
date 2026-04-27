#[cfg(feature = "resolve")]
use {
    crate::{
        get_associated_token_address, get_token_program_for_mint, read_pubkey, ClientError,
        TOKEN_PROGRAM_ID,
    },
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const HUMA_FINANCE_PROGRAM_ID: Address =
    address!("HumaXepHnjaRCpjYTokxY4UtaJcmx41prQ8cxGmFC5fn");
pub const HUMA_CONFIG: Address = address!("Fh2WKYCJfota6k76gDGnhTELUuhPa7FHQvVza4cE11ja");
pub const POOL_CONFIG: Address = address!("28hFhD21Nka3stL27a8zZ4nRLgaDVxRYwJgeEVgeakzS");
pub const POOL_STATE: Address = address!("iFgP2EbzHUZzMjqbjaagJQ8zmn6as3Hw95aVUKm67od");
pub const MODE_CONFIG: Address = address!("3FhoMDyKzQqxtGxnz9DfysfoGQKvgDnSFjoDGgguDCQN");
pub const PST_MINT: Address = address!("59obFNBzyTBGowrkif5uK7ojS58vsuWz3ZCvg6tfZAGw");
pub const POOL_AUTHORITY: Address = address!("9936VFvgRmW1STvdgeyPQaKHDx5DwBtbhZkT3HcdL3QK");
pub const POOL_UNDERLYING_TOKEN: Address = address!("6Xh2Jg9sWJE16VQGppJFTHvQ8Vii3ABUvUF8Pwcwy7Vq");

// PoolConfig account layout offsets
// Layout: ... [32 underlying_mint] @ 105
#[cfg(feature = "resolve")]
const OFFSET_UNDERLYING_MINT: usize = 105;

/// Pre-resolved addresses for building an Huma Finance swap instruction offline.
pub struct HumaFinanceSwapInput {
    pub payer: Address,
    pub underlying_mint: Address,
    pub pool_underlying_token: Address,
    pub depositor_underlying_token: Address,
    pub depositor_mode_token: Address,
    pub underlying_token_program: Address,
}

/// Build Huma Finance swap AccountMeta list from pre-resolved addresses (no RPC needed).
fn build_accounts(input: &HumaFinanceSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(HUMA_FINANCE_PROGRAM_ID, false),
        AccountMeta::new(input.payer, true),
        AccountMeta::new_readonly(HUMA_CONFIG, false),
        AccountMeta::new_readonly(POOL_CONFIG, false),
        AccountMeta::new(POOL_STATE, false),
        AccountMeta::new_readonly(MODE_CONFIG, false),
        AccountMeta::new(PST_MINT, false),
        AccountMeta::new_readonly(POOL_AUTHORITY, false),
        AccountMeta::new_readonly(input.underlying_mint, false),
        AccountMeta::new(input.pool_underlying_token, false),
        AccountMeta::new(input.depositor_underlying_token, false),
        AccountMeta::new(input.depositor_mode_token, false),
        AccountMeta::new_readonly(input.underlying_token_program, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
    ]
}

/// Resolve accounts and data for an Huma Finance swap via RPC.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let pool_config_data = rpc.get_account(&POOL_CONFIG).await?.data;
    let underlying_mint = read_pubkey(&pool_config_data, OFFSET_UNDERLYING_MINT)?;
    let underlying_token_program = get_token_program_for_mint(rpc, &underlying_mint).await?;
    let pool_underlying_token =
        get_associated_token_address(&POOL_AUTHORITY, &underlying_mint, &underlying_token_program);
    let user_underlying_token =
        get_associated_token_address(user, &underlying_mint, &underlying_token_program);
    let user_mode_token = get_associated_token_address(user, &PST_MINT, &TOKEN_PROGRAM_ID);

    let input = HumaFinanceSwapInput {
        payer: *user,
        underlying_mint,
        pool_underlying_token,
        depositor_underlying_token: user_underlying_token,
        depositor_mode_token: user_mode_token,
        underlying_token_program,
    };

    Ok((build_accounts(&input), vec![]))
}
