#[cfg(feature = "resolve")]
use crate::error::ClientError;
use solana_address::Address;

pub const FUTARCHY_PROGRAM_ID: Address =
    Address::from_str_const("FUTARELBfJfQ8RDGhg1wdhddq1odMAJUePHFuBYfUxKq");

// Futarchy Dao layout (from on-chain IDL: amm_v0.3.json)
//
// The Dao account embeds a FutarchyAmm, whose first field is a Borsh enum
// (PoolState). Because enums have variable sizes, the byte offsets for
// baseMint / quoteMint / ammBaseVault / ammQuoteVault depend on the variant.
//
// PoolState variants:
//   0 = Spot     → 1 (tag) + Pool (132 bytes) = 133 bytes
//   1 = Futarchy → 1 (tag) + 3 × Pool (396 bytes) = 397 bytes
//
// Pool = TwapOracle (100 bytes) + 4 × u64 (32 bytes) = 132 bytes
// TwapOracle = u128 + i64 + i64 + u128 + u128 + u128 + u128 + u32 = 100 bytes
//
// After PoolState comes totalLiquidity (u128, 16 bytes), then the Pubkeys.
#[cfg(feature = "resolve")]
const POOL_STATE_TAG_OFFSET: usize = 8;
#[cfg(feature = "resolve")]
const POOL_SIZE: usize = 132;

#[cfg(feature = "resolve")]
fn compute_amm_field_offsets(
    pool_state_variant: u8,
) -> Result<(usize, usize, usize, usize), ClientError> {
    let pool_state_size = match pool_state_variant {
        0 => 1 + POOL_SIZE,     // Spot
        1 => 1 + POOL_SIZE * 3, // Futarchy (spot + pass + fail)
        _ => {
            return Err(ClientError::InvalidAccountData(
                "Unknown Futarchy PoolState variant".to_string(),
            ))
        }
    };

    let total_liquidity_offset = 8 + pool_state_size; // after discriminator + PoolState
    let base_mint_offset = total_liquidity_offset + 16; // after u128 totalLiquidity
    let quote_mint_offset = base_mint_offset + 32;
    let amm_base_vault_offset = quote_mint_offset + 32;
    let amm_quote_vault_offset = amm_base_vault_offset + 32;

    Ok((
        base_mint_offset,
        quote_mint_offset,
        amm_base_vault_offset,
        amm_quote_vault_offset,
    ))
}

#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &solana_rpc_client::nonblocking::rpc_client::RpcClient,
    dao: Option<&Address>,
    swap_type: u8,
    _mint_a: &Address,
    _mint_b: &Address,
    user: &Address,
) -> Result<(Vec<solana_instruction::AccountMeta>, Vec<u8>), ClientError> {
    use solana_instruction::AccountMeta;

    // Futarchy requires an explicit DAO address — the embedded PoolState enum
    // makes getProgramAccounts with fixed memcmp offsets impractical.
    let dao_pubkey = dao.ok_or(ClientError::InvalidAccountData(
        "Futarchy requires an explicit DAO address (variable-size PoolState enum prevents auto-discovery)".to_string(),
    ))?;

    let account = rpc.get_account(dao_pubkey).await?;
    let dao_data = account.data;

    if dao_data.len() <= POOL_STATE_TAG_OFFSET {
        return Err(ClientError::InvalidAccountData(
            "Dao account data too short".to_string(),
        ));
    }

    let pool_state_variant = dao_data[POOL_STATE_TAG_OFFSET];
    let (base_mint_offset, quote_mint_offset, base_vault_offset, quote_vault_offset) =
        compute_amm_field_offsets(pool_state_variant)?;

    let base_mint = crate::read_pubkey(&dao_data, base_mint_offset)?;
    let quote_mint = crate::read_pubkey(&dao_data, quote_mint_offset)?;
    let amm_base_vault = crate::read_pubkey(&dao_data, base_vault_offset)?;
    let amm_quote_vault = crate::read_pubkey(&dao_data, quote_vault_offset)?;

    let user_base_ata =
        crate::get_associated_token_address(user, &base_mint, &crate::TOKEN_PROGRAM_ID);
    let user_quote_ata =
        crate::get_associated_token_address(user, &quote_mint, &crate::TOKEN_PROGRAM_ID);

    let (event_authority, _) =
        Address::find_program_address(&[b"__event_authority"], &FUTARCHY_PROGRAM_ID);

    let accounts = vec![
        AccountMeta::new_readonly(FUTARCHY_PROGRAM_ID, false),
        AccountMeta::new(*dao_pubkey, false),
        AccountMeta::new(user_base_ata, false),
        AccountMeta::new(user_quote_ata, false),
        AccountMeta::new(amm_base_vault, false),
        AccountMeta::new(amm_quote_vault, false),
        AccountMeta::new_readonly(*user, true),
        AccountMeta::new_readonly(crate::TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(event_authority, false),
        AccountMeta::new_readonly(FUTARCHY_PROGRAM_ID, false),
    ];

    Ok((accounts, vec![swap_type]))
}
