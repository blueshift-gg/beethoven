use {
    crate::{SolFi, SOLFI_PROGRAM_ID, SWAP_DISCRIMINATOR},
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

pub struct SolFiSwapRemaining<'info> {
    pub solfi_program: &'info AccountView,
    pub market_account: &'info AccountView,
    pub base_vault: &'info AccountView,
    pub quote_vault: &'info AccountView,
    pub token_program: &'info AccountView,
    pub instructions_sysvar: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for SolFiSwapRemaining<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < 6 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [solfi_program, market_account, base_vault, quote_vault, token_program, instructions_sysvar, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(SolFiSwapRemaining {
            solfi_program,
            market_account,
            base_vault,
            quote_vault,
            token_program,
            instructions_sysvar,
        })
    }
}

impl<'info> SwapWithParameters<'info> for SolFi {
    type Remaining = SolFiSwapRemaining<'info>;
    type Extra = ();

    fn try_parse_remaining(
        remaining: &'info [AccountView],
    ) -> Result<Self::Remaining, ProgramError> {
        SolFiSwapRemaining::try_from(remaining)
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
            InstructionAccount::writable(remaining.base_vault.address()),
            InstructionAccount::writable(remaining.quote_vault.address()),
            InstructionAccount::writable(user_base_ata.address()),
            InstructionAccount::writable(user_quote_ata.address()),
            InstructionAccount::readonly(remaining.token_program.address()),
            InstructionAccount::readonly(remaining.instructions_sysvar.address()),
        ];

        let account_infos = [
            params.user_wallet,
            remaining.market_account,
            remaining.base_vault,
            remaining.quote_vault,
            user_base_ata,
            user_quote_ata,
            remaining.token_program,
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
            program_id: &SOLFI_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 18)
            },
        };

        invoke_signed(&instruction, &account_infos, signer_seeds)
    }
}
