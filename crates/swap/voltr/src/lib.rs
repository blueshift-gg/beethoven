#![no_std]

use {
    beethoven_core::Swap,
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::{address, Address},
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const VOLTR_PROGRAM_ID: Address = address!("vVoLTRjQmtFpiYoegx285Ze4gsLJ8ZxgFKVcuvmG1a8");

const DEPOSIT_VAULT_DISCRIMINATOR: [u8; 8] = [126, 224, 21, 255, 228, 53, 117, 33];
const INSTANT_WITHDRAW_VAULT_DISCRIMINATOR: [u8; 8] = [221, 56, 115, 168, 128, 220, 235, 245];

pub const DEPOSIT_VAULT_NUM_ACCOUNTS: usize = 14;
pub const INSTANT_WITHDRAW_VAULT_NUM_ACCOUNTS: usize = 13;
const MAX_ACCOUNTS: usize = 13;

pub struct Voltr;

pub enum VoltrSwapData {
    DepositVault,
    InstantWithdrawVault {
        is_amount_in_lp: bool,
        is_withdraw_all: bool,
    },
}

impl VoltrSwapData {
    pub const INSTANT_WITHDRAW_VAULT_DATA_LEN: usize = 2;
}

impl TryFrom<&[u8]> for VoltrSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        match data.len() {
            0 => Ok(VoltrSwapData::DepositVault),
            2 => Ok(VoltrSwapData::InstantWithdrawVault {
                is_amount_in_lp: data[0] != 0,
                is_withdraw_all: data[1] != 0,
            }),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

pub struct VoltrSwapBaseAccounts<'info> {
    pub voltr_program: &'info AccountView,
    pub user_transfer_authority: &'info AccountView,
    pub protocol: &'info AccountView,
    pub vault: &'info AccountView,
    pub vault_asset_mint: &'info AccountView,
    pub vault_lp_mint: &'info AccountView,
}

pub enum VoltrSwapLegAccounts<'info> {
    DepositVault {
        user_asset_ata: &'info AccountView,
        vault_asset_idle_ata: &'info AccountView,
        vault_asset_idle_auth: &'info AccountView,
        user_lp_ata: &'info AccountView,
        vault_lp_mint_auth: &'info AccountView,
    },
    InstantWithdrawVault {
        user_lp_ata: &'info AccountView,
        vault_asset_idle_ata: &'info AccountView,
        vault_asset_idle_auth: &'info AccountView,
        user_asset_ata: &'info AccountView,
    },
}

pub struct VoltrSwapTailAccounts<'info> {
    pub asset_token_program: &'info AccountView,
    pub lp_token_program: &'info AccountView,
    pub system_program: &'info AccountView,
}

pub struct VoltrSwapAccounts<'info> {
    base: VoltrSwapBaseAccounts<'info>,
    leg: VoltrSwapLegAccounts<'info>,
    tail: VoltrSwapTailAccounts<'info>,
}

impl VoltrSwapAccounts<'_> {
    pub const MIN_NUM_ACCOUNTS: usize = INSTANT_WITHDRAW_VAULT_NUM_ACCOUNTS;
}

impl<'info> TryFrom<&'info [AccountView]> for VoltrSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [voltr_program, user_transfer_authority, protocol, vault, vault_asset_mint, vault_lp_mint, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if accounts.len() < Self::MIN_NUM_ACCOUNTS {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let base = VoltrSwapBaseAccounts {
            voltr_program,
            user_transfer_authority,
            protocol,
            vault,
            vault_asset_mint,
            vault_lp_mint,
        };

        let (leg, index) = match accounts.len() {
            DEPOSIT_VAULT_NUM_ACCOUNTS => {
                let leg = VoltrSwapLegAccounts::DepositVault {
                    user_asset_ata: &accounts[6],
                    vault_asset_idle_ata: &accounts[7],
                    vault_asset_idle_auth: &accounts[8],
                    user_lp_ata: &accounts[9],
                    vault_lp_mint_auth: &accounts[10],
                };

                (leg, 11)
            }
            INSTANT_WITHDRAW_VAULT_NUM_ACCOUNTS => {
                let leg = VoltrSwapLegAccounts::InstantWithdrawVault {
                    user_lp_ata: &accounts[6],
                    vault_asset_idle_ata: &accounts[7],
                    vault_asset_idle_auth: &accounts[8],
                    user_asset_ata: &accounts[9],
                };

                (leg, 10)
            }
            _ => return Err(ProgramError::NotEnoughAccountKeys),
        };

        let tail = VoltrSwapTailAccounts {
            asset_token_program: &accounts[index],
            lp_token_program: &accounts[index + 1],
            system_program: &accounts[index + 2],
        };

        Ok(Self { base, leg, tail })
    }
}

impl<'info> Swap<'info> for Voltr {
    type Accounts = VoltrSwapAccounts<'info>;
    type Data = VoltrSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        _minimum_out_amount: u64,
        data: &VoltrSwapData,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let mut instruction_accounts = MaybeUninit::<[InstructionAccount; MAX_ACCOUNTS]>::uninit();
        let instruction_accounts_ptr = instruction_accounts.as_mut_ptr() as *mut InstructionAccount;

        unsafe {
            core::ptr::write(
                instruction_accounts_ptr,
                InstructionAccount::writable_signer(ctx.base.user_transfer_authority.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(1),
                InstructionAccount::readonly(ctx.base.protocol.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(2),
                InstructionAccount::writable(ctx.base.vault.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(3),
                InstructionAccount::readonly(ctx.base.vault_asset_mint.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(4),
                InstructionAccount::writable(ctx.base.vault_lp_mint.address()),
            );

            let index = match ctx.leg {
                VoltrSwapLegAccounts::DepositVault {
                    user_asset_ata,
                    vault_asset_idle_ata,
                    vault_asset_idle_auth,
                    user_lp_ata,
                    vault_lp_mint_auth,
                } => {
                    core::ptr::write(
                        instruction_accounts_ptr.add(5),
                        InstructionAccount::writable(user_asset_ata.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(6),
                        InstructionAccount::writable(vault_asset_idle_ata.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(7),
                        InstructionAccount::readonly(vault_asset_idle_auth.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(8),
                        InstructionAccount::writable(user_lp_ata.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(9),
                        InstructionAccount::readonly(vault_lp_mint_auth.address()),
                    );

                    10
                }
                VoltrSwapLegAccounts::InstantWithdrawVault {
                    user_lp_ata,
                    vault_asset_idle_ata,
                    vault_asset_idle_auth,
                    user_asset_ata,
                } => {
                    core::ptr::write(
                        instruction_accounts_ptr.add(5),
                        InstructionAccount::writable(user_lp_ata.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(6),
                        InstructionAccount::writable(vault_asset_idle_ata.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(7),
                        InstructionAccount::writable(vault_asset_idle_auth.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(8),
                        InstructionAccount::writable(user_asset_ata.address()),
                    );

                    9
                }
            };

            core::ptr::write(
                instruction_accounts_ptr.add(index),
                InstructionAccount::readonly(ctx.tail.asset_token_program.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(index + 1),
                InstructionAccount::readonly(ctx.tail.lp_token_program.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(index + 2),
                InstructionAccount::readonly(ctx.tail.system_program.address()),
            );
        };

        let instruction_accounts =
            unsafe { core::slice::from_raw_parts(instruction_accounts_ptr, MAX_ACCOUNTS) };

        let mut account_views = [ctx.base.user_transfer_authority; MAX_ACCOUNTS];
        account_views[1] = ctx.base.protocol;
        account_views[2] = ctx.base.vault;
        account_views[3] = ctx.base.vault_asset_mint;
        account_views[4] = ctx.base.vault_lp_mint;

        let index = match ctx.leg {
            VoltrSwapLegAccounts::DepositVault {
                user_asset_ata,
                vault_asset_idle_ata,
                vault_asset_idle_auth,
                user_lp_ata,
                vault_lp_mint_auth,
            } => {
                account_views[5] = user_asset_ata;
                account_views[6] = vault_asset_idle_ata;
                account_views[7] = vault_asset_idle_auth;
                account_views[8] = user_lp_ata;
                account_views[9] = vault_lp_mint_auth;

                10
            }
            VoltrSwapLegAccounts::InstantWithdrawVault {
                user_lp_ata,
                vault_asset_idle_ata,
                vault_asset_idle_auth,
                user_asset_ata,
            } => {
                account_views[5] = user_lp_ata;
                account_views[6] = vault_asset_idle_ata;
                account_views[7] = vault_asset_idle_auth;
                account_views[8] = user_asset_ata;

                9
            }
        };

        account_views[index] = ctx.tail.asset_token_program;
        account_views[index + 1] = ctx.tail.lp_token_program;
        account_views[index + 2] = ctx.tail.system_program;

        let account_views = &account_views[..MAX_ACCOUNTS];

        let data_len = match data {
            VoltrSwapData::DepositVault => 16,
            VoltrSwapData::InstantWithdrawVault { .. } => 18,
        };

        let mut instruction_data = MaybeUninit::<[u8; 18]>::uninit();

        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            let discriminator = match data {
                VoltrSwapData::DepositVault => DEPOSIT_VAULT_DISCRIMINATOR,
                VoltrSwapData::InstantWithdrawVault { .. } => INSTANT_WITHDRAW_VAULT_DISCRIMINATOR,
            };
            core::ptr::copy_nonoverlapping(discriminator.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);

            match data {
                VoltrSwapData::DepositVault => {}
                VoltrSwapData::InstantWithdrawVault {
                    is_amount_in_lp,
                    is_withdraw_all,
                } => {
                    core::ptr::write(ptr.add(16), *is_amount_in_lp as u8);
                    core::ptr::write(ptr.add(17), *is_withdraw_all as u8);
                }
            }
        }

        let instruction = InstructionView {
            program_id: &VOLTR_PROGRAM_ID,
            accounts: instruction_accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, data_len)
            },
        };

        invoke_signed_with_bounds::<MAX_ACCOUNTS, _>(&instruction, account_views, signer_seeds)
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
