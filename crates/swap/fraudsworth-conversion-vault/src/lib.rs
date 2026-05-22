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

pub const FRAUDSWORTH_CONVERSION_VAULT_PROGRAM_ID: Address =
    address!("5uawA6ehYTu69Ggvm3LSK84qFawPKxbWgfngwj15NRJ");

const CONVERT_V2_DISCRIMINATOR: [u8; 8] = [2, 169, 12, 141, 64, 38, 20, 20];

pub struct FraudsworthConversionVault;

pub struct FraudsworthConversionVaultSwapData {
    pub pre_balance: u64,
}

impl FraudsworthConversionVaultSwapData {
    pub const DATA_LEN: usize = 8;
}

impl TryFrom<&[u8]> for FraudsworthConversionVaultSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            pre_balance: u64::from_le_bytes(data[..8].try_into().unwrap()),
        })
    }
}

impl FraudsworthConversionVaultSwapAccounts<'_> {
    // CRIME, FRAUD and PROFIT all have 4 transfer hook accounts
    // 1 (protocol program) + 9 (base) + 4 (input transfer hooks) + 4 (output transfer hooks)
    pub const NUM_ACCOUNTS: usize = 18;
}

pub struct FraudsworthConversionVaultSwapAccounts<'info> {
    pub fraudsworth_conversion_vault_program: &'info AccountView,
    pub user: &'info AccountView,
    pub vault_config: &'info AccountView,
    pub user_input_account: &'info AccountView,
    pub user_output_account: &'info AccountView,
    pub input_mint: &'info AccountView,
    pub output_mint: &'info AccountView,
    pub vault_input: &'info AccountView,
    pub vault_output: &'info AccountView,
    pub token_program: &'info AccountView,
    pub input_mint_extra_account_meta_list: &'info AccountView,
    pub input_mint_whitelist_source: &'info AccountView,
    pub input_mint_whitelist_destination: &'info AccountView,
    pub input_mint_transfer_hook_program: &'info AccountView,
    pub output_mint_extra_account_meta_list: &'info AccountView,
    pub output_mint_whitelist_source: &'info AccountView,
    pub output_mint_whitelist_destination: &'info AccountView,
    pub output_mint_transfer_hook_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for FraudsworthConversionVaultSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [fraudsworth_conversion_vault_program, user, vault_config, user_input_account, user_output_account, input_mint, output_mint, vault_input, vault_output, token_program, input_mint_extra_account_meta_list, input_mint_whitelist_source, input_mint_whitelist_destination, input_mint_transfer_hook_program, output_mint_extra_account_meta_list, output_mint_whitelist_source, output_mint_whitelist_destination, output_mint_transfer_hook_program] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(Self {
            fraudsworth_conversion_vault_program,
            user,
            vault_config,
            user_input_account,
            user_output_account,
            input_mint,
            output_mint,
            vault_input,
            vault_output,
            token_program,
            input_mint_extra_account_meta_list,
            input_mint_whitelist_source,
            input_mint_whitelist_destination,
            input_mint_transfer_hook_program,
            output_mint_extra_account_meta_list,
            output_mint_whitelist_source,
            output_mint_whitelist_destination,
            output_mint_transfer_hook_program,
        })
    }
}

impl<'info> Swap<'info> for FraudsworthConversionVault {
    type Accounts = FraudsworthConversionVaultSwapAccounts<'info>;
    type Data = FraudsworthConversionVaultSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let instruction_accounts = [
            InstructionAccount::readonly_signer(ctx.user.address()),
            InstructionAccount::readonly(ctx.vault_config.address()),
            InstructionAccount::writable(ctx.user_input_account.address()),
            InstructionAccount::writable(ctx.user_output_account.address()),
            InstructionAccount::readonly(ctx.input_mint.address()),
            InstructionAccount::readonly(ctx.output_mint.address()),
            InstructionAccount::writable(ctx.vault_input.address()),
            InstructionAccount::writable(ctx.vault_output.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
            InstructionAccount::readonly(ctx.input_mint_extra_account_meta_list.address()),
            InstructionAccount::readonly(ctx.input_mint_whitelist_source.address()),
            InstructionAccount::readonly(ctx.input_mint_whitelist_destination.address()),
            InstructionAccount::readonly(ctx.input_mint_transfer_hook_program.address()),
            InstructionAccount::readonly(ctx.output_mint_extra_account_meta_list.address()),
            InstructionAccount::readonly(ctx.output_mint_whitelist_source.address()),
            InstructionAccount::readonly(ctx.output_mint_whitelist_destination.address()),
            InstructionAccount::readonly(ctx.output_mint_transfer_hook_program.address()),
        ];

        let account_views = [
            ctx.user,
            ctx.vault_config,
            ctx.user_input_account,
            ctx.user_output_account,
            ctx.input_mint,
            ctx.output_mint,
            ctx.vault_input,
            ctx.vault_output,
            ctx.token_program,
            ctx.input_mint_extra_account_meta_list,
            ctx.input_mint_whitelist_source,
            ctx.input_mint_whitelist_destination,
            ctx.input_mint_transfer_hook_program,
            ctx.output_mint_extra_account_meta_list,
            ctx.output_mint_whitelist_source,
            ctx.output_mint_whitelist_destination,
            ctx.output_mint_transfer_hook_program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 32]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(CONVERT_V2_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(16),
                8,
            );
            core::ptr::copy_nonoverlapping(data.pre_balance.to_le_bytes().as_ptr(), ptr.add(24), 8);
        }

        let instruction = InstructionView {
            program_id: &FRAUDSWORTH_CONVERSION_VAULT_PROGRAM_ID,
            accounts: &instruction_accounts,
            data: unsafe { instruction_data.assume_init_ref() },
        };

        invoke_signed(&instruction, &account_views, signer_seeds)
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
