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

pub const MARGINFI_PROGRAM_ID: Address =
    Address::from_str_const("MFv2hWf31Z9kbCa1snEPYctwafyhdvnV7FZnsebVacA");
pub const LENDING_ACCOUNT_DEPOSIT_DISCRIMINATOR: [u8; 8] = [171, 94, 235, 103, 82, 64, 212, 140];
pub const DEPOSIT_DATA_LEN: usize = 18;

pub struct Marginfi;

pub struct MarginfiDepositData {
    pub deposit_up_to_amount: Option<u8>,
}

impl MarginfiDepositData {
    pub const DATA_LEN: usize = 2;
}

impl TryFrom<&[u8]> for MarginfiDepositData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            deposit_up_to_amount: if data[0] == 0 { None } else { Some(data[1]) },
        })
    }
}

pub struct MarginfiDepositAccounts<'info> {
    pub marginfi_program: &'info AccountView,
    pub group: &'info AccountView,
    pub marginfi_account: &'info AccountView,
    pub authority: &'info AccountView,
    pub bank: &'info AccountView,
    pub signer_token_account: &'info AccountView,
    pub liquidity_vault: &'info AccountView,
    pub token_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for MarginfiDepositAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [marginfi_program, group, marginfi_account, authority, bank, signer_token_account, liquidity_vault, token_program, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(MarginfiDepositAccounts {
            marginfi_program,
            group,
            marginfi_account,
            authority,
            bank,
            signer_token_account,
            liquidity_vault,
            token_program,
        })
    }
}

impl<'info> Deposit<'info> for Marginfi {
    type Accounts = MarginfiDepositAccounts<'info>;
    type Data = MarginfiDepositData;

    fn deposit_signed(
        ctx: &MarginfiDepositAccounts<'info>,
        amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::readonly(ctx.group.address()),
            InstructionAccount::writable(ctx.marginfi_account.address()),
            InstructionAccount::writable_signer(ctx.authority.address()),
            InstructionAccount::writable(ctx.bank.address()),
            InstructionAccount::writable(ctx.signer_token_account.address()),
            InstructionAccount::writable(ctx.liquidity_vault.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
        ];

        let account_infos = [
            ctx.group,
            ctx.marginfi_account,
            ctx.authority,
            ctx.bank,
            ctx.signer_token_account,
            ctx.liquidity_vault,
            ctx.token_program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; DEPOSIT_DATA_LEN]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(LENDING_ACCOUNT_DEPOSIT_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            match data.deposit_up_to_amount {
                None => {
                    *ptr.add(16) = 0;
                    *ptr.add(17) = 0;
                }
                Some(v) => {
                    *ptr.add(16) = 1;
                    *ptr.add(17) = v;
                }
            }
        }

        let deposit_ix = InstructionView {
            program_id: &MARGINFI_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe { instruction_data.assume_init_ref() },
        };

        invoke_signed(&deposit_ix, &account_infos, signer_seeds)?;

        Ok(())
    }

    fn deposit(
        ctx: &MarginfiDepositAccounts<'info>,
        amount: u64,
        data: &Self::Data,
    ) -> ProgramResult {
        Self::deposit_signed(ctx, amount, data, &[])
    }
}
