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

pub const MANIFEST_PROGRAM_ID: Address =
    Address::from_str_const("MNFSTqtC93rEfYHB6hF82sKdZpUDFWkViLByLd1k1Ms");

const SWAP_DISCRIMINATOR: u8 = 13;

pub struct Manifest;

pub struct ManifestSwapData {
    pub is_base_in: bool,
    pub is_exact_in: bool,
}

impl TryFrom<&[u8]> for ManifestSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < 2 {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self {
            is_base_in: data[0] != 0,
            is_exact_in: data[1] != 0,
        })
    }
}

pub struct ManifestSwapAccounts<'info> {
    pub manifest_program: &'info AccountView,
    pub payer: &'info AccountView,
    pub owner: &'info AccountView,
    pub market: &'info AccountView,
    pub system_program: &'info AccountView,
    pub trader_base: &'info AccountView,
    pub trader_quote: &'info AccountView,
    pub base_vault: &'info AccountView,
    pub quote_vault: &'info AccountView,
    pub token_program_base: &'info AccountView,
    pub base_mint: &'info AccountView,
    pub token_program_quote: &'info AccountView,
    pub quote_mint: &'info AccountView,
    pub global: &'info AccountView,
    pub global_vault: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for ManifestSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < 15 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [manifest_program, payer, owner, market, system_program, trader_base, trader_quote, base_vault, quote_vault, token_program_base, base_mint, token_program_quote, quote_mint, global, global_vault, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(ManifestSwapAccounts {
            manifest_program,
            payer,
            owner,
            market,
            system_program,
            trader_base,
            trader_quote,
            base_vault,
            quote_vault,
            token_program_base,
            base_mint,
            token_program_quote,
            quote_mint,
            global,
            global_vault,
        })
    }
}

impl<'info> Swap<'info> for Manifest {
    type Accounts = ManifestSwapAccounts<'info>;
    type Data = ManifestSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::writable_signer(ctx.payer.address()),
            InstructionAccount::readonly_signer(ctx.owner.address()),
            InstructionAccount::writable(ctx.market.address()),
            InstructionAccount::readonly(ctx.system_program.address()),
            InstructionAccount::writable(ctx.trader_base.address()),
            InstructionAccount::writable(ctx.trader_quote.address()),
            InstructionAccount::writable(ctx.base_vault.address()),
            InstructionAccount::writable(ctx.quote_vault.address()),
            InstructionAccount::readonly(ctx.token_program_base.address()),
            InstructionAccount::readonly(ctx.base_mint.address()),
            InstructionAccount::readonly(ctx.token_program_quote.address()),
            InstructionAccount::readonly(ctx.quote_mint.address()),
            InstructionAccount::writable(ctx.global.address()),
            InstructionAccount::writable(ctx.global_vault.address()),
        ];

        let account_infos = [
            ctx.payer,
            ctx.owner,
            ctx.market,
            ctx.system_program,
            ctx.trader_base,
            ctx.trader_quote,
            ctx.base_vault,
            ctx.quote_vault,
            ctx.token_program_base,
            ctx.base_mint,
            ctx.token_program_quote,
            ctx.quote_mint,
            ctx.global,
            ctx.global_vault,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 19]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::write(ptr, SWAP_DISCRIMINATOR);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(1), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(9),
                8,
            );
            core::ptr::write(ptr.add(17), data.is_base_in as u8);
            core::ptr::write(ptr.add(18), data.is_exact_in as u8);
        }

        let instruction = InstructionView {
            program_id: &MANIFEST_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 19)
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
