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

pub const METEORA_DAMM_PROGRAM_ID: Address =
    address!("Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB");

const SWAP_DISCRIMINATOR: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];

pub struct MeteoraDamm;

impl MeteoraDammSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 16;
}

pub struct MeteoraDammSwapAccounts<'info> {
    pub meteora_damm_program: &'info AccountView,
    pub pool: &'info AccountView,
    pub user_source_token: &'info AccountView,
    pub user_destination_token: &'info AccountView,
    pub a_vault: &'info AccountView,
    pub b_vault: &'info AccountView,
    pub a_token_vault: &'info AccountView,
    pub b_token_vault: &'info AccountView,
    pub a_vault_lp_mint: &'info AccountView,
    pub b_vault_lp_mint: &'info AccountView,
    pub a_vault_lp: &'info AccountView,
    pub b_vault_lp: &'info AccountView,
    pub protocol_token_fee: &'info AccountView,
    pub user: &'info AccountView,
    pub vault_program: &'info AccountView,
    pub token_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for MeteoraDammSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [meteora_damm_program, pool, user_source_token, user_destination_token, a_vault, b_vault, a_token_vault, b_token_vault, a_vault_lp_mint, b_vault_lp_mint, a_vault_lp, b_vault_lp, protocol_token_fee, user, vault_program, token_program, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(MeteoraDammSwapAccounts {
            meteora_damm_program,
            pool,
            user_source_token,
            user_destination_token,
            a_vault,
            b_vault,
            a_token_vault,
            b_token_vault,
            a_vault_lp_mint,
            b_vault_lp_mint,
            a_vault_lp,
            b_vault_lp,
            protocol_token_fee,
            user,
            vault_program,
            token_program,
        })
    }
}

impl<'info> Swap<'info> for MeteoraDamm {
    type Accounts = MeteoraDammSwapAccounts<'info>;
    type Data = ();

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        _data: &(),
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::writable(ctx.pool.address()),
            InstructionAccount::writable(ctx.user_source_token.address()),
            InstructionAccount::writable(ctx.user_destination_token.address()),
            InstructionAccount::writable(ctx.a_vault.address()),
            InstructionAccount::writable(ctx.b_vault.address()),
            InstructionAccount::writable(ctx.a_token_vault.address()),
            InstructionAccount::writable(ctx.b_token_vault.address()),
            InstructionAccount::writable(ctx.a_vault_lp_mint.address()),
            InstructionAccount::writable(ctx.b_vault_lp_mint.address()),
            InstructionAccount::writable(ctx.a_vault_lp.address()),
            InstructionAccount::writable(ctx.b_vault_lp.address()),
            InstructionAccount::writable(ctx.protocol_token_fee.address()),
            InstructionAccount::readonly_signer(ctx.user.address()),
            InstructionAccount::readonly(ctx.vault_program.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
        ];

        let account_infos = [
            ctx.pool,
            ctx.user_source_token,
            ctx.user_destination_token,
            ctx.a_vault,
            ctx.b_vault,
            ctx.a_token_vault,
            ctx.b_token_vault,
            ctx.a_vault_lp_mint,
            ctx.b_vault_lp_mint,
            ctx.a_vault_lp,
            ctx.b_vault_lp,
            ctx.protocol_token_fee,
            ctx.user,
            ctx.vault_program,
            ctx.token_program,
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
            program_id: &METEORA_DAMM_PROGRAM_ID,
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
