#![no_std]

use {
    beethoven_core::{Swap, SwapTokenAccounts},
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::{address, Address},
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const RAYDIUM_CLMM_PROGRAM_ID: Address =
    address!("CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK");

const SWAP_V2_DISCRIMINATOR: [u8; 8] = [43, 4, 237, 11, 26, 201, 30, 98];
const MAX_TICK_ARRAY_SIZE: usize = 16;
const MAX_ACCOUNTS: usize = RaydiumClmmSwapAccounts::NUM_ACCOUNTS + MAX_TICK_ARRAY_SIZE;

pub struct RaydiumClmm;

pub struct RaydiumClmmSwapData {
    pub sqrt_price_limit_x64: u128,
    pub is_base_input: bool,
}

impl RaydiumClmmSwapData {
    // 16 - sqrt_price_limit_x64
    // 1 - is_base_input
    pub const DATA_LEN: usize = 17;
}

impl TryFrom<&[u8]> for RaydiumClmmSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }
        let sqrt_price_limit_x64 = u128::from_le_bytes(data[0..16].try_into().unwrap());
        Ok(Self {
            sqrt_price_limit_x64,
            is_base_input: data[16] != 0,
        })
    }
}

impl RaydiumClmmSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 14;
}

pub struct RaydiumClmmSwapAccounts<'info> {
    pub raydium_clmm_program: &'info AccountView,
    pub payer: &'info AccountView,
    pub amm_config: &'info AccountView,
    pub pool_state: &'info AccountView,
    pub input_token_account: &'info AccountView,
    pub output_token_account: &'info AccountView,
    pub input_vault: &'info AccountView,
    pub output_vault: &'info AccountView,
    pub observation_state: &'info AccountView,
    pub token_program: &'info AccountView,
    pub token_program_2022: &'info AccountView,
    pub memo_program: &'info AccountView,
    pub input_vault_mint: &'info AccountView,
    pub output_vault_mint: &'info AccountView,
    pub remaining_accounts: &'info [AccountView],
}

impl<'info> TryFrom<&'info [AccountView]> for RaydiumClmmSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [raydium_clmm_program, payer, amm_config, pool_state, input_token_account, output_token_account, input_vault, output_vault, observation_state, token_program, token_program_2022, memo_program, input_vault_mint, output_vault_mint, remaining_accounts @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(RaydiumClmmSwapAccounts {
            raydium_clmm_program,
            payer,
            amm_config,
            pool_state,
            input_token_account,
            output_token_account,
            input_vault,
            output_vault,
            observation_state,
            token_program,
            token_program_2022,
            memo_program,
            input_vault_mint,
            output_vault_mint,
            remaining_accounts,
        })
    }
}

impl<'info> Swap<'info> for RaydiumClmm {
    type Accounts = RaydiumClmmSwapAccounts<'info>;
    type Data = RaydiumClmmSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &RaydiumClmmSwapData,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let total_accounts = 13 + ctx.remaining_accounts.len();

        let mut account_metas = MaybeUninit::<[InstructionAccount; MAX_ACCOUNTS]>::uninit();
        let account_metas_ptr = account_metas.as_mut_ptr() as *mut InstructionAccount;

        unsafe {
            core::ptr::write(
                account_metas_ptr,
                InstructionAccount::readonly_signer(ctx.payer.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(1),
                InstructionAccount::readonly(ctx.amm_config.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(2),
                InstructionAccount::writable(ctx.pool_state.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(3),
                InstructionAccount::writable(ctx.input_token_account.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(4),
                InstructionAccount::writable(ctx.output_token_account.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(5),
                InstructionAccount::writable(ctx.input_vault.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(6),
                InstructionAccount::writable(ctx.output_vault.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(7),
                InstructionAccount::writable(ctx.observation_state.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(8),
                InstructionAccount::readonly(ctx.token_program.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(9),
                InstructionAccount::readonly(ctx.token_program_2022.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(10),
                InstructionAccount::readonly(ctx.memo_program.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(11),
                InstructionAccount::readonly(ctx.input_vault_mint.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(12),
                InstructionAccount::readonly(ctx.output_vault_mint.address()),
            );

            for (index, account) in ctx.remaining_accounts.iter().enumerate() {
                core::ptr::write(
                    account_metas_ptr.add(13 + index),
                    InstructionAccount::from(account),
                );
            }
        }

        let account_metas =
            unsafe { core::slice::from_raw_parts(account_metas_ptr, total_accounts) };

        let mut account_infos = [ctx.payer; MAX_ACCOUNTS];
        account_infos[1] = ctx.amm_config;
        account_infos[2] = ctx.pool_state;
        account_infos[3] = ctx.input_token_account;
        account_infos[4] = ctx.output_token_account;
        account_infos[5] = ctx.input_vault;
        account_infos[6] = ctx.output_vault;
        account_infos[7] = ctx.observation_state;
        account_infos[8] = ctx.token_program;
        account_infos[9] = ctx.token_program_2022;
        account_infos[10] = ctx.memo_program;
        account_infos[11] = ctx.input_vault_mint;
        account_infos[12] = ctx.output_vault_mint;
        for (index, account) in ctx.remaining_accounts.iter().enumerate() {
            account_infos[13 + index] = account;
        }
        let account_infos = &account_infos[..total_accounts];

        let mut instruction_data = MaybeUninit::<[u8; 41]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(SWAP_V2_DISCRIMINATOR.as_ptr(), ptr, 8);

            let base_amount = if data.is_base_input {
                in_amount
            } else {
                minimum_out_amount
            };
            let quote_amount = if data.is_base_input {
                minimum_out_amount
            } else {
                in_amount
            };

            core::ptr::copy_nonoverlapping(base_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(quote_amount.to_le_bytes().as_ptr(), ptr.add(16), 8);
            let lim = data.sqrt_price_limit_x64.to_le_bytes();
            core::ptr::copy_nonoverlapping(lim.as_ptr(), ptr.add(24), 16);
            *ptr.add(40) = u8::from(data.is_base_input);
        }

        let instruction = InstructionView {
            program_id: &RAYDIUM_CLMM_PROGRAM_ID,
            accounts: account_metas,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 41)
            },
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

impl<'info> SwapTokenAccounts<'info> for RaydiumClmm {
    type Accounts = RaydiumClmmSwapAccounts<'info>;
    type Data = RaydiumClmmSwapData;

    fn token_accounts(
        ctx: &Self::Accounts,
        _data: &Self::Data,
    ) -> (&'info AccountView, &'info AccountView) {
        (ctx.input_token_account, ctx.output_token_account)
    }
}
