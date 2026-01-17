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

pub const ALDRIN_PROGRAM_ID: Address =
    Address::from_str_const("AMM55ShdkoGRB5jVYPjWziwk8m5MpwyDgsMWHaMSQWH6");

const SWAP_DISCRIMINATOR: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];

pub struct Aldrin;

#[repr(u8)]
pub enum Side {
    Bid = 0,
    Ask = 1,
}

pub struct AldrinSwapData {
    pub side: Side,
}

impl TryFrom<&[u8]> for AldrinSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let side = match data[0] {
            0 => Side::Bid,
            1 => Side::Ask,
            _ => return Err(ProgramError::InvalidInstructionData),
        };
        Ok(Self { side })
    }
}

pub struct AldrinSwapAccounts<'info> {
    pub aldrin_program: &'info AccountView,
    pub pool: &'info AccountView,
    pub pool_signer: &'info AccountView,
    pub pool_mint: &'info AccountView,
    pub base_token_vault: &'info AccountView,
    pub quote_token_vault: &'info AccountView,
    pub fee_pool_token_account: &'info AccountView,
    pub wallet_authority: &'info AccountView,
    pub user_base_token_account: &'info AccountView,
    pub user_quote_token_account: &'info AccountView,
    pub token_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for AldrinSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < 11 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [aldrin_program, pool, pool_signer, pool_mint, base_token_vault, quote_token_vault, fee_pool_token_account, wallet_authority, user_base_token_account, user_quote_token_account, token_program, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(AldrinSwapAccounts {
            aldrin_program,
            pool,
            pool_signer,
            pool_mint,
            base_token_vault,
            quote_token_vault,
            fee_pool_token_account,
            wallet_authority,
            user_base_token_account,
            user_quote_token_account,
            token_program,
        })
    }
}

impl<'info> Swap<'info> for Aldrin {
    type Accounts = AldrinSwapAccounts<'info>;
    type Data = AldrinSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::readonly(ctx.pool.address()),
            InstructionAccount::readonly(ctx.pool_signer.address()),
            InstructionAccount::writable(ctx.pool_mint.address()),
            InstructionAccount::writable(ctx.base_token_vault.address()),
            InstructionAccount::writable(ctx.quote_token_vault.address()),
            InstructionAccount::writable(ctx.fee_pool_token_account.address()),
            InstructionAccount::readonly_signer(ctx.wallet_authority.address()),
            InstructionAccount::writable(ctx.user_base_token_account.address()),
            InstructionAccount::writable(ctx.user_quote_token_account.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
        ];

        let account_infos = [
            ctx.pool,
            ctx.pool_signer,
            ctx.pool_mint,
            ctx.base_token_vault,
            ctx.quote_token_vault,
            ctx.fee_pool_token_account,
            ctx.wallet_authority,
            ctx.user_base_token_account,
            ctx.user_quote_token_account,
            ctx.token_program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 25]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(SWAP_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(16),
                8,
            );
            let side_byte = match data.side {
                Side::Bid => 0u8,
                Side::Ask => 1u8,
            };
            core::ptr::write(ptr.add(24), side_byte);
        }

        let instruction = InstructionView {
            program_id: &ALDRIN_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 25)
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
