#![no_std]

use {
    beethoven_core::Swap,
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const HADRON_PROGRAM_ID: Address =
    Address::from_str_const("Q72w4coozA552keKDdeeh2EyQw32qfMFsHPu6cbatom");

const SWAP_DISCRIMINATOR: u8 = 3;
// spread config, sysvar instructions
const MAX_ACCOUNTS: usize = HadronSwapAccounts::NUM_ACCOUNTS + 2;

pub struct Hadron;

#[repr(u8)]
pub enum IsX {
    No = 0,
    Yes = 1,
}

pub struct HadronSwapData {
    pub is_x: IsX,
    pub expiration: i64,
}

impl HadronSwapData {
    pub const DATA_LEN: usize = 9;
}

impl TryFrom<&[u8]> for HadronSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let is_x = match data[0] {
            0 => IsX::No,
            1 => IsX::Yes,
            _ => return Err(ProgramError::InvalidInstructionData),
        };
        let expiration = i64::from_le_bytes(data[1..9].try_into().unwrap());
        Ok(Self { is_x, expiration })
    }
}

impl HadronSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 16;
}

pub struct HadronSwapAccounts<'info> {
    pub hadron_program: &'info AccountView,
    pub token_program_x: &'info AccountView,
    pub token_program_y: &'info AccountView,
    pub config: &'info AccountView,
    pub midprice_oracle: &'info AccountView,
    pub curve_meta: &'info AccountView,
    pub curve_prefabs: &'info AccountView,
    pub user: &'info AccountView,
    pub user_source: &'info AccountView,
    pub vault_source: &'info AccountView,
    pub vault_dest: &'info AccountView,
    pub user_dest: &'info AccountView,
    pub fee_config_pda: &'info AccountView,
    pub fee_recipient_ata: &'info AccountView,
    pub clock: &'info AccountView,
    pub curve_updates: &'info AccountView,
    pub remaining_accounts: &'info [AccountView],
}

impl<'info> TryFrom<&'info [AccountView]> for HadronSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [hadron_program, token_program_x, token_program_y, config, midprice_oracle, curve_meta, curve_prefabs, user, user_source, vault_source, vault_dest, user_dest, fee_config_pda, fee_recipient_ata, clock, curve_updates, remaining_accounts @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(HadronSwapAccounts {
            hadron_program,
            token_program_x,
            token_program_y,
            config,
            midprice_oracle,
            curve_meta,
            curve_prefabs,
            user,
            user_source,
            vault_source,
            vault_dest,
            user_dest,
            fee_config_pda,
            fee_recipient_ata,
            clock,
            curve_updates,
            remaining_accounts,
        })
    }
}

impl<'info> Swap<'info> for Hadron {
    type Accounts = HadronSwapAccounts<'info>;
    type Data = HadronSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let mut account_metas = MaybeUninit::<[InstructionAccount; MAX_ACCOUNTS]>::uninit();
        let account_metas_ptr = account_metas.as_mut_ptr() as *mut InstructionAccount;

        unsafe {
            core::ptr::write(
                account_metas_ptr,
                InstructionAccount::readonly(ctx.token_program_x.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(1),
                InstructionAccount::readonly(ctx.token_program_y.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(2),
                InstructionAccount::readonly(ctx.config.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(3),
                InstructionAccount::readonly(ctx.midprice_oracle.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(4),
                InstructionAccount::readonly(ctx.curve_meta.address()),
            );

            core::ptr::write(
                account_metas_ptr.add(5),
                InstructionAccount::writable(ctx.curve_prefabs.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(6),
                InstructionAccount::readonly(ctx.config.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(7),
                InstructionAccount::readonly_signer(ctx.user.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(8),
                InstructionAccount::writable(ctx.user_source.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(9),
                InstructionAccount::writable(ctx.vault_source.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(10),
                InstructionAccount::writable(ctx.vault_dest.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(11),
                InstructionAccount::writable(ctx.user_dest.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(12),
                InstructionAccount::readonly(ctx.fee_config_pda.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(13),
                InstructionAccount::writable(ctx.fee_recipient_ata.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(14),
                InstructionAccount::readonly(ctx.clock.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(15),
                InstructionAccount::writable(ctx.curve_updates.address()),
            );

            for (index, account) in ctx.remaining_accounts.iter().enumerate() {
                core::ptr::write(
                    account_metas_ptr.add(16 + index),
                    InstructionAccount::from(account),
                );
            }
        }

        let account_metas = unsafe { core::slice::from_raw_parts(account_metas_ptr, MAX_ACCOUNTS) };

        let mut account_infos = [ctx.token_program_x; MAX_ACCOUNTS];
        account_infos[1] = ctx.token_program_y;
        account_infos[2] = ctx.config;
        account_infos[3] = ctx.midprice_oracle;
        account_infos[4] = ctx.curve_meta;
        account_infos[5] = ctx.curve_prefabs;
        account_infos[6] = ctx.config;
        account_infos[7] = ctx.user;
        account_infos[8] = ctx.user_source;
        account_infos[9] = ctx.vault_source;
        account_infos[10] = ctx.vault_dest;
        account_infos[11] = ctx.user_dest;
        account_infos[12] = ctx.fee_config_pda;
        account_infos[13] = ctx.fee_recipient_ata;
        account_infos[14] = ctx.clock;
        account_infos[15] = ctx.curve_updates;
        for (index, account) in ctx.remaining_accounts.iter().enumerate() {
            account_infos[16 + index] = account;
        }
        let account_infos = &account_infos[..MAX_ACCOUNTS];

        let mut instruction_data = MaybeUninit::<[u8; 26]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::write(ptr.add(0), SWAP_DISCRIMINATOR);
            let is_x_byte = match data.is_x {
                IsX::No => 0u8,
                IsX::Yes => 1u8,
            };
            core::ptr::write(ptr.add(1), is_x_byte);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(2), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(10),
                8,
            );
            core::ptr::copy_nonoverlapping(data.expiration.to_le_bytes().as_ptr(), ptr.add(18), 8);
        }

        let instruction = InstructionView {
            program_id: &HADRON_PROGRAM_ID,
            accounts: account_metas,
            data: unsafe { instruction_data.assume_init_ref() },
        };

        invoke_signed_with_bounds::<MAX_ACCOUNTS>(&instruction, account_infos, signer_seeds)
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
