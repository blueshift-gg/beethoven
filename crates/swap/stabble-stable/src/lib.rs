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

pub const STABBLE_STABLE_PROGRAM_ID: Address =
    address!("swapNyd8XiQwJ6ianp9snpu4brUqFxadzvHebnAXjJZ");

const SWAP_V2_DISCRIMINATOR: [u8; 8] = [43, 4, 237, 11, 26, 201, 30, 98];

pub struct StabbleStable;

impl StabbleStableSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 16;
}

pub struct StabbleStableSwapAccounts<'info> {
    pub stabble_program: &'info AccountView,
    pub user: &'info AccountView,
    pub mint_in: &'info AccountView,
    pub mint_out: &'info AccountView,
    pub user_token_in: &'info AccountView,
    pub user_token_out: &'info AccountView,
    pub vault_token_in: &'info AccountView,
    pub vault_token_out: &'info AccountView,
    pub beneficiary_token_out: &'info AccountView,
    pub pool: &'info AccountView,
    pub withdraw_authority: &'info AccountView,
    pub vault: &'info AccountView,
    pub vault_authority: &'info AccountView,
    pub vault_program: &'info AccountView,
    pub token_program: &'info AccountView,
    pub token_2022_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for StabbleStableSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [stabble_program, user, mint_in, mint_out, user_token_in, user_token_out, vault_token_in, vault_token_out, beneficiary_token_out, pool, withdraw_authority, vault, vault_authority, vault_program, token_program, token_2022_program, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(StabbleStableSwapAccounts {
            stabble_program,
            user,
            mint_in,
            mint_out,
            user_token_in,
            user_token_out,
            vault_token_in,
            vault_token_out,
            beneficiary_token_out,
            pool,
            withdraw_authority,
            vault,
            vault_authority,
            vault_program,
            token_program,
            token_2022_program,
        })
    }
}

impl<'info> Swap<'info> for StabbleStable {
    type Accounts = StabbleStableSwapAccounts<'info>;
    type Data = ();

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        _data: &(),
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::writable_signer(ctx.user.address()),
            InstructionAccount::readonly(ctx.mint_in.address()),
            InstructionAccount::readonly(ctx.mint_out.address()),
            InstructionAccount::writable(ctx.user_token_in.address()),
            InstructionAccount::writable(ctx.user_token_out.address()),
            InstructionAccount::writable(ctx.vault_token_in.address()),
            InstructionAccount::writable(ctx.vault_token_out.address()),
            InstructionAccount::writable(ctx.beneficiary_token_out.address()),
            InstructionAccount::writable(ctx.pool.address()),
            InstructionAccount::readonly(ctx.withdraw_authority.address()),
            InstructionAccount::readonly(ctx.vault.address()),
            InstructionAccount::readonly(ctx.vault_authority.address()),
            InstructionAccount::readonly(ctx.vault_program.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
            InstructionAccount::readonly(ctx.token_2022_program.address()),
        ];

        let account_infos = [
            ctx.user,
            ctx.mint_in,
            ctx.mint_out,
            ctx.user_token_in,
            ctx.user_token_out,
            ctx.vault_token_in,
            ctx.vault_token_out,
            ctx.beneficiary_token_out,
            ctx.pool,
            ctx.withdraw_authority,
            ctx.vault,
            ctx.vault_authority,
            ctx.vault_program,
            ctx.token_program,
            ctx.token_2022_program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 25]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(SWAP_V2_DISCRIMINATOR.as_ptr(), ptr, 8);
            *ptr.add(8) = 1u8;
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(9), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(17),
                8,
            );
        }

        let instruction = InstructionView {
            program_id: &STABBLE_STABLE_PROGRAM_ID,
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
