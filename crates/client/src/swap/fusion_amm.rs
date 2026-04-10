#[cfg(feature = "resolve")]
use {
    crate::{
        discover_pool_with_flip, get_associated_token_address, get_token_program_for_mint,
        read_i32_le, read_pubkey, read_u16_le,
    },
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    crate::{ClientError, MEMO_PROGRAM_ID},
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const FUSION_AMM_PROGRAM_ID: Address = address!("fUSioN9YKKSa3CUC2YUc4tPkHJ5Y6XW1yz8y6F7qWz9");

// FusionPool account data layout
// Layout: [8 discriminator] [1 bump] [2 version] [32 token_mint_a] [32 token_mint_b] [32 token_vault_a]
// [32 token_vault_b] [2 tick_spacing] [2 tick_spacing_seed] [2 fee_rate] [2 protocol_fee_rate]
// [4 unused_0] [16 liquidity] [16 sqrt_price] [4 tick_current_index] ...
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_MINT_A: usize = 11;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_MINT_B: usize = 43;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_VAULT_A: usize = 75;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_VAULT_B: usize = 107;
#[cfg(feature = "resolve")]
const OFFSET_TICK_SPACING: usize = 139;
#[cfg(feature = "resolve")]
const OFFSET_TICK_CURRENT_INDEX: usize = 183;

const TICK_ARRAY_SIZE: i32 = 88;

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum AccountsType {
    TransferHookA,
    TransferHookB,
    TransferHookInput,
    TransferHookIntermediate,
    TransferHookOutput,
    SupplementalTickArrays,
    SupplementalTickArraysOne,
    SupplementalTickArraysTwo,
}

#[derive(Clone, Copy)]
pub struct RemainingAccountsSlice {
    pub accounts_type: AccountsType,
    pub length: u8,
}

#[derive(Clone)]
pub struct RemainingAccountsInfo {
    pub slices: Vec<RemainingAccountsSlice>,
}

fn initializable_start_tick(tick_index: i32, tick_spacing: u16) -> i32 {
    let ticks_in_array = TICK_ARRAY_SIZE * i32::from(tick_spacing);
    tick_index.div_euclid(ticks_in_array) * ticks_in_array
}

fn tick_array_pda(fusion_pool: &Address, start_tick_index: i32) -> Address {
    Address::find_program_address(
        &[
            b"tick_array",
            fusion_pool.as_ref(),
            &start_tick_index.to_le_bytes(),
        ],
        &FUSION_AMM_PROGRAM_ID,
    )
    .0
}

/// Pre-resolved addresses for building a Fusion AMM swap instruction offline.
pub struct FusionAmmSwapInput {
    pub token_program_a: Address,
    pub token_program_b: Address,
    pub token_authority: Address,
    pub fusion_pool: Address,
    pub token_mint_a: Address,
    pub token_mint_b: Address,
    pub token_owner_account_a: Address,
    pub token_vault_a: Address,
    pub token_owner_account_b: Address,
    pub token_vault_b: Address,
    pub tick_array_0: Address,
    pub tick_array_1: Address,
    pub tick_array_2: Address,
}

/// Build Fusion AMM swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &FusionAmmSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(FUSION_AMM_PROGRAM_ID, false),
        AccountMeta::new_readonly(input.token_program_a, false),
        AccountMeta::new_readonly(input.token_program_b, false),
        AccountMeta::new_readonly(MEMO_PROGRAM_ID, false),
        AccountMeta::new(input.token_authority, true),
        AccountMeta::new(input.fusion_pool, false),
        AccountMeta::new_readonly(input.token_mint_a, false),
        AccountMeta::new_readonly(input.token_mint_b, false),
        AccountMeta::new(input.token_owner_account_a, false),
        AccountMeta::new(input.token_owner_account_b, false),
        AccountMeta::new(input.token_vault_a, false),
        AccountMeta::new(input.token_vault_b, false),
        AccountMeta::new(input.tick_array_0, false),
        AccountMeta::new(input.tick_array_1, false),
        AccountMeta::new(input.tick_array_2, false),
    ]
}

/// Build Fusion AMM extra data: [sqrt_price_limit, amount_specified_is_input, a_to_b, remaining_accounts_info].
pub fn build_extra_data(
    sqrt_price_limit: u128,
    amount_specified_is_input: bool,
    a_to_b: bool,
    remaining_accounts_info: Option<RemainingAccountsInfo>,
) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&sqrt_price_limit.to_le_bytes());
    data.push(amount_specified_is_input as u8);
    data.push(a_to_b as u8);

    match remaining_accounts_info {
        None => data.push(0),
        Some(info) => {
            data.push(1);
            let n: u32 = info
                .slices
                .len()
                .try_into()
                .expect("remaining_accounts_info.slices.len() must fit u32 (Borsh Vec)");
            data.extend_from_slice(&n.to_le_bytes());
            for s in &info.slices {
                data.push(s.accounts_type as u8);
                data.push(s.length);
            }
        }
    }

    data
}

#[cfg(feature = "resolve")]
#[allow(clippy::too_many_arguments)]
pub async fn resolve(
    rpc: &RpcClient,
    fusion_pool: Option<&Address>,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
    sqrt_price_limit: u128,
    amount_specified_is_input: bool,
    a_to_b: bool,
    remaining_accounts_info: Option<RemainingAccountsInfo>,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let (pool_pubkey, pool_data) = match fusion_pool {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = discover_pool_with_flip(
                rpc,
                &FUSION_AMM_PROGRAM_ID,
                OFFSET_TOKEN_MINT_A,
                OFFSET_TOKEN_MINT_B,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let token_mint_a = read_pubkey(&pool_data, OFFSET_TOKEN_MINT_A)?;
    let token_mint_b = read_pubkey(&pool_data, OFFSET_TOKEN_MINT_B)?;

    // `mint_a` / `mint_b` are input / output; `a_to_b` is fusion pool direction (A->B or B->A).
    let mints_ok = if a_to_b {
        *mint_a == token_mint_a && *mint_b == token_mint_b
    } else {
        *mint_a == token_mint_b && *mint_b == token_mint_a
    };
    if !mints_ok {
        let (expected_in, expected_out) = if a_to_b {
            (token_mint_a, token_mint_b)
        } else {
            (token_mint_b, token_mint_a)
        };
        return Err(ClientError::MintMismatch {
            expected: format!(
                "a_to_b={}: input {} output {} (pool A {} B {})",
                a_to_b, expected_in, expected_out, token_mint_a, token_mint_b
            ),
            got: format!("{} / {}", mint_a, mint_b),
        });
    }

    let token_vault_a = read_pubkey(&pool_data, OFFSET_TOKEN_VAULT_A)?;
    let token_vault_b = read_pubkey(&pool_data, OFFSET_TOKEN_VAULT_B)?;

    let tick_spacing = read_u16_le(&pool_data, OFFSET_TICK_SPACING)?;
    let tick_current = read_i32_le(&pool_data, OFFSET_TICK_CURRENT_INDEX)?;

    let step = TICK_ARRAY_SIZE * i32::from(tick_spacing);
    let s0 = initializable_start_tick(tick_current, tick_spacing);
    let (s1, s2) = if a_to_b {
        (s0 - step, s0 - 2 * step)
    } else {
        (s0 + step, s0 + 2 * step)
    };

    let tick_array_0 = tick_array_pda(&pool_pubkey, s0);
    let tick_array_1 = tick_array_pda(&pool_pubkey, s1);
    let tick_array_2 = tick_array_pda(&pool_pubkey, s2);

    let token_program_a = get_token_program_for_mint(rpc, &token_mint_a).await?;
    let token_program_b = get_token_program_for_mint(rpc, &token_mint_b).await?;

    let token_owner_a = get_associated_token_address(user, &token_mint_a, &token_program_a);
    let token_owner_b = get_associated_token_address(user, &token_mint_b, &token_program_b);

    let input = FusionAmmSwapInput {
        token_program_a,
        token_program_b,
        token_authority: *user,
        fusion_pool: pool_pubkey,
        token_mint_a,
        token_mint_b,
        token_owner_account_a: token_owner_a,
        token_vault_a,
        token_owner_account_b: token_owner_b,
        token_vault_b,
        tick_array_0,
        tick_array_1,
        tick_array_2,
    };

    Ok((
        build_accounts(&input),
        build_extra_data(
            sqrt_price_limit,
            amount_specified_is_input,
            a_to_b,
            remaining_accounts_info,
        ),
    ))
}
