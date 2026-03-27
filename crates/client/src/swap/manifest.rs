use {solana_address::Address, solana_instruction::AccountMeta};

pub const MANIFEST_PROGRAM_ID: Address =
    Address::from_str_const("MNFSTqtC93rEfYHB6hF82sKdZpUDFWkViLByLd1k1Ms");

// Manifest MarketFixed layout offsets (from market.rs source)
// Layout: [8-byte discriminant] [1-byte version] [1-byte base_mint_decimals]
//         [1-byte quote_mint_decimals] [1-byte base_vault_bump] [1-byte quote_vault_bump]
//         [3-byte padding] [32-byte base_mint] [32-byte quote_mint]
//         [32-byte base_vault] [32-byte quote_vault] ...
#[cfg(feature = "resolve")]
const OFFSET_BASE_MINT: usize = 16;
#[cfg(feature = "resolve")]
const OFFSET_QUOTE_MINT: usize = 48;
#[cfg(feature = "resolve")]
const OFFSET_BASE_VAULT: usize = 80;
#[cfg(feature = "resolve")]
const OFFSET_QUOTE_VAULT: usize = 112;

/// Pre-resolved addresses for building a Manifest swap instruction offline.
pub struct ManifestSwapInput {
    pub user: Address,
    pub market: Address,
    pub trader_base: Address,
    pub trader_quote: Address,
    pub base_vault: Address,
    pub quote_vault: Address,
    pub base_token_program: Address,
    pub base_mint: Address,
    pub quote_token_program: Address,
    pub quote_mint: Address,
    pub global: Address,
    pub global_vault: Address,
}

/// Build Manifest swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &ManifestSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(MANIFEST_PROGRAM_ID, false),
        AccountMeta::new(input.user, true),
        AccountMeta::new_readonly(input.user, true),
        AccountMeta::new(input.market, false),
        AccountMeta::new_readonly(crate::SYSTEM_PROGRAM_ID, false),
        AccountMeta::new(input.trader_base, false),
        AccountMeta::new(input.trader_quote, false),
        AccountMeta::new(input.base_vault, false),
        AccountMeta::new(input.quote_vault, false),
        AccountMeta::new_readonly(input.base_token_program, false),
        AccountMeta::new_readonly(input.base_mint, false),
        AccountMeta::new_readonly(input.quote_token_program, false),
        AccountMeta::new_readonly(input.quote_mint, false),
        AccountMeta::new(input.global, false),
        AccountMeta::new(input.global_vault, false),
    ]
}

/// Build Manifest extra data: [is_base_in, is_exact_in].
pub fn build_extra_data(is_base_in: bool, is_exact_in: bool) -> Vec<u8> {
    vec![is_base_in as u8, is_exact_in as u8]
}

/// Resolve accounts and data for a Manifest swap via RPC.
///
/// `mint_a` is the input mint (what you're selling). `is_base_in` is inferred
/// by comparing `mint_a` against the market's base mint.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &solana_rpc_client::nonblocking::rpc_client::RpcClient,
    market: Option<&Address>,
    is_exact_in: bool,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), crate::error::ClientError> {
    let (market_pubkey, market_data) = match market {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = crate::discover_pool_with_flip(
                rpc,
                &MANIFEST_PROGRAM_ID,
                OFFSET_BASE_MINT,
                OFFSET_QUOTE_MINT,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let base_mint = crate::read_pubkey(&market_data, OFFSET_BASE_MINT)?;
    let quote_mint = crate::read_pubkey(&market_data, OFFSET_QUOTE_MINT)?;
    let base_vault = crate::read_pubkey(&market_data, OFFSET_BASE_VAULT)?;
    let quote_vault = crate::read_pubkey(&market_data, OFFSET_QUOTE_VAULT)?;

    let is_base_in = *mint_a == base_mint;

    let base_token_program = crate::get_token_program_for_mint(rpc, &base_mint).await?;
    let quote_token_program = crate::get_token_program_for_mint(rpc, &quote_mint).await?;

    let trader_base = crate::get_associated_token_address(user, &base_mint, &base_token_program);
    let trader_quote = crate::get_associated_token_address(user, &quote_mint, &quote_token_program);

    let (global, _) = Address::find_program_address(&[b"global"], &MANIFEST_PROGRAM_ID);
    let (global_vault, _) = Address::find_program_address(&[b"global-vault"], &MANIFEST_PROGRAM_ID);

    let input = ManifestSwapInput {
        user: *user,
        market: market_pubkey,
        trader_base,
        trader_quote,
        base_vault,
        quote_vault,
        base_token_program,
        base_mint,
        quote_token_program,
        quote_mint,
        global,
        global_vault,
    };

    Ok((
        build_accounts(&input),
        build_extra_data(is_base_in, is_exact_in),
    ))
}
