#![no_std]

use {
    beethoven_core::Swap,
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::{address, Address},
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const METEORA_DYNAMIC_BONDING_CURVE_PROGRAM_ID: Address =
    address!("dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN");

const SWAP2_DISCRIMINATOR: [u8; 8] = [65, 75, 63, 76, 235, 91, 91, 136];

#[derive(PartialEq, Copy, Clone)]
pub enum SwapMode {
    ExactIn = 0,
    PartialFill = 1,
    ExactOut = 2,
}

pub struct MeteoraDynamicBondingCurve;

impl MeteoraDynamicBondingCurveSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 16;
}

pub struct MeteoraDynamicBondingCurveSwapAccounts<'info> {
    pub dynamic_bonding_curve_program: &'info AccountView,
    pub pool_authority: &'info AccountView,
    pub config: &'info AccountView,
    pub pool: &'info AccountView,
    pub input_token_account: &'info AccountView,
    pub output_token_account: &'info AccountView,
    pub base_vault: &'info AccountView,
    pub quote_vault: &'info AccountView,
    pub base_mint: &'info AccountView,
    pub quote_mint: &'info AccountView,
    pub payer: &'info AccountView,
    pub token_base_program: &'info AccountView,
    pub token_quote_program: &'info AccountView,
    pub referral_token_account: &'info AccountView,
    pub event_authority: &'info AccountView,
    pub program: &'info AccountView,
}

pub struct MeteoraDynamicBondingCurveSwapData {
    pub swap_mode: SwapMode,
}

impl MeteoraDynamicBondingCurveSwapData {
    pub const DATA_LEN: usize = 1;
}

impl TryFrom<&[u8]> for MeteoraDynamicBondingCurveSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self {
            swap_mode: match data[0] {
                0 => SwapMode::ExactIn,
                1 => SwapMode::PartialFill,
                2 => SwapMode::ExactOut,
                _ => return Err(ProgramError::InvalidInstructionData),
            },
        })
    }
}

impl<'info> TryFrom<&'info [AccountView]> for MeteoraDynamicBondingCurveSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [dynamic_bonding_curve_program, pool_authority, config, pool, input_token_account, output_token_account, base_vault, quote_vault, base_mint, quote_mint, payer, token_base_program, token_quote_program, referral_token_account, event_authority, program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(MeteoraDynamicBondingCurveSwapAccounts {
            dynamic_bonding_curve_program,
            pool_authority,
            config,
            pool,
            input_token_account,
            output_token_account,
            base_vault,
            quote_vault,
            base_mint,
            quote_mint,
            payer,
            token_base_program,
            token_quote_program,
            referral_token_account,
            event_authority,
            program,
        })
    }
}

impl<'info> Swap<'info> for MeteoraDynamicBondingCurve {
    type Accounts = MeteoraDynamicBondingCurveSwapAccounts<'info>;
    type Data = MeteoraDynamicBondingCurveSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let (amount_0, amount_1) = if data.swap_mode == SwapMode::ExactOut {
            (minimum_out_amount, in_amount)
        } else {
            (in_amount, minimum_out_amount)
        };

        let referral =
            if *ctx.referral_token_account.address() == METEORA_DYNAMIC_BONDING_CURVE_PROGRAM_ID {
                InstructionAccount::readonly(ctx.referral_token_account.address())
            } else {
                InstructionAccount::writable(ctx.referral_token_account.address())
            };

        let accounts = [
            InstructionAccount::readonly(ctx.pool_authority.address()),
            InstructionAccount::readonly(ctx.config.address()),
            InstructionAccount::writable(ctx.pool.address()),
            InstructionAccount::writable(ctx.input_token_account.address()),
            InstructionAccount::writable(ctx.output_token_account.address()),
            InstructionAccount::writable(ctx.base_vault.address()),
            InstructionAccount::writable(ctx.quote_vault.address()),
            InstructionAccount::readonly(ctx.base_mint.address()),
            InstructionAccount::readonly(ctx.quote_mint.address()),
            InstructionAccount::readonly_signer(ctx.payer.address()),
            InstructionAccount::readonly(ctx.token_base_program.address()),
            InstructionAccount::readonly(ctx.token_quote_program.address()),
            referral,
            InstructionAccount::readonly(ctx.event_authority.address()),
            InstructionAccount::readonly(ctx.program.address()),
        ];

        let account_infos = [
            ctx.pool_authority,
            ctx.config,
            ctx.pool,
            ctx.input_token_account,
            ctx.output_token_account,
            ctx.base_vault,
            ctx.quote_vault,
            ctx.base_mint,
            ctx.quote_mint,
            ctx.payer,
            ctx.token_base_program,
            ctx.token_quote_program,
            ctx.referral_token_account,
            ctx.event_authority,
            ctx.program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 25]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(SWAP2_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(amount_0.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(amount_1.to_le_bytes().as_ptr(), ptr.add(16), 8);
            *ptr.add(24) = data.swap_mode as u8;
        }

        let instruction = InstructionView {
            program_id: &METEORA_DYNAMIC_BONDING_CURVE_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe { instruction_data.assume_init_ref() },
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
