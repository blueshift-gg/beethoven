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

pub const HUMA_FINANCE_PROGRAM_ID: Address =
    address!("HumaXepHnjaRCpjYTokxY4UtaJcmx41prQ8cxGmFC5fn");

const DEPOSIT_DISCRIMINATOR: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182];

// "NO_COMMITMENT" borsh-encoded string: 4-byte LE length + 13 UTF-8 bytes
const NO_COMMITMENT_LEN: [u8; 4] = [13, 0, 0, 0];
const NO_COMMITMENT_BYTES: [u8; 13] = [78, 79, 95, 67, 79, 77, 77, 73, 84, 77, 69, 78, 84];

pub struct HumaFinance;

pub struct HumaFinanceSwapAccounts<'info> {
    pub depositor: &'info AccountView,
    pub huma_config: &'info AccountView,
    pub pool_config: &'info AccountView,
    pub pool_state: &'info AccountView,
    pub mode_config: &'info AccountView,
    pub mode_mint: &'info AccountView,
    pub pool_authority: &'info AccountView,
    pub underlying_mint: &'info AccountView,
    pub pool_underlying_token: &'info AccountView,
    pub depositor_underlying_token: &'info AccountView,
    pub depositor_mode_token: &'info AccountView,
    pub underlying_token_program: &'info AccountView,
    pub mode_token_program: &'info AccountView,
}

impl HumaFinanceSwapAccounts<'_> {
    pub const MIN_NUM_ACCOUNTS: usize = 14;
}

impl<'info> TryFrom<&'info [AccountView]> for HumaFinanceSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [_huma_finance_program, depositor, huma_config, pool_config, pool_state, mode_config, mode_mint, pool_authority, underlying_mint, pool_underlying_token, depositor_underlying_token, depositor_mode_token, underlying_token_program, mode_token_program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(HumaFinanceSwapAccounts {
            depositor,
            huma_config,
            pool_config,
            pool_state,
            mode_config,
            mode_mint,
            pool_authority,
            underlying_mint,
            pool_underlying_token,
            depositor_underlying_token,
            depositor_mode_token,
            underlying_token_program,
            mode_token_program,
        })
    }
}

impl<'info> Swap<'info> for HumaFinance {
    type Accounts = HumaFinanceSwapAccounts<'info>;
    type Data = ();

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        _minimum_out_amount: u64,
        _data: &(),
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let instruction_accounts = [
            InstructionAccount::writable_signer(ctx.depositor.address()),
            InstructionAccount::readonly(ctx.huma_config.address()),
            InstructionAccount::readonly(ctx.pool_config.address()),
            InstructionAccount::writable(ctx.pool_state.address()),
            InstructionAccount::readonly(ctx.mode_config.address()),
            InstructionAccount::writable(ctx.mode_mint.address()),
            InstructionAccount::readonly(ctx.pool_authority.address()),
            InstructionAccount::readonly(ctx.underlying_mint.address()),
            InstructionAccount::writable(ctx.pool_underlying_token.address()),
            InstructionAccount::writable(ctx.depositor_underlying_token.address()),
            InstructionAccount::writable(ctx.depositor_mode_token.address()),
            InstructionAccount::readonly(ctx.underlying_token_program.address()),
            InstructionAccount::readonly(ctx.mode_token_program.address()),
        ];

        let account_views = [
            ctx.depositor,
            ctx.huma_config,
            ctx.pool_config,
            ctx.pool_state,
            ctx.mode_config,
            ctx.mode_mint,
            ctx.pool_authority,
            ctx.underlying_mint,
            ctx.pool_underlying_token,
            ctx.depositor_underlying_token,
            ctx.depositor_mode_token,
            ctx.underlying_token_program,
            ctx.mode_token_program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 34]>::uninit();

        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(DEPOSIT_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(NO_COMMITMENT_LEN.as_ptr(), ptr.add(16), 4);
            core::ptr::copy_nonoverlapping(NO_COMMITMENT_BYTES.as_ptr(), ptr.add(20), 13);
            // commitment_auto_renewal = false
            core::ptr::write(ptr.add(33), 0);
        }

        let instruction = InstructionView {
            program_id: &HUMA_FINANCE_PROGRAM_ID,
            accounts: &instruction_accounts,
            data: unsafe { instruction_data.assume_init_ref() },
        };

        invoke_signed(&instruction, &account_views, signer_seeds)
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
