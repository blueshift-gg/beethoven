use {
    crate::{Perena, PERENA_PROGRAM_ID, SWAP_DISCRIMINATOR},
    beethoven_core::{SwapParameters, SwapWithParameters},
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub struct PerenaSwapRemaining<'info> {
    pub perena_program: &'info AccountView,
    pub pool: &'info AccountView,
    pub in_vault: &'info AccountView,
    pub out_vault: &'info AccountView,
    pub numeraire_config: &'info AccountView,
    pub token_program: &'info AccountView,
    pub token_2022_program: &'info AccountView,
    pub mint_in: &'info AccountView,
    pub mint_out: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for PerenaSwapRemaining<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < 9 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [perena_program, pool, in_vault, out_vault, numeraire_config, token_program, token_2022_program, mint_in, mint_out, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(PerenaSwapRemaining {
            perena_program,
            pool,
            in_vault,
            out_vault,
            numeraire_config,
            token_program,
            token_2022_program,
            mint_in,
            mint_out,
        })
    }
}

pub struct PerenaSwapExtra {
    pub in_index: u8,
    pub out_index: u8,
}

impl TryFrom<&[u8]> for PerenaSwapExtra {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < 2 {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self {
            in_index: data[0],
            out_index: data[1],
        })
    }
}

impl<'info> SwapWithParameters<'info> for Perena {
    type Remaining = PerenaSwapRemaining<'info>;
    type Extra = PerenaSwapExtra;

    fn try_parse_remaining(
        remaining: &'info [AccountView],
    ) -> Result<Self::Remaining, ProgramError> {
        PerenaSwapRemaining::try_from(remaining)
    }

    fn swap_with_parameters_signed(
        params: &SwapParameters<'info>,
        remaining: &Self::Remaining,
        in_amount: u64,
        minimum_out_amount: u64,
        extra: &Self::Extra,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::writable(remaining.pool.address()),
            InstructionAccount::writable(remaining.mint_in.address()),
            InstructionAccount::writable(remaining.mint_out.address()),
            InstructionAccount::writable(params.in_ata.address()),
            InstructionAccount::writable(params.out_ata.address()),
            InstructionAccount::writable(remaining.in_vault.address()),
            InstructionAccount::writable(remaining.out_vault.address()),
            InstructionAccount::readonly(remaining.numeraire_config.address()),
            InstructionAccount::writable_signer(params.user_wallet.address()),
            InstructionAccount::readonly(remaining.token_program.address()),
            InstructionAccount::readonly(remaining.token_2022_program.address()),
        ];

        let account_infos = [
            remaining.pool,
            remaining.mint_in,
            remaining.mint_out,
            params.in_ata,
            params.out_ata,
            remaining.in_vault,
            remaining.out_vault,
            remaining.numeraire_config,
            params.user_wallet,
            remaining.token_program,
            remaining.token_2022_program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 26]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(SWAP_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::write(ptr.add(8), extra.in_index);
            core::ptr::write(ptr.add(9), extra.out_index);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(10), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(18),
                8,
            );
        }

        let instruction = InstructionView {
            program_id: &PERENA_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 26)
            },
        };

        invoke_signed(&instruction, &account_infos, signer_seeds)
    }
}
