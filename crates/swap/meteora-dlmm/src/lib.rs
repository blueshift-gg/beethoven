#![no_std]

use {
    beethoven_core::Swap,
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::{address, Address},
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const METEORA_DLMM_PROGRAM_ID: Address =
    address!("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");

const SWAP2_DISCRIMINATOR: [u8; 8] = [65, 75, 63, 76, 235, 91, 91, 136];

// loose limit without blowing the stack, based on average bin array accounts passed in mainnet transactions
const MAX_ACCOUNTS: usize = 32;

// 8 - discriminator
// 8 - in amount
// 8 - minimum out amount
// 8 - remaining accounts info length (4 - vec header, max 4 from slices)
const MAX_IX_DATA: usize = 32;

pub struct MeteoraDlmm;

pub struct MeteoraDlmmSwapAccounts<'info> {
    pub meteora_dlmm_program: &'info AccountView,
    pub lb_pair: &'info AccountView,
    pub bin_array_bitmap_extension: &'info AccountView,
    pub reserve_x: &'info AccountView,
    pub reserve_y: &'info AccountView,
    pub user_token_in: &'info AccountView,
    pub user_token_out: &'info AccountView,
    pub token_x_mint: &'info AccountView,
    pub token_y_mint: &'info AccountView,
    pub oracle: &'info AccountView,
    pub host_fee_in: &'info AccountView,
    pub user: &'info AccountView,
    pub token_x_program: &'info AccountView,
    pub token_y_program: &'info AccountView,
    pub memo_program: &'info AccountView,
    pub event_authority: &'info AccountView,
    pub program: &'info AccountView,
    pub remaining_accounts: &'info [AccountView],
}

/// Borsh-encoded third argument to `swap2`: `RemainingAccountsInfo` (`{ slices }` in Anchor TS).
///
/// Instruction layout: `swap2(amount_in, min_amount_out, remaining_accounts_info)` per IDL;
/// the TS client builds this from `getPotentialToken2022IxDataAndAccounts(ActionType.Liquidity)`.
pub struct MeteoraDlmmSwapData<'a> {
    pub remaining_accounts_info: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for MeteoraDlmmSwapData<'a> {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(MeteoraDlmmSwapData {
            remaining_accounts_info: data,
        })
    }
}

impl<'info> TryFrom<&'info [AccountView]> for MeteoraDlmmSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [meteora_dlmm_program, lb_pair, bin_array_bitmap_extension, reserve_x, reserve_y, user_token_in, user_token_out, token_x_mint, token_y_mint, oracle, host_fee_in, user, token_x_program, token_y_program, memo_program, event_authority, program, remaining_accounts @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(MeteoraDlmmSwapAccounts {
            meteora_dlmm_program,
            lb_pair,
            bin_array_bitmap_extension,
            reserve_x,
            reserve_y,
            user_token_in,
            user_token_out,
            token_x_mint,
            token_y_mint,
            oracle,
            host_fee_in,
            user,
            token_x_program,
            token_y_program,
            memo_program,
            event_authority,
            program,
            remaining_accounts,
        })
    }
}

impl<'info> Swap<'info> for MeteoraDlmm {
    type Accounts = MeteoraDlmmSwapAccounts<'info>;
    type Data = MeteoraDlmmSwapData<'info>;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let tail = data.remaining_accounts_info;

        // 8 (discriminator) + 8 (in amount) + 8 (minimum out amount) + tail.len()
        let data_len = 24 + tail.len();

        if data_len > MAX_IX_DATA {
            return Err(ProgramError::InvalidInstructionData);
        }

        let total_accounts = 16 + ctx.remaining_accounts.len();

        let mut instruction_data = MaybeUninit::<[u8; MAX_IX_DATA]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(SWAP2_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(16),
                8,
            );
            core::ptr::copy_nonoverlapping(tail.as_ptr(), ptr.add(24), tail.len());
        }

        let mut instruction_accounts = MaybeUninit::<[InstructionAccount; MAX_ACCOUNTS]>::uninit();
        let instruction_accounts_ptr = instruction_accounts.as_mut_ptr() as *mut InstructionAccount;

        unsafe {
            core::ptr::write(
                instruction_accounts_ptr,
                InstructionAccount::writable(ctx.lb_pair.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(1),
                InstructionAccount::readonly(ctx.bin_array_bitmap_extension.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(2),
                InstructionAccount::writable(ctx.reserve_x.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(3),
                InstructionAccount::writable(ctx.reserve_y.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(4),
                InstructionAccount::writable(ctx.user_token_in.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(5),
                InstructionAccount::writable(ctx.user_token_out.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(6),
                InstructionAccount::readonly(ctx.token_x_mint.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(7),
                InstructionAccount::readonly(ctx.token_y_mint.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(8),
                InstructionAccount::writable(ctx.oracle.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(9),
                if ctx.host_fee_in.address().eq(&METEORA_DLMM_PROGRAM_ID) {
                    InstructionAccount::readonly(ctx.host_fee_in.address())
                } else {
                    InstructionAccount::writable(ctx.host_fee_in.address())
                },
            );
            core::ptr::write(
                instruction_accounts_ptr.add(10),
                InstructionAccount::readonly_signer(ctx.user.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(11),
                InstructionAccount::readonly(ctx.token_x_program.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(12),
                InstructionAccount::readonly(ctx.token_y_program.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(13),
                InstructionAccount::readonly(ctx.memo_program.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(14),
                InstructionAccount::readonly(ctx.event_authority.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(15),
                InstructionAccount::readonly(ctx.program.address()),
            );
            for (index, account) in ctx.remaining_accounts.iter().enumerate() {
                core::ptr::write(
                    instruction_accounts_ptr.add(16 + index),
                    InstructionAccount::from(account),
                );
            }
        }

        let instruction_accounts =
            unsafe { core::slice::from_raw_parts(instruction_accounts_ptr, total_accounts) };

        let mut account_views = [ctx.lb_pair; MAX_ACCOUNTS];
        account_views[1] = ctx.bin_array_bitmap_extension;
        account_views[2] = ctx.reserve_x;
        account_views[3] = ctx.reserve_y;
        account_views[4] = ctx.user_token_in;
        account_views[5] = ctx.user_token_out;
        account_views[6] = ctx.token_x_mint;
        account_views[7] = ctx.token_y_mint;
        account_views[8] = ctx.oracle;
        account_views[9] = ctx.host_fee_in;
        account_views[10] = ctx.user;
        account_views[11] = ctx.token_x_program;
        account_views[12] = ctx.token_y_program;
        account_views[13] = ctx.memo_program;
        account_views[14] = ctx.event_authority;
        account_views[15] = ctx.program;
        for (index, account) in ctx.remaining_accounts.iter().enumerate() {
            account_views[16 + index] = account;
        }
        let account_views = &account_views[..total_accounts];

        let instruction = InstructionView {
            program_id: &METEORA_DLMM_PROGRAM_ID,
            accounts: instruction_accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, data_len)
            },
        };

        invoke_signed_with_bounds::<MAX_ACCOUNTS, _>(&instruction, account_views, signer_seeds)
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
