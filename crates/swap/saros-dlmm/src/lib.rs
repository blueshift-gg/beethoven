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

pub const SAROS_DLMM_PROGRAM_ID: Address = address!("1qbkdrr3z4ryLA7pZykqxvxWPoeifcVKo6ZG9CfkvVE");

const SWAP_DISCRIMINATOR: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];
// max 2 HookBinArray accounts expected
const MAX_REMAINING_ACCOUNTS: usize = 2;
const MAX_ACCOUNTS: usize = SarosDlmmSwapAccounts::NUM_ACCOUNTS + MAX_REMAINING_ACCOUNTS;

pub enum SwapType {
    ExactInput,
    ExactOutput,
}

pub struct SarosDlmmSwapData {
    pub swap_for_y: bool,
    pub swap_type: SwapType,
}

impl SarosDlmmSwapData {
    pub const DATA_LEN: usize = 2;
}

impl TryFrom<&[u8]> for SarosDlmmSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(SarosDlmmSwapData {
            swap_for_y: data[0] == 1,
            swap_type: match data[1] {
                0 => SwapType::ExactInput,
                1 => SwapType::ExactOutput,
                _ => return Err(ProgramError::InvalidInstructionData),
            },
        })
    }
}

pub struct SarosDlmm;

impl SarosDlmmSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 18;
}

pub struct SarosDlmmSwapAccounts<'info> {
    pub saros_dlmm_program: &'info AccountView,
    pub pair: &'info AccountView,
    pub token_mint_x: &'info AccountView,
    pub token_mint_y: &'info AccountView,
    pub bin_array_lower: &'info AccountView,
    pub bin_array_upper: &'info AccountView,
    pub token_vault_x: &'info AccountView,
    pub token_vault_y: &'info AccountView,
    pub user_vault_x: &'info AccountView,
    pub user_vault_y: &'info AccountView,
    pub user: &'info AccountView,
    pub token_program_x: &'info AccountView,
    pub token_program_y: &'info AccountView,
    pub memo_program: &'info AccountView,
    pub hook: &'info AccountView,
    pub hooks_program: &'info AccountView,
    pub event_authority: &'info AccountView,
    pub program: &'info AccountView,
    pub remaining_accounts: &'info [AccountView],
}

impl<'info> TryFrom<&'info [AccountView]> for SarosDlmmSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [saros_dlmm_program, pair, token_mint_x, token_mint_y, bin_array_lower, bin_array_upper, token_vault_x, token_vault_y, user_vault_x, user_vault_y, user, token_program_x, token_program_y, memo_program, hook, hooks_program, event_authority, program, remaining_accounts @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(SarosDlmmSwapAccounts {
            saros_dlmm_program,
            pair,
            token_mint_x,
            token_mint_y,
            bin_array_lower,
            bin_array_upper,
            token_vault_x,
            token_vault_y,
            user_vault_x,
            user_vault_y,
            user,
            token_program_x,
            token_program_y,
            memo_program,
            hook,
            hooks_program,
            event_authority,
            program,
            remaining_accounts,
        })
    }
}

impl<'info> Swap<'info> for SarosDlmm {
    type Accounts = SarosDlmmSwapAccounts<'info>;
    type Data = SarosDlmmSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let total_accounts = 17 + ctx.remaining_accounts.len();

        let mut account_metas = MaybeUninit::<[InstructionAccount; MAX_ACCOUNTS]>::uninit();
        let account_metas_ptr = account_metas.as_mut_ptr() as *mut InstructionAccount;

        unsafe {
            core::ptr::write(
                account_metas_ptr,
                InstructionAccount::writable(ctx.pair.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(1),
                InstructionAccount::readonly(ctx.token_mint_x.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(2),
                InstructionAccount::readonly(ctx.token_mint_y.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(3),
                InstructionAccount::writable(ctx.bin_array_lower.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(4),
                InstructionAccount::writable(ctx.bin_array_upper.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(5),
                InstructionAccount::writable(ctx.token_vault_x.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(6),
                InstructionAccount::writable(ctx.token_vault_y.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(7),
                InstructionAccount::writable(ctx.user_vault_x.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(8),
                InstructionAccount::writable(ctx.user_vault_y.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(9),
                InstructionAccount::writable_signer(ctx.user.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(10),
                InstructionAccount::readonly(ctx.token_program_x.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(11),
                InstructionAccount::readonly(ctx.token_program_y.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(12),
                InstructionAccount::readonly(ctx.memo_program.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(13),
                InstructionAccount::writable(ctx.hook.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(14),
                InstructionAccount::readonly(ctx.hooks_program.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(15),
                InstructionAccount::readonly(ctx.event_authority.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(16),
                InstructionAccount::readonly(ctx.program.address()),
            );
            for (index, account) in ctx.remaining_accounts.iter().enumerate() {
                core::ptr::write(
                    account_metas_ptr.add(17 + index),
                    InstructionAccount::from(account),
                );
            }
        }

        let account_metas =
            unsafe { core::slice::from_raw_parts(account_metas_ptr, total_accounts) };

        let mut account_infos = [ctx.pair; MAX_ACCOUNTS];
        account_infos[1] = ctx.token_mint_x;
        account_infos[2] = ctx.token_mint_y;
        account_infos[3] = ctx.bin_array_lower;
        account_infos[4] = ctx.bin_array_upper;
        account_infos[5] = ctx.token_vault_x;
        account_infos[6] = ctx.token_vault_y;
        account_infos[7] = ctx.user_vault_x;
        account_infos[8] = ctx.user_vault_y;
        account_infos[9] = ctx.user;
        account_infos[10] = ctx.token_program_x;
        account_infos[11] = ctx.token_program_y;
        account_infos[12] = ctx.memo_program;
        account_infos[13] = ctx.hook;
        account_infos[14] = ctx.hooks_program;
        account_infos[15] = ctx.event_authority;
        account_infos[16] = ctx.program;
        for (index, account) in ctx.remaining_accounts.iter().enumerate() {
            account_infos[17 + index] = account;
        }
        let account_infos = &account_infos[..total_accounts];

        let mut instruction_data = MaybeUninit::<[u8; 26]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(SWAP_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(16),
                8,
            );
            core::ptr::write(ptr.add(24), data.swap_for_y as u8);
            let swap_type_byte = match data.swap_type {
                SwapType::ExactInput => 0,
                SwapType::ExactOutput => 1,
            };
            core::ptr::write(ptr.add(25), swap_type_byte);
        }

        let instruction = InstructionView {
            program_id: &SAROS_DLMM_PROGRAM_ID,
            accounts: account_metas,
            data: unsafe { instruction_data.assume_init_ref() },
        };

        invoke_signed_with_bounds::<MAX_ACCOUNTS, _>(&instruction, account_infos, signer_seeds)?;

        Ok(())
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
