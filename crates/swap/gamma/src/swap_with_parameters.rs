use {
    crate::{Gamma, GAMMA_PROGRAM_ID, SWAP_DISCRIMINATOR},
    beethoven_core::{SwapParameters, SwapWithParameters},
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub struct GammaSwapRemaining<'info> {
    pub gamma_program: &'info AccountView,
    pub authority: &'info AccountView,
    pub amm_config: &'info AccountView,
    pub pool_state: &'info AccountView,
    pub input_vault: &'info AccountView,
    pub output_vault: &'info AccountView,
    pub input_token_program: &'info AccountView,
    pub output_token_program: &'info AccountView,
    pub input_token_mint: &'info AccountView,
    pub output_token_mint: &'info AccountView,
    pub observation_state: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for GammaSwapRemaining<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < 11 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [gamma_program, authority, amm_config, pool_state, input_vault, output_vault, input_token_program, output_token_program, input_token_mint, output_token_mint, observation_state, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(GammaSwapRemaining {
            gamma_program,
            authority,
            amm_config,
            pool_state,
            input_vault,
            output_vault,
            input_token_program,
            output_token_program,
            input_token_mint,
            output_token_mint,
            observation_state,
        })
    }
}

impl<'info> SwapWithParameters<'info> for Gamma {
    type Remaining = GammaSwapRemaining<'info>;
    type Extra = ();

    fn try_parse_remaining(
        remaining: &'info [AccountView],
    ) -> Result<Self::Remaining, ProgramError> {
        GammaSwapRemaining::try_from(remaining)
    }

    fn swap_with_parameters_signed(
        params: &SwapParameters<'info>,
        remaining: &Self::Remaining,
        in_amount: u64,
        minimum_out_amount: u64,
        _extra: &(),
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::readonly_signer(params.user_wallet.address()),
            InstructionAccount::readonly(remaining.authority.address()),
            InstructionAccount::readonly(remaining.amm_config.address()),
            InstructionAccount::writable(remaining.pool_state.address()),
            InstructionAccount::writable(params.in_ata.address()),
            InstructionAccount::writable(params.out_ata.address()),
            InstructionAccount::writable(remaining.input_vault.address()),
            InstructionAccount::writable(remaining.output_vault.address()),
            InstructionAccount::readonly(remaining.input_token_program.address()),
            InstructionAccount::readonly(remaining.output_token_program.address()),
            InstructionAccount::readonly(remaining.input_token_mint.address()),
            InstructionAccount::readonly(remaining.output_token_mint.address()),
            InstructionAccount::writable(remaining.observation_state.address()),
        ];

        let account_infos = [
            params.user_wallet,
            remaining.authority,
            remaining.amm_config,
            remaining.pool_state,
            params.in_ata,
            params.out_ata,
            remaining.input_vault,
            remaining.output_vault,
            remaining.input_token_program,
            remaining.output_token_program,
            remaining.input_token_mint,
            remaining.output_token_mint,
            remaining.observation_state,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 24]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(SWAP_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(16),
                8,
            );
        }

        let instruction = InstructionView {
            program_id: &GAMMA_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 24)
            },
        };

        invoke_signed(&instruction, &account_infos, signer_seeds)
    }
}
