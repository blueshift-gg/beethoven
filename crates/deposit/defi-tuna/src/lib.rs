#![no_std]

use {
    beethoven_core::Deposit,
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::{address, Address},
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const DEFI_TUNA_PROGRAM_ID: Address = address!("tuna4uSQZncNeeiAMKbstuxA9CUkHH6HmC64wgmnogD");
pub const DEPOSIT_DISCRIMINATOR: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182];

pub struct DefiTuna;

pub struct DefiTunaDepositAccounts<'info> {
    pub defi_tuna_program: &'info AccountView,
    pub authority: &'info AccountView,
    pub mint: &'info AccountView,
    pub tuna_config: &'info AccountView,
    pub lending_position: &'info AccountView,
    pub vault: &'info AccountView,
    pub vault_ata: &'info AccountView,
    pub authority_ata: &'info AccountView,
    pub token_program: &'info AccountView,
    pub memo_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for DefiTunaDepositAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [defi_tuna_program, authority, mint, tuna_config, lending_position, vault, vault_ata, authority_ata, token_program, memo_program, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(DefiTunaDepositAccounts {
            defi_tuna_program,
            authority,
            mint,
            tuna_config,
            lending_position,
            vault,
            vault_ata,
            authority_ata,
            token_program,
            memo_program,
        })
    }
}

impl<'info> Deposit<'info> for DefiTuna {
    type Accounts = DefiTunaDepositAccounts<'info>;
    type Data = ();

    fn deposit_signed(
        ctx: &DefiTunaDepositAccounts<'info>,
        amount: u64,
        _data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::writable_signer(ctx.authority.address()),
            InstructionAccount::readonly(ctx.mint.address()),
            InstructionAccount::readonly(ctx.tuna_config.address()),
            InstructionAccount::writable(ctx.lending_position.address()),
            InstructionAccount::writable(ctx.vault.address()),
            InstructionAccount::writable(ctx.vault_ata.address()),
            InstructionAccount::writable(ctx.authority_ata.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
            InstructionAccount::readonly(ctx.memo_program.address()),
        ];

        let account_infos = [
            ctx.authority,
            ctx.mint,
            ctx.tuna_config,
            ctx.lending_position,
            ctx.vault,
            ctx.vault_ata,
            ctx.authority_ata,
            ctx.token_program,
            ctx.memo_program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 16]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(DEPOSIT_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
        }

        let deposit_ix = InstructionView {
            program_id: &DEFI_TUNA_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe { instruction_data.assume_init_ref() },
        };

        invoke_signed(&deposit_ix, &account_infos, signer_seeds)
    }

    fn deposit(
        ctx: &DefiTunaDepositAccounts<'info>,
        amount: u64,
        data: &Self::Data,
    ) -> ProgramResult {
        Self::deposit_signed(ctx, amount, data, &[])
    }
}
