use {
    crate::{Futarchy, FUTARCHY_PROGRAM_ID, SWAP_DISCRIMINATOR},
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

pub struct FutarchySwapRemaining<'info> {
    pub futarchy_program: &'info AccountView,
    pub dao: &'info AccountView,
    pub amm_base_vault: &'info AccountView,
    pub amm_quote_vault: &'info AccountView,
    pub token_program: &'info AccountView,
    pub event_authority: &'info AccountView,
    pub program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for FutarchySwapRemaining<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < 7 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [futarchy_program, dao, amm_base_vault, amm_quote_vault, token_program, event_authority, program, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(FutarchySwapRemaining {
            futarchy_program,
            dao,
            amm_base_vault,
            amm_quote_vault,
            token_program,
            event_authority,
            program,
        })
    }
}

impl<'info> SwapWithParameters<'info> for Futarchy {
    type Remaining = FutarchySwapRemaining<'info>;
    type Extra = ();

    fn try_parse_remaining(
        remaining: &'info [AccountView],
    ) -> Result<Self::Remaining, ProgramError> {
        FutarchySwapRemaining::try_from(remaining)
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
        let base_vault_mint = unsafe { &*(token_account_mint(remaining.amm_base_vault).as_ptr() as *const Address) };
        let is_base_in = address_eq(in_ata_mint, base_vault_mint);
        let swap_type_byte = if is_base_in { 1u8 } else { 0u8 };

        let (user_base_account, user_quote_account) = if is_base_in {
            (params.in_ata, params.out_ata)
        } else {
            (params.out_ata, params.in_ata)
        };

        let accounts = [
            InstructionAccount::writable(remaining.dao.address()),
            InstructionAccount::writable(user_base_account.address()),
            InstructionAccount::writable(user_quote_account.address()),
            InstructionAccount::writable(remaining.amm_base_vault.address()),
            InstructionAccount::writable(remaining.amm_quote_vault.address()),
            InstructionAccount::readonly_signer(params.user_wallet.address()),
            InstructionAccount::readonly(remaining.token_program.address()),
            InstructionAccount::readonly(remaining.event_authority.address()),
            InstructionAccount::readonly(remaining.program.address()),
        ];

        let account_infos = [
            remaining.dao,
            user_base_account,
            user_quote_account,
            remaining.amm_base_vault,
            remaining.amm_quote_vault,
            params.user_wallet,
            remaining.token_program,
            remaining.event_authority,
            remaining.program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 25]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(SWAP_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::write(ptr.add(16), swap_type_byte);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(17),
                8,
            );
        }

        let instruction = InstructionView {
            program_id: &FUTARCHY_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 25)
            },
        };

        invoke_signed(&instruction, &account_infos, signer_seeds)
    }
}
