#![no_std]

use {
    beethoven_core::Swap,
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const OMNIPAIR_PROGRAM_ID: Address =
    Address::from_str_const("omnixgS8fnqHfCcTGKWj6JtKjzpJZ1Y5y9pyFkQDkYE");

const SWAP_DISCRIMINATOR: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];

pub struct Omnipair;

impl OmnipairSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 15;
}

pub struct OmnipairSwapAccounts<'info> {
    pub omnipair_program: &'info AccountView,
    pub pair: &'info AccountView,
    pub rate_model: &'info AccountView,
    pub futarchy_authority: &'info AccountView,
    pub token_in_vault: &'info AccountView,
    pub token_out_vault: &'info AccountView,
    pub user_token_in_account: &'info AccountView,
    pub user_token_out_account: &'info AccountView,
    pub token_in_mint: &'info AccountView,
    pub token_out_mint: &'info AccountView,
    pub user: &'info AccountView,
    pub token_program: &'info AccountView,
    pub token_2022_program: &'info AccountView,
    pub event_authority: &'info AccountView,
    pub program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for OmnipairSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < OmnipairSwapAccounts::NUM_ACCOUNTS {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [omnipair_program, pair, rate_model, futarchy_authority, token_in_vault, token_out_vault, user_token_in_account, user_token_out_account, token_in_mint, token_out_mint, user, token_program, token_2022_program, event_authority, program, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(OmnipairSwapAccounts {
            omnipair_program,
            pair,
            rate_model,
            futarchy_authority,
            token_in_vault,
            token_out_vault,
            user_token_in_account,
            user_token_out_account,
            token_in_mint,
            token_out_mint,
            user,
            token_program,
            token_2022_program,
            event_authority,
            program,
        })
    }
}

impl<'info> Swap<'info> for Omnipair {
    type Accounts = OmnipairSwapAccounts<'info>;
    type Data = ();

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        _data: &(),
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::writable(ctx.pair.address()),
            InstructionAccount::writable(ctx.rate_model.address()),
            InstructionAccount::readonly(ctx.futarchy_authority.address()),
            InstructionAccount::writable(ctx.token_in_vault.address()),
            InstructionAccount::writable(ctx.token_out_vault.address()),
            InstructionAccount::writable(ctx.user_token_in_account.address()),
            InstructionAccount::writable(ctx.user_token_out_account.address()),
            InstructionAccount::readonly(ctx.token_in_mint.address()),
            InstructionAccount::readonly(ctx.token_out_mint.address()),
            InstructionAccount::readonly_signer(ctx.user.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
            InstructionAccount::readonly(ctx.token_2022_program.address()),
            InstructionAccount::readonly(ctx.event_authority.address()),
            InstructionAccount::readonly(ctx.program.address()),
        ];

        let account_infos = [
            ctx.pair,
            ctx.rate_model,
            ctx.futarchy_authority,
            ctx.token_in_vault,
            ctx.token_out_vault,
            ctx.user_token_in_account,
            ctx.user_token_out_account,
            ctx.token_in_mint,
            ctx.token_out_mint,
            ctx.user,
            ctx.token_program,
            ctx.token_2022_program,
            ctx.event_authority,
            ctx.program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 24]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(SWAP_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(16),
                8,
            );
        }

        let instruction = InstructionView {
            program_id: &OMNIPAIR_PROGRAM_ID,
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
