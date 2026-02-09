use {
    crate::{SolFiV2, SOLFI_V2_PROGRAM_ID, SWAP_DISCRIMINATOR},
    beethoven_core::{SwapParameters, SwapWithParameters, token_account_mint},
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::{address_eq, Address},
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub struct SolFiV2SwapRemaining<'info> {
    pub solfi_v2_program: &'info AccountView,
    pub market_account: &'info AccountView,
    pub oracle_account: &'info AccountView,
    pub config_account: &'info AccountView,
    pub base_vault: &'info AccountView,
    pub quote_vault: &'info AccountView,
    pub base_mint: &'info AccountView,
    pub quote_mint: &'info AccountView,
    pub base_token_program: &'info AccountView,
    pub quote_token_program: &'info AccountView,
    pub instructions_sysvar: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for SolFiV2SwapRemaining<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < 11 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [solfi_v2_program, market_account, oracle_account, config_account, base_vault, quote_vault, base_mint, quote_mint, base_token_program, quote_token_program, instructions_sysvar, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(SolFiV2SwapRemaining {
            solfi_v2_program,
            market_account,
            oracle_account,
            config_account,
            base_vault,
            quote_vault,
            base_mint,
            quote_mint,
            base_token_program,
            quote_token_program,
            instructions_sysvar,
        })
    }
}

impl<'info> SwapWithParameters<'info> for SolFiV2 {
    type Remaining = SolFiV2SwapRemaining<'info>;
    type Extra = ();

    fn try_parse_remaining(
        remaining: &'info [AccountView],
    ) -> Result<Self::Remaining, ProgramError> {
        SolFiV2SwapRemaining::try_from(remaining)
    }

    fn swap_with_parameters_signed(
        params: &SwapParameters<'info>,
        remaining: &Self::Remaining,
        in_amount: u64,
        minimum_out_amount: u64,
        _extra: &(),
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let in_ata_mint = unsafe { &*(token_account_mint(params.in_ata).as_ptr() as *const Address) };
        let base_vault_mint = unsafe { &*(token_account_mint(remaining.base_vault).as_ptr() as *const Address) };
        let is_base_in = address_eq(in_ata_mint, base_vault_mint);
        let is_quote_to_base = !is_base_in;

        let (user_base_ata, user_quote_ata) = if is_base_in {
            (params.in_ata, params.out_ata)
        } else {
            (params.out_ata, params.in_ata)
        };

        let accounts = [
            InstructionAccount::writable_signer(params.user_wallet.address()),
            InstructionAccount::writable(remaining.market_account.address()),
            InstructionAccount::readonly(remaining.oracle_account.address()),
            InstructionAccount::readonly(remaining.config_account.address()),
            InstructionAccount::writable(remaining.base_vault.address()),
            InstructionAccount::writable(remaining.quote_vault.address()),
            InstructionAccount::writable(user_base_ata.address()),
            InstructionAccount::writable(user_quote_ata.address()),
            InstructionAccount::readonly(remaining.base_mint.address()),
            InstructionAccount::readonly(remaining.quote_mint.address()),
            InstructionAccount::readonly(remaining.base_token_program.address()),
            InstructionAccount::readonly(remaining.quote_token_program.address()),
            InstructionAccount::readonly(remaining.instructions_sysvar.address()),
        ];

        let account_infos = [
            params.user_wallet,
            remaining.market_account,
            remaining.oracle_account,
            remaining.config_account,
            remaining.base_vault,
            remaining.quote_vault,
            user_base_ata,
            user_quote_ata,
            remaining.base_mint,
            remaining.quote_mint,
            remaining.base_token_program,
            remaining.quote_token_program,
            remaining.instructions_sysvar,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 18]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::write(ptr, SWAP_DISCRIMINATOR);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(1), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(9),
                8,
            );
            core::ptr::write(ptr.add(17), is_quote_to_base as u8);
        }

        let instruction = InstructionView {
            program_id: &SOLFI_V2_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 18)
            },
        };

        invoke_signed(&instruction, &account_infos, signer_seeds)
    }
}
