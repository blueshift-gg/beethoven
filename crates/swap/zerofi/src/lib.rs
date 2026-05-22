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

pub const ZEROFI_PROGRAM_ID: Address = address!("ZERor4xhbUycZ6gb9ntrhqscUcZmAbQDjEAtCf4hbZY");

const SWAP_DISCRIMINATOR: u8 = 6;

pub struct Zerofi;

impl ZerofiSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 10;
}

pub struct ZerofiSwapAccounts<'info> {
    pub zerofi_program: &'info AccountView,
    pub market: &'info AccountView,
    pub cfg_in: &'info AccountView,
    pub ta_in: &'info AccountView,
    pub cfg_out: &'info AccountView,
    pub ta_out: &'info AccountView,
    pub usr_ta_in: &'info AccountView,
    pub usr_ta_out: &'info AccountView,
    pub token_program: &'info AccountView,
    pub sysvar_instructions: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for ZerofiSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [zerofi_program, market, cfg_in, ta_in, cfg_out, ta_out, usr_ta_in, usr_ta_out, token_program, sysvar_instructions] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(ZerofiSwapAccounts {
            zerofi_program,
            market,
            cfg_in,
            ta_in,
            cfg_out,
            ta_out,
            usr_ta_in,
            usr_ta_out,
            token_program,
            sysvar_instructions,
        })
    }
}

impl<'info> Swap<'info> for Zerofi {
    type Accounts = ZerofiSwapAccounts<'info>;
    type Data = ();

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        _data: &(),
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::writable(ctx.market.address()),
            InstructionAccount::writable(ctx.cfg_in.address()),
            InstructionAccount::writable(ctx.ta_in.address()),
            InstructionAccount::writable(ctx.cfg_out.address()),
            InstructionAccount::writable(ctx.ta_out.address()),
            InstructionAccount::writable(ctx.usr_ta_in.address()),
            InstructionAccount::writable(ctx.usr_ta_out.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
            InstructionAccount::readonly(ctx.sysvar_instructions.address()),
        ];

        let account_infos = [
            ctx.market,
            ctx.cfg_in,
            ctx.ta_in,
            ctx.cfg_out,
            ctx.ta_out,
            ctx.usr_ta_in,
            ctx.usr_ta_out,
            ctx.token_program,
            ctx.sysvar_instructions,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 17]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::write(ptr, SWAP_DISCRIMINATOR);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(1), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(9),
                8,
            );
        }

        let instruction = InstructionView {
            program_id: &ZEROFI_PROGRAM_ID,
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
