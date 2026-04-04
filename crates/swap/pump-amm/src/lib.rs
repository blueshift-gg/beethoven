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

pub const PUMP_AMM_PROGRAM_ID: Address =
    Address::from_str_const("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");

const BUY_DISCRIMINATOR: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
// Optional associated token account of the UserVolumeAccumulator for Pump AMM program
const MAX_REMAINING_ACCOUNTS: usize = 1;
const MAX_ACCOUNTS: usize = PumpAmmSwapAccounts::NUM_ACCOUNTS + MAX_REMAINING_ACCOUNTS;

pub struct PumpAmm;

pub struct PumpAmmSwapData {
    pub track_volume: Option<bool>,
}

impl PumpAmmSwapData {
    pub const DATA_LEN: usize = 2;
}

impl TryFrom<&[u8]> for PumpAmmSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }
        let (tag, b) = (data[0], data[1]);
        let track_volume = match tag {
            0 if b == 0 => None,
            0 => return Err(ProgramError::InvalidInstructionData),
            1 => Some(match b {
                0 => false,
                1 => true,
                _ => return Err(ProgramError::InvalidInstructionData),
            }),
            _ => return Err(ProgramError::InvalidInstructionData),
        };
        Ok(Self { track_volume })
    }
}

impl PumpAmmSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 24;
}

pub struct PumpAmmSwapAccounts<'info> {
    pub pump_amm_program: &'info AccountView,
    pub pool: &'info AccountView,
    pub user: &'info AccountView,
    pub global_config: &'info AccountView,
    pub base_mint: &'info AccountView,
    pub quote_mint: &'info AccountView,
    pub user_base_token_account: &'info AccountView,
    pub user_quote_token_account: &'info AccountView,
    pub pool_base_token_account: &'info AccountView,
    pub pool_quote_token_account: &'info AccountView,
    pub protocol_fee_recipient: &'info AccountView,
    pub protocol_fee_recipient_token_account: &'info AccountView,
    pub base_token_program: &'info AccountView,
    pub quote_token_program: &'info AccountView,
    pub system_program: &'info AccountView,
    pub associated_token_program: &'info AccountView,
    pub event_authority: &'info AccountView,
    pub program: &'info AccountView,
    pub coin_creator_vault_ata: &'info AccountView,
    pub coin_creator_vault_authority: &'info AccountView,
    pub global_volume_accumulator: &'info AccountView,
    pub user_volume_accumulator: &'info AccountView,
    pub fee_config: &'info AccountView,
    pub fee_program: &'info AccountView,
    pub remaining_accounts: &'info [AccountView],
}

impl<'info> TryFrom<&'info [AccountView]> for PumpAmmSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [pump_amm_program, pool, user, global_config, base_mint, quote_mint, user_base_token_account, user_quote_token_account, pool_base_token_account, pool_quote_token_account, protocol_fee_recipient, protocol_fee_recipient_token_account, base_token_program, quote_token_program, system_program, associated_token_program, event_authority, program, coin_creator_vault_ata, coin_creator_vault_authority, global_volume_accumulator, user_volume_accumulator, fee_config, fee_program, remaining_accounts @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(PumpAmmSwapAccounts {
            pump_amm_program,
            pool,
            user,
            global_config,
            base_mint,
            quote_mint,
            user_base_token_account,
            user_quote_token_account,
            pool_base_token_account,
            pool_quote_token_account,
            protocol_fee_recipient,
            protocol_fee_recipient_token_account,
            base_token_program,
            quote_token_program,
            system_program,
            associated_token_program,
            event_authority,
            program,
            coin_creator_vault_ata,
            coin_creator_vault_authority,
            global_volume_accumulator,
            user_volume_accumulator,
            fee_config,
            fee_program,
            remaining_accounts,
        })
    }
}

impl<'info> Swap<'info> for PumpAmm {
    type Accounts = PumpAmmSwapAccounts<'info>;
    type Data = PumpAmmSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let total_accounts = 23 + ctx.remaining_accounts.len();

        let mut account_metas = MaybeUninit::<[InstructionAccount; MAX_ACCOUNTS]>::uninit();
        let account_metas_ptr = account_metas.as_mut_ptr() as *mut InstructionAccount;

        unsafe {
            core::ptr::write(
                account_metas_ptr,
                InstructionAccount::writable(ctx.pool.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(1),
                InstructionAccount::writable_signer(ctx.user.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(2),
                InstructionAccount::readonly(ctx.global_config.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(3),
                InstructionAccount::readonly(ctx.base_mint.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(4),
                InstructionAccount::readonly(ctx.quote_mint.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(5),
                InstructionAccount::writable(ctx.user_base_token_account.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(6),
                InstructionAccount::writable(ctx.user_quote_token_account.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(7),
                InstructionAccount::writable(ctx.pool_base_token_account.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(8),
                InstructionAccount::writable(ctx.pool_quote_token_account.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(9),
                InstructionAccount::readonly(ctx.protocol_fee_recipient.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(10),
                InstructionAccount::writable(ctx.protocol_fee_recipient_token_account.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(11),
                InstructionAccount::readonly(ctx.base_token_program.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(12),
                InstructionAccount::readonly(ctx.quote_token_program.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(13),
                InstructionAccount::readonly(ctx.system_program.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(14),
                InstructionAccount::readonly(ctx.associated_token_program.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(15),
                InstructionAccount::readonly(ctx.event_authority.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(16),
                InstructionAccount::readonly(ctx.program.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(17),
                InstructionAccount::writable(ctx.coin_creator_vault_ata.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(18),
                InstructionAccount::readonly(ctx.coin_creator_vault_authority.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(19),
                InstructionAccount::readonly(ctx.global_volume_accumulator.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(20),
                InstructionAccount::writable(ctx.user_volume_accumulator.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(21),
                InstructionAccount::readonly(ctx.fee_config.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(22),
                InstructionAccount::readonly(ctx.fee_program.address()),
            );

            for (index, account) in ctx.remaining_accounts.iter().enumerate() {
                core::ptr::write(
                    account_metas_ptr.add(23 + index),
                    InstructionAccount::from(account),
                );
            }
        }

        let account_metas =
            unsafe { core::slice::from_raw_parts(account_metas_ptr, total_accounts) };

        let mut account_infos = [ctx.pool; MAX_ACCOUNTS];
        account_infos[1] = ctx.user;
        account_infos[2] = ctx.global_config;
        account_infos[3] = ctx.base_mint;
        account_infos[4] = ctx.quote_mint;
        account_infos[5] = ctx.user_base_token_account;
        account_infos[6] = ctx.user_quote_token_account;
        account_infos[7] = ctx.pool_base_token_account;
        account_infos[8] = ctx.pool_quote_token_account;
        account_infos[9] = ctx.protocol_fee_recipient;
        account_infos[10] = ctx.protocol_fee_recipient_token_account;
        account_infos[11] = ctx.base_token_program;
        account_infos[12] = ctx.quote_token_program;
        account_infos[13] = ctx.system_program;
        account_infos[14] = ctx.associated_token_program;
        account_infos[15] = ctx.event_authority;
        account_infos[16] = ctx.program;
        account_infos[17] = ctx.coin_creator_vault_ata;
        account_infos[18] = ctx.coin_creator_vault_authority;
        account_infos[19] = ctx.global_volume_accumulator;
        account_infos[20] = ctx.user_volume_accumulator;
        account_infos[21] = ctx.fee_config;
        account_infos[22] = ctx.fee_program;
        for (index, account) in ctx.remaining_accounts.iter().enumerate() {
            account_infos[23 + index] = account;
        }
        let account_infos = &account_infos[..total_accounts];

        let mut instruction_data = MaybeUninit::<[u8; 26]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(BUY_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(8),
                8,
            );
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(16), 8);
            let bytes = match data.track_volume {
                None => [0, 0],
                Some(b) => [1, b as u8],
            };
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.add(24), 2);
        }

        let instruction = InstructionView {
            program_id: &PUMP_AMM_PROGRAM_ID,
            accounts: account_metas,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 26)
            },
        };

        invoke_signed_with_bounds::<MAX_ACCOUNTS>(&instruction, account_infos, signer_seeds)?;

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
