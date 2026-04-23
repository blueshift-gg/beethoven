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

pub const SOLV_FINANCE_PROGRAM_ID: Address =
    address!("soLv1S6GsAEVEnXmVY3oz6GtrNJteQ28iTyRQrHXvkz");

const VAULT_DEPOSIT_DISCRIMINATOR: u8 = 0;

pub struct SolvFinance;

impl SolvFinanceSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 11;
}

pub struct SolvFinanceSwapAccounts<'info> {
    pub solv_finance_program: &'info AccountView,
    pub user: &'info AccountView,
    pub user_token_ta: &'info AccountView,
    pub user_target_ta: &'info AccountView,
    pub treasurer_token_ta: &'info AccountView,
    pub multisig: &'info AccountView,
    pub mint_token: &'info AccountView,
    pub mint_target: &'info AccountView,
    pub vault: &'info AccountView,
    pub token_program: &'info AccountView,
    pub associated_token_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for SolvFinanceSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [solv_finance_program, user, user_token_ta, user_target_ta, treasurer_token_ta, multisig, mint_token, mint_target, vault, token_program, associated_token_program, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(SolvFinanceSwapAccounts {
            solv_finance_program,
            user,
            user_token_ta,
            user_target_ta,
            treasurer_token_ta,
            multisig,
            mint_token,
            mint_target,
            vault,
            token_program,
            associated_token_program,
        })
    }
}

impl<'info> Swap<'info> for SolvFinance {
    type Accounts = SolvFinanceSwapAccounts<'info>;
    type Data = ();

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        _data: &(),
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let instruction_accounts = [
            InstructionAccount::writable_signer(ctx.user.address()),
            InstructionAccount::writable(ctx.user_token_ta.address()),
            InstructionAccount::writable(ctx.user_target_ta.address()),
            InstructionAccount::writable(ctx.treasurer_token_ta.address()),
            InstructionAccount::readonly(ctx.multisig.address()),
            InstructionAccount::readonly(ctx.mint_token.address()),
            InstructionAccount::writable(ctx.mint_target.address()),
            InstructionAccount::writable(ctx.vault.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
            InstructionAccount::readonly(ctx.associated_token_program.address()),
        ];

        let account_views = [
            ctx.user,
            ctx.user_token_ta,
            ctx.user_target_ta,
            ctx.treasurer_token_ta,
            ctx.multisig,
            ctx.mint_token,
            ctx.mint_target,
            ctx.vault,
            ctx.token_program,
            ctx.associated_token_program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 17]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::write(ptr, VAULT_DEPOSIT_DISCRIMINATOR);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(1), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(9),
                8,
            );
        }

        let instruction = InstructionView {
            program_id: &SOLV_FINANCE_PROGRAM_ID,
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
