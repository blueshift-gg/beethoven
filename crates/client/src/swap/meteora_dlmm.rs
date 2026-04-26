use {
    crate::MEMO_PROGRAM_ID,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};
#[cfg(feature = "resolve")]
use {
    crate::{
        discover_pool_with_flip, get_associated_token_address, get_token_program_for_mint,
        read_i32, read_pubkey, read_u64, ClientError,
    },
    ruint::aliases::{U1024, U512},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    spl_transfer_hook_interface::solana_pubkey::Pubkey,
};

pub const METEORA_DLMM_PROGRAM_ID: Address =
    address!("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");
pub const EVENT_AUTHORITY: Address = address!("D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6");

pub enum AccountsType {
    TransferHookX,
    TransferHookY,
}

// LbPair account layout offsets
// Layout: ... [4 active_id] @ 68 ... [32 token_x_mint] @ 88 [32 token_y_mint] [32 reserve_x] [32 reserve_y] ... [32 oracle] @ 552 [128 bin_array_bitmap]
#[cfg(feature = "resolve")]
const OFFSET_ACTIVE_ID: usize = 68;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_X_MINT: usize = 88;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_Y_MINT: usize = 120;
#[cfg(feature = "resolve")]
const OFFSET_RESERVE_X: usize = 152;
#[cfg(feature = "resolve")]
const OFFSET_RESERVE_Y: usize = 184;
#[cfg(feature = "resolve")]
const OFFSET_ORACLE: usize = 552;
#[cfg(feature = "resolve")]
const OFFSET_BIN_ARRAY_BITMAP: usize = 584;

// BinArrayBitmapExtension account layout offsets
// Layout: [8 discrimninator] [32 lb_pair] [768 positive_bin_array_bitmap] [768 negative_bin_array_bitmap]
#[cfg(feature = "resolve")]
const OFFSET_BIN_ARRAY_BITMAP_EXTENSION_POSITIVE_BIN_ARRAY_BITMAP: usize = 40;

// from constants section of IDL
#[cfg(feature = "resolve")]
const MAX_BINS_PER_ARRAY: i64 = 70;
#[cfg(feature = "resolve")]
const BIN_ARRAY_BITMAP_SIZE: i64 = 512;
#[cfg(feature = "resolve")]
const EXTENSION_BIN_ARRAY_BITMAP_SIZE: u8 = 12;

#[cfg(feature = "resolve")]
const DEFAULT_BIN_ARRAY_SWAP_COUNT: usize = 3;

/// Pre-resolved addresses for building an Meteora DLMM swap instruction offline.
pub struct MeteoraDlmmSwapInput {
    pub lb_pair: Address,
    pub bin_array_bitmap_extension: Option<Address>,
    pub reserve_x: Address,
    pub reserve_y: Address,
    pub user_token_in: Address,
    pub user_token_out: Address,
    pub token_x_mint: Address,
    pub token_y_mint: Address,
    pub oracle: Address,
    pub user: Address,
    pub token_x_program: Address,
    pub token_y_program: Address,
    pub transfer_hook_x_accounts: Option<Vec<AccountMeta>>,
    pub transfer_hook_y_accounts: Option<Vec<AccountMeta>>,
    pub bin_array_accounts: Option<Vec<AccountMeta>>,
}

pub fn derive_bin_array_bitmap_extension_pda(lb_pair: &Address) -> Address {
    Address::find_program_address(&[b"bitmap", lb_pair.as_ref()], &METEORA_DLMM_PROGRAM_ID).0
}

pub fn derive_bin_array_pda(lb_pair: &Address, index: i64) -> Address {
    Address::find_program_address(
        &[b"bin_array", lb_pair.as_ref(), index.to_le_bytes().as_ref()],
        &METEORA_DLMM_PROGRAM_ID,
    )
    .0
}

// --- Bin array discovery (Meteora dlmm-sdk `getBinArrayForSwap` / `findNextBinArrayIndexWithLiquidity`) ---

#[cfg(feature = "resolve")]
struct LbPairBinWalk {
    active_id: i32,
    bin_array_bitmap: [u64; 16],
}

#[cfg(feature = "resolve")]
struct BitmapExtension {
    positive: [[u64; 8]; 12],
    negative: [[u64; 8]; 12],
}

#[cfg(feature = "resolve")]
fn address_to_pubkey(a: &Address) -> Pubkey {
    Pubkey::new_from_array(a.to_bytes())
}

#[cfg(feature = "resolve")]
fn parse_lb_pair_bin_walk(data: &[u8]) -> Result<LbPairBinWalk, ClientError> {
    // offset bin_array_bitmap + 16 (u64) * 8
    const OFFSET_END_BIN_ARRAY_BITMAP: usize = OFFSET_BIN_ARRAY_BITMAP + 128;
    if data.len() < OFFSET_END_BIN_ARRAY_BITMAP {
        return Err(ClientError::InvalidAccountData(format!(
            "lb_pair data too short for bin walk: {} < {}",
            data.len(),
            OFFSET_END_BIN_ARRAY_BITMAP
        )));
    }
    let active_id = read_i32(data, OFFSET_ACTIVE_ID)?;
    let off = OFFSET_BIN_ARRAY_BITMAP;
    let mut bin_array_bitmap = [0u64; 16];
    for (i, item) in bin_array_bitmap.iter_mut().enumerate() {
        let s = off + i * 8;
        *item = read_u64(data, s)?;
    }
    Ok(LbPairBinWalk {
        active_id,
        bin_array_bitmap,
    })
}

#[cfg(feature = "resolve")]
fn parse_bin_array_bitmap_extension(data: &[u8]) -> Result<BitmapExtension, ClientError> {
    // 8 (discriminator) + 32 (Pubkey) + 12 (rows) * 8 (u64) * 8 (u64) * 2 (positive and negative)
    const OFFSET_END_BIN_ARRAY_BITMAP_EXTENSION: usize = 8 + 32 + 12 * 8 * 8 * 2;
    if data.len() < OFFSET_END_BIN_ARRAY_BITMAP_EXTENSION {
        return Err(ClientError::InvalidAccountData(
            "bin_array_bitmap_extension too short".into(),
        ));
    }
    let mut pos = OFFSET_BIN_ARRAY_BITMAP_EXTENSION_POSITIVE_BIN_ARRAY_BITMAP;
    let mut positive: [[u64; 8]; 12] = [[0u64; 8]; 12];
    for row in &mut positive {
        for w in row.iter_mut() {
            *w = read_u64(data, pos)?;
            pos += 8;
        }
    }
    let mut negative = [[0u64; 8]; 12];
    for row in &mut negative {
        for w in row.iter_mut() {
            *w = read_u64(data, pos)?;
            pos += 8;
        }
    }
    Ok(BitmapExtension { positive, negative })
}

#[cfg(feature = "resolve")]
fn bin_id_to_bin_array_index(bin_id: i32) -> i32 {
    let max_bins_per_array = MAX_BINS_PER_ARRAY as i32;
    let div = bin_id / max_bins_per_array;
    let rem = bin_id % max_bins_per_array;
    if bin_id < 0 && rem != 0 {
        div - 1
    } else {
        div
    }
}

#[cfg(feature = "resolve")]
fn is_overflow_default_bin_array_bitmap(bin_array_index: i32) -> bool {
    let bitmap_size = BIN_ARRAY_BITMAP_SIZE as i32;
    !(-bitmap_size..=bitmap_size - 1).contains(&bin_array_index)
}

#[cfg(feature = "resolve")]
fn extension_bitmap_range() -> (i32, i32) {
    let bitmap_size = BIN_ARRAY_BITMAP_SIZE as i32;
    let factor = i32::from(EXTENSION_BIN_ARRAY_BITMAP_SIZE) + 1;
    let min = -bitmap_size * factor;
    let max = bitmap_size * factor - 1;
    (min, max)
}

#[cfg(feature = "resolve")]
fn get_bin_array_offset(bin_array_index: i32) -> Option<usize> {
    let bitmap_size = BIN_ARRAY_BITMAP_SIZE as i32;
    let offset = bin_array_index.checked_add(bitmap_size)?;
    usize::try_from(offset).ok()
}

#[cfg(feature = "resolve")]
fn next_bin_array_index_with_liquidity_internal(
    swap_for_y: bool,
    start_array_index: i32,
    bin_array_bitmap: &[u64; 16],
) -> Option<(i32, bool)> {
    let bitmap_size = BIN_ARRAY_BITMAP_SIZE as i32;
    let min_bitmap_id = -bitmap_size;
    let max_bitmap_id = bitmap_size - 1;
    let array_offset = get_bin_array_offset(start_array_index)?;
    let bitmap = U1024::from_limbs(*bin_array_bitmap);

    if swap_for_y {
        let bitmap_range = usize::try_from(max_bitmap_id.checked_sub(min_bitmap_id)?).ok()?;
        let shift = bitmap_range.checked_sub(array_offset)?;
        let offset_bitmap = bitmap << shift;

        if offset_bitmap == U1024::ZERO {
            Some((min_bitmap_id.checked_sub(1)?, false))
        } else {
            let next_bit = i32::try_from(offset_bitmap.leading_zeros()).ok()?;
            Some((start_array_index.checked_sub(next_bit)?, true))
        }
    } else {
        let offset_bitmap = bitmap >> array_offset;
        if offset_bitmap == U1024::ZERO {
            Some((max_bitmap_id.checked_add(1)?, false))
        } else {
            let next_bit = i32::try_from(offset_bitmap.trailing_zeros()).ok()?;
            Some((start_array_index.checked_add(next_bit)?, true))
        }
    }
}

#[cfg(feature = "resolve")]
fn get_bitmap_offset(bin_array_index: i32) -> Option<usize> {
    let bitmap_size = BIN_ARRAY_BITMAP_SIZE as i32;
    let offset = if bin_array_index > 0 {
        bin_array_index.checked_div(bitmap_size)?.checked_sub(1)?
    } else {
        let t = bin_array_index.checked_add(1)?.checked_neg()?;
        t.checked_div(bitmap_size)?.checked_sub(1)?
    };
    usize::try_from(offset).ok()
}

#[cfg(feature = "resolve")]
fn bin_array_offset_in_bitmap(bin_array_index: i32) -> Option<usize> {
    let bitmap_size = BIN_ARRAY_BITMAP_SIZE as i32;
    if bin_array_index > 0 {
        usize::try_from(bin_array_index.checked_rem(bitmap_size)?).ok()
    } else {
        let t = bin_array_index.checked_add(1)?.checked_neg()?;
        usize::try_from(t.checked_rem(bitmap_size)?).ok()
    }
}

#[cfg(feature = "resolve")]
fn to_bin_array_index(offset: usize, bin_array_offset: usize, is_positive: bool) -> Option<i32> {
    let bitmap_size = BIN_ARRAY_BITMAP_SIZE as i32;
    let offset_i32 = i32::try_from(offset).ok()?;
    let bin_array_offset_i32 = i32::try_from(bin_array_offset).ok()?;

    if is_positive {
        Some(
            (offset_i32 + 1)
                .checked_mul(bitmap_size)?
                .checked_add(bin_array_offset_i32)?,
        )
    } else {
        Some(
            ((offset_i32 + 1)
                .checked_mul(bitmap_size)?
                .checked_add(bin_array_offset_i32)?)
            .checked_neg()?
            .checked_sub(1)?,
        )
    }
}

#[cfg(feature = "resolve")]
fn bit_in_extension(ext: &BitmapExtension, bin_array_index: i32) -> Option<bool> {
    let offset = get_bitmap_offset(bin_array_index)?;
    let bitmap = if bin_array_index < 0 {
        U512::from_limbs(ext.negative[offset])
    } else {
        U512::from_limbs(ext.positive[offset])
    };
    let bit_offset = bin_array_offset_in_bitmap(bin_array_index)?;
    Some(bitmap.bit(bit_offset))
}

#[cfg(feature = "resolve")]
fn iter_bitmap_extension(ext: &BitmapExtension, start_index: i32, end_index: i32) -> Option<i32> {
    if start_index == end_index {
        return bit_in_extension(ext, start_index)?.then_some(start_index);
    }

    let ext_rows = usize::from(EXTENSION_BIN_ARRAY_BITMAP_SIZE);
    let bitmap_size = BIN_ARRAY_BITMAP_SIZE as usize;
    let offset = get_bitmap_offset(start_index)?;
    let start_offset = bin_array_offset_in_bitmap(start_index)?;

    if start_index < 0 {
        if start_index < end_index {
            for i in (0..=offset).rev() {
                let mut bitmap = U512::from_limbs(ext.negative[i]);
                if i == offset {
                    let shift = bitmap_size.checked_sub(start_offset)?.checked_sub(1)?;
                    bitmap <<= shift;
                    if bitmap == U512::ZERO {
                        continue;
                    }
                    let offset_in_bitmap = start_offset
                        .checked_sub(bitmap.leading_zeros())?
                        .checked_sub(1)?;
                    return to_bin_array_index(i, offset_in_bitmap, false);
                }
                if bitmap == U512::ZERO {
                    continue;
                }
                let offset_in_bitmap = bitmap_size
                    .checked_sub(bitmap.leading_zeros())?
                    .checked_sub(1)?;
                return to_bin_array_index(i, offset_in_bitmap, false);
            }
        } else {
            for i in offset..ext_rows {
                let mut bitmap = U512::from_limbs(ext.negative[i]);
                if i == offset {
                    bitmap >>= start_offset;
                    if bitmap == U512::ZERO {
                        continue;
                    }
                    let offset_in_bitmap = start_offset
                        .checked_add(bitmap.trailing_zeros())?
                        .checked_sub(1)?;
                    return to_bin_array_index(i, offset_in_bitmap, false);
                }
                if bitmap == U512::ZERO {
                    continue;
                }
                return to_bin_array_index(i, bitmap.trailing_zeros(), false);
            }
        }
    } else if start_index < end_index {
        for i in offset..ext_rows {
            let mut bitmap = U512::from_limbs(ext.positive[i]);
            if i == offset {
                bitmap >>= start_offset;
                if bitmap == U512::ZERO {
                    continue;
                }
                let offset_in_bitmap = start_offset
                    .checked_add(bitmap.trailing_zeros())?
                    .checked_sub(1)?;
                return to_bin_array_index(i, offset_in_bitmap, true);
            }
            if bitmap == U512::ZERO {
                continue;
            }
            return to_bin_array_index(i, bitmap.trailing_zeros(), true);
        }
    } else {
        for i in (0..=offset).rev() {
            let mut bitmap = U512::from_limbs(ext.positive[i]);
            if i == offset {
                let shift = bitmap_size.checked_sub(start_offset)?.checked_sub(1)?;
                bitmap <<= shift;
                if bitmap == U512::ZERO {
                    continue;
                }
                let offset_in_bitmap = start_offset
                    .checked_sub(bitmap.leading_zeros())?
                    .checked_sub(1)?;
                return to_bin_array_index(i, offset_in_bitmap, true);
            }
            if bitmap == U512::ZERO {
                continue;
            }
            let offset_in_bitmap = bitmap_size
                .checked_sub(bitmap.leading_zeros())?
                .checked_sub(1)?;
            return to_bin_array_index(i, offset_in_bitmap, true);
        }
    }
    None
}

#[cfg(feature = "resolve")]
fn next_bin_array_index_with_liquidity_extension(
    ext: &BitmapExtension,
    swap_for_y: bool,
    start_index: i32,
) -> Option<(i32, bool)> {
    let bitmap_size = BIN_ARRAY_BITMAP_SIZE as i32;
    let (min_bitmap_id, max_bitmap_id) = extension_bitmap_range();

    if start_index > 0 {
        if swap_for_y {
            match iter_bitmap_extension(ext, start_index, bitmap_size) {
                Some(value) => Some((value, true)),
                None => Some((bitmap_size - 1, false)),
            }
        } else {
            iter_bitmap_extension(ext, start_index, max_bitmap_id).map(|value| (value, true))
        }
    } else if swap_for_y {
        iter_bitmap_extension(ext, start_index, min_bitmap_id).map(|value| (value, true))
    } else {
        match iter_bitmap_extension(ext, start_index, -bitmap_size - 1) {
            Some(value) => Some((value, true)),
            None => Some((-bitmap_size, false)),
        }
    }
}

#[cfg(feature = "resolve")]
fn collect_bin_array_pubkeys_for_swap(
    lb_pair: &Address,
    swap_for_y: bool,
    active_id: i32,
    bin_array_bitmap: &[u64; 16],
    extension: Option<&BitmapExtension>,
    count: usize,
) -> Vec<Address> {
    let mut start_bin_array_idx = bin_id_to_bin_array_index(active_id);
    let mut bin_array_indexes = Vec::with_capacity(count);
    let increment: i32 = if swap_for_y { -1 } else { 1 };

    loop {
        if bin_array_indexes.len() == count {
            break;
        }

        if is_overflow_default_bin_array_bitmap(start_bin_array_idx) {
            let Some(ext) = extension else {
                break;
            };
            let Some((next_bin_array_idx, has_liquidity)) =
                next_bin_array_index_with_liquidity_extension(ext, swap_for_y, start_bin_array_idx)
            else {
                break;
            };

            if has_liquidity {
                bin_array_indexes.push(next_bin_array_idx);
                let Some(next_start_idx) = next_bin_array_idx.checked_add(increment) else {
                    break;
                };
                start_bin_array_idx = next_start_idx;
            } else {
                start_bin_array_idx = next_bin_array_idx;
            }
        } else {
            let Some((next_bin_array_idx, has_liquidity)) =
                next_bin_array_index_with_liquidity_internal(
                    swap_for_y,
                    start_bin_array_idx,
                    bin_array_bitmap,
                )
            else {
                break;
            };

            if has_liquidity {
                bin_array_indexes.push(next_bin_array_idx);
                let Some(next_start_idx) = next_bin_array_idx.checked_add(increment) else {
                    break;
                };
                start_bin_array_idx = next_start_idx;
            } else {
                start_bin_array_idx = next_bin_array_idx;
            }
        }
    }

    bin_array_indexes
        .into_iter()
        .map(|idx| derive_bin_array_pda(lb_pair, i64::from(idx)))
        .collect()
}

#[cfg(feature = "resolve")]
async fn fetch_bin_array_metas(
    rpc: &RpcClient,
    addresses: &[Address],
) -> Result<Vec<AccountMeta>, ClientError> {
    let pks: Vec<Pubkey> = addresses.iter().map(address_to_pubkey).collect();
    let accounts = rpc
        .get_multiple_accounts(&pks)
        .await
        .map_err(|e| ClientError::Rpc(e.to_string()))?;
    let mut account_metas = Vec::with_capacity(pks.len());
    for (pk, acc) in pks.iter().zip(accounts.iter()) {
        if acc.is_none() {
            return Err(ClientError::AccountNotFound(format!("bin_array {}", pk)));
        }
        account_metas.push(AccountMeta::new(*pk, false));
    }
    Ok(account_metas)
}

/// Build Meteora DLMM extra data (`swap2`'s `remaining_accounts_info`).
///
/// Wire format is Borsh: `Vec<RemainingAccountsSlice>`.
/// - empty vec => `[0,0,0,0]`
/// - otherwise Liquidity uses 2 slices: TransferHookX and TransferHookY, each with a `length` (u8)
pub fn build_extra_data(
    transfer_hook_x_accounts: &Option<Vec<AccountMeta>>,
    transfer_hook_y_accounts: &Option<Vec<AccountMeta>>,
) -> Vec<u8> {
    let x_len = transfer_hook_x_accounts
        .as_ref()
        .map(|v| v.len())
        .unwrap_or(0);
    let y_len = transfer_hook_y_accounts
        .as_ref()
        .map(|v| v.len())
        .unwrap_or(0);

    if x_len == 0 && y_len == 0 {
        return vec![0, 0, 0, 0];
    }

    let x_len_u8 = u8::try_from(x_len).unwrap_or(u8::MAX);
    let y_len_u8 = u8::try_from(y_len).unwrap_or(u8::MAX);

    let mut slices_len: u32 = 0;
    if x_len != 0 {
        slices_len += 1;
    }
    if y_len != 0 {
        slices_len += 1;
    }

    let mut data = Vec::with_capacity(4 + (slices_len as usize) * 2);
    data.extend_from_slice(&slices_len.to_le_bytes());

    if x_len != 0 {
        data.push(AccountsType::TransferHookX as u8);
        data.push(x_len_u8);
    }
    if y_len != 0 {
        data.push(AccountsType::TransferHookY as u8);
        data.push(y_len_u8);
    }

    data
}

/// Build Meteora DLMM swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &MeteoraDlmmSwapInput) -> Vec<AccountMeta> {
    let mut meta = vec![
        AccountMeta::new_readonly(METEORA_DLMM_PROGRAM_ID, false),
        AccountMeta::new(input.lb_pair, false),
        // pass program itself if there's no bin array bitmap extension
        AccountMeta::new_readonly(
            input
                .bin_array_bitmap_extension
                .unwrap_or(METEORA_DLMM_PROGRAM_ID),
            false,
        ),
        AccountMeta::new(input.reserve_x, false),
        AccountMeta::new(input.reserve_y, false),
        AccountMeta::new(input.user_token_in, false),
        AccountMeta::new(input.user_token_out, false),
        AccountMeta::new_readonly(input.token_x_mint, false),
        AccountMeta::new_readonly(input.token_y_mint, false),
        AccountMeta::new(input.oracle, false),
        AccountMeta::new_readonly(METEORA_DLMM_PROGRAM_ID, false),
        AccountMeta::new(input.user, true),
        AccountMeta::new_readonly(input.token_x_program, false),
        AccountMeta::new_readonly(input.token_y_program, false),
        AccountMeta::new_readonly(MEMO_PROGRAM_ID, false),
        AccountMeta::new_readonly(EVENT_AUTHORITY, false),
        AccountMeta::new_readonly(METEORA_DLMM_PROGRAM_ID, false),
    ];

    if let Some(transfer_hook_x_accounts) = &input.transfer_hook_x_accounts {
        meta.extend_from_slice(transfer_hook_x_accounts);
    }
    if let Some(transfer_hook_y_accounts) = &input.transfer_hook_y_accounts {
        meta.extend_from_slice(transfer_hook_y_accounts);
    }

    if let Some(bin_array_accounts) = &input.bin_array_accounts {
        meta.extend_from_slice(bin_array_accounts);
    }

    meta
}

/// Resolve accounts and data for an Meteora DLMM swap via RPC.
///
/// Requires transfer hook accounts to be explicitly passed in as arguments.
#[cfg(feature = "resolve")]
#[allow(clippy::too_many_arguments)]
pub async fn resolve(
    rpc: &RpcClient,
    lb_pair: Option<&Address>,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
    bin_array_count: Option<u8>,
    transfer_hook_x_accounts: Option<Vec<AccountMeta>>,
    transfer_hook_y_accounts: Option<Vec<AccountMeta>>,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let (pair_pubkey, pair_data) = match lb_pair {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = discover_pool_with_flip(
                rpc,
                &METEORA_DLMM_PROGRAM_ID,
                OFFSET_TOKEN_X_MINT,
                OFFSET_TOKEN_Y_MINT,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let token_x_mint = read_pubkey(&pair_data, OFFSET_TOKEN_X_MINT)?;
    let token_y_mint = read_pubkey(&pair_data, OFFSET_TOKEN_Y_MINT)?;

    let reserve_x = read_pubkey(&pair_data, OFFSET_RESERVE_X)?;
    let reserve_y = read_pubkey(&pair_data, OFFSET_RESERVE_Y)?;
    let oracle = read_pubkey(&pair_data, OFFSET_ORACLE)?;

    let token_x_program = get_token_program_for_mint(rpc, &token_x_mint).await?;
    let token_y_program = get_token_program_for_mint(rpc, &token_y_mint).await?;

    let user_token_x_account = get_associated_token_address(user, &token_x_mint, &token_x_program);
    let user_token_y_account = get_associated_token_address(user, &token_y_mint, &token_y_program);

    // determine user token account order based on mint order
    let (user_token_in, user_token_out) = if *mint_a == token_x_mint && *mint_b == token_y_mint {
        (user_token_x_account, user_token_y_account)
    } else if *mint_a == token_y_mint && *mint_b == token_x_mint {
        (user_token_y_account, user_token_x_account)
    } else {
        return Err(crate::error::ClientError::MintMismatch {
            expected: format!(
                "({}, {}) or ({}, {})",
                token_x_mint, token_y_mint, token_y_mint, token_x_mint
            ),
            got: format!("({}, {})", mint_a, mint_b),
        });
    };

    let bin_array_bitmap_extension_pda = derive_bin_array_bitmap_extension_pda(&pair_pubkey);

    let bin_array_bitmap_extension = match rpc.get_account(&bin_array_bitmap_extension_pda).await {
        Ok(_) => Some(bin_array_bitmap_extension_pda),
        Err(_) => None,
    };

    let bin_array_bitmap_extension_data = if let Some(ref ext_addr) = bin_array_bitmap_extension {
        rpc.get_account(ext_addr).await.ok().map(|a| a.data)
    } else {
        None
    };

    let swap_for_y = *mint_a == token_x_mint && *mint_b == token_y_mint;

    let data = build_extra_data(&transfer_hook_x_accounts, &transfer_hook_y_accounts);

    let lb_pair_bin_walk = parse_lb_pair_bin_walk(&pair_data)?;
    let bitmap_extension = bin_array_bitmap_extension_data
        .as_ref()
        .and_then(|d| parse_bin_array_bitmap_extension(d).ok());
    let bin_array_pubkeys = collect_bin_array_pubkeys_for_swap(
        &pair_pubkey,
        swap_for_y,
        lb_pair_bin_walk.active_id,
        &lb_pair_bin_walk.bin_array_bitmap,
        bitmap_extension.as_ref(),
        bin_array_count
            .map(|c| c as usize)
            .unwrap_or(DEFAULT_BIN_ARRAY_SWAP_COUNT),
    );
    let bin_array_metas = fetch_bin_array_metas(rpc, &bin_array_pubkeys).await?;

    let input = MeteoraDlmmSwapInput {
        lb_pair: pair_pubkey,
        bin_array_bitmap_extension,
        reserve_x,
        reserve_y,
        user_token_in,
        user_token_out,
        token_x_mint: *mint_a,
        token_y_mint: *mint_b,
        oracle,
        user: *user,
        token_x_program,
        token_y_program,
        transfer_hook_x_accounts,
        transfer_hook_y_accounts,
        bin_array_accounts: Some(bin_array_metas),
    };

    Ok((build_accounts(&input), data))
}
