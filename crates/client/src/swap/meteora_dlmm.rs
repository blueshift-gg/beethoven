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
    num_bigint::BigUint,
    num_traits::{ToPrimitive, Zero},
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
const BITMAP_TYPE_U1024_BITS: usize = 1024;

#[cfg(feature = "resolve")]
const DEFAULT_BIN_ARRAY_SWAP_COUNT: usize = 4;

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
fn bin_id_to_bin_array_index(bin_id: i64) -> i64 {
    let div = bin_id / MAX_BINS_PER_ARRAY;
    let rem = bin_id % MAX_BINS_PER_ARRAY;
    if bin_id < 0 && rem != 0 {
        div - 1
    } else {
        div
    }
}

#[cfg(feature = "resolve")]
fn get_bin_array_lower_upper_bin_id(bin_array_index: i64) -> (i64, i64) {
    let lower = bin_array_index * MAX_BINS_PER_ARRAY;
    let upper = lower + MAX_BINS_PER_ARRAY - 1;
    (lower, upper)
}

#[cfg(feature = "resolve")]
fn is_overflow_default_bin_array_bitmap(bin_array_index: i64) -> bool {
    !(-BIN_ARRAY_BITMAP_SIZE..=BIN_ARRAY_BITMAP_SIZE - 1).contains(&bin_array_index)
}

#[cfg(feature = "resolve")]
fn extension_bitmap_range() -> (i64, i64) {
    let factor = i64::from(EXTENSION_BIN_ARRAY_BITMAP_SIZE) + 1;
    let min = -BIN_ARRAY_BITMAP_SIZE * factor;
    let max = BIN_ARRAY_BITMAP_SIZE * factor - 1;
    (min, max)
}

#[cfg(feature = "resolve")]
fn u512_from_u64x8_le(chunks: &[u64; 8]) -> BigUint {
    let mut v = BigUint::zero();
    for (i, &w) in chunks.iter().enumerate() {
        v += BigUint::from(w) << (i * 64);
    }
    v
}

#[cfg(feature = "resolve")]
fn u1024_from_u64x16_le(chunks: &[u64; 16]) -> BigUint {
    let mut v = BigUint::zero();
    for (i, &w) in chunks.iter().enumerate() {
        v += BigUint::from(w) << (i * 64);
    }
    v
}

#[cfg(feature = "resolve")]
fn bit_u1024(b: &BigUint, i: usize) -> bool {
    if i >= BITMAP_TYPE_U1024_BITS {
        return false;
    }
    (b >> i) & BigUint::from(1u32) != BigUint::zero()
}

#[cfg(feature = "resolve")]
fn most_significant_bit(number: &BigUint, bit_length: usize) -> Option<u32> {
    if number.is_zero() {
        return None;
    }
    let highest_index = bit_length - 1;
    for i in (0..bit_length).rev() {
        if bit_u1024(number, i) {
            return Some((highest_index - i) as u32);
        }
    }
    None
}

#[cfg(feature = "resolve")]
fn least_significant_bit(number: &BigUint, bit_length: usize) -> Option<u32> {
    if number.is_zero() {
        return None;
    }
    for i in 0..bit_length {
        if bit_u1024(number, i) {
            return Some(i as u32);
        }
    }
    None
}

#[cfg(feature = "resolve")]
fn get_bin_array_offset(bin_array_index: i64) -> usize {
    let m = BIN_ARRAY_BITMAP_SIZE as u64;
    if bin_array_index > 0 {
        (bin_array_index as u64 % m) as usize
    } else {
        let t = (-(bin_array_index as i128 + 1)) as u64;
        (t % m) as usize
    }
}

#[cfg(feature = "resolve")]
fn get_bitmap_offset(bin_array_index: i64) -> isize {
    if bin_array_index > 0 {
        (bin_array_index as i128 / 512 - 1) as isize
    } else {
        let t = -(bin_array_index as i128 + 1);
        (t / 512 - 1) as isize
    }
}

#[cfg(feature = "resolve")]
fn ext_bitmap_row_index(bin_array_index: i64) -> usize {
    get_bitmap_offset(bin_array_index).clamp(0, EXTENSION_BIN_ARRAY_BITMAP_SIZE as isize - 1)
        as usize
}

#[cfg(feature = "resolve")]
fn find_set_bit(start_index: i64, end_index: i64, ext: &BitmapExtension) -> Option<i64> {
    if start_index <= end_index {
        let mut i = start_index;
        while i <= end_index {
            let bin_array_offset = get_bin_array_offset(i);
            let row = ext_bitmap_row_index(i);
            let chunks = if i > 0 {
                &ext.positive[row]
            } else {
                &ext.negative[row]
            };
            let bitmap = u512_from_u64x8_le(chunks);
            if (bitmap >> bin_array_offset) & BigUint::from(1u32) != BigUint::zero() {
                return Some(i);
            }
            i += 1;
        }
    } else {
        let mut i = start_index;
        while i >= end_index {
            let bin_array_offset = get_bin_array_offset(i);
            let row = ext_bitmap_row_index(i);
            let chunks = if i > 0 {
                &ext.positive[row]
            } else {
                &ext.negative[row]
            };
            let bitmap = u512_from_u64x8_le(chunks);
            if (bitmap >> bin_array_offset) & BigUint::from(1u32) != BigUint::zero() {
                return Some(i);
            }
            i -= 1;
        }
    }
    None
}

#[cfg(feature = "resolve")]
fn find_next_bin_array_index_with_liquidity(
    swap_for_y: bool,
    active_bin_id: i64,
    bin_array_bitmap: &[u64; 16],
    bin_array_bitmap_extension: Option<&BitmapExtension>,
) -> Option<i64> {
    let lower_internal = -BIN_ARRAY_BITMAP_SIZE;
    let upper_internal = BIN_ARRAY_BITMAP_SIZE - 1;
    let mut start_bin_array_index = bin_id_to_bin_array_index(active_bin_id);

    loop {
        if is_overflow_default_bin_array_bitmap(start_bin_array_index) {
            let ext = bin_array_bitmap_extension?;
            let (min_bin_array_index, max_bin_array_index) = extension_bitmap_range();

            if start_bin_array_index < 0 {
                if swap_for_y {
                    return find_set_bit(start_bin_array_index, min_bin_array_index, ext);
                }
                if let Some(i) =
                    find_set_bit(start_bin_array_index, -BIN_ARRAY_BITMAP_SIZE - 1, ext)
                {
                    return Some(i);
                }
                start_bin_array_index = -BIN_ARRAY_BITMAP_SIZE;
            } else if swap_for_y {
                if let Some(i) = find_set_bit(start_bin_array_index, BIN_ARRAY_BITMAP_SIZE, ext) {
                    return Some(i);
                }
                start_bin_array_index = BIN_ARRAY_BITMAP_SIZE - 1;
            } else {
                return find_set_bit(start_bin_array_index, max_bin_array_index, ext);
            }
        } else {
            let bitmap = u1024_from_u64x16_le(bin_array_bitmap);
            let offset = (start_bin_array_index as i128) + (BIN_ARRAY_BITMAP_SIZE as i128);
            let offset_u = offset.to_usize()?;

            if swap_for_y {
                let upper_bit_range = BITMAP_TYPE_U1024_BITS - 1 - offset_u;
                let cropped_bitmap = bitmap << upper_bit_range;
                if let Some(msb) = most_significant_bit(&cropped_bitmap, BITMAP_TYPE_U1024_BITS) {
                    return Some(start_bin_array_index - msb as i64);
                }
                start_bin_array_index = lower_internal - 1;
            } else {
                let lower_bit_range = offset_u;
                let cropped_bitmap = bitmap >> lower_bit_range;
                if let Some(lsb) = least_significant_bit(&cropped_bitmap, BITMAP_TYPE_U1024_BITS) {
                    return Some(start_bin_array_index + lsb as i64);
                }
                start_bin_array_index = upper_internal + 1;
            }
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
    let mut bin_array_pubkeys: Vec<Address> = Vec::new();
    let mut active_id_to_loop = active_id as i64;

    while bin_array_pubkeys.len() < count {
        let Some(bin_array_index) = find_next_bin_array_index_with_liquidity(
            swap_for_y,
            active_id_to_loop,
            bin_array_bitmap,
            extension,
        ) else {
            break;
        };
        let pda = derive_bin_array_pda(lb_pair, bin_array_index);
        if bin_array_pubkeys.contains(&pda) {
            break;
        }
        bin_array_pubkeys.push(pda);
        let (lower_bin_id, upper_bin_id) = get_bin_array_lower_upper_bin_id(bin_array_index);
        active_id_to_loop = if swap_for_y {
            lower_bin_id - 1
        } else {
            upper_bin_id + 1
        };
    }
    bin_array_pubkeys
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
pub async fn resolve(
    rpc: &RpcClient,
    lb_pair: Option<&Address>,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
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
        DEFAULT_BIN_ARRAY_SWAP_COUNT,
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
