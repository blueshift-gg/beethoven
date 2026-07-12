#![no_std]

use {
    beethoven_core::Deposit,
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::{address, Address},
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const BANKINECO_PROGRAM_ID: Address = address!("save8RQVPMWNTzU18t3GBvBkN9hT7jsGjiCQ28FpD9H");

const MINT_W_YIELDING_GEN_DISCRIMINATOR: [u8; 8] = [31, 100, 17, 215, 62, 12, 31, 2];

pub struct Bankineco;

pub struct BankinecoDepositData {
    pub min_bank_mint_minted: u64,
}

impl BankinecoDepositData {
    pub const DATA_LEN: usize = 8;
}

impl TryFrom<&[u8]> for BankinecoDepositData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self {
            min_bank_mint_minted: u64::from_le_bytes(data[0..8].try_into().unwrap()),
        })
    }
}

pub struct BankinecoDepositAccounts<'info> {
    pub bankineco_program: &'info AccountView,
    pub user: &'info AccountView,
    pub bank_state: &'info AccountView,
    pub vault_state: &'info AccountView,
    pub oracle_state: &'info AccountView,
    pub yielding_mint: &'info AccountView,
    pub bank_mint: &'info AccountView,
    pub yielding_user_ta: &'info AccountView,
    pub bank_mint_user_ta: &'info AccountView,
    pub yielding_vault_ata: &'info AccountView,
    pub team_state: &'info AccountView,
    pub fee_team_ata: &'info AccountView,
    pub system_program: &'info AccountView,
    pub token_program: &'info AccountView,
    pub yielding_mint_program: &'info AccountView,
    pub associated_token_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for BankinecoDepositAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [bankineco_program, user, bank_state, vault_state, oracle_state, yielding_mint, bank_mint, yielding_user_ta, bank_mint_user_ta, yielding_vault_ata, team_state, fee_team_ata, system_program, token_program, yielding_mint_program, associated_token_program, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(BankinecoDepositAccounts {
            bankineco_program,
            user,
            bank_state,
            vault_state,
            oracle_state,
            yielding_mint,
            bank_mint,
            yielding_user_ta,
            bank_mint_user_ta,
            yielding_vault_ata,
            team_state,
            fee_team_ata,
            system_program,
            token_program,
            yielding_mint_program,
            associated_token_program,
        })
    }
}

impl<'info> Deposit<'info> for Bankineco {
    type Accounts = BankinecoDepositAccounts<'info>;
    type Data = BankinecoDepositData;

    fn deposit_signed(
        ctx: &Self::Accounts,
        amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::readonly_signer(ctx.user.address()),
            InstructionAccount::writable(ctx.bank_state.address()),
            InstructionAccount::writable(ctx.vault_state.address()),
            InstructionAccount::readonly(ctx.oracle_state.address()),
            InstructionAccount::readonly(ctx.yielding_mint.address()),
            InstructionAccount::writable(ctx.bank_mint.address()),
            InstructionAccount::writable(ctx.yielding_user_ta.address()),
            InstructionAccount::writable(ctx.bank_mint_user_ta.address()),
            InstructionAccount::writable(ctx.yielding_vault_ata.address()),
            InstructionAccount::writable(ctx.team_state.address()),
            InstructionAccount::writable(ctx.fee_team_ata.address()),
            InstructionAccount::readonly(ctx.system_program.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
            InstructionAccount::readonly(ctx.yielding_mint_program.address()),
            InstructionAccount::readonly(ctx.associated_token_program.address()),
        ];

        let account_infos = [
            ctx.user,
            ctx.bank_state,
            ctx.vault_state,
            ctx.oracle_state,
            ctx.yielding_mint,
            ctx.bank_mint,
            ctx.yielding_user_ta,
            ctx.bank_mint_user_ta,
            ctx.yielding_vault_ata,
            ctx.team_state,
            ctx.fee_team_ata,
            ctx.system_program,
            ctx.token_program,
            ctx.yielding_mint_program,
            ctx.associated_token_program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 24]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(MINT_W_YIELDING_GEN_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(
                data.min_bank_mint_minted.to_le_bytes().as_ptr(),
                ptr.add(16),
                8,
            );
        }

        let instruction = InstructionView {
            program_id: &BANKINECO_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe { instruction_data.assume_init_ref() },
        };

        invoke_signed(&instruction, &account_infos, signer_seeds)
    }

    fn deposit(ctx: &Self::Accounts, amount: u64, data: &Self::Data) -> ProgramResult {
        Self::deposit_signed(ctx, amount, data, &[])
    }
}
