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

pub const TESSERA_V_PROGRAM_ID: Address = address!("TessVdML9pBGgG9yGks7o4HewRaXVAMuoVj4x83GLQH");

const SWAP_DISCRIMINATOR: u8 = 16;

pub struct TesseraV;

pub struct TesseraVSwapData {
    pub is_a_to_b: bool,
}

impl TesseraVSwapData {
    pub const DATA_LEN: usize = 1;
}

impl TryFrom<&[u8]> for TesseraVSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self {
            is_a_to_b: data[0] != 0,
        })
    }
}

impl TesseraVSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 13;
}

pub struct TesseraVSwapAccounts<'info> {
    pub tessera_v_program: &'info AccountView,
    pub global_state: &'info AccountView,
    pub market: &'info AccountView,
    pub user: &'info AccountView,
    pub vault_a: &'info AccountView,
    pub vault_b: &'info AccountView,
    pub user_ata_a: &'info AccountView,
    pub user_ata_b: &'info AccountView,
    pub mint_a: &'info AccountView,
    pub mint_b: &'info AccountView,
    pub token_program_a: &'info AccountView,
    pub token_program_b: &'info AccountView,
    pub sysvar_instructions: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for TesseraVSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [tessera_v_program, global_state, market, user, vault_a, vault_b, user_ata_a, user_ata_b, mint_a, mint_b, token_program_a, token_program_b, sysvar_instructions] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(TesseraVSwapAccounts {
            tessera_v_program,
            global_state,
            market,
            user,
            vault_a,
            vault_b,
            user_ata_a,
            user_ata_b,
            mint_a,
            mint_b,
            token_program_a,
            token_program_b,
            sysvar_instructions,
        })
    }
}

impl<'info> Swap<'info> for TesseraV {
    type Accounts = TesseraVSwapAccounts<'info>;
    type Data = TesseraVSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::readonly(ctx.global_state.address()),
            InstructionAccount::writable(ctx.market.address()),
            InstructionAccount::writable_signer(ctx.user.address()),
            InstructionAccount::writable(ctx.vault_a.address()),
            InstructionAccount::writable(ctx.vault_b.address()),
            InstructionAccount::writable(ctx.user_ata_a.address()),
            InstructionAccount::writable(ctx.user_ata_b.address()),
            InstructionAccount::readonly(ctx.mint_a.address()),
            InstructionAccount::readonly(ctx.mint_b.address()),
            InstructionAccount::readonly(ctx.token_program_a.address()),
            InstructionAccount::readonly(ctx.token_program_b.address()),
            InstructionAccount::readonly(ctx.sysvar_instructions.address()),
        ];

        let account_infos = [
            ctx.global_state,
            ctx.market,
            ctx.user,
            ctx.vault_a,
            ctx.vault_b,
            ctx.user_ata_a,
            ctx.user_ata_b,
            ctx.mint_a,
            ctx.mint_b,
            ctx.token_program_a,
            ctx.token_program_b,
            ctx.sysvar_instructions,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 18]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::write(ptr, SWAP_DISCRIMINATOR);
            core::ptr::write(ptr.add(1), data.is_a_to_b as u8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(2), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(10),
                8,
            );
        }

        let instruction = InstructionView {
            program_id: &TESSERA_V_PROGRAM_ID,
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
