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

pub const XORCA_PROGRAM_ID: Address = address!("StaKE6XNKVVhG8Qu9hDJBqCW3eRe7MDGLz17nJZetLT");
const STAKE_DISCRIMINATOR: u8 = 0;

pub struct Xorca;

impl XorcaSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 9;
}

pub struct XorcaSwapAccounts<'info> {
    pub xorca_program: &'info AccountView,
    pub staker: &'info AccountView,
    pub vault: &'info AccountView,
    pub staker_orca_ata: &'info AccountView,
    pub staker_xorca_ata: &'info AccountView,
    pub xorca_mint: &'info AccountView,
    pub state: &'info AccountView,
    pub orca_mint: &'info AccountView,
    pub token_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for XorcaSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [xorca_program, staker, vault, staker_orca_ata, staker_xorca_ata, xorca_mint, state, orca_mint, token_program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(XorcaSwapAccounts {
            xorca_program,
            staker,
            vault,
            staker_orca_ata,
            staker_xorca_ata,
            xorca_mint,
            state,
            orca_mint,
            token_program,
        })
    }
}

impl<'info> Swap<'info> for Xorca {
    type Accounts = XorcaSwapAccounts<'info>;
    type Data = ();

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        _minimum_out_amount: u64,
        _data: &(),
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let instruction_accounts = [
            InstructionAccount::writable_signer(ctx.staker.address()),
            InstructionAccount::writable(ctx.vault.address()),
            InstructionAccount::writable(ctx.staker_orca_ata.address()),
            InstructionAccount::writable(ctx.staker_xorca_ata.address()),
            InstructionAccount::writable(ctx.xorca_mint.address()),
            InstructionAccount::readonly(ctx.state.address()),
            InstructionAccount::readonly(ctx.orca_mint.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
        ];

        let account_views = [
            ctx.staker,
            ctx.vault,
            ctx.staker_orca_ata,
            ctx.staker_xorca_ata,
            ctx.xorca_mint,
            ctx.state,
            ctx.orca_mint,
            ctx.token_program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 9]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::write(ptr, STAKE_DISCRIMINATOR);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(1), 8);
        }

        let instruction = InstructionView {
            program_id: &XORCA_PROGRAM_ID,
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
