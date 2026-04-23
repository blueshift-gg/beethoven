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

pub const FRAUDSWORTH_TAX_PROGRAM_ID: Address =
    address!("43fZGRtmEsP7ExnJE1dbTbNjaP1ncvVmMPusSeksWGEj");

const SWAP_SOL_BUY_DISCRIMINATOR: [u8; 8] = [158, 213, 169, 65, 11, 116, 176, 25];
const SWAP_SOL_SELL_DISCRIMINATOR: [u8; 8] = [136, 242, 218, 149, 17, 222, 250, 240];

const BUY_ACCOUNTS_LEN: usize = 24;
const SELL_ACCOUNTS_LEN: usize = 25;
const MAX_ACCOUNTS: usize = SELL_ACCOUNTS_LEN;

pub struct FraudsworthTax;

pub struct FraudsworthTaxSwapData {
    pub is_buy: bool,
    pub is_crime: bool,
}

impl FraudsworthTaxSwapData {
    pub const DATA_LEN: usize = 2;
}

impl TryFrom<&[u8]> for FraudsworthTaxSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            is_buy: data[0] != 0,
            is_crime: data[1] != 0,
        })
    }
}

impl FraudsworthTaxSwapAccounts<'_> {
    pub const MIN_NUM_ACCOUNTS: usize = BUY_ACCOUNTS_LEN + 1;
}

pub struct FraudsworthTaxSwapAccounts<'info> {
    pub fraudsworth_tax_program: &'info AccountView,
    pub user: &'info AccountView,
    pub epoch_state: &'info AccountView,
    pub swap_authority: &'info AccountView,
    pub tax_authority: &'info AccountView,
    pub pool: &'info AccountView,
    pub pool_vault_a: &'info AccountView,
    pub pool_vault_b: &'info AccountView,
    pub mint_a: &'info AccountView,
    pub mint_b: &'info AccountView,
    pub user_token_a: &'info AccountView,
    pub user_token_b: &'info AccountView,
    pub stake_pool: &'info AccountView,
    pub staking_escrow: &'info AccountView,
    pub carnage_vault: &'info AccountView,
    pub treasury: &'info AccountView,
    pub wsol_intermediary: Option<&'info AccountView>,
    pub amm_program: &'info AccountView,
    pub token_program_a: &'info AccountView,
    pub token_program_b: &'info AccountView,
    pub system_program: &'info AccountView,
    pub staking_program: &'info AccountView,
    pub extra_account_meta_list: &'info AccountView,
    pub whitelist_source: &'info AccountView,
    pub whitelist_destination: &'info AccountView,
    pub transfer_hook_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for FraudsworthTaxSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [fraudsworth_tax_program, user, epoch_state, swap_authority, tax_authority, pool, pool_vault_a, pool_vault_b, mint_a, mint_b, user_token_a, user_token_b, stake_pool, staking_escrow, carnage_vault, treasury, tail @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        let (
            wsol_intermediary,
            amm_program,
            token_program_a,
            token_program_b,
            system_program,
            staking_program,
            extra_account_meta_list,
            whitelist_source,
            whitelist_destination,
            transfer_hook_program,
        ) = match tail {
            [amm_program, token_program_a, token_program_b, system_program, staking_program, extra_account_meta_list, whitelist_source, whitelist_destination, transfer_hook_program] => {
                (
                    None,
                    amm_program,
                    token_program_a,
                    token_program_b,
                    system_program,
                    staking_program,
                    extra_account_meta_list,
                    whitelist_source,
                    whitelist_destination,
                    transfer_hook_program,
                )
            }
            [wsol_intermediary, amm_program, token_program_a, token_program_b, system_program, staking_program, extra_account_meta_list, whitelist_source, whitelist_destination, transfer_hook_program] => {
                (
                    Some(wsol_intermediary),
                    amm_program,
                    token_program_a,
                    token_program_b,
                    system_program,
                    staking_program,
                    extra_account_meta_list,
                    whitelist_source,
                    whitelist_destination,
                    transfer_hook_program,
                )
            }
            _ => return Err(ProgramError::NotEnoughAccountKeys),
        };

        Ok(Self {
            fraudsworth_tax_program,
            user,
            epoch_state,
            swap_authority,
            tax_authority,
            pool,
            pool_vault_a,
            pool_vault_b,
            mint_a,
            mint_b,
            user_token_a,
            user_token_b,
            stake_pool,
            staking_escrow,
            carnage_vault,
            treasury,
            wsol_intermediary,
            amm_program,
            token_program_a,
            token_program_b,
            system_program,
            staking_program,
            extra_account_meta_list,
            whitelist_source,
            whitelist_destination,
            transfer_hook_program,
        })
    }
}

impl<'info> Swap<'info> for FraudsworthTax {
    type Accounts = FraudsworthTaxSwapAccounts<'info>;
    type Data = FraudsworthTaxSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let (total_accounts, discriminator) = if data.is_buy {
            if ctx.wsol_intermediary.is_some() {
                return Err(ProgramError::InvalidAccountData);
            }
            (BUY_ACCOUNTS_LEN, SWAP_SOL_BUY_DISCRIMINATOR)
        } else {
            if ctx.wsol_intermediary.is_none() {
                return Err(ProgramError::NotEnoughAccountKeys);
            }
            (SELL_ACCOUNTS_LEN, SWAP_SOL_SELL_DISCRIMINATOR)
        };

        let mut instruction_accounts = MaybeUninit::<[InstructionAccount; MAX_ACCOUNTS]>::uninit();
        let instruction_accounts_ptr = instruction_accounts.as_mut_ptr() as *mut InstructionAccount;

        unsafe {
            core::ptr::write(
                instruction_accounts_ptr,
                InstructionAccount::writable_signer(ctx.user.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(1),
                InstructionAccount::readonly(ctx.epoch_state.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(2),
                if data.is_buy {
                    InstructionAccount::readonly(ctx.swap_authority.address())
                } else {
                    InstructionAccount::writable(ctx.swap_authority.address())
                },
            );
            core::ptr::write(
                instruction_accounts_ptr.add(3),
                InstructionAccount::readonly(ctx.tax_authority.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(4),
                InstructionAccount::writable(ctx.pool.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(5),
                InstructionAccount::writable(ctx.pool_vault_a.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(6),
                InstructionAccount::writable(ctx.pool_vault_b.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(7),
                InstructionAccount::readonly(ctx.mint_a.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(8),
                InstructionAccount::readonly(ctx.mint_b.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(9),
                InstructionAccount::writable(ctx.user_token_a.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(10),
                InstructionAccount::writable(ctx.user_token_b.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(11),
                InstructionAccount::writable(ctx.stake_pool.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(12),
                InstructionAccount::writable(ctx.staking_escrow.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(13),
                InstructionAccount::writable(ctx.carnage_vault.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(14),
                InstructionAccount::writable(ctx.treasury.address()),
            );

            let mut index = 15;
            if let Some(wsol_intermediary) = ctx.wsol_intermediary {
                core::ptr::write(
                    instruction_accounts_ptr.add(index),
                    InstructionAccount::writable(wsol_intermediary.address()),
                );
                index += 1;
            }

            core::ptr::write(
                instruction_accounts_ptr.add(index),
                InstructionAccount::readonly(ctx.amm_program.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(index + 1),
                InstructionAccount::readonly(ctx.token_program_a.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(index + 2),
                InstructionAccount::readonly(ctx.token_program_b.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(index + 3),
                InstructionAccount::readonly(ctx.system_program.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(index + 4),
                InstructionAccount::readonly(ctx.staking_program.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(index + 5),
                InstructionAccount::readonly(ctx.extra_account_meta_list.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(index + 6),
                InstructionAccount::readonly(ctx.whitelist_source.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(index + 7),
                InstructionAccount::readonly(ctx.whitelist_destination.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(index + 8),
                InstructionAccount::readonly(ctx.transfer_hook_program.address()),
            );
        }

        let instruction_accounts =
            unsafe { core::slice::from_raw_parts(instruction_accounts_ptr, total_accounts) };

        let mut account_infos = [ctx.user; MAX_ACCOUNTS];
        account_infos[1] = ctx.epoch_state;
        account_infos[2] = ctx.swap_authority;
        account_infos[3] = ctx.tax_authority;
        account_infos[4] = ctx.pool;
        account_infos[5] = ctx.pool_vault_a;
        account_infos[6] = ctx.pool_vault_b;
        account_infos[7] = ctx.mint_a;
        account_infos[8] = ctx.mint_b;
        account_infos[9] = ctx.user_token_a;
        account_infos[10] = ctx.user_token_b;
        account_infos[11] = ctx.stake_pool;
        account_infos[12] = ctx.staking_escrow;
        account_infos[13] = ctx.carnage_vault;
        account_infos[14] = ctx.treasury;

        let mut index = 15;
        if let Some(wsol_intermediary) = ctx.wsol_intermediary {
            account_infos[index] = wsol_intermediary;
            index += 1;
        }
        account_infos[index] = ctx.amm_program;
        account_infos[index + 1] = ctx.token_program_a;
        account_infos[index + 2] = ctx.token_program_b;
        account_infos[index + 3] = ctx.system_program;
        account_infos[index + 4] = ctx.staking_program;
        account_infos[index + 5] = ctx.extra_account_meta_list;
        account_infos[index + 6] = ctx.whitelist_source;
        account_infos[index + 7] = ctx.whitelist_destination;
        account_infos[index + 8] = ctx.transfer_hook_program;

        let account_infos = &account_infos[..total_accounts];

        let mut instruction_data = MaybeUninit::<[u8; 25]>::uninit();

        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(discriminator.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(16),
                8,
            );
            core::ptr::write(ptr.add(24), data.is_crime as u8);
        }

        let instruction = InstructionView {
            program_id: &FRAUDSWORTH_TAX_PROGRAM_ID,
            accounts: instruction_accounts,
            data: unsafe { instruction_data.assume_init_ref() },
        };

        invoke_signed_with_bounds::<MAX_ACCOUNTS, _>(&instruction, account_infos, signer_seeds)
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
