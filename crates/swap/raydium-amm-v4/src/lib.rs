#![no_std]

use {
    beethoven_core::{Swap, SwapTokenAccounts},
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::{address, Address},
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const RAYDIUM_AMM_V4_PROGRAM_ID: Address =
    address!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");

const SWAP_BASE_IN_V2_TAG: u8 = 16;

pub struct RaydiumAmmV4;

impl RaydiumAmmV4SwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 9;
}

pub struct RaydiumAmmV4SwapAccounts<'info> {
    pub raydium_amm_v4_program: &'info AccountView,
    pub token_program: &'info AccountView,
    pub amm_id: &'info AccountView,
    pub amm_authority: &'info AccountView,
    pub amm_coin_vault: &'info AccountView,
    pub amm_pc_vault: &'info AccountView,
    pub user_source_token_account: &'info AccountView,
    pub user_dest_token_account: &'info AccountView,
    pub user_wallet_account: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for RaydiumAmmV4SwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [raydium_amm_v4_program, token_program, amm_id, amm_authority, amm_coin_vault, amm_pc_vault, user_source_token_account, user_dest_token_account, user_wallet_account, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(RaydiumAmmV4SwapAccounts {
            raydium_amm_v4_program,
            token_program,
            amm_id,
            amm_authority,
            amm_coin_vault,
            amm_pc_vault,
            user_source_token_account,
            user_dest_token_account,
            user_wallet_account,
        })
    }
}

impl<'info> Swap<'info> for RaydiumAmmV4 {
    type Accounts = RaydiumAmmV4SwapAccounts<'info>;
    type Data = ();

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        _data: &(),
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::readonly(ctx.token_program.address()),
            InstructionAccount::writable(ctx.amm_id.address()),
            InstructionAccount::readonly(ctx.amm_authority.address()),
            InstructionAccount::writable(ctx.amm_coin_vault.address()),
            InstructionAccount::writable(ctx.amm_pc_vault.address()),
            InstructionAccount::writable(ctx.user_source_token_account.address()),
            InstructionAccount::writable(ctx.user_dest_token_account.address()),
            InstructionAccount::writable_signer(ctx.user_wallet_account.address()),
        ];

        let account_infos = [
            ctx.token_program,
            ctx.amm_id,
            ctx.amm_authority,
            ctx.amm_coin_vault,
            ctx.amm_pc_vault,
            ctx.user_source_token_account,
            ctx.user_dest_token_account,
            ctx.user_wallet_account,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 17]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            *ptr = SWAP_BASE_IN_V2_TAG;
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(1), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(9),
                8,
            );
        }

        let instruction = InstructionView {
            program_id: &RAYDIUM_AMM_V4_PROGRAM_ID,
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

impl<'info> SwapTokenAccounts<'info> for RaydiumAmmV4 {
    type Accounts = RaydiumAmmV4SwapAccounts<'info>;
    type Data = ();

    fn token_accounts(
        ctx: &Self::Accounts,
        _data: &Self::Data,
    ) -> (&'info AccountView, &'info AccountView) {
        (ctx.user_source_token_account, ctx.user_dest_token_account)
    }
}
