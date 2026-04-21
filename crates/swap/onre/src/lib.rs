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

pub const ONRE_PROGRAM_ID: Address = address!("onreuGhHHgVzMWSkj2oQDLDtvvGvoepBPkqyaubFcwe");

const TAKE_OFFER_PERMISSIONLESS_DISCRIMINATOR: [u8; 8] = [37, 190, 224, 77, 197, 39, 203, 230];

pub struct Onre;

impl OnreSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 22;
}

pub struct OnreSwapAccounts<'info> {
    pub onre_program: &'info AccountView,
    pub offer: &'info AccountView,
    pub state: &'info AccountView,
    pub boss: &'info AccountView,
    pub vault_authority: &'info AccountView,
    pub vault_token_in_account: &'info AccountView,
    pub vault_token_out_account: &'info AccountView,
    pub permissionless_authority: &'info AccountView,
    pub permissionless_token_in_account: &'info AccountView,
    pub permissionless_token_out_account: &'info AccountView,
    pub token_in_mint: &'info AccountView,
    pub token_in_program: &'info AccountView,
    pub token_out_mint: &'info AccountView,
    pub token_out_program: &'info AccountView,
    pub user_token_in_account: &'info AccountView,
    pub user_token_out_account: &'info AccountView,
    pub boss_token_in_account: &'info AccountView,
    pub mint_authority: &'info AccountView,
    pub instructions_sysvar: &'info AccountView,
    pub user: &'info AccountView,
    pub associated_token_program: &'info AccountView,
    pub system_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for OnreSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [onre_program, offer, state, boss, vault_authority, vault_token_in_account, vault_token_out_account, permissionless_authority, permissionless_token_in_account, permissionless_token_out_account, token_in_mint, token_in_program, token_out_mint, token_out_program, user_token_in_account, user_token_out_account, boss_token_in_account, mint_authority, instructions_sysvar, user, associated_token_program, system_program, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(OnreSwapAccounts {
            onre_program,
            offer,
            state,
            boss,
            vault_authority,
            vault_token_in_account,
            vault_token_out_account,
            permissionless_authority,
            permissionless_token_in_account,
            permissionless_token_out_account,
            token_in_mint,
            token_in_program,
            token_out_mint,
            token_out_program,
            user_token_in_account,
            user_token_out_account,
            boss_token_in_account,
            mint_authority,
            instructions_sysvar,
            user,
            associated_token_program,
            system_program,
        })
    }
}

impl<'info> Swap<'info> for Onre {
    type Accounts = OnreSwapAccounts<'info>;
    type Data = ();

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        _minimum_out_amount: u64,
        _data: &(),
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::writable(ctx.offer.address()),
            InstructionAccount::readonly(ctx.state.address()),
            InstructionAccount::readonly(ctx.boss.address()),
            InstructionAccount::readonly(ctx.vault_authority.address()),
            InstructionAccount::writable(ctx.vault_token_in_account.address()),
            InstructionAccount::writable(ctx.vault_token_out_account.address()),
            InstructionAccount::readonly(ctx.permissionless_authority.address()),
            InstructionAccount::writable(ctx.permissionless_token_in_account.address()),
            InstructionAccount::writable(ctx.permissionless_token_out_account.address()),
            InstructionAccount::writable(ctx.token_in_mint.address()),
            InstructionAccount::readonly(ctx.token_in_program.address()),
            InstructionAccount::writable(ctx.token_out_mint.address()),
            InstructionAccount::readonly(ctx.token_out_program.address()),
            InstructionAccount::writable(ctx.user_token_in_account.address()),
            InstructionAccount::writable(ctx.user_token_out_account.address()),
            InstructionAccount::writable(ctx.boss_token_in_account.address()),
            InstructionAccount::readonly(ctx.mint_authority.address()),
            InstructionAccount::readonly(ctx.instructions_sysvar.address()),
            InstructionAccount::writable_signer(ctx.user.address()),
            InstructionAccount::readonly(ctx.associated_token_program.address()),
            InstructionAccount::readonly(ctx.system_program.address()),
        ];

        let account_infos = [
            ctx.offer,
            ctx.state,
            ctx.boss,
            ctx.vault_authority,
            ctx.vault_token_in_account,
            ctx.vault_token_out_account,
            ctx.permissionless_authority,
            ctx.permissionless_token_in_account,
            ctx.permissionless_token_out_account,
            ctx.token_in_mint,
            ctx.token_in_program,
            ctx.token_out_mint,
            ctx.token_out_program,
            ctx.user_token_in_account,
            ctx.user_token_out_account,
            ctx.boss_token_in_account,
            ctx.mint_authority,
            ctx.instructions_sysvar,
            ctx.user,
            ctx.associated_token_program,
            ctx.system_program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 17]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(
                TAKE_OFFER_PERMISSIONLESS_DISCRIMINATOR.as_ptr(),
                ptr,
                8,
            );
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            // approval_message - None
            core::ptr::write(ptr.add(16), 0u8);
        }

        let instruction = InstructionView {
            program_id: &ONRE_PROGRAM_ID,
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
