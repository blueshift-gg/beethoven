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

pub const SOLFI_PROGRAM_ID: Address =
    Address::from_str_const("SoLFiHG9TfgtdUXUjWAxi3LtvYuFyDLVhBWxdMZxyCe");

const SWAP_DISCRIMINATOR: u8 = 7;

pub struct SolFi;

pub struct SolFiSwapData {
    pub is_quote_to_base: bool,
}

impl TryFrom<&[u8]> for SolFiSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self {
            is_quote_to_base: data[0] != 0,
        })
    }
}

pub struct SolFiSwapAccounts<'info> {
    pub solfi_program: &'info AccountView,
    pub token_transfer_authority: &'info AccountView,
    pub market_account: &'info AccountView,
    pub base_vault: &'info AccountView,
    pub quote_vault: &'info AccountView,
    pub user_base_ata: &'info AccountView,
    pub user_quote_ata: &'info AccountView,
    pub token_program: &'info AccountView,
    pub instructions_sysvar: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for SolFiSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < 9 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [solfi_program, token_transfer_authority, market_account, base_vault, quote_vault, user_base_ata, user_quote_ata, token_program, instructions_sysvar, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(SolFiSwapAccounts {
            solfi_program,
            token_transfer_authority,
            market_account,
            base_vault,
            quote_vault,
            user_base_ata,
            user_quote_ata,
            token_program,
            instructions_sysvar,
        })
    }
}

impl<'info> Swap<'info> for SolFi {
    type Accounts = SolFiSwapAccounts<'info>;
    type Data = SolFiSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::writable_signer(ctx.token_transfer_authority.address()),
            InstructionAccount::writable(ctx.market_account.address()),
            InstructionAccount::writable(ctx.base_vault.address()),
            InstructionAccount::writable(ctx.quote_vault.address()),
            InstructionAccount::writable(ctx.user_base_ata.address()),
            InstructionAccount::writable(ctx.user_quote_ata.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
            InstructionAccount::readonly(ctx.instructions_sysvar.address()),
        ];

        let account_infos = [
            ctx.token_transfer_authority,
            ctx.market_account,
            ctx.base_vault,
            ctx.quote_vault,
            ctx.user_base_ata,
            ctx.user_quote_ata,
            ctx.token_program,
            ctx.instructions_sysvar,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 18]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::write(ptr, SWAP_DISCRIMINATOR);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(1), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(9),
                8,
            );
            core::ptr::write(ptr.add(17), data.is_quote_to_base as u8);
        }

        let instruction = InstructionView {
            program_id: &SOLFI_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 18)
            },
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
