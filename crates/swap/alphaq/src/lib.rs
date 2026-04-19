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

pub const ALPHAQ_PROGRAM_ID: Address = address!("ALPHAQmeA7bjrVuccPsYPiCvsi428SNwte66Srvs4pHA");

const SWAP_DISCRIMINATOR: u8 = 12;

pub struct AlphaQ;

pub struct AlphaqSwapData {
    pub a_to_b: bool,
}

impl AlphaqSwapData {
    pub const DATA_LEN: usize = 1;
}

impl TryFrom<&[u8]> for AlphaqSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(AlphaqSwapData {
            a_to_b: data[0] != 0,
        })
    }
}

impl AlphaqSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 13;
}

pub struct AlphaqSwapAccounts<'info> {
    pub alphaq_program: &'info AccountView,
    pub user: &'info AccountView,
    pub market: &'info AccountView,
    pub market_state: &'info AccountView,
    pub user_token_account_a: &'info AccountView,
    pub user_token_account_b: &'info AccountView,
    pub vault_token_account_a: &'info AccountView,
    pub vault_token_account_b: &'info AccountView,
    pub token_authority_a: &'info AccountView,
    pub token_authority_b: &'info AccountView,
    pub vendor_key: &'info AccountView,
    pub token_program: &'info AccountView,
    pub instructions_sysvar: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for AlphaqSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [alphaq_program, user, market, market_state, user_token_account_a, user_token_account_b, vault_token_account_a, vault_token_account_b, token_authority_a, token_authority_b, vendor_key, token_program, instructions_sysvar] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(AlphaqSwapAccounts {
            alphaq_program,
            user,
            market,
            market_state,
            user_token_account_a,
            user_token_account_b,
            vault_token_account_a,
            vault_token_account_b,
            token_authority_a,
            token_authority_b,
            vendor_key,
            token_program,
            instructions_sysvar,
        })
    }
}

impl<'info> Swap<'info> for AlphaQ {
    type Accounts = AlphaqSwapAccounts<'info>;
    type Data = AlphaqSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &AlphaqSwapData,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::writable_signer(ctx.user.address()),
            InstructionAccount::writable(ctx.market.address()),
            InstructionAccount::writable(ctx.market_state.address()),
            InstructionAccount::writable(ctx.user_token_account_a.address()),
            InstructionAccount::writable(ctx.user_token_account_b.address()),
            InstructionAccount::writable(ctx.vault_token_account_a.address()),
            InstructionAccount::writable(ctx.vault_token_account_b.address()),
            InstructionAccount::writable(ctx.token_authority_a.address()),
            InstructionAccount::writable(ctx.token_authority_b.address()),
            InstructionAccount::writable(ctx.vendor_key.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
            InstructionAccount::readonly(ctx.instructions_sysvar.address()),
        ];

        let account_infos = [
            ctx.user,
            ctx.market,
            ctx.market_state,
            ctx.user_token_account_a,
            ctx.user_token_account_b,
            ctx.vault_token_account_a,
            ctx.vault_token_account_b,
            ctx.token_authority_a,
            ctx.token_authority_b,
            ctx.vendor_key,
            ctx.token_program,
            ctx.instructions_sysvar,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 18]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::write(ptr, SWAP_DISCRIMINATOR);
            core::ptr::write(ptr.add(1), data.a_to_b as u8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(2), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(10),
                8,
            );
        }

        let instruction = InstructionView {
            program_id: &ALPHAQ_PROGRAM_ID,
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
