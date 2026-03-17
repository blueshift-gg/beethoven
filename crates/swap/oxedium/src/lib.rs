#![no_std]

use {
    beethoven_core::Swap,
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

/// Oxedium program ID on Solana mainnet.
///
/// Oxedium is a single-sided liquidity protocol. Liquidity providers deposit
/// a single token into a vault; traders swap between vaults using Pyth oracle
/// prices to determine the exchange rate.
pub const OXEDIUM_PROGRAM_ID: Address =
    Address::from_str_const("oV3SkLhiXSG946FaqDf1yNocFMhE1ZvomGsoWF8Mzap");

/// Anchor discriminator for the `swap` instruction (sha256("global:swap")[0..8]).
const SWAP_DISCRIMINATOR: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];

pub struct Oxedium;

/// No protocol-specific extra data is required for an Oxedium swap.
/// `in_amount` and `minimum_out_amount` map directly to the instruction args.
impl OxediumSwapAccounts<'_> {
    /// Total number of accounts in `remainingAccounts` including the leading
    /// program-ID sentinel account used for protocol detection.
    pub const NUM_ACCOUNTS: usize = 16;
}

/// Accounts required for an Oxedium swap CPI.
///
/// Expected order in `remainingAccounts`:
///
/// | # | Account              | Writable | Signer |
/// |---|----------------------|----------|--------|
/// | 0 | oxedium_program      | no       | no     | ← protocol detector
/// | 1 | signer               | yes      | yes    |
/// | 2 | token_mint_in        | no       | no     |
/// | 3 | token_mint_out       | no       | no     |
/// | 4 | pyth_price_in        | no       | no     |
/// | 5 | pyth_price_out       | no       | no     |
/// | 6 | signer_ata_in        | yes      | no     |
/// | 7 | signer_ata_out       | yes      | no     |
/// | 8 | vault_pda_in         | yes      | no     |
/// | 9 | vault_pda_out        | yes      | no     |
/// |10 | vault_ata_in         | yes      | no     |
/// |11 | vault_ata_out        | yes      | no     |
/// |12 | oxe_global_pda       | no       | no     |
/// |13 | associated_token_prog| no       | no     |
/// |14 | token_program        | no       | no     |
/// |15 | system_program       | no       | no     |
pub struct OxediumSwapAccounts<'info> {
    pub oxedium_program: &'info AccountView,
    pub signer: &'info AccountView,
    pub token_mint_in: &'info AccountView,
    pub token_mint_out: &'info AccountView,
    pub pyth_price_in: &'info AccountView,
    pub pyth_price_out: &'info AccountView,
    pub signer_ata_in: &'info AccountView,
    pub signer_ata_out: &'info AccountView,
    pub vault_pda_in: &'info AccountView,
    pub vault_pda_out: &'info AccountView,
    pub vault_ata_in: &'info AccountView,
    pub vault_ata_out: &'info AccountView,
    pub oxe_global_pda: &'info AccountView,
    pub associated_token_program: &'info AccountView,
    pub token_program: &'info AccountView,
    pub system_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for OxediumSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < 16 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [
            oxedium_program,
            signer,
            token_mint_in,
            token_mint_out,
            pyth_price_in,
            pyth_price_out,
            signer_ata_in,
            signer_ata_out,
            vault_pda_in,
            vault_pda_out,
            vault_ata_in,
            vault_ata_out,
            oxe_global_pda,
            associated_token_program,
            token_program,
            system_program,
            ..
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(OxediumSwapAccounts {
            oxedium_program,
            signer,
            token_mint_in,
            token_mint_out,
            pyth_price_in,
            pyth_price_out,
            signer_ata_in,
            signer_ata_out,
            vault_pda_in,
            vault_pda_out,
            vault_ata_in,
            vault_ata_out,
            oxe_global_pda,
            associated_token_program,
            token_program,
            system_program,
        })
    }
}

impl<'info> Swap<'info> for Oxedium {
    type Accounts = OxediumSwapAccounts<'info>;
    /// No extra data — `in_amount` and `minimum_out_amount` are the only args.
    type Data = ();

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        _data: &(),
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::writable_signer(ctx.signer.address()),
            InstructionAccount::readonly(ctx.token_mint_in.address()),
            InstructionAccount::readonly(ctx.token_mint_out.address()),
            InstructionAccount::readonly(ctx.pyth_price_in.address()),
            InstructionAccount::readonly(ctx.pyth_price_out.address()),
            InstructionAccount::writable(ctx.signer_ata_in.address()),
            InstructionAccount::writable(ctx.signer_ata_out.address()),
            InstructionAccount::writable(ctx.vault_pda_in.address()),
            InstructionAccount::writable(ctx.vault_pda_out.address()),
            InstructionAccount::writable(ctx.vault_ata_in.address()),
            InstructionAccount::writable(ctx.vault_ata_out.address()),
            InstructionAccount::readonly(ctx.oxe_global_pda.address()),
            InstructionAccount::readonly(ctx.associated_token_program.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
            InstructionAccount::readonly(ctx.system_program.address()),
        ];

        let account_infos = [
            ctx.signer,
            ctx.token_mint_in,
            ctx.token_mint_out,
            ctx.pyth_price_in,
            ctx.pyth_price_out,
            ctx.signer_ata_in,
            ctx.signer_ata_out,
            ctx.vault_pda_in,
            ctx.vault_pda_out,
            ctx.vault_ata_in,
            ctx.vault_ata_out,
            ctx.oxe_global_pda,
            ctx.associated_token_program,
            ctx.token_program,
            ctx.system_program,
        ];

        // Instruction data: 8-byte Anchor discriminator + amount_in + minimum_out_amount
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
            program_id: &OXEDIUM_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 24)
            },
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
