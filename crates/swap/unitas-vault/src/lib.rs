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

pub const UNITAS_VAULT_PROGRAM_ID: Address =
    address!("VALT7AM76ZWfRhjVeYQRrLvNRLvqBzNs8dTsAcLW3jj");

const STAKE_USDU_MINT_SUSDU_DISCRIMINATOR: [u8; 8] = [20, 15, 120, 241, 40, 12, 245, 17];

pub struct UnitasVault;

impl UnitasVaultSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 18;
}

pub struct UnitasVaultSwapAccounts<'info> {
    pub unitas_vault_program: &'info AccountView,
    pub caller: &'info AccountView,
    pub receiver: &'info AccountView,
    pub receiver_susdu_token_account: &'info AccountView,
    pub caller_usdu_token_account: &'info AccountView,
    pub access_registry: &'info AccountView,
    pub vault_stake_pool_usdu_token_account: &'info AccountView,
    pub susdu_minter: &'info AccountView,
    pub usdu_token: &'info AccountView,
    pub susdu_token: &'info AccountView,
    pub vault_state: &'info AccountView,
    pub vault_config: &'info AccountView,
    pub susdu_config: &'info AccountView,
    pub susdu_program: &'info AccountView,
    pub usdu_token_program: &'info AccountView,
    pub susdu_token_program: &'info AccountView,
    pub associated_token_program: &'info AccountView,
    pub system_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for UnitasVaultSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [unitas_vault_program, caller, receiver, receiver_susdu_token_account, caller_usdu_token_account, access_registry, vault_stake_pool_usdu_token_account, susdu_minter, usdu_token, susdu_token, vault_state, vault_config, susdu_config, susdu_program, usdu_token_program, susdu_token_program, associated_token_program, system_program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(UnitasVaultSwapAccounts {
            unitas_vault_program,
            caller,
            receiver,
            receiver_susdu_token_account,
            caller_usdu_token_account,
            access_registry,
            vault_stake_pool_usdu_token_account,
            susdu_minter,
            usdu_token,
            susdu_token,
            vault_state,
            vault_config,
            susdu_config,
            susdu_program,
            usdu_token_program,
            susdu_token_program,
            associated_token_program,
            system_program,
        })
    }
}

impl<'info> Swap<'info> for UnitasVault {
    type Accounts = UnitasVaultSwapAccounts<'info>;
    type Data = ();

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        _minimum_out_amount: u64,
        _data: &(),
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let instruction_accounts = [
            InstructionAccount::writable_signer(ctx.caller.address()),
            InstructionAccount::writable_signer(ctx.receiver.address()),
            InstructionAccount::writable(ctx.receiver_susdu_token_account.address()),
            InstructionAccount::writable(ctx.caller_usdu_token_account.address()),
            InstructionAccount::readonly(ctx.access_registry.address()),
            InstructionAccount::writable(ctx.vault_stake_pool_usdu_token_account.address()),
            InstructionAccount::readonly(ctx.susdu_minter.address()),
            InstructionAccount::writable(ctx.usdu_token.address()),
            InstructionAccount::writable(ctx.susdu_token.address()),
            InstructionAccount::readonly(ctx.vault_state.address()),
            InstructionAccount::writable(ctx.vault_config.address()),
            InstructionAccount::writable(ctx.susdu_config.address()),
            InstructionAccount::readonly(ctx.susdu_program.address()),
            InstructionAccount::readonly(ctx.usdu_token_program.address()),
            InstructionAccount::readonly(ctx.susdu_token_program.address()),
            InstructionAccount::readonly(ctx.associated_token_program.address()),
            InstructionAccount::readonly(ctx.system_program.address()),
        ];

        let account_views = [
            ctx.caller,
            ctx.receiver,
            ctx.receiver_susdu_token_account,
            ctx.caller_usdu_token_account,
            ctx.access_registry,
            ctx.vault_stake_pool_usdu_token_account,
            ctx.susdu_minter,
            ctx.usdu_token,
            ctx.susdu_token,
            ctx.vault_state,
            ctx.vault_config,
            ctx.susdu_config,
            ctx.susdu_program,
            ctx.usdu_token_program,
            ctx.susdu_token_program,
            ctx.associated_token_program,
            ctx.system_program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 16]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(STAKE_USDU_MINT_SUSDU_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
        }

        let instruction = InstructionView {
            program_id: ctx.unitas_vault_program.address(),
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
