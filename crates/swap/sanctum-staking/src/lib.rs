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

pub const SANCTUM_STAKING_PROGRAM_ID: Address =
    address!("bon4Kh3x1uQK16w9b9DKgz3Aw4AP1pZxBJk55Q6Sosb");
const DEPOSIT_DISCRIMINATOR: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182];

pub struct SanctumStaking;

impl SanctumStakingSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 9;
}

pub struct SanctumStakingSwapAccounts<'info> {
    pub sanctum_staking_program: &'info AccountView,
    pub authority: &'info AccountView,
    pub deposit_from: &'info AccountView,
    pub mint_to: &'info AccountView,
    pub vault: &'info AccountView,
    pub bonded_mint: &'info AccountView,
    pub bond_mint_authority: &'info AccountView,
    pub bond_pool: &'info AccountView,
    pub token_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for SanctumStakingSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [sanctum_staking_program, authority, deposit_from, mint_to, vault, bonded_mint, bond_mint_authority, bond_pool, token_program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(SanctumStakingSwapAccounts {
            sanctum_staking_program,
            authority,
            deposit_from,
            mint_to,
            vault,
            bonded_mint,
            bond_mint_authority,
            bond_pool,
            token_program,
        })
    }
}

impl<'info> Swap<'info> for SanctumStaking {
    type Accounts = SanctumStakingSwapAccounts<'info>;
    type Data = ();

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        _minimum_out_amount: u64,
        _data: &(),
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let instruction_accounts = [
            InstructionAccount::readonly_signer(ctx.authority.address()),
            InstructionAccount::writable(ctx.deposit_from.address()),
            InstructionAccount::writable(ctx.mint_to.address()),
            InstructionAccount::writable(ctx.vault.address()),
            InstructionAccount::writable(ctx.bonded_mint.address()),
            InstructionAccount::readonly(ctx.bond_mint_authority.address()),
            InstructionAccount::readonly(ctx.bond_pool.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
        ];

        let account_views = [
            ctx.authority,
            ctx.deposit_from,
            ctx.mint_to,
            ctx.vault,
            ctx.bonded_mint,
            ctx.bond_mint_authority,
            ctx.bond_pool,
            ctx.token_program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 16]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(DEPOSIT_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
        }

        let instruction = InstructionView {
            program_id: &SANCTUM_STAKING_PROGRAM_ID,
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
