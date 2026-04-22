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

pub const ORE_LST_PROGRAM_ID: Address = address!("LStwN2E5Uw6MCtuxHRLhy8RY9hxqW2XRpLzettb696y");

pub const WRAP_DISCRIMINATOR: u8 = 3;
pub const UNWRAP_DISCRIMINATOR: u8 = 2;

pub struct OreLst;

pub struct OreLstSwapData {
    pub is_wrap: bool,
}

impl OreLstSwapData {
    pub const DATA_LEN: usize = 1;
}

impl TryFrom<&[u8]> for OreLstSwapData {
    type Error = ProgramError;
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self {
            is_wrap: data[0] != 0,
        })
    }
}

impl OreLstSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 17;
}

pub struct OreLstSwapAccounts<'info> {
    pub ore_lst_program: &'info AccountView,
    pub signer: &'info AccountView,
    pub payer: &'info AccountView,
    pub sender_ore: &'info AccountView,
    pub sender_store: &'info AccountView,
    pub ore_mint: &'info AccountView,
    pub store_mint: &'info AccountView,
    pub stake: &'info AccountView,
    pub stake_tokens: &'info AccountView,
    pub treasury: &'info AccountView,
    pub treasury_tokens: &'info AccountView,
    pub vault: &'info AccountView,
    pub vault_tokens: &'info AccountView,
    pub system_program: &'info AccountView,
    pub token_program: &'info AccountView,
    pub associated_token_program: &'info AccountView,
    pub ore_stake_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for OreLstSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [ore_lst_program, signer, payer, sender_ore, sender_store, ore_mint, store_mint, stake, stake_tokens, treasury, treasury_tokens, vault, vault_tokens, system_program, token_program, associated_token_program, ore_stake_program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(OreLstSwapAccounts {
            ore_lst_program,
            signer,
            payer,
            sender_ore,
            sender_store,
            ore_mint,
            store_mint,
            stake,
            stake_tokens,
            treasury,
            treasury_tokens,
            vault,
            vault_tokens,
            system_program,
            token_program,
            associated_token_program,
            ore_stake_program,
        })
    }
}

impl<'info> Swap<'info> for OreLst {
    type Accounts = OreLstSwapAccounts<'info>;
    type Data = OreLstSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        _minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::writable_signer(ctx.signer.address()),
            InstructionAccount::writable_signer(ctx.payer.address()),
            InstructionAccount::writable(ctx.sender_ore.address()),
            InstructionAccount::writable(ctx.sender_store.address()),
            InstructionAccount::writable(ctx.ore_mint.address()),
            InstructionAccount::writable(ctx.store_mint.address()),
            InstructionAccount::writable(ctx.stake.address()),
            InstructionAccount::writable(ctx.stake_tokens.address()),
            InstructionAccount::writable(ctx.treasury.address()),
            InstructionAccount::writable(ctx.treasury_tokens.address()),
            InstructionAccount::writable(ctx.vault.address()),
            InstructionAccount::writable(ctx.vault_tokens.address()),
            InstructionAccount::readonly(ctx.system_program.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
            InstructionAccount::readonly(ctx.associated_token_program.address()),
            InstructionAccount::readonly(ctx.ore_stake_program.address()),
        ];

        let account_infos = [
            ctx.signer,
            ctx.payer,
            ctx.sender_ore,
            ctx.sender_store,
            ctx.ore_mint,
            ctx.store_mint,
            ctx.stake,
            ctx.stake_tokens,
            ctx.treasury,
            ctx.treasury_tokens,
            ctx.vault,
            ctx.vault_tokens,
            ctx.system_program,
            ctx.token_program,
            ctx.associated_token_program,
            ctx.ore_stake_program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 9]>::uninit();

        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::write(
                ptr,
                if data.is_wrap {
                    WRAP_DISCRIMINATOR
                } else {
                    UNWRAP_DISCRIMINATOR
                },
            );
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(1), 8);
        }

        let instruction = InstructionView {
            program_id: &ORE_LST_PROGRAM_ID,
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
