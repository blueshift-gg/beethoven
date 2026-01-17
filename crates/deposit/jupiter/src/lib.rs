#![no_std]

use {
    beethoven_core::Deposit,
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const JUPITER_EARN_PROGRAM_ID: Address = Address::new_from_array([0u8; 32]);
pub const DEPOSIT_DISCRIMINATOR: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182];

pub struct JupiterEarn;

pub struct JupiterEarnDepositAccounts<'info> {
    pub lending_program: &'info AccountView,
    pub signer: &'info AccountView,
    pub depositor_token_account: &'info AccountView,
    pub recipient_token_account: &'info AccountView,
    pub mint: &'info AccountView,
    pub lending_admin: &'info AccountView,
    pub lending: &'info AccountView,
    pub f_token_mint: &'info AccountView,
    pub supply_token_reserves_liquidity: &'info AccountView,
    pub lending_supply_position_on_liquidity: &'info AccountView,
    pub rate_model: &'info AccountView,
    pub vault: &'info AccountView,
    pub liquidity: &'info AccountView,
    pub liquidity_program: &'info AccountView,
    pub rewards_rate_model: &'info AccountView,
    pub token_program: &'info AccountView,
    pub associated_token_program: &'info AccountView,
    pub system_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for JupiterEarnDepositAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < 18 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [lending_program, signer, depositor_token_account, recipient_token_account, mint, lending_admin, lending, f_token_mint, supply_token_reserves_liquidity, lending_supply_position_on_liquidity, rate_model, vault, liquidity, liquidity_program, rewards_rate_model, token_program, associated_token_program, system_program, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(JupiterEarnDepositAccounts {
            signer,
            depositor_token_account,
            recipient_token_account,
            mint,
            lending_admin,
            lending,
            f_token_mint,
            supply_token_reserves_liquidity,
            lending_supply_position_on_liquidity,
            rate_model,
            vault,
            liquidity,
            liquidity_program,
            rewards_rate_model,
            token_program,
            associated_token_program,
            system_program,
            lending_program,
        })
    }
}

impl<'info> Deposit<'info> for JupiterEarn {
    type Accounts = JupiterEarnDepositAccounts<'info>;

    fn deposit_signed(
        ctx: &JupiterEarnDepositAccounts<'info>,
        amount: u64,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::writable_signer(ctx.signer.address()),
            InstructionAccount::writable(ctx.depositor_token_account.address()),
            InstructionAccount::writable(ctx.recipient_token_account.address()),
            InstructionAccount::readonly(ctx.mint.address()),
            InstructionAccount::readonly(ctx.lending_admin.address()),
            InstructionAccount::writable(ctx.lending.address()),
            InstructionAccount::writable(ctx.f_token_mint.address()),
            InstructionAccount::writable(ctx.supply_token_reserves_liquidity.address()),
            InstructionAccount::writable(ctx.lending_supply_position_on_liquidity.address()),
            InstructionAccount::readonly(ctx.rate_model.address()),
            InstructionAccount::writable(ctx.vault.address()),
            InstructionAccount::writable(ctx.liquidity.address()),
            InstructionAccount::writable(ctx.liquidity_program.address()),
            InstructionAccount::readonly(ctx.rewards_rate_model.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
            InstructionAccount::readonly(ctx.associated_token_program.address()),
            InstructionAccount::readonly(ctx.system_program.address()),
        ];

        let account_infos = [
            ctx.signer,
            ctx.depositor_token_account,
            ctx.recipient_token_account,
            ctx.mint,
            ctx.lending_admin,
            ctx.lending,
            ctx.f_token_mint,
            ctx.supply_token_reserves_liquidity,
            ctx.lending_supply_position_on_liquidity,
            ctx.rate_model,
            ctx.vault,
            ctx.liquidity,
            ctx.liquidity_program,
            ctx.rewards_rate_model,
            ctx.token_program,
            ctx.associated_token_program,
            ctx.system_program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 16]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(DEPOSIT_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
        }

        let deposit_ix = InstructionView {
            program_id: &JUPITER_EARN_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 16)
            },
        };

        invoke_signed(&deposit_ix, &account_infos, signer_seeds)?;

        Ok(())
    }

    fn deposit(ctx: &JupiterEarnDepositAccounts<'info>, amount: u64) -> ProgramResult {
        Self::deposit_signed(ctx, amount, &[])
    }
}
