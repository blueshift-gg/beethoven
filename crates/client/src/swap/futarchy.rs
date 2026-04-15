use solana_address::Address;
#[cfg(feature = "resolve")]
use {
    crate::{
        error::ClientError, get_associated_token_address, get_token_program_for_mint, read_pubkey,
        TOKEN_PROGRAM_ID,
    },
    solana_instruction::AccountMeta,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

pub const FUTARCHY_PROGRAM_ID: Address =
    Address::from_str_const("FUTARELBfJfQ8RDGhg1wdhddq1odMAJUePHFuBYfUxKq");

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum SwapType {
    Buy = 0,
    Sell = 1,
}

pub fn build_extra_data(swap_type: SwapType) -> Vec<u8> {
    vec![swap_type as u8]
}

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
    rpc: &RpcClient,
    dao: Option<&Address>,
    swap_type: &SwapType,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
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

    let base_mint = read_pubkey(&dao_data, base_mint_offset)?;
    let quote_mint = read_pubkey(&dao_data, quote_mint_offset)?;
    let amm_base_vault = read_pubkey(&dao_data, base_vault_offset)?;
    let amm_quote_vault = read_pubkey(&dao_data, quote_vault_offset)?;

    let pair_matches = (*mint_a == base_mint && *mint_b == quote_mint)
        || (*mint_a == quote_mint && *mint_b == base_mint);
    if !pair_matches {
        return Err(ClientError::MintMismatch {
            expected: format!("{}/{}", base_mint, quote_mint),
            got: format!("{}/{}", mint_a, mint_b),
        });
    }

    let base_token_program = get_token_program_for_mint(rpc, &base_mint).await?;
    let quote_token_program = get_token_program_for_mint(rpc, &quote_mint).await?;
    if base_token_program != TOKEN_PROGRAM_ID || quote_token_program != TOKEN_PROGRAM_ID {
        return Err(ClientError::InvalidAccountData(
            "Futarchy mints must be owned by the SPL Token program".to_string(),
        ));
    }

    let user_base_ata = get_associated_token_address(user, &base_mint, &TOKEN_PROGRAM_ID);
    let user_quote_ata = get_associated_token_address(user, &quote_mint, &TOKEN_PROGRAM_ID);

    let (event_authority, _) =
        Address::find_program_address(&[b"__event_authority"], &FUTARCHY_PROGRAM_ID);

    let accounts = vec![
        AccountMeta::new_readonly(FUTARCHY_PROGRAM_ID, false),
        AccountMeta::new(*dao_pubkey, false),
        AccountMeta::new(user_base_ata, false),
        AccountMeta::new(user_quote_ata, false),
        AccountMeta::new(amm_base_vault, false),
        AccountMeta::new(amm_quote_vault, false),
        AccountMeta::new(*user, true),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(event_authority, false),
        AccountMeta::new_readonly(FUTARCHY_PROGRAM_ID, false),
    ];

    Ok((accounts, build_extra_data(*swap_type)))
}
