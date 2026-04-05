#[cfg(feature = "resolve")]
use crate::{
    discover_pool_with_flip, get_associated_token_address, get_token_program_for_mint, read_pubkey,
    ClientError,
};
use {
    crate::MEMO_PROGRAM_ID,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const SAROS_DLMM_PROGRAM_ID: Address = address!("1qbkdrr3z4ryLA7pZykqxvxWPoeifcVKo6ZG9CfkvVE");
pub const EVENT_AUTHORITY: Address = address!("AQjz6RZK93SLjxfDGKL9nCYQNSjEbQSdETxwR63jXV8m");
pub const SAROS_MDMA_HOOKS_PROGRAM_ID: Address =
    address!("mdmavMvJpF4ZcLJNg6VSjuKVMiBo5uKwERTg1ZB9yUH");

// Pair account layout offsets (8-byte Anchor discriminator; fields match Saros Pair IDL)
// Layout: [8 discriminator] [1 bump] [32 liquidity_book_config] [1 bin_step] [1 bin_step_seed]
//         [32 token_mint_x] [32 token_mint_y] [20 static_fee_parameters] [4 active_id] [24 dynamic_fee_parameters] [8 protocolFeesX] [8 protocolFeesY] [1 option] [32 hook]
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_MINT_X: usize = 43;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_MINT_Y: usize = 75;
#[cfg(feature = "resolve")]
const OFFSET_ACTIVE_ID: usize = 127;
#[cfg(feature = "resolve")]
const OFFSET_HOOK_OPTION: usize = 171;
// https://github.com/saros-xyz/saros-dlmm-sdk-rs/blob/main/saros-sdk/src/state/bin.rs#L17
#[cfg(feature = "resolve")]
const BIN_ARRAY_SIZE: u32 = 256;

#[cfg(feature = "resolve")]
fn read_u32_le(data: &[u8], offset: usize) -> Result<u32, ClientError> {
    if data.len() < offset + 4 {
        return Err(ClientError::InvalidAccountData(format!(
            "Account data too short for u32 at offset {}",
            offset
        )));
    }
    let bytes: [u8; 4] = data[offset..offset + 4].try_into().expect("length checked");
    Ok(u32::from_le_bytes(bytes))
}

#[cfg(feature = "resolve")]
fn read_optional_hook_pubkey(data: &[u8], offset: usize) -> Result<Option<Address>, ClientError> {
    if data.len() < offset + 1 {
        return Err(ClientError::InvalidAccountData(
            "Account data too short for hook option".into(),
        ));
    }
    match data[offset] {
        0 => Ok(None),
        1 => Ok(Some(read_pubkey(data, offset + 1)?)),
        b => Err(ClientError::InvalidAccountData(format!(
            "Invalid Option tag for hook: {}",
            b
        ))),
    }
}

// https://github.com/saros-xyz/saros-dlmm-sdk-rs/blob/main/saros-sdk/src/state/pair.rs#L163
#[cfg(feature = "resolve")]
fn bin_array_index(active_id: u32) -> u32 {
    let mut bin_array_index = (active_id / BIN_ARRAY_SIZE) as i32;
    if active_id % BIN_ARRAY_SIZE < BIN_ARRAY_SIZE / 2 {
        bin_array_index -= 1;
    }
    bin_array_index as u32
}

// https://github.com/saros-xyz/saros-dlmm-sdk-rs/blob/5fcea41728dc52b76e6b4589ddd540e4a30e952f/saros-sdk/src/utils/helper.rs#L51
#[cfg(feature = "resolve")]
fn derive_bin_array_pda(index: u32, pair: &Address) -> Address {
    let (addr, _) = Address::find_program_address(
        &[b"bin_array", pair.as_ref(), &index.to_le_bytes()],
        &SAROS_DLMM_PROGRAM_ID,
    );
    addr
}

// https://github.com/saros-xyz/saros-dlmm-sdk-rs/blob/5fcea41728dc52b76e6b4589ddd540e4a30e952f/saros-sdk/src/utils/helper.rs#L46
#[cfg(feature = "resolve")]
fn get_pair_bin_array(bin_array_index: u32, pair: &Address) -> (Address, Address) {
    let lower = derive_bin_array_pda(bin_array_index, pair);
    let upper = derive_bin_array_pda(bin_array_index + 1, pair);
    (lower, upper)
}

// https://github.com/saros-xyz/saros-dlmm-sdk-rs/blob/5fcea41728dc52b76e6b4589ddd540e4a30e952f/saros-sdk/src/utils/helper.rs#L72
#[cfg(feature = "resolve")]
fn get_swap_hook_bin_array(hook: &Address, index: u32) -> Address {
    let (addr, _) = Address::find_program_address(
        &[b"bin_array", hook.as_ref(), &index.to_le_bytes()],
        &SAROS_MDMA_HOOKS_PROGRAM_ID,
    );
    addr
}

pub enum SwapType {
    ExactInput,
    ExactOutput,
}

impl TryFrom<u8> for SwapType {
    type Error = ClientError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SwapType::ExactInput),
            1 => Ok(SwapType::ExactOutput),
            _ => Err(ClientError::InvalidAccountData(format!(
                "Invalid swap type: {value}"
            ))),
        }
    }
}

/// Pre-resolved addresses for building a Saros DLMM swap instruction offline.
pub struct SarosDlmmSwapInput {
    pub pair: Address,
    pub token_mint_x: Address,
    pub token_mint_y: Address,
    pub bin_array_lower: Address,
    pub bin_array_upper: Address,
    pub token_vault_x: Address,
    pub token_vault_y: Address,
    pub user_vault_x: Address,
    pub user_vault_y: Address,
    pub user: Address,
    pub token_program_x: Address,
    pub token_program_y: Address,
    pub hook: Address,
    pub active_hook_bin_arrays: Option<(Address, Address)>,
}

/// Build Saros DLMM swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &SarosDlmmSwapInput) -> Vec<AccountMeta> {
    let mut metas = vec![
        AccountMeta::new_readonly(SAROS_DLMM_PROGRAM_ID, false),
        AccountMeta::new(input.pair, false),
        AccountMeta::new_readonly(input.token_mint_x, false),
        AccountMeta::new_readonly(input.token_mint_y, false),
        AccountMeta::new(input.bin_array_lower, false),
        AccountMeta::new(input.bin_array_upper, false),
        AccountMeta::new(input.token_vault_x, false),
        AccountMeta::new(input.token_vault_y, false),
        AccountMeta::new(input.user_vault_x, false),
        AccountMeta::new(input.user_vault_y, false),
        AccountMeta::new(input.user, true),
        AccountMeta::new_readonly(input.token_program_x, false),
        AccountMeta::new_readonly(input.token_program_y, false),
        AccountMeta::new_readonly(MEMO_PROGRAM_ID, false),
        AccountMeta::new(input.hook, false),
        AccountMeta::new_readonly(SAROS_MDMA_HOOKS_PROGRAM_ID, false),
        AccountMeta::new_readonly(EVENT_AUTHORITY, false),
        AccountMeta::new_readonly(SAROS_DLMM_PROGRAM_ID, false),
    ];
    if let Some((active_hook_lower, active_hook_upper)) = input.active_hook_bin_arrays {
        metas.push(AccountMeta::new(active_hook_lower, false));
        metas.push(AccountMeta::new(active_hook_upper, false));
    }
    metas
}

/// Build Saros DLMM extra data: [swap_for_y, swap_type].
pub fn build_extra_data(swap_for_y: bool, swap_type: SwapType) -> Vec<u8> {
    vec![swap_for_y as u8, swap_type as u8]
}

#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &solana_rpc_client::nonblocking::rpc_client::RpcClient,
    pair: Option<&Address>,
    swap_for_y: bool,
    swap_type: u8,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let (pair_pubkey, pair_data) = match pair {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = discover_pool_with_flip(
                rpc,
                &SAROS_DLMM_PROGRAM_ID,
                OFFSET_TOKEN_MINT_X,
                OFFSET_TOKEN_MINT_Y,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let token_mint_x = read_pubkey(&pair_data, OFFSET_TOKEN_MINT_X)?;
    let token_mint_y = read_pubkey(&pair_data, OFFSET_TOKEN_MINT_Y)?;

    if *mint_a != token_mint_x && *mint_a != token_mint_y {
        return Err(ClientError::MintMismatch {
            expected: format!("{} or {}", token_mint_x, token_mint_y),
            got: mint_a.to_string(),
        });
    }

    let active_id = read_u32_le(&pair_data, OFFSET_ACTIVE_ID)?;
    let bin_array_index = bin_array_index(active_id);
    let (bin_array_lower, bin_array_upper) = get_pair_bin_array(bin_array_index, &pair_pubkey);

    let token_program_x = get_token_program_for_mint(rpc, &token_mint_x).await?;
    let token_program_y = get_token_program_for_mint(rpc, &token_mint_y).await?;

    let token_vault_x = get_associated_token_address(&pair_pubkey, &token_mint_x, &token_program_x);
    let token_vault_y = get_associated_token_address(&pair_pubkey, &token_mint_y, &token_program_y);

    let user_vault_x = get_associated_token_address(user, &token_mint_x, &token_program_x);
    let user_vault_y = get_associated_token_address(user, &token_mint_y, &token_program_y);

    let hook = read_optional_hook_pubkey(&pair_data, OFFSET_HOOK_OPTION)?.unwrap_or_default();

    let active_hook_bin_arrays = if hook != Address::default() {
        let idx_upper = bin_array_index.checked_add(1).ok_or_else(|| {
            ClientError::InvalidAccountData("bin_array_index overflow for hook bin".into())
        })?;
        Some((
            get_swap_hook_bin_array(&hook, bin_array_index),
            get_swap_hook_bin_array(&hook, idx_upper),
        ))
    } else {
        None
    };

    let input = SarosDlmmSwapInput {
        pair: pair_pubkey,
        token_mint_x,
        token_mint_y,
        bin_array_lower,
        bin_array_upper,
        token_vault_x,
        token_vault_y,
        user_vault_x,
        user_vault_y,
        user: *user,
        token_program_x,
        token_program_y,
        hook,
        active_hook_bin_arrays,
    };

    let swap_type = SwapType::try_from(swap_type)?;

    Ok((
        build_accounts(&input),
        build_extra_data(swap_for_y, swap_type),
    ))
}
