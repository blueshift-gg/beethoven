use {solana_instruction::AccountMeta, solana_pubkey::Pubkey};

pub const METEORA_DLMM_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");

const MEMO_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");

#[cfg(feature = "resolve")]
const OFFSET_ACTIVE_ID: usize = 76;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_X_MINT: usize = 96;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_Y_MINT: usize = 128;
#[cfg(feature = "resolve")]
const OFFSET_RESERVE_X: usize = 160;
#[cfg(feature = "resolve")]
const OFFSET_RESERVE_Y: usize = 192;
#[cfg(feature = "resolve")]
const OFFSET_ORACLE: usize = 560;
#[cfg(feature = "resolve")]
const OFFSET_BIN_ARRAY_BITMAP: usize = 592;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_MINT_X_PROGRAM_FLAG: usize = 888;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_MINT_Y_PROGRAM_FLAG: usize = 889;

#[cfg(feature = "resolve")]
const OFFSET_POSITIVE_BITMAPS: usize = 40;
#[cfg(feature = "resolve")]
const OFFSET_NEGATIVE_BITMAPS: usize = 808;

#[cfg(feature = "resolve")]
const MAX_BIN_PER_ARRAY: i32 = 70;
#[cfg(feature = "resolve")]
const INTERNAL_BITMAP_MIN: i32 = -512;
#[cfg(feature = "resolve")]
const INTERNAL_BITMAP_MAX: i32 = 511;
#[cfg(feature = "resolve")]
const EXTERNAL_BITMAP_MIN: i32 = -6656;
#[cfg(feature = "resolve")]
const EXTERNAL_BITMAP_MAX: i32 = 6655;
#[cfg(feature = "resolve")]
const INTERNAL_BITMAP_WORDS: usize = 16;
#[cfg(feature = "resolve")]
const EXT_BITMAP_SEGMENTS: usize = 12;
#[cfg(feature = "resolve")]
const EXT_BITMAP_WORDS: usize = 8;
#[cfg(feature = "resolve")]
const MAX_BIN_ARRAY_ACCOUNTS: usize = 5;

pub struct MeteoraDlmmSwapInput {
    pub lb_pair: Pubkey,
    pub bin_array_bitmap_extension: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub user_token_in: Pubkey,
    pub user_token_out: Pubkey,
    pub token_x_mint: Pubkey,
    pub token_y_mint: Pubkey,
    pub oracle: Pubkey,
    pub host_fee_in: Pubkey,
    pub user: Pubkey,
    pub token_x_program: Pubkey,
    pub token_y_program: Pubkey,
    pub event_authority: Pubkey,
    pub bin_array_accounts: Vec<Pubkey>,
}

pub fn build_accounts(input: &MeteoraDlmmSwapInput) -> Vec<AccountMeta> {
    let mut accounts = vec![
        AccountMeta::new_readonly(METEORA_DLMM_PROGRAM_ID, false),
        AccountMeta::new(input.lb_pair, false),
        AccountMeta::new_readonly(input.bin_array_bitmap_extension, false),
        AccountMeta::new(input.reserve_x, false),
        AccountMeta::new(input.reserve_y, false),
        AccountMeta::new(input.user_token_in, false),
        AccountMeta::new(input.user_token_out, false),
        AccountMeta::new_readonly(input.token_x_mint, false),
        AccountMeta::new_readonly(input.token_y_mint, false),
        AccountMeta::new(input.oracle, false),
        AccountMeta::new(input.host_fee_in, false),
        AccountMeta::new_readonly(input.user, true),
        AccountMeta::new_readonly(input.token_x_program, false),
        AccountMeta::new_readonly(input.token_y_program, false),
        AccountMeta::new_readonly(MEMO_PROGRAM_ID, false),
        AccountMeta::new_readonly(input.event_authority, false),
        AccountMeta::new_readonly(METEORA_DLMM_PROGRAM_ID, false),
    ];

    accounts.extend(
        input
            .bin_array_accounts
            .iter()
            .map(|pubkey| AccountMeta::new(*pubkey, false)),
    );

    accounts
}

pub fn build_extra_data() -> Vec<u8> {
    vec![]
}

#[cfg(feature = "resolve")]
#[derive(Clone, Debug)]
struct LbPairState {
    active_id: i32,
    token_x_mint: Pubkey,
    token_y_mint: Pubkey,
    reserve_x: Pubkey,
    reserve_y: Pubkey,
    oracle: Pubkey,
    bin_array_bitmap: [u64; INTERNAL_BITMAP_WORDS],
    token_mint_x_program_flag: u8,
    token_mint_y_program_flag: u8,
}

#[cfg(feature = "resolve")]
#[derive(Clone, Debug)]
struct BitmapExtensionState {
    positive_bitmaps: [[u64; EXT_BITMAP_WORDS]; EXT_BITMAP_SEGMENTS],
    negative_bitmaps: [[u64; EXT_BITMAP_WORDS]; EXT_BITMAP_SEGMENTS],
}

#[cfg(feature = "resolve")]
impl LbPairState {
    fn parse(data: &[u8]) -> Result<Self, crate::error::ClientError> {
        Ok(Self {
            active_id: read_i32(data, OFFSET_ACTIVE_ID)?,
            token_x_mint: crate::read_pubkey(data, OFFSET_TOKEN_X_MINT)?,
            token_y_mint: crate::read_pubkey(data, OFFSET_TOKEN_Y_MINT)?,
            reserve_x: crate::read_pubkey(data, OFFSET_RESERVE_X)?,
            reserve_y: crate::read_pubkey(data, OFFSET_RESERVE_Y)?,
            oracle: crate::read_pubkey(data, OFFSET_ORACLE)?,
            bin_array_bitmap: read_u64_words::<INTERNAL_BITMAP_WORDS>(
                data,
                OFFSET_BIN_ARRAY_BITMAP,
            )?,
            token_mint_x_program_flag: read_u8(data, OFFSET_TOKEN_MINT_X_PROGRAM_FLAG)?,
            token_mint_y_program_flag: read_u8(data, OFFSET_TOKEN_MINT_Y_PROGRAM_FLAG)?,
        })
    }

    fn infer_swap_direction(
        &self,
        mint_a: &Pubkey,
        mint_b: &Pubkey,
    ) -> Result<bool, crate::error::ClientError> {
        if *mint_a == self.token_x_mint && *mint_b == self.token_y_mint {
            Ok(true)
        } else if *mint_a == self.token_y_mint && *mint_b == self.token_x_mint {
            Ok(false)
        } else {
            Err(crate::error::ClientError::MintMismatch {
                expected: format!("{} / {}", self.token_x_mint, self.token_y_mint),
                got: format!("{} / {}", mint_a, mint_b),
            })
        }
    }
}

#[cfg(feature = "resolve")]
impl BitmapExtensionState {
    fn parse(data: &[u8]) -> Result<Self, crate::error::ClientError> {
        Ok(Self {
            positive_bitmaps: read_bitmap_matrix(data, OFFSET_POSITIVE_BITMAPS)?,
            negative_bitmaps: read_bitmap_matrix(data, OFFSET_NEGATIVE_BITMAPS)?,
        })
    }
}

#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &solana_rpc_client::nonblocking::rpc_client::RpcClient,
    lb_pair: &Pubkey,
    mint_a: &Pubkey,
    mint_b: &Pubkey,
    user: &Pubkey,
) -> Result<(Vec<AccountMeta>, Vec<u8>), crate::error::ClientError> {
    let lb_pair_account = rpc.get_account(lb_pair).await?;
    if lb_pair_account.owner != METEORA_DLMM_PROGRAM_ID {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "LB pair {} is not owned by Meteora DLMM",
            lb_pair
        )));
    }

    let lb_pair_state = LbPairState::parse(&lb_pair_account.data)?;
    let swap_for_y = lb_pair_state.infer_swap_direction(mint_a, mint_b)?;

    let token_x_program = token_program_from_flag(lb_pair_state.token_mint_x_program_flag)?;
    let token_y_program = token_program_from_flag(lb_pair_state.token_mint_y_program_flag)?;

    let bitmap_extension = derive_bin_array_bitmap_extension(*lb_pair);
    let mut fetched_accounts = rpc
        .get_multiple_accounts(&[
            lb_pair_state.token_x_mint,
            lb_pair_state.token_y_mint,
            bitmap_extension,
        ])
        .await?;

    let bitmap_extension_state = fetched_accounts
        .pop()
        .ok_or_else(|| {
            crate::error::ClientError::InvalidAccountData(
                "Missing bitmap extension fetch result".to_string(),
            )
        })?
        .map(|account| {
            if account.owner != METEORA_DLMM_PROGRAM_ID {
                return Err(crate::error::ClientError::InvalidAccountData(format!(
                    "Bitmap extension {} is not owned by Meteora DLMM",
                    bitmap_extension
                )));
            }

            BitmapExtensionState::parse(&account.data).map(Some)
        })
        .transpose()?
        .flatten();

    let token_y_mint_account = fetched_accounts
        .pop()
        .ok_or_else(|| {
            crate::error::ClientError::InvalidAccountData(
                "Missing token Y mint fetch result".to_string(),
            )
        })?
        .ok_or_else(|| {
            crate::error::ClientError::AccountNotFound(lb_pair_state.token_y_mint.to_string())
        })?;
    let token_x_mint_account = fetched_accounts
        .pop()
        .ok_or_else(|| {
            crate::error::ClientError::InvalidAccountData(
                "Missing token X mint fetch result".to_string(),
            )
        })?
        .ok_or_else(|| {
            crate::error::ClientError::AccountNotFound(lb_pair_state.token_x_mint.to_string())
        })?;

    validate_transfer_hook_support(
        &lb_pair_state.token_x_mint,
        &token_x_program,
        &token_x_mint_account.owner,
        &token_x_mint_account.data,
    )?;
    validate_transfer_hook_support(
        &lb_pair_state.token_y_mint,
        &token_y_program,
        &token_y_mint_account.owner,
        &token_y_mint_account.data,
    )?;

    let (user_token_in, user_token_out) = if swap_for_y {
        (
            crate::get_associated_token_address(
                user,
                &lb_pair_state.token_x_mint,
                &token_x_program,
            ),
            crate::get_associated_token_address(
                user,
                &lb_pair_state.token_y_mint,
                &token_y_program,
            ),
        )
    } else {
        (
            crate::get_associated_token_address(
                user,
                &lb_pair_state.token_y_mint,
                &token_y_program,
            ),
            crate::get_associated_token_address(
                user,
                &lb_pair_state.token_x_mint,
                &token_x_program,
            ),
        )
    };

    let bin_array_accounts = resolve_bin_array_accounts(
        *lb_pair,
        lb_pair_state.active_id,
        &lb_pair_state.bin_array_bitmap,
        bitmap_extension_state.as_ref(),
        swap_for_y,
        MAX_BIN_ARRAY_ACCOUNTS,
    );

    if bin_array_accounts.is_empty() {
        return Err(crate::error::ClientError::InvalidAccountData(
            "Meteora DLMM has no bin arrays with liquidity for this direction".to_string(),
        ));
    }

    let input = MeteoraDlmmSwapInput {
        lb_pair: *lb_pair,
        bin_array_bitmap_extension: bitmap_extension_state
            .as_ref()
            .map(|_| bitmap_extension)
            .unwrap_or(METEORA_DLMM_PROGRAM_ID),
        reserve_x: lb_pair_state.reserve_x,
        reserve_y: lb_pair_state.reserve_y,
        user_token_in,
        user_token_out,
        token_x_mint: lb_pair_state.token_x_mint,
        token_y_mint: lb_pair_state.token_y_mint,
        oracle: lb_pair_state.oracle,
        host_fee_in: METEORA_DLMM_PROGRAM_ID,
        user: *user,
        token_x_program,
        token_y_program,
        event_authority: derive_event_authority(),
        bin_array_accounts,
    };

    Ok((build_accounts(&input), build_extra_data()))
}

#[cfg(feature = "resolve")]
fn validate_transfer_hook_support(
    mint: &Pubkey,
    token_program: &Pubkey,
    owner: &Pubkey,
    data: &[u8],
) -> Result<(), crate::error::ClientError> {
    use spl_token_2022_interface::{
        extension::{transfer_hook, StateWithExtensions},
        state::Mint,
    };

    if *token_program != crate::TOKEN_2022_PROGRAM_ID {
        return Ok(());
    }

    if *owner != crate::TOKEN_2022_PROGRAM_ID {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "Mint {} is flagged as Token-2022 but owned by {}",
            mint, owner
        )));
    }

    let mint_state = StateWithExtensions::<Mint>::unpack(data).map_err(|err| {
        crate::error::ClientError::InvalidAccountData(format!(
            "Failed to parse Token-2022 mint {}: {}",
            mint, err
        ))
    })?;

    if transfer_hook::get_program_id(&mint_state).is_some() {
        return Err(crate::error::ClientError::InvalidAccountData(
            "Meteora DLMM transfer hooks are not supported".to_string(),
        ));
    }

    Ok(())
}

#[cfg(feature = "resolve")]
fn token_program_from_flag(flag: u8) -> Result<Pubkey, crate::error::ClientError> {
    match flag {
        0 => Ok(crate::TOKEN_PROGRAM_ID),
        1 => Ok(crate::TOKEN_2022_PROGRAM_ID),
        _ => Err(crate::error::ClientError::InvalidAccountData(format!(
            "Invalid Meteora DLMM token program flag: {}",
            flag
        ))),
    }
}

#[cfg(feature = "resolve")]
fn read_i32(data: &[u8], offset: usize) -> Result<i32, crate::error::ClientError> {
    if data.len() < offset + 4 {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "Account data too short: {} bytes, need at least {}",
            data.len(),
            offset + 4
        )));
    }

    Ok(i32::from_le_bytes(
        data[offset..offset + 4].try_into().unwrap(),
    ))
}

#[cfg(feature = "resolve")]
fn read_u8(data: &[u8], offset: usize) -> Result<u8, crate::error::ClientError> {
    data.get(offset).copied().ok_or_else(|| {
        crate::error::ClientError::InvalidAccountData(format!(
            "Account data too short: {} bytes, need at least {}",
            data.len(),
            offset + 1
        ))
    })
}

#[cfg(feature = "resolve")]
fn read_u64_words<const N: usize>(
    data: &[u8],
    offset: usize,
) -> Result<[u64; N], crate::error::ClientError> {
    let mut words = [0u64; N];
    let byte_len = N * core::mem::size_of::<u64>();

    if data.len() < offset + byte_len {
        return Err(crate::error::ClientError::InvalidAccountData(format!(
            "Account data too short: {} bytes, need at least {}",
            data.len(),
            offset + byte_len
        )));
    }

    for (index, word) in words.iter_mut().enumerate() {
        let start = offset + index * 8;
        *word = u64::from_le_bytes(data[start..start + 8].try_into().unwrap());
    }

    Ok(words)
}

#[cfg(feature = "resolve")]
fn read_bitmap_matrix(
    data: &[u8],
    offset: usize,
) -> Result<[[u64; EXT_BITMAP_WORDS]; EXT_BITMAP_SEGMENTS], crate::error::ClientError> {
    let mut matrix = [[0u64; EXT_BITMAP_WORDS]; EXT_BITMAP_SEGMENTS];

    for (segment_index, segment) in matrix.iter_mut().enumerate() {
        *segment = read_u64_words::<EXT_BITMAP_WORDS>(
            data,
            offset + segment_index * EXT_BITMAP_WORDS * 8,
        )?;
    }

    Ok(matrix)
}

#[cfg(feature = "resolve")]
fn bin_id_to_bin_array_index(bin_id: i32) -> i32 {
    let idx = bin_id / MAX_BIN_PER_ARRAY;
    let rem = bin_id % MAX_BIN_PER_ARRAY;

    if bin_id.is_negative() && rem != 0 {
        idx - 1
    } else {
        idx
    }
}

#[cfg(feature = "resolve")]
fn bit_is_set(bitmap: &[u64], bit_index: usize) -> bool {
    let word_index = bit_index / 64;
    let bit_offset = bit_index % 64;
    bitmap
        .get(word_index)
        .map(|word| (word & (1u64 << bit_offset)) != 0)
        .unwrap_or(false)
}

#[cfg(feature = "resolve")]
fn internal_bitmap_bit_is_set(bitmap: &[u64; INTERNAL_BITMAP_WORDS], bin_array_index: i32) -> bool {
    if !(INTERNAL_BITMAP_MIN..=INTERNAL_BITMAP_MAX).contains(&bin_array_index) {
        return false;
    }

    let offset = (bin_array_index - INTERNAL_BITMAP_MIN) as usize;
    bit_is_set(bitmap, offset)
}

#[cfg(feature = "resolve")]
fn external_bitmap_bit_is_set(bitmap: &BitmapExtensionState, bin_array_index: i32) -> bool {
    if !(EXTERNAL_BITMAP_MIN..=EXTERNAL_BITMAP_MAX).contains(&bin_array_index)
        || (INTERNAL_BITMAP_MIN..=INTERNAL_BITMAP_MAX).contains(&bin_array_index)
    {
        return false;
    }

    if bin_array_index > 0 {
        let bitmap_offset = (bin_array_index / 512 - 1) as usize;
        let bit_offset = (bin_array_index % 512) as usize;
        bitmap
            .positive_bitmaps
            .get(bitmap_offset)
            .map(|segment| bit_is_set(segment, bit_offset))
            .unwrap_or(false)
    } else {
        let bitmap_offset = ((-(bin_array_index + 1)) / 512 - 1) as usize;
        let bit_offset = ((-(bin_array_index + 1)) % 512) as usize;
        bitmap
            .negative_bitmaps
            .get(bitmap_offset)
            .map(|segment| bit_is_set(segment, bit_offset))
            .unwrap_or(false)
    }
}

#[cfg(feature = "resolve")]
fn derive_bin_array_bitmap_extension(lb_pair: Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[b"bin_array_bitmap", lb_pair.as_ref()],
        &METEORA_DLMM_PROGRAM_ID,
    )
    .0
}

#[cfg(feature = "resolve")]
fn derive_bin_array(lb_pair: Pubkey, bin_array_index: i32) -> Pubkey {
    Pubkey::find_program_address(
        &[
            b"bin_array",
            lb_pair.as_ref(),
            &i64::from(bin_array_index).to_le_bytes(),
        ],
        &METEORA_DLMM_PROGRAM_ID,
    )
    .0
}

#[cfg(feature = "resolve")]
fn derive_event_authority() -> Pubkey {
    Pubkey::find_program_address(&[b"__event_authority"], &METEORA_DLMM_PROGRAM_ID).0
}

#[cfg(feature = "resolve")]
fn resolve_bin_array_accounts(
    lb_pair: Pubkey,
    active_id: i32,
    internal_bitmap: &[u64; INTERNAL_BITMAP_WORDS],
    bitmap_extension: Option<&BitmapExtensionState>,
    swap_for_y: bool,
    take_count: usize,
) -> Vec<Pubkey> {
    let mut bin_array_accounts = Vec::with_capacity(take_count);
    let mut index = bin_id_to_bin_array_index(active_id);
    let min_index = if bitmap_extension.is_some() {
        EXTERNAL_BITMAP_MIN
    } else {
        INTERNAL_BITMAP_MIN
    };
    let max_index = if bitmap_extension.is_some() {
        EXTERNAL_BITMAP_MAX
    } else {
        INTERNAL_BITMAP_MAX
    };

    while index >= min_index && index <= max_index && bin_array_accounts.len() < take_count {
        let has_liquidity = if (INTERNAL_BITMAP_MIN..=INTERNAL_BITMAP_MAX).contains(&index) {
            internal_bitmap_bit_is_set(internal_bitmap, index)
        } else {
            bitmap_extension
                .map(|bitmap| external_bitmap_bit_is_set(bitmap, index))
                .unwrap_or(false)
        };

        if has_liquidity {
            bin_array_accounts.push(derive_bin_array(lb_pair, index));
        }

        if swap_for_y {
            if index == min_index {
                break;
            }
            index -= 1;
        } else {
            if index == max_index {
                break;
            }
            index += 1;
        }
    }

    bin_array_accounts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bin_id_to_bin_array_index_handles_negative_rounding() {
        assert_eq!(bin_id_to_bin_array_index(0), 0);
        assert_eq!(bin_id_to_bin_array_index(69), 0);
        assert_eq!(bin_id_to_bin_array_index(70), 1);
        assert_eq!(bin_id_to_bin_array_index(-1), -1);
        assert_eq!(bin_id_to_bin_array_index(-70), -1);
        assert_eq!(bin_id_to_bin_array_index(-71), -2);
    }

    #[test]
    fn test_lb_pair_parse_reads_current_offsets() {
        let mut data = vec![0u8; OFFSET_TOKEN_MINT_Y_PROGRAM_FLAG + 1];
        data[OFFSET_ACTIVE_ID..OFFSET_ACTIVE_ID + 4].copy_from_slice(&123i32.to_le_bytes());
        data[OFFSET_TOKEN_X_MINT..OFFSET_TOKEN_X_MINT + 32].copy_from_slice(&[1u8; 32]);
        data[OFFSET_TOKEN_Y_MINT..OFFSET_TOKEN_Y_MINT + 32].copy_from_slice(&[2u8; 32]);
        data[OFFSET_RESERVE_X..OFFSET_RESERVE_X + 32].copy_from_slice(&[3u8; 32]);
        data[OFFSET_RESERVE_Y..OFFSET_RESERVE_Y + 32].copy_from_slice(&[4u8; 32]);
        data[OFFSET_ORACLE..OFFSET_ORACLE + 32].copy_from_slice(&[5u8; 32]);
        data[OFFSET_BIN_ARRAY_BITMAP..OFFSET_BIN_ARRAY_BITMAP + 8]
            .copy_from_slice(&7u64.to_le_bytes());
        data[OFFSET_TOKEN_MINT_X_PROGRAM_FLAG] = 1;
        data[OFFSET_TOKEN_MINT_Y_PROGRAM_FLAG] = 0;

        let state = LbPairState::parse(&data).unwrap();

        assert_eq!(state.active_id, 123);
        assert_eq!(state.token_x_mint, Pubkey::new_from_array([1u8; 32]));
        assert_eq!(state.token_y_mint, Pubkey::new_from_array([2u8; 32]));
        assert_eq!(state.reserve_x, Pubkey::new_from_array([3u8; 32]));
        assert_eq!(state.reserve_y, Pubkey::new_from_array([4u8; 32]));
        assert_eq!(state.oracle, Pubkey::new_from_array([5u8; 32]));
        assert_eq!(state.bin_array_bitmap[0], 7);
        assert_eq!(state.token_mint_x_program_flag, 1);
        assert_eq!(state.token_mint_y_program_flag, 0);
    }

    #[test]
    fn test_resolve_bin_array_accounts_crosses_internal_and_external_ranges() {
        let lb_pair = Pubkey::new_unique();
        let mut internal_bitmap = [0u64; INTERNAL_BITMAP_WORDS];
        let mut bitmap_extension = BitmapExtensionState {
            positive_bitmaps: [[0u64; EXT_BITMAP_WORDS]; EXT_BITMAP_SEGMENTS],
            negative_bitmaps: [[0u64; EXT_BITMAP_WORDS]; EXT_BITMAP_SEGMENTS],
        };

        let internal_offset = (511 - INTERNAL_BITMAP_MIN) as usize;
        internal_bitmap[internal_offset / 64] |= 1u64 << (internal_offset % 64);

        let external_positive_offset = 520usize - 512;
        bitmap_extension.positive_bitmaps[0][external_positive_offset / 64] |=
            1u64 << (external_positive_offset % 64);

        let accounts = resolve_bin_array_accounts(
            lb_pair,
            510 * MAX_BIN_PER_ARRAY,
            &internal_bitmap,
            Some(&bitmap_extension),
            false,
            2,
        );

        assert_eq!(accounts.len(), 2);
        assert_eq!(accounts[0], derive_bin_array(lb_pair, 511));
        assert_eq!(accounts[1], derive_bin_array(lb_pair, 520));
    }
}
