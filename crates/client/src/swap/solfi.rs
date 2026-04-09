use {crate::SYSVAR_INSTRUCTIONS_ID, solana_address::Address, solana_instruction::AccountMeta};
#[cfg(feature = "resolve")]
use {
    crate::{
        discover_pool_with_flip, get_associated_token_address, get_token_program_for_mint,
        read_pubkey, ClientError,
    },
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

pub const SOLFI_PROGRAM_ID: Address =
    Address::from_str_const("SoLFiHG9TfgtdUXUjWAxi3LtvYuFyDLVhBWxdMZxyCe");

// Market account layout offsets
#[cfg(feature = "resolve")]
const OFFSET_MARKET_BASE_MINT: usize = 2664;
#[cfg(feature = "resolve")]
const OFFSET_MARKET_QUOTE_MINT: usize = 2696;
#[cfg(feature = "resolve")]
const OFFSET_MARKET_TOKEN_A_VAULT: usize = 2736;
#[cfg(feature = "resolve")]
const OFFSET_MARKET_TOKEN_B_VAULT: usize = 2768;

/// Pre-resolved addresses for building a SolFi swap instruction offline.
pub struct SolFiSwapInput {
    pub token_transfer_authority: Address,
    pub market_account: Address,
    pub base_vault: Address,
    pub quote_vault: Address,
    pub user_base_ata: Address,
    pub user_quote_ata: Address,
    pub token_program: Address,
}

/// Build SolFi swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &SolFiSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(SOLFI_PROGRAM_ID, false),
        AccountMeta::new(input.token_transfer_authority, true),
        AccountMeta::new(input.market_account, false),
        AccountMeta::new(input.base_vault, false),
        AccountMeta::new(input.quote_vault, false),
        AccountMeta::new(input.user_base_ata, false),
        AccountMeta::new(input.user_quote_ata, false),
        AccountMeta::new_readonly(input.token_program, false),
        AccountMeta::new_readonly(SYSVAR_INSTRUCTIONS_ID, false),
    ]
}

/// SolFi extra data: [is_quote_to_base]
pub fn build_extra_data(is_quote_to_base: bool) -> Vec<u8> {
    vec![is_quote_to_base as u8]
}

/// Resolve accounts and data for a SolFi swap via RPC.
///
/// `mint_a` is the input mint (what you're selling). Direction is inferred
/// by comparing `mint_a` against the pair's token0.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    market: Option<&Address>,
    is_quote_to_base: bool,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let (market_pubkey, market_data) = match market {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = discover_pool_with_flip(
                rpc,
                &SOLFI_PROGRAM_ID,
                OFFSET_MARKET_BASE_MINT,
                OFFSET_MARKET_QUOTE_MINT,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let base_mint = read_pubkey(&market_data, OFFSET_MARKET_BASE_MINT)?;
    let quote_mint = read_pubkey(&market_data, OFFSET_MARKET_QUOTE_MINT)?;

    if *mint_a != base_mint && *mint_a != quote_mint {
        return Err(ClientError::MintMismatch {
            expected: format!("{} or {}", base_mint, quote_mint),
            got: mint_a.to_string(),
        });
    }

    if *mint_b != base_mint && *mint_b != quote_mint {
        return Err(ClientError::MintMismatch {
            expected: format!("{} or {}", base_mint, quote_mint),
            got: mint_b.to_string(),
        });
    }

    let base_vault = read_pubkey(&market_data, OFFSET_MARKET_TOKEN_A_VAULT)?;
    let quote_vault = read_pubkey(&market_data, OFFSET_MARKET_TOKEN_B_VAULT)?;
    let token_program = get_token_program_for_mint(rpc, &base_mint).await?;
    let user_base_ata = get_associated_token_address(user, &base_mint, &token_program);
    let user_quote_ata = get_associated_token_address(user, &quote_mint, &token_program);

    let input = SolFiSwapInput {
        token_transfer_authority: *user,
        market_account: market_pubkey,
        base_vault,
        quote_vault,
        user_base_ata,
        user_quote_ata,
        token_program,
    };

    Ok((build_accounts(&input), build_extra_data(is_quote_to_base)))
}
