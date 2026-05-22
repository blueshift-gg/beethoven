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

pub const BISONFI_PROGRAM_ID: Address = address!("BiSoNHVpsVZW2F7rx2eQ59yQwKxzU5NvBcmKshCSUypi");

const SWAP2_DISCRIMINATOR: u8 = 7;

pub struct Bisonfi;

pub struct BisonfiSwapData {
    pub b_to_a: bool,
    pub exact_out: bool,
}

impl BisonfiSwapData {
    pub const DATA_LEN: usize = 2;
}

impl TryFrom<&[u8]> for BisonfiSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self {
            b_to_a: data[0] != 0,
            exact_out: data[1] != 0,
        })
    }
}

impl BisonfiSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 10;
}

pub struct BisonfiSwapAccounts<'info> {
    pub bisonfi_program: &'info AccountView,
    pub user: &'info AccountView,
    pub market: &'info AccountView,
    pub market_ta_a: &'info AccountView,
    pub market_ta_b: &'info AccountView,
    pub user_ata_a: &'info AccountView,
    pub user_ata_b: &'info AccountView,
    pub token_prog_a: &'info AccountView,
    pub token_prog_b: &'info AccountView,
    pub logger: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for BisonfiSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [bisonfi_program, user, market, market_ta_a, market_ta_b, user_ata_a, user_ata_b, token_prog_a, token_prog_b, logger] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(BisonfiSwapAccounts {
            bisonfi_program,
            user,
            market,
            market_ta_a,
            market_ta_b,
            user_ata_a,
            user_ata_b,
            token_prog_a,
            token_prog_b,
            logger,
        })
    }
}

impl<'info> Swap<'info> for Bisonfi {
    type Accounts = BisonfiSwapAccounts<'info>;
    type Data = BisonfiSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::writable_signer(ctx.user.address()),
            InstructionAccount::writable(ctx.market.address()),
            InstructionAccount::writable(ctx.market_ta_a.address()),
            InstructionAccount::writable(ctx.market_ta_b.address()),
            InstructionAccount::writable(ctx.user_ata_a.address()),
            InstructionAccount::writable(ctx.user_ata_b.address()),
            InstructionAccount::readonly(ctx.token_prog_a.address()),
            InstructionAccount::readonly(ctx.token_prog_b.address()),
            InstructionAccount::readonly(ctx.logger.address()),
        ];

        let account_infos = [
            ctx.user,
            ctx.market,
            ctx.market_ta_a,
            ctx.market_ta_b,
            ctx.user_ata_a,
            ctx.user_ata_b,
            ctx.token_prog_a,
            ctx.token_prog_b,
            ctx.logger,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 19]>::uninit();

        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            *ptr = SWAP2_DISCRIMINATOR;
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(1), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(9),
                8,
            );
            *ptr.add(17) = data.b_to_a as u8;
            *ptr.add(18) = data.exact_out as u8;
        }

        let instruction = InstructionView {
            program_id: &BISONFI_PROGRAM_ID,
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
