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

pub const METEORA_DAMM_V2_PROGRAM_ID: Address =
    address!("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");

const SWAP2_DISCRIMINATOR: [u8; 8] = [65, 75, 63, 76, 235, 91, 91, 136];

pub struct MeteoraDammV2;

#[repr(u8)]
pub enum SwapType {
    ExactIn = 0,
    PartialFill = 1,
    ExactOut = 2,
}

pub struct MeteoraDammV2SwapData {
    pub swap_type: SwapType,
}

impl MeteoraDammV2SwapData {
    pub const DATA_LEN: usize = 1;
}

impl TryFrom<&[u8]> for MeteoraDammV2SwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let swap_type = match data[0] {
            0 => SwapType::ExactIn,
            1 => SwapType::PartialFill,
            2 => SwapType::ExactOut,
            _ => return Err(ProgramError::InvalidInstructionData),
        };
        Ok(Self { swap_type })
    }
}

impl MeteoraDammV2SwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 15;
}

pub struct MeteoraDammV2SwapAccounts<'info> {
    pub cp_amm_program: &'info AccountView,
    pub pool_authority: &'info AccountView,
    pub pool: &'info AccountView,
    pub input_token_account: &'info AccountView,
    pub output_token_account: &'info AccountView,
    pub token_a_vault: &'info AccountView,
    pub token_b_vault: &'info AccountView,
    pub token_a_mint: &'info AccountView,
    pub token_b_mint: &'info AccountView,
    pub payer: &'info AccountView,
    pub token_a_program: &'info AccountView,
    pub token_b_program: &'info AccountView,
    pub referral_token_account: &'info AccountView,
    pub event_authority: &'info AccountView,
    pub program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for MeteoraDammV2SwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [cp_amm_program, pool_authority, pool, input_token_account, output_token_account, token_a_vault, token_b_vault, token_a_mint, token_b_mint, payer, token_a_program, token_b_program, referral_token_account, event_authority, program, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(MeteoraDammV2SwapAccounts {
            cp_amm_program,
            pool_authority,
            pool,
            input_token_account,
            output_token_account,
            token_a_vault,
            token_b_vault,
            token_a_mint,
            token_b_mint,
            payer,
            token_a_program,
            token_b_program,
            referral_token_account,
            event_authority,
            program,
        })
    }
}

fn referral_account_meta(addr: &Address) -> InstructionAccount<'_> {
    if *addr == METEORA_DAMM_V2_PROGRAM_ID {
        InstructionAccount::readonly(addr)
    } else {
        InstructionAccount::writable(addr)
    }
}

impl<'info> Swap<'info> for MeteoraDammV2 {
    type Accounts = MeteoraDammV2SwapAccounts<'info>;
    type Data = MeteoraDammV2SwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::readonly(ctx.pool_authority.address()),
            InstructionAccount::writable(ctx.pool.address()),
            InstructionAccount::writable(ctx.input_token_account.address()),
            InstructionAccount::writable(ctx.output_token_account.address()),
            InstructionAccount::writable(ctx.token_a_vault.address()),
            InstructionAccount::writable(ctx.token_b_vault.address()),
            InstructionAccount::readonly(ctx.token_a_mint.address()),
            InstructionAccount::readonly(ctx.token_b_mint.address()),
            InstructionAccount::readonly_signer(ctx.payer.address()),
            InstructionAccount::readonly(ctx.token_a_program.address()),
            InstructionAccount::readonly(ctx.token_b_program.address()),
            referral_account_meta(ctx.referral_token_account.address()),
            InstructionAccount::readonly(ctx.event_authority.address()),
            InstructionAccount::readonly(ctx.program.address()),
        ];

        let account_infos = [
            ctx.pool_authority,
            ctx.pool,
            ctx.input_token_account,
            ctx.output_token_account,
            ctx.token_a_vault,
            ctx.token_b_vault,
            ctx.token_a_mint,
            ctx.token_b_mint,
            ctx.payer,
            ctx.token_a_program,
            ctx.token_b_program,
            ctx.referral_token_account,
            ctx.event_authority,
            ctx.program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 25]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(SWAP2_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(16),
                8,
            );
            let swap_type_byte = match data.swap_type {
                SwapType::ExactIn => 0u8,
                SwapType::PartialFill => 1u8,
                SwapType::ExactOut => 2u8,
            };
            core::ptr::write(ptr.add(24), swap_type_byte);
        }

        let instruction = InstructionView {
            program_id: &METEORA_DAMM_V2_PROGRAM_ID,
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
