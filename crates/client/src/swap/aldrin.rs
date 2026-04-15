use solana_address::Address;
#[cfg(feature = "resolve")]
use {
    crate::{
        discover_pool_with_flip, get_associated_token_address, read_pubkey, ClientError,
        TOKEN_PROGRAM_ID,
    },
    solana_instruction::AccountMeta,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

pub const ALDRIN_PROGRAM_ID: Address =
    Address::from_str_const("AMM55ShdkoGRB5jVYPjWziwk8m5MpwyDgsMWHaMSQWH6");

// Pool account layout offsets
// Layout: [8 discriminator] [32 lpTokenFreezeVault] [32 poolMint]
//         [32 baseTokenVault] [32 baseTokenMint] [32 quoteTokenVault]
//         [32 quoteTokenMint] [32 poolSigner] [1 poolSignerNonce]
//         [32 authority] [32 initializerAccount] [32 feeBaseAccount]
//         [32 feeQuoteAccount] [32 feePoolTokenAccount] [48 fees]
#[cfg(feature = "resolve")]
const OFFSET_POOL_MINT: usize = 40;
#[cfg(feature = "resolve")]
const OFFSET_BASE_TOKEN_VAULT: usize = 72;
#[cfg(feature = "resolve")]
const OFFSET_BASE_TOKEN_MINT: usize = 104;
#[cfg(feature = "resolve")]
const OFFSET_QUOTE_TOKEN_VAULT: usize = 136;
#[cfg(feature = "resolve")]
const OFFSET_QUOTE_TOKEN_MINT: usize = 168;
#[cfg(feature = "resolve")]
const OFFSET_POOL_SIGNER: usize = 200;
#[cfg(feature = "resolve")]
const OFFSET_FEE_POOL_TOKEN_ACCOUNT: usize = 361;

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Side {
    Bid,
    Ask,
}

/// Pre-resolved addresses for building a Aldrin swap instruction offline.
pub struct AldrinSwapInput {
    pub pool: Address,
    pub pool_signer: Address,
    pub pool_mint: Address,
    pub base_token_vault: Address,
    pub quote_token_vault: Address,
    pub fee_pool_token_account: Address,
    pub wallet_authority: Address,
    pub user_base_token_account: Address,
    pub user_quote_token_account: Address,
}

/// Build Aldrin swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &AldrinSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(ALDRIN_PROGRAM_ID, false),
        AccountMeta::new_readonly(input.pool, false),
        AccountMeta::new_readonly(input.pool_signer, false),
        AccountMeta::new(input.pool_mint, false),
        AccountMeta::new(input.base_token_vault, false),
        AccountMeta::new(input.quote_token_vault, false),
        AccountMeta::new(input.fee_pool_token_account, false),
        AccountMeta::new(input.wallet_authority, true),
        AccountMeta::new(input.user_base_token_account, false),
        AccountMeta::new(input.user_quote_token_account, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
    ]
}

/// Build Aldrin extra data: [side].
pub fn build_extra_data(side: Side) -> Vec<u8> {
    let side_byte = match side {
        Side::Bid => 0,
        Side::Ask => 1,
    };

    vec![side_byte]
}

#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    pool: Option<&Address>,
    side: &Side,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let (pool_pubkey, pool_data) = match pool {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = discover_pool_with_flip(
                rpc,
                &ALDRIN_PROGRAM_ID,
                OFFSET_BASE_TOKEN_MINT,
                OFFSET_QUOTE_TOKEN_MINT,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let pool_mint = read_pubkey(&pool_data, OFFSET_POOL_MINT)?;
    let base_token_vault = read_pubkey(&pool_data, OFFSET_BASE_TOKEN_VAULT)?;
    let base_token_mint = read_pubkey(&pool_data, OFFSET_BASE_TOKEN_MINT)?;
    let quote_token_vault = read_pubkey(&pool_data, OFFSET_QUOTE_TOKEN_VAULT)?;
    let quote_token_mint = read_pubkey(&pool_data, OFFSET_QUOTE_TOKEN_MINT)?;
    let pool_signer = read_pubkey(&pool_data, OFFSET_POOL_SIGNER)?;
    let fee_pool_token_account = read_pubkey(&pool_data, OFFSET_FEE_POOL_TOKEN_ACCOUNT)?;

    let user_base_ata = get_associated_token_address(user, &base_token_mint, &TOKEN_PROGRAM_ID);
    let user_quote_ata = get_associated_token_address(user, &quote_token_mint, &TOKEN_PROGRAM_ID);

    let input = AldrinSwapInput {
        pool: pool_pubkey,
        pool_signer,
        pool_mint,
        base_token_vault,
        quote_token_vault,
        fee_pool_token_account,
        wallet_authority: *user,
        user_base_token_account: user_base_ata,
        user_quote_token_account: user_quote_ata,
    };

    Ok((build_accounts(&input), build_extra_data(*side)))
}
