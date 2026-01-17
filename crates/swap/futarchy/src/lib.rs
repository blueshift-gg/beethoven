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

pub const FUTARCHY_PROGRAM_ID: Address =
    Address::from_str_const("FUTARELBfJfQ8RDGhg1wdhddq1odMAJUePHFuBYfUxKq");

const SWAP_DISCRIMINATOR: [u8; 8] = [167, 97, 12, 231, 237, 78, 166, 251];

pub struct Futarchy;

#[repr(u8)]
pub enum SwapType {
    Buy = 0,
    Sell = 1,
}

pub struct FutarchySwapData {
    pub swap_type: SwapType,
}

impl TryFrom<&[u8]> for FutarchySwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let swap_type = match data[0] {
            0 => SwapType::Buy,
            1 => SwapType::Sell,
            _ => return Err(ProgramError::InvalidInstructionData),
        };
        Ok(Self { swap_type })
    }
}

pub struct FutarchySwapAccounts<'info> {
    pub futarchy_program: &'info AccountView,
    pub dao: &'info AccountView,
    pub user_base_account: &'info AccountView,
    pub user_quote_account: &'info AccountView,
    pub amm_base_vault: &'info AccountView,
    pub amm_quote_vault: &'info AccountView,
    pub user: &'info AccountView,
    pub token_program: &'info AccountView,
    pub event_authority: &'info AccountView,
    pub program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for FutarchySwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < 10 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [futarchy_program, dao, user_base_account, user_quote_account, amm_base_vault, amm_quote_vault, user, token_program, event_authority, program, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(FutarchySwapAccounts {
            futarchy_program,
            dao,
            user_base_account,
            user_quote_account,
            amm_base_vault,
            amm_quote_vault,
            user,
            token_program,
            event_authority,
            program,
        })
    }
}

impl<'info> Swap<'info> for Futarchy {
    type Accounts = FutarchySwapAccounts<'info>;
    type Data = FutarchySwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::writable(ctx.dao.address()),
            InstructionAccount::writable(ctx.user_base_account.address()),
            InstructionAccount::writable(ctx.user_quote_account.address()),
            InstructionAccount::writable(ctx.amm_base_vault.address()),
            InstructionAccount::writable(ctx.amm_quote_vault.address()),
            InstructionAccount::readonly_signer(ctx.user.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
            InstructionAccount::readonly(ctx.event_authority.address()),
            InstructionAccount::readonly(ctx.program.address()),
        ];

        let account_infos = [
            ctx.dao,
            ctx.user_base_account,
            ctx.user_quote_account,
            ctx.amm_base_vault,
            ctx.amm_quote_vault,
            ctx.user,
            ctx.token_program,
            ctx.event_authority,
            ctx.program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 25]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(SWAP_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            let swap_type_byte = match data.swap_type {
                SwapType::Buy => 0u8,
                SwapType::Sell => 1u8,
            };
            core::ptr::write(ptr.add(16), swap_type_byte);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(17),
                8,
            );
        }

        let instruction = InstructionView {
            program_id: &FUTARCHY_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 25)
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
