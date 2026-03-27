use solana_address::Address;

pub const ALDRIN_PROGRAM_ID: Address =
    Address::from_str_const("AMM55ShdkoGRB5jVYPjWziwk8m5MpwyDgsMWHaMSQWH6");

// Aldrin Pool V1 layout offsets (from poolsV1.json IDL)
// Layout: [8-byte discriminator] [32-byte lpTokenFreezeVault] [32-byte poolMint]
//         [32-byte baseTokenVault] [32-byte baseTokenMint] [32-byte quoteTokenVault]
//         [32-byte quoteTokenMint] [32-byte poolSigner] [1-byte poolSignerNonce]
//         [32-byte authority] [32-byte initializerAccount] [32-byte feeBaseAccount]
//         [32-byte feeQuoteAccount] [32-byte feePoolTokenAccount] [Fees struct]
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

#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &solana_rpc_client::nonblocking::rpc_client::RpcClient,
    pool: Option<&Address>,
    side: u8,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<solana_instruction::AccountMeta>, Vec<u8>), crate::error::ClientError> {
    use solana_instruction::AccountMeta;

    let (pool_pubkey, pool_data) = match pool {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = crate::discover_pool_with_flip(
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

    let pool_mint = crate::read_pubkey(&pool_data, OFFSET_POOL_MINT)?;
    let base_token_vault = crate::read_pubkey(&pool_data, OFFSET_BASE_TOKEN_VAULT)?;
    let base_token_mint = crate::read_pubkey(&pool_data, OFFSET_BASE_TOKEN_MINT)?;
    let quote_token_vault = crate::read_pubkey(&pool_data, OFFSET_QUOTE_TOKEN_VAULT)?;
    let quote_token_mint = crate::read_pubkey(&pool_data, OFFSET_QUOTE_TOKEN_MINT)?;
    let pool_signer = crate::read_pubkey(&pool_data, OFFSET_POOL_SIGNER)?;
    let fee_pool_token_account = crate::read_pubkey(&pool_data, OFFSET_FEE_POOL_TOKEN_ACCOUNT)?;

    let user_base_ata =
        crate::get_associated_token_address(user, &base_token_mint, &crate::TOKEN_PROGRAM_ID);
    let user_quote_ata =
        crate::get_associated_token_address(user, &quote_token_mint, &crate::TOKEN_PROGRAM_ID);

    let accounts = vec![
        AccountMeta::new_readonly(ALDRIN_PROGRAM_ID, false),
        AccountMeta::new_readonly(pool_pubkey, false),
        AccountMeta::new_readonly(pool_signer, false),
        AccountMeta::new(pool_mint, false),
        AccountMeta::new(base_token_vault, false),
        AccountMeta::new(quote_token_vault, false),
        AccountMeta::new(fee_pool_token_account, false),
        AccountMeta::new_readonly(*user, true),
        AccountMeta::new(user_base_ata, false),
        AccountMeta::new(user_quote_ata, false),
        AccountMeta::new_readonly(crate::TOKEN_PROGRAM_ID, false),
    ];

    Ok((accounts, vec![side]))
}
