use {
    crate::{Manifest, MANIFEST_PROGRAM_ID, SWAP_DISCRIMINATOR},
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

pub struct ManifestSwapRemaining<'info> {
    pub manifest_program: &'info AccountView,
    pub owner: &'info AccountView,
    pub market: &'info AccountView,
    pub system_program: &'info AccountView,
    pub base_vault: &'info AccountView,
    pub quote_vault: &'info AccountView,
    pub token_program_base: &'info AccountView,
    pub token_program_quote: &'info AccountView,
    pub base_mint: &'info AccountView,
    pub quote_mint: &'info AccountView,
    pub global: &'info AccountView,
    pub global_vault: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for ManifestSwapRemaining<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < 12 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [manifest_program, owner, market, system_program, base_vault, quote_vault, token_program_base, token_program_quote, base_mint, quote_mint, global, global_vault, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(ManifestSwapRemaining {
            manifest_program,
            owner,
            market,
            system_program,
            base_vault,
            quote_vault,
            token_program_base,
            token_program_quote,
            base_mint,
            quote_mint,
            global,
            global_vault,
        })
    }
}

pub struct ManifestSwapExtra {
    pub is_exact_in: bool,
}

impl TryFrom<&[u8]> for ManifestSwapExtra {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self {
            is_exact_in: data[0] != 0,
        })
    }
}

impl<'info> SwapWithParameters<'info> for Manifest {
    type Remaining = ManifestSwapRemaining<'info>;
    type Extra = ManifestSwapExtra;

    fn try_parse_remaining(
        remaining: &'info [AccountView],
    ) -> Result<Self::Remaining, ProgramError> {
        ManifestSwapRemaining::try_from(remaining)
    }

    fn swap_with_parameters_signed(
        params: &SwapParameters<'info>,
        remaining: &Self::Remaining,
        in_amount: u64,
        minimum_out_amount: u64,
        extra: &Self::Extra,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let in_ata_mint = unsafe { &*(token_account_mint(params.in_ata).as_ptr() as *const Address) };
        let base_vault_mint = unsafe { &*(token_account_mint(remaining.base_vault).as_ptr() as *const Address) };
        let is_base_in = address_eq(in_ata_mint, base_vault_mint);

        let (trader_base, trader_quote) = if is_base_in {
            (params.in_ata, params.out_ata)
        } else {
            (params.out_ata, params.in_ata)
        };

        let accounts = [
            InstructionAccount::writable_signer(params.user_wallet.address()),
            InstructionAccount::readonly_signer(remaining.owner.address()),
            InstructionAccount::writable(remaining.market.address()),
            InstructionAccount::readonly(remaining.system_program.address()),
            InstructionAccount::writable(trader_base.address()),
            InstructionAccount::writable(trader_quote.address()),
            InstructionAccount::writable(remaining.base_vault.address()),
            InstructionAccount::writable(remaining.quote_vault.address()),
            InstructionAccount::readonly(remaining.token_program_base.address()),
            InstructionAccount::readonly(remaining.base_mint.address()),
            InstructionAccount::readonly(remaining.token_program_quote.address()),
            InstructionAccount::readonly(remaining.quote_mint.address()),
            InstructionAccount::writable(remaining.global.address()),
            InstructionAccount::writable(remaining.global_vault.address()),
        ];

        let account_infos = [
            params.user_wallet,
            remaining.owner,
            remaining.market,
            remaining.system_program,
            trader_base,
            trader_quote,
            remaining.base_vault,
            remaining.quote_vault,
            remaining.token_program_base,
            remaining.base_mint,
            remaining.token_program_quote,
            remaining.quote_mint,
            remaining.global,
            remaining.global_vault,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 19]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::write(ptr, SWAP_DISCRIMINATOR);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(1), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(9),
                8,
            );
            core::ptr::write(ptr.add(17), is_base_in as u8);
            core::ptr::write(ptr.add(18), extra.is_exact_in as u8);
        }

        let instruction = InstructionView {
            program_id: &MANIFEST_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 19)
            },
        };

        invoke_signed(&instruction, &account_infos, signer_seeds)
    }
}
