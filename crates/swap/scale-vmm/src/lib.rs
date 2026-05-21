#![no_std]

use {
    beethoven_core::{Swap, SwapTokenAccounts},
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const SCALE_VMM_PROGRAM_ID: Address =
    Address::from_str_const("SCALEWoRSpVZpMRqHEcDfNvBh3nUSe34jDr9r689gLa");

const BUY_DISCRIMINATOR: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
const SELL_DISCRIMINATOR: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];

const MAX_BENEFICIARY_ACCOUNTS: usize = 5;

pub struct ScaleVmm;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ScaleVmmSide {
    Buy = 0,
    Sell = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScaleVmmSwapData {
    pub side: ScaleVmmSide,
}

impl ScaleVmmSwapData {
    pub const DATA_LEN: usize = 1;
}

impl TryFrom<&[u8]> for ScaleVmmSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let side = match data[0] {
            0 => ScaleVmmSide::Buy,
            1 => ScaleVmmSide::Sell,
            _ => return Err(ProgramError::InvalidInstructionData),
        };

        Ok(Self { side })
    }
}

pub struct ScaleVmmSwapAccounts<'info> {
    pub scale_vmm_program: &'info AccountView,
    pub pair: &'info AccountView,
    pub user: &'info AccountView,
    pub mint_a: &'info AccountView,
    pub mint_b: &'info AccountView,
    pub user_ta_a: &'info AccountView,
    pub user_ta_b: &'info AccountView,
    pub vault_a: &'info AccountView,
    pub vault_b: &'info AccountView,
    pub platform_fee_ta_a: &'info AccountView,
    pub token_program_a: &'info AccountView,
    pub token_program_b: &'info AccountView,
    pub system_program: &'info AccountView,
    pub config: &'info AccountView,
    pub amm_program: &'info AccountView,
    pub amm_pool: &'info AccountView,
    pub amm_vault_a: &'info AccountView,
    pub amm_vault_b: &'info AccountView,
    pub amm_config: &'info AccountView,
    pub beneficiary_accounts: &'info [AccountView],
}

impl ScaleVmmSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 19;
}

const CPI_FIXED_ACCOUNTS: usize = ScaleVmmSwapAccounts::NUM_ACCOUNTS - 1;
const MAX_ACCOUNTS: usize = CPI_FIXED_ACCOUNTS + MAX_BENEFICIARY_ACCOUNTS;

impl<'info> TryFrom<&'info [AccountView]> for ScaleVmmSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [scale_vmm_program, pair, user, mint_a, mint_b, user_ta_a, user_ta_b, vault_a, vault_b, platform_fee_ta_a, token_program_a, token_program_b, system_program, config, amm_program, amm_pool, amm_vault_a, amm_vault_b, amm_config, beneficiary_accounts @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if beneficiary_accounts.len() > MAX_BENEFICIARY_ACCOUNTS {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            scale_vmm_program,
            pair,
            user,
            mint_a,
            mint_b,
            user_ta_a,
            user_ta_b,
            vault_a,
            vault_b,
            platform_fee_ta_a,
            token_program_a,
            token_program_b,
            system_program,
            config,
            amm_program,
            amm_pool,
            amm_vault_a,
            amm_vault_b,
            amm_config,
            beneficiary_accounts,
        })
    }
}

fn build_instruction_data(
    side: ScaleVmmSide,
    in_amount: u64,
    minimum_out_amount: u64,
) -> MaybeUninit<[u8; 24]> {
    let discriminator = match side {
        ScaleVmmSide::Buy => &BUY_DISCRIMINATOR,
        ScaleVmmSide::Sell => &SELL_DISCRIMINATOR,
    };

    let mut instruction_data = MaybeUninit::<[u8; 24]>::uninit();
    unsafe {
        let ptr = instruction_data.as_mut_ptr() as *mut u8;
        core::ptr::copy_nonoverlapping(discriminator.as_ptr(), ptr, 8);
        core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
        core::ptr::copy_nonoverlapping(minimum_out_amount.to_le_bytes().as_ptr(), ptr.add(16), 8);
    }

    instruction_data
}

impl<'info> Swap<'info> for ScaleVmm {
    type Accounts = ScaleVmmSwapAccounts<'info>;
    type Data = ScaleVmmSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let instruction_data = build_instruction_data(data.side, in_amount, minimum_out_amount);

        let total_accounts = CPI_FIXED_ACCOUNTS + ctx.beneficiary_accounts.len();

        let mut account_metas = MaybeUninit::<[InstructionAccount; MAX_ACCOUNTS]>::uninit();
        let account_metas_ptr = account_metas.as_mut_ptr() as *mut InstructionAccount;

        unsafe {
            core::ptr::write(
                account_metas_ptr,
                InstructionAccount::writable(ctx.pair.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(1),
                InstructionAccount::writable_signer(ctx.user.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(2),
                InstructionAccount::readonly(ctx.mint_a.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(3),
                InstructionAccount::readonly(ctx.mint_b.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(4),
                InstructionAccount::writable(ctx.user_ta_a.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(5),
                InstructionAccount::writable(ctx.user_ta_b.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(6),
                InstructionAccount::writable(ctx.vault_a.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(7),
                InstructionAccount::writable(ctx.vault_b.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(8),
                InstructionAccount::writable(ctx.platform_fee_ta_a.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(9),
                InstructionAccount::readonly(ctx.token_program_a.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(10),
                InstructionAccount::readonly(ctx.token_program_b.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(11),
                InstructionAccount::readonly(ctx.system_program.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(12),
                InstructionAccount::readonly(ctx.config.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(13),
                InstructionAccount::readonly(ctx.amm_program.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(14),
                InstructionAccount::writable(ctx.amm_pool.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(15),
                InstructionAccount::writable(ctx.amm_vault_a.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(16),
                InstructionAccount::writable(ctx.amm_vault_b.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(17),
                InstructionAccount::readonly(ctx.amm_config.address()),
            );

            for (index, account) in ctx.beneficiary_accounts.iter().enumerate() {
                core::ptr::write(
                    account_metas_ptr.add(CPI_FIXED_ACCOUNTS + index),
                    InstructionAccount::from(account),
                );
            }
        }

        let account_metas = unsafe {
            core::slice::from_raw_parts(
                account_metas_ptr as *const InstructionAccount,
                total_accounts,
            )
        };

        let mut account_infos = [ctx.pair; MAX_ACCOUNTS];
        account_infos[0] = ctx.pair;
        account_infos[1] = ctx.user;
        account_infos[2] = ctx.mint_a;
        account_infos[3] = ctx.mint_b;
        account_infos[4] = ctx.user_ta_a;
        account_infos[5] = ctx.user_ta_b;
        account_infos[6] = ctx.vault_a;
        account_infos[7] = ctx.vault_b;
        account_infos[8] = ctx.platform_fee_ta_a;
        account_infos[9] = ctx.token_program_a;
        account_infos[10] = ctx.token_program_b;
        account_infos[11] = ctx.system_program;
        account_infos[12] = ctx.config;
        account_infos[13] = ctx.amm_program;
        account_infos[14] = ctx.amm_pool;
        account_infos[15] = ctx.amm_vault_a;
        account_infos[16] = ctx.amm_vault_b;
        account_infos[17] = ctx.amm_config;
        for (index, account) in ctx.beneficiary_accounts.iter().enumerate() {
            account_infos[CPI_FIXED_ACCOUNTS + index] = account;
        }
        let account_infos = &account_infos[..total_accounts];

        let instruction = InstructionView {
            program_id: &SCALE_VMM_PROGRAM_ID,
            accounts: account_metas,
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

impl<'info> SwapTokenAccounts<'info> for ScaleVmm {
    type Accounts = ScaleVmmSwapAccounts<'info>;
    type Data = ScaleVmmSwapData;

    fn token_accounts(
        ctx: &Self::Accounts,
        data: &Self::Data,
    ) -> (&'info AccountView, &'info AccountView) {
        match data.side {
            ScaleVmmSide::Buy => (ctx.user_ta_b, ctx.user_ta_a),
            ScaleVmmSide::Sell => (ctx.user_ta_a, ctx.user_ta_b),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scale_vmm_swap_data() {
        let buy = ScaleVmmSwapData::try_from(&[0u8][..]).unwrap();
        assert_eq!(buy.side, ScaleVmmSide::Buy);

        let sell = ScaleVmmSwapData::try_from(&[1u8][..]).unwrap();
        assert_eq!(sell.side, ScaleVmmSide::Sell);

        let empty: &[u8] = &[];

        assert_eq!(
            ScaleVmmSwapData::try_from(empty).unwrap_err(),
            ProgramError::InvalidInstructionData
        );
        assert_eq!(
            ScaleVmmSwapData::try_from(&[2u8][..]).unwrap_err(),
            ProgramError::InvalidInstructionData
        );
    }

    #[test]
    fn test_instruction_data_layout() {
        let amount = 42u64;
        let min_out = 7u64;

        let buy = build_instruction_data(ScaleVmmSide::Buy, amount, min_out);
        let buy = unsafe { &*buy.as_ptr() };
        assert_eq!(&buy[0..8], &BUY_DISCRIMINATOR);
        assert_eq!(&buy[8..16], &amount.to_le_bytes());
        assert_eq!(&buy[16..24], &min_out.to_le_bytes());

        let sell = build_instruction_data(ScaleVmmSide::Sell, amount, min_out);
        let sell = unsafe { &*sell.as_ptr() };
        assert_eq!(&sell[0..8], &SELL_DISCRIMINATOR);
        assert_eq!(&sell[8..16], &amount.to_le_bytes());
        assert_eq!(&sell[16..24], &min_out.to_le_bytes());
    }
}
