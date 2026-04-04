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

pub const SAROS_AMM_PROGRAM_ID: Address = address!("SSwapUtytfBdBn1b9NUGG6foMVPtcWgpRU32HToDUZr");

const SWAP_DISCRIMINATOR: u8 = 1;

pub struct SarosAmm;

impl SarosAmmSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 11;
}

pub struct SarosAmmSwapAccounts<'info> {
    pub saros_program: &'info AccountView,
    pub swap_info: &'info AccountView,
    pub authority_info: &'info AccountView,
    pub user_transfer_authority_info: &'info AccountView,
    pub source_info: &'info AccountView,
    pub swap_source_info: &'info AccountView,
    pub swap_destination_info: &'info AccountView,
    pub destination_info: &'info AccountView,
    pub pool_mint_info: &'info AccountView,
    pub pool_fee_account_info: &'info AccountView,
    pub token_program_info: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for SarosAmmSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [saros_program, swap_info, authority_info, user_transfer_authority_info, source_info, swap_source_info, swap_destination_info, destination_info, pool_mint_info, pool_fee_account_info, token_program_info, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(SarosAmmSwapAccounts {
            saros_program,
            swap_info,
            authority_info,
            user_transfer_authority_info,
            source_info,
            swap_source_info,
            swap_destination_info,
            destination_info,
            pool_mint_info,
            pool_fee_account_info,
            token_program_info,
        })
    }
}

impl<'info> Swap<'info> for SarosAmm {
    type Accounts = SarosAmmSwapAccounts<'info>;
    type Data = ();

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        _data: &(),
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::readonly(ctx.swap_info.address()),
            InstructionAccount::readonly(ctx.authority_info.address()),
            InstructionAccount::writable_signer(ctx.user_transfer_authority_info.address()),
            InstructionAccount::writable(ctx.source_info.address()),
            InstructionAccount::writable(ctx.swap_source_info.address()),
            InstructionAccount::writable(ctx.swap_destination_info.address()),
            InstructionAccount::writable(ctx.destination_info.address()),
            InstructionAccount::writable(ctx.pool_mint_info.address()),
            InstructionAccount::writable(ctx.pool_fee_account_info.address()),
            InstructionAccount::readonly(ctx.token_program_info.address()),
        ];

        let account_infos = [
            ctx.swap_info,
            ctx.authority_info,
            ctx.user_transfer_authority_info,
            ctx.source_info,
            ctx.swap_source_info,
            ctx.swap_destination_info,
            ctx.destination_info,
            ctx.pool_mint_info,
            ctx.pool_fee_account_info,
            ctx.token_program_info,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 17]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            *ptr = SWAP_DISCRIMINATOR;
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(1), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(9),
                8,
            );
        }

        let instruction = InstructionView {
            program_id: &SAROS_AMM_PROGRAM_ID,
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
