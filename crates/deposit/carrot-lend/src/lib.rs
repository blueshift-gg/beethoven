#![no_std]

use {
    beethoven_core::Deposit,
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const CARROT_LEND_PROGRAM_ID: Address =
    Address::from_str_const("C73nDAFn23RYwiFa6vtHshSbcg8x6BLYjw3bERJ3vHxf");
pub const LENDING_ACCOUNT_DEPOSIT_DISCRIMINATOR: [u8; 8] = [171, 94, 235, 103, 82, 64, 212, 140];
pub const DEPOSIT_DATA_LEN: usize = 17;
pub const MAX_ACCOUNTS: usize = 13;

pub struct CarrotLend;

pub struct CarrotLendDepositData {
    pub deposit_up_to_amount: u8,
}

impl CarrotLendDepositData {
    pub const DATA_LEN: usize = 1;
}

impl TryFrom<&[u8]> for CarrotLendDepositData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            deposit_up_to_amount: data[0],
        })
    }
}

pub struct CarrotLendDepositAccounts<'info> {
    pub clend_program: &'info AccountView,
    pub clend_group: &'info AccountView,
    pub clend_account: &'info AccountView,
    pub signer: &'info AccountView,
    pub bank: &'info AccountView,
    pub signer_token_account: &'info AccountView,
    pub bank_liquidity_vault: &'info AccountView,
    pub token_program: &'info AccountView,
    pub remaining_accounts: &'info [AccountView],
}

impl<'info> TryFrom<&'info [AccountView]> for CarrotLendDepositAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [clend_program, clend_group, clend_account, signer, bank, signer_token_account, bank_liquidity_vault, token_program, remaining_accounts @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(CarrotLendDepositAccounts {
            clend_program,
            clend_group,
            clend_account,
            signer,
            bank,
            signer_token_account,
            bank_liquidity_vault,
            token_program,
            remaining_accounts,
        })
    }
}

impl<'info> Deposit<'info> for CarrotLend {
    type Accounts = CarrotLendDepositAccounts<'info>;
    type Data = CarrotLendDepositData;

    fn deposit_signed(
        ctx: &CarrotLendDepositAccounts<'info>,
        amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let total_accounts = 7 + ctx.remaining_accounts.len();
        if total_accounts > MAX_ACCOUNTS {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let mut account_metas = MaybeUninit::<[InstructionAccount; MAX_ACCOUNTS]>::uninit();
        let account_metas_ptr = account_metas.as_mut_ptr() as *mut InstructionAccount;

        unsafe {
            core::ptr::write(
                account_metas_ptr,
                InstructionAccount::readonly(ctx.clend_group.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(1),
                InstructionAccount::writable(ctx.clend_account.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(2),
                InstructionAccount::writable_signer(ctx.signer.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(3),
                InstructionAccount::writable(ctx.bank.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(4),
                InstructionAccount::writable(ctx.signer_token_account.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(5),
                InstructionAccount::writable(ctx.bank_liquidity_vault.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(6),
                InstructionAccount::readonly(ctx.token_program.address()),
            );

            for (index, account) in ctx.remaining_accounts.iter().enumerate() {
                core::ptr::write(
                    account_metas_ptr.add(7 + index),
                    InstructionAccount::from(account),
                );
            }
        }

        let account_metas =
            unsafe { core::slice::from_raw_parts(account_metas_ptr, total_accounts) };

        let mut account_infos = [ctx.clend_group; MAX_ACCOUNTS];
        account_infos[1] = ctx.clend_account;
        account_infos[2] = ctx.signer;
        account_infos[3] = ctx.bank;
        account_infos[4] = ctx.signer_token_account;
        account_infos[5] = ctx.bank_liquidity_vault;
        account_infos[6] = ctx.token_program;
        for (index, account) in ctx.remaining_accounts.iter().enumerate() {
            account_infos[7 + index] = account;
        }
        let account_infos = &account_infos[..total_accounts];

        let mut instruction_data = MaybeUninit::<[u8; DEPOSIT_DATA_LEN]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(LENDING_ACCOUNT_DEPOSIT_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            *ptr.add(16) = data.deposit_up_to_amount;
        }

        let deposit_ix = InstructionView {
            program_id: &CARROT_LEND_PROGRAM_ID,
            accounts: account_metas,
            data: unsafe { instruction_data.assume_init_ref() },
        };

        invoke_signed_with_bounds::<MAX_ACCOUNTS>(&deposit_ix, account_infos, signer_seeds)?;

        Ok(())
    }

    fn deposit(
        ctx: &CarrotLendDepositAccounts<'info>,
        amount: u64,
        data: &Self::Data,
    ) -> ProgramResult {
        Self::deposit_signed(ctx, amount, data, &[])
    }
}
