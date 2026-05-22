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

pub const SCORCH_PROGRAM_ID: Address = address!("SCoRcH8c2dpjvcJD6FiPbCSQyQgu3PcUAWj2Xxx3mqn");

const SWAP_DISCRIMINATOR: u8 = 2;

pub struct ScorchSwapData(pub [u8; 17]);

impl ScorchSwapData {
    pub const DATA_LEN: usize = 17;
}

impl TryFrom<&[u8]> for ScorchSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }
        let mut out = [0u8; 17];
        out.copy_from_slice(&data[..Self::DATA_LEN]);
        Ok(ScorchSwapData(out))
    }
}

pub struct Scorch;

impl ScorchSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 18;
}

pub struct ScorchSwapAccounts<'info> {
    pub scorch_program: &'info AccountView,
    pub market: &'info AccountView,
    pub payer: &'info AccountView,
    pub user_ata_a: &'info AccountView,
    pub user_ata_b: &'info AccountView,
    pub market_ta_a: &'info AccountView,
    pub market_ta_b: &'info AccountView,
    pub mint_a: &'info AccountView,
    pub mint_b: &'info AccountView,
    pub token_program_a: &'info AccountView,
    pub token_program_b: &'info AccountView,
    pub memo_program: &'info AccountView,
    pub oracle_program: &'info AccountView,
    pub acc1: &'info AccountView,
    pub state_a: &'info AccountView,
    pub state_b: &'info AccountView,
    pub state_c: &'info AccountView,
    pub sysvar_instructions: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for ScorchSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [scorch_program, market, payer, user_ata_a, user_ata_b, market_ta_a, market_ta_b, mint_a, mint_b, token_program_a, token_program_b, memo_program, core_program, acc1, state_a, state_b, state_c, sysvar_instructions] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(ScorchSwapAccounts {
            scorch_program,
            market,
            payer,
            user_ata_a,
            user_ata_b,
            market_ta_a,
            market_ta_b,
            mint_a,
            mint_b,
            token_program_a,
            token_program_b,
            memo_program,
            oracle_program: core_program,
            acc1,
            state_a,
            state_b,
            state_c,
            sysvar_instructions,
        })
    }
}

impl<'info> Swap<'info> for Scorch {
    type Accounts = ScorchSwapAccounts<'info>;
    type Data = ScorchSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &ScorchSwapData,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::writable(ctx.market.address()),
            InstructionAccount::writable_signer(ctx.payer.address()),
            InstructionAccount::writable(ctx.user_ata_a.address()),
            InstructionAccount::writable(ctx.user_ata_b.address()),
            InstructionAccount::writable(ctx.market_ta_a.address()),
            InstructionAccount::writable(ctx.market_ta_b.address()),
            InstructionAccount::readonly(ctx.mint_a.address()),
            InstructionAccount::readonly(ctx.mint_b.address()),
            InstructionAccount::readonly(ctx.token_program_a.address()),
            InstructionAccount::readonly(ctx.token_program_b.address()),
            InstructionAccount::readonly(ctx.memo_program.address()),
            InstructionAccount::readonly(ctx.oracle_program.address()),
            InstructionAccount::readonly(ctx.acc1.address()),
            InstructionAccount::writable(ctx.state_a.address()),
            InstructionAccount::writable(ctx.state_b.address()),
            InstructionAccount::writable(ctx.state_c.address()),
            InstructionAccount::readonly(ctx.sysvar_instructions.address()),
        ];

        let account_infos = [
            ctx.market,
            ctx.payer,
            ctx.user_ata_a,
            ctx.user_ata_b,
            ctx.market_ta_a,
            ctx.market_ta_b,
            ctx.mint_a,
            ctx.mint_b,
            ctx.token_program_a,
            ctx.token_program_b,
            ctx.memo_program,
            ctx.oracle_program,
            ctx.acc1,
            ctx.state_a,
            ctx.state_b,
            ctx.state_c,
            ctx.sysvar_instructions,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 34]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            *ptr = SWAP_DISCRIMINATOR;
            core::ptr::copy_nonoverlapping(data.0.as_ptr(), ptr.add(1), 17);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(18), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(26),
                8,
            );
        }

        let instruction = InstructionView {
            program_id: &SCORCH_PROGRAM_ID,
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
