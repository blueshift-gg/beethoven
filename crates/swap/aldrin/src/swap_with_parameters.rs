use {
    crate::{Aldrin, ALDRIN_PROGRAM_ID, SWAP_DISCRIMINATOR},
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

pub struct AldrinSwapRemaining<'info> {
    pub aldrin_program: &'info AccountView,
    pub pool: &'info AccountView,
    pub pool_signer: &'info AccountView,
    pub pool_mint: &'info AccountView,
    pub base_token_vault: &'info AccountView,
    pub quote_token_vault: &'info AccountView,
    pub fee_pool_token_account: &'info AccountView,
    pub token_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for AldrinSwapRemaining<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < 8 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [aldrin_program, pool, pool_signer, pool_mint, base_token_vault, quote_token_vault, fee_pool_token_account, token_program, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(AldrinSwapRemaining {
            aldrin_program,
            pool,
            pool_signer,
            pool_mint,
            base_token_vault,
            quote_token_vault,
            fee_pool_token_account,
            token_program,
        })
    }
}

impl<'info> SwapWithParameters<'info> for Aldrin {
    type Remaining = AldrinSwapRemaining<'info>;
    type Extra = ();

    fn try_parse_remaining(
        remaining: &'info [AccountView],
    ) -> Result<Self::Remaining, ProgramError> {
        AldrinSwapRemaining::try_from(remaining)
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
        let base_vault_mint = unsafe { &*(token_account_mint(remaining.base_token_vault).as_ptr() as *const Address) };
        let is_base_in = address_eq(in_ata_mint, base_vault_mint);
        let side_byte = if is_base_in { 1u8 } else { 0u8 };

        let (user_base_token_account, user_quote_token_account) = if is_base_in {
            (params.in_ata, params.out_ata)
        } else {
            (params.out_ata, params.in_ata)
        };

        let accounts = [
            InstructionAccount::readonly(remaining.pool.address()),
            InstructionAccount::readonly(remaining.pool_signer.address()),
            InstructionAccount::writable(remaining.pool_mint.address()),
            InstructionAccount::writable(remaining.base_token_vault.address()),
            InstructionAccount::writable(remaining.quote_token_vault.address()),
            InstructionAccount::writable(remaining.fee_pool_token_account.address()),
            InstructionAccount::readonly_signer(params.user_wallet.address()),
            InstructionAccount::writable(user_base_token_account.address()),
            InstructionAccount::writable(user_quote_token_account.address()),
            InstructionAccount::readonly(remaining.token_program.address()),
        ];

        let account_infos = [
            remaining.pool,
            remaining.pool_signer,
            remaining.pool_mint,
            remaining.base_token_vault,
            remaining.quote_token_vault,
            remaining.fee_pool_token_account,
            params.user_wallet,
            user_base_token_account,
            user_quote_token_account,
            remaining.token_program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 25]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(SWAP_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(16),
                8,
            );
            core::ptr::write(ptr.add(24), side_byte);
        }

        let instruction = InstructionView {
            program_id: &ALDRIN_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 25)
            },
        };

        invoke_signed(&instruction, &account_infos, signer_seeds)
    }
}
