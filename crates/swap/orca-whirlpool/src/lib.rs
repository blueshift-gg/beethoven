#![no_std]

use {
    beethoven_core::{Swap, SwapTokenAccounts},
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::{address, Address},
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const WHIRLPOOL_PROGRAM_ID: Address = address!("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc");

const SWAP_V2_DISCRIMINATOR: [u8; 8] = [43, 4, 237, 11, 26, 201, 30, 98];
// 8 - discriminator
// 8 - in_amount
// 8 - minimum_out_amount
// 16 - sqrt_price_limit
// 1 - amount_specified_is_input
// 1 - a_to_b
// 1 - remaining_accounts_info Option header
const SWAP_V2_PREFIX_LEN: usize = 43;
const VEC_HEADER_LEN: usize = 4;
const MAX_IX_DATA_LEN: usize = 512;

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum AccountsType {
    TransferHookA,
    TransferHookB,
    TransferHookReward,
    TransferHookInput,
    TransferHookIntermediate,
    TransferHookOutput,
    SupplementalTickArrays,
    SupplementalTickArraysOne,
    SupplementalTickArraysTwo,
    TransferHookDepositA,
    TransferHookDepositB,
    TransferHookWithdrawalA,
    TransferHookWithdrawalB,
}

#[derive(Clone, Copy)]
pub struct RemainingAccountsSlice {
    pub accounts_type: AccountsType,
    pub length: u8,
}

impl RemainingAccountsSlice {
    pub const DATA_LEN: usize = 2;
}

// implemented as Vec<RemainingAccountsSlice> in source code
pub struct RemainingAccountsInfo<'info> {
    pub slices: &'info [RemainingAccountsSlice],
}

pub struct OrcaWhirlpoolSwapData<'info> {
    sqrt_price_limit: u128,
    amount_specified_is_input: bool,
    a_to_b: bool,
    remaining_accounts_info: Option<RemainingAccountsInfo<'info>>,
}

impl OrcaWhirlpoolSwapData<'_> {
    // 16 - sqrt_price_limit
    // 1 - amount_specified_is_input
    // 1 - a_to_b
    // 1 - remaining_accounts_info Option header
    //
    // When the Option tag is non-zero: 4-byte LE `Vec<RemainingAccountsSlice>` length,
    // then `count * size_of::<RemainingAccountsSlice>()` bytes of slice records.
    pub const DATA_LEN: usize = 19;

    /// Returns the total encoded length of a payload starting with these bytes
    ///
    /// If remaining_accounts_info is None, returns [`Self::DATA_LEN`].
    ///
    /// If remaining_accounts_info is Some, returns [`Self::DATA_LEN`] + 4 + n * [`RemainingAccountsSlice::DATA_LEN`].
    ///
    /// Requires at least [`Self::DATA_LEN`] bytes to read the Option tag; if `Some`, reads the
    /// 4-byte vec length at offset 19.
    pub fn encoded_len(data: &[u8]) -> Result<usize, ProgramError> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        // n = 0 if remaining_accounts_info is None
        if data[Self::DATA_LEN - 1] == 0 {
            return Ok(Self::DATA_LEN);
        }

        // check if data len is enough to read the vec header
        if data.len() < Self::DATA_LEN + VEC_HEADER_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        let mut len_le = MaybeUninit::<[u8; VEC_HEADER_LEN]>::uninit();

        unsafe {
            core::ptr::copy_nonoverlapping(
                data.as_ptr().add(Self::DATA_LEN),
                len_le.as_mut_ptr().cast(),
                VEC_HEADER_LEN,
            );
        }

        let len_le = unsafe { len_le.assume_init() };
        let n = u32::from_le_bytes(len_le) as usize;

        let slice_bytes = n
            .checked_mul(RemainingAccountsSlice::DATA_LEN)
            .ok_or(ProgramError::InvalidInstructionData)?;

        Self::DATA_LEN
            .checked_add(VEC_HEADER_LEN)
            .and_then(|o| o.checked_add(slice_bytes))
            .ok_or(ProgramError::InvalidInstructionData)
            .and_then(|total| {
                // data len is not enough to read the remaining accounts info
                if data.len() < total {
                    return Err(ProgramError::InvalidInstructionData);
                }
                Ok(total)
            })
    }
}

impl<'info> TryFrom<&'info [u8]> for OrcaWhirlpoolSwapData<'info> {
    type Error = ProgramError;

    fn try_from(data: &'info [u8]) -> Result<Self, Self::Error> {
        let total = Self::encoded_len(data)?;

        if data.len() != total {
            return Err(ProgramError::InvalidInstructionData);
        }

        // remaining_accounts_info is None if the Option tag is 0
        let remaining_accounts_info = if data[Self::DATA_LEN - 1] == 0 {
            None
        } else {
            // remaining_accounts_info is Some if the Option tag is not 0
            let mut len_le = MaybeUninit::<[u8; VEC_HEADER_LEN]>::uninit();

            unsafe {
                core::ptr::copy_nonoverlapping(
                    data.as_ptr().add(Self::DATA_LEN),
                    len_le.as_mut_ptr().cast(),
                    VEC_HEADER_LEN,
                );
            }

            let len_le = unsafe { len_le.assume_init() };
            let n = u32::from_le_bytes(len_le) as usize;
            let slices_start = Self::DATA_LEN + VEC_HEADER_LEN;
            let slices_bytes = n
                .checked_mul(RemainingAccountsSlice::DATA_LEN)
                .ok_or(ProgramError::InvalidInstructionData)?;
            let raw = &data[slices_start..slices_start + slices_bytes];

            for chunk in raw.chunks_exact(RemainingAccountsSlice::DATA_LEN) {
                let tag = chunk[0];

                // check if tag is invalid AccountsType
                if tag > AccountsType::TransferHookWithdrawalB as u8 {
                    return Err(ProgramError::InvalidInstructionData);
                }
            }

            let slices = unsafe {
                core::slice::from_raw_parts(raw.as_ptr() as *const RemainingAccountsSlice, n)
            };

            Some(RemainingAccountsInfo { slices })
        };

        Ok(Self {
            sqrt_price_limit: u128::from_le_bytes(data[0..16].try_into().unwrap()),
            amount_specified_is_input: data[16] == 1,
            a_to_b: data[17] == 1,
            remaining_accounts_info,
        })
    }
}

pub struct OrcaWhirlpool;

impl OrcaWhirlpoolSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 16;
}

pub struct OrcaWhirlpoolSwapAccounts<'info> {
    pub whirlpool_program: &'info AccountView,
    pub token_program_a: &'info AccountView,
    pub token_program_b: &'info AccountView,
    pub memo_program: &'info AccountView,
    pub token_authority: &'info AccountView,
    pub whirlpool: &'info AccountView,
    pub token_mint_a: &'info AccountView,
    pub token_mint_b: &'info AccountView,
    pub token_owner_account_a: &'info AccountView,
    pub token_vault_a: &'info AccountView,
    pub token_owner_account_b: &'info AccountView,
    pub token_vault_b: &'info AccountView,
    pub tick_array_0: &'info AccountView,
    pub tick_array_1: &'info AccountView,
    pub tick_array_2: &'info AccountView,
    pub oracle: &'info AccountView,
    pub remaining_accounts: &'info [AccountView],
}

impl<'info> TryFrom<&'info [AccountView]> for OrcaWhirlpoolSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [whirlpool_program, token_program_a, token_program_b, memo_program, token_authority, whirlpool, token_mint_a, token_mint_b, token_owner_account_a, token_vault_a, token_owner_account_b, token_vault_b, tick_array_0, tick_array_1, tick_array_2, oracle, remaining_accounts @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(OrcaWhirlpoolSwapAccounts {
            whirlpool_program,
            token_program_a,
            token_program_b,
            memo_program,
            token_authority,
            whirlpool,
            token_mint_a,
            token_mint_b,
            token_owner_account_a,
            token_vault_a,
            token_owner_account_b,
            token_vault_b,
            tick_array_0,
            tick_array_1,
            tick_array_2,
            oracle,
            remaining_accounts,
        })
    }
}

impl<'info> Swap<'info> for OrcaWhirlpool {
    type Accounts = OrcaWhirlpoolSwapAccounts<'info>;
    type Data = OrcaWhirlpoolSwapData<'info>;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::readonly(ctx.token_program_a.address()),
            InstructionAccount::readonly(ctx.token_program_b.address()),
            InstructionAccount::readonly(ctx.memo_program.address()),
            InstructionAccount::readonly_signer(ctx.token_authority.address()),
            InstructionAccount::writable(ctx.whirlpool.address()),
            InstructionAccount::readonly(ctx.token_mint_a.address()),
            InstructionAccount::readonly(ctx.token_mint_b.address()),
            InstructionAccount::writable(ctx.token_owner_account_a.address()),
            InstructionAccount::writable(ctx.token_vault_a.address()),
            InstructionAccount::writable(ctx.token_owner_account_b.address()),
            InstructionAccount::writable(ctx.token_vault_b.address()),
            InstructionAccount::writable(ctx.tick_array_0.address()),
            InstructionAccount::writable(ctx.tick_array_1.address()),
            InstructionAccount::writable(ctx.tick_array_2.address()),
            InstructionAccount::writable(ctx.oracle.address()),
            InstructionAccount::readonly(ctx.whirlpool_program.address()),
        ];

        let account_infos = [
            ctx.token_program_a,
            ctx.token_program_b,
            ctx.memo_program,
            ctx.token_authority,
            ctx.whirlpool,
            ctx.token_mint_a,
            ctx.token_mint_b,
            ctx.token_owner_account_a,
            ctx.token_vault_a,
            ctx.token_owner_account_b,
            ctx.token_vault_b,
            ctx.tick_array_0,
            ctx.tick_array_1,
            ctx.tick_array_2,
            ctx.oracle,
            ctx.whirlpool_program,
        ];

        let ix_len = match &data.remaining_accounts_info {
            Some(info) => {
                let slices_len: usize = info.slices.len();
                let slice_bytes = slices_len
                    .checked_mul(RemainingAccountsSlice::DATA_LEN)
                    .ok_or(ProgramError::InvalidInstructionData)?;
                SWAP_V2_PREFIX_LEN
                    .checked_add(VEC_HEADER_LEN)
                    .and_then(|o| o.checked_add(slice_bytes))
                    .ok_or(ProgramError::InvalidInstructionData)?
            }
            None => SWAP_V2_PREFIX_LEN,
        };

        if ix_len > MAX_IX_DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        let mut instruction_data = MaybeUninit::<[u8; MAX_IX_DATA_LEN]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(SWAP_V2_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(16),
                8,
            );
            core::ptr::copy_nonoverlapping(
                data.sqrt_price_limit.to_le_bytes().as_ptr(),
                ptr.add(24),
                16,
            );
            ptr.add(40).write(data.amount_specified_is_input as u8);
            ptr.add(41).write(data.a_to_b as u8);
            ptr.add(42)
                .write(data.remaining_accounts_info.is_some() as u8);

            if let Some(remaining_accounts_info) = &data.remaining_accounts_info {
                let slices_len = remaining_accounts_info.slices.len() as u32;
                core::ptr::copy_nonoverlapping(
                    slices_len.to_le_bytes().as_ptr(),
                    ptr.add(SWAP_V2_PREFIX_LEN + 1),
                    VEC_HEADER_LEN,
                );
                let body_base = SWAP_V2_PREFIX_LEN + 1 + VEC_HEADER_LEN;
                for (i, slice) in remaining_accounts_info.slices.iter().enumerate() {
                    core::ptr::copy_nonoverlapping(
                        slice as *const RemainingAccountsSlice as *const u8,
                        ptr.add(body_base + i * RemainingAccountsSlice::DATA_LEN),
                        RemainingAccountsSlice::DATA_LEN,
                    );
                }
            }
        }

        let instruction = InstructionView {
            program_id: &WHIRLPOOL_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, ix_len)
            },
        };

        invoke_signed(&instruction, &account_infos, signer_seeds)
    }

    fn swap(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
    ) -> ProgramResult {
        Self::swap_signed(ctx, in_amount, minimum_out_amount, data, &[])
    }
}

impl<'info> SwapTokenAccounts<'info> for OrcaWhirlpool {
    type Accounts = OrcaWhirlpoolSwapAccounts<'info>;
    type Data = OrcaWhirlpoolSwapData<'info>;

    fn token_accounts(
        ctx: &Self::Accounts,
        data: &Self::Data,
    ) -> (&'info AccountView, &'info AccountView) {
        if data.a_to_b {
            (ctx.token_owner_account_a, ctx.token_owner_account_b)
        } else {
            (ctx.token_owner_account_b, ctx.token_owner_account_a)
        }
    }
}
