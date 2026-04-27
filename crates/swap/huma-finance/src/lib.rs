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

pub const HUMA_FINANCE_PROGRAM_ID: Address =
    address!("HumaXepHnjaRCpjYTokxY4UtaJcmx41prQ8cxGmFC5fn");

const DEPOSIT_DISCRIMINATOR: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182];
const INSTANT_WITHDRAW_DISCRIMINATOR: [u8; 8] = [171, 49, 145, 176, 48, 101, 112, 162];

// "NO_COMMITMENT" borsh-encoded string: 4-byte LE length + 13 UTF-8 bytes
const NO_COMMITMENT_LEN: [u8; 4] = [13, 0, 0, 0];
const NO_COMMITMENT_BYTES: [u8; 13] = [78, 79, 95, 67, 79, 77, 77, 73, 84, 77, 69, 78, 84];

const DEPOSIT_ACCOUNTS_LEN: usize = 13;
const INSTANT_WITHDRAW_ACCOUNTS_LEN: usize = 15;
const MAX_ACCOUNTS: usize = INSTANT_WITHDRAW_ACCOUNTS_LEN;

// 8 - discriminator
// 8 - assets
// 4 - no_commitment_len
// 13 - no_commitment_bytes
// 1 - commitment_auto_renewal
const DEPOSIT_DATA_LEN: usize = 34;
// 8 - discriminator
// 16 - shares
const INSTANT_WITHDRAW_DATA_LEN: usize = 24;
const MAX_DATA_LEN: usize = DEPOSIT_DATA_LEN;

pub struct HumaFinance;

pub struct HumaFinanceSwapBaseAccounts<'info> {
    // payer is also depositor for deposit and lender for withdrawals
    pub payer: &'info AccountView,
    pub huma_config: &'info AccountView,
    pub pool_config: &'info AccountView,
    pub pool_state: &'info AccountView,
}

pub enum HumaFinanceSwapLegAccounts<'info> {
    Deposit {
        mode_config: &'info AccountView,
        mode_mint: &'info AccountView,
        pool_authority: &'info AccountView,
        underlying_mint: &'info AccountView,
        pool_underlying_token: &'info AccountView,
        depositor_underlying_token: &'info AccountView,
        depositor_mode_token: &'info AccountView,
        underlying_token_program: &'info AccountView,
        mode_token_program: &'info AccountView,
    },
    InstantWithdraw {
        instant_withdrawal_lender_config: &'info AccountView,
        mode_config: &'info AccountView,
        mode_mint: &'info AccountView,
        lender_state: &'info AccountView,
        underlying_mint: &'info AccountView,
        pool_authority: &'info AccountView,
        pool_underlying_token: &'info AccountView,
        lender_underlying_token: &'info AccountView,
        lender_mode_token: &'info AccountView,
        underlying_token_program: &'info AccountView,
        mode_token_program: &'info AccountView,
    },
}

pub struct HumaFinanceSwapAccounts<'info> {
    pub base: HumaFinanceSwapBaseAccounts<'info>,
    pub leg: HumaFinanceSwapLegAccounts<'info>,
}

impl HumaFinanceSwapAccounts<'_> {
    pub const MIN_NUM_ACCOUNTS: usize = 14;
}

impl<'info> TryFrom<&'info [AccountView]> for HumaFinanceSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [_huma_finance_program, depositor, huma_config, pool_config, pool_state, ..] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if accounts.len() < Self::MIN_NUM_ACCOUNTS {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let base = HumaFinanceSwapBaseAccounts {
            payer: depositor,
            huma_config,
            pool_config,
            pool_state,
        };

        let leg = match accounts.len() - 1 {
            DEPOSIT_ACCOUNTS_LEN => HumaFinanceSwapLegAccounts::Deposit {
                mode_config: &accounts[5],
                mode_mint: &accounts[6],
                pool_authority: &accounts[7],
                underlying_mint: &accounts[8],
                pool_underlying_token: &accounts[9],
                depositor_underlying_token: &accounts[10],
                depositor_mode_token: &accounts[11],
                underlying_token_program: &accounts[12],
                mode_token_program: &accounts[13],
            },
            INSTANT_WITHDRAW_ACCOUNTS_LEN => HumaFinanceSwapLegAccounts::InstantWithdraw {
                instant_withdrawal_lender_config: &accounts[5],
                mode_config: &accounts[6],
                mode_mint: &accounts[7],
                lender_state: &accounts[8],
                underlying_mint: &accounts[9],
                pool_authority: &accounts[10],
                pool_underlying_token: &accounts[11],
                lender_underlying_token: &accounts[12],
                lender_mode_token: &accounts[13],
                underlying_token_program: &accounts[14],
                mode_token_program: &accounts[15],
            },
            _ => return Err(ProgramError::NotEnoughAccountKeys),
        };

        Ok(HumaFinanceSwapAccounts { base, leg })
    }
}

impl<'info> Swap<'info> for HumaFinance {
    type Accounts = HumaFinanceSwapAccounts<'info>;
    type Data = ();

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        _minimum_out_amount: u64,
        _data: &(),
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        // lender state needs to exist for lender before invoking instant_withdraw

        let mut instruction_accounts = MaybeUninit::<[InstructionAccount; MAX_ACCOUNTS]>::uninit();
        let instruction_accounts_ptr = instruction_accounts.as_mut_ptr() as *mut InstructionAccount;

        unsafe {
            core::ptr::write(
                instruction_accounts_ptr,
                InstructionAccount::writable_signer(ctx.base.payer.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(1),
                InstructionAccount::readonly(ctx.base.huma_config.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(2),
                InstructionAccount::readonly(ctx.base.pool_config.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(3),
                InstructionAccount::writable(ctx.base.pool_state.address()),
            );

            match ctx.leg {
                HumaFinanceSwapLegAccounts::Deposit {
                    mode_config,
                    mode_mint,
                    pool_authority,
                    underlying_mint,
                    pool_underlying_token,
                    depositor_underlying_token,
                    depositor_mode_token,
                    underlying_token_program,
                    mode_token_program,
                } => {
                    core::ptr::write(
                        instruction_accounts_ptr.add(4),
                        InstructionAccount::readonly(mode_config.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(5),
                        InstructionAccount::writable(mode_mint.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(6),
                        InstructionAccount::readonly(pool_authority.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(7),
                        InstructionAccount::readonly(underlying_mint.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(8),
                        InstructionAccount::writable(pool_underlying_token.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(9),
                        InstructionAccount::writable(depositor_underlying_token.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(10),
                        InstructionAccount::writable(depositor_mode_token.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(11),
                        InstructionAccount::readonly(underlying_token_program.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(12),
                        InstructionAccount::readonly(mode_token_program.address()),
                    );
                }
                HumaFinanceSwapLegAccounts::InstantWithdraw {
                    instant_withdrawal_lender_config,
                    mode_config,
                    mode_mint,
                    lender_state,
                    underlying_mint,
                    pool_authority,
                    pool_underlying_token,
                    lender_underlying_token,
                    lender_mode_token,
                    underlying_token_program,
                    mode_token_program,
                } => {
                    core::ptr::write(
                        instruction_accounts_ptr.add(4),
                        InstructionAccount::readonly(instant_withdrawal_lender_config.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(5),
                        InstructionAccount::readonly(mode_config.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(6),
                        InstructionAccount::writable(mode_mint.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(7),
                        InstructionAccount::writable(lender_state.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(8),
                        InstructionAccount::readonly(underlying_mint.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(9),
                        InstructionAccount::readonly(pool_authority.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(10),
                        InstructionAccount::writable(pool_underlying_token.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(11),
                        InstructionAccount::writable(lender_underlying_token.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(12),
                        InstructionAccount::writable(lender_mode_token.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(13),
                        InstructionAccount::readonly(underlying_token_program.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(14),
                        InstructionAccount::readonly(mode_token_program.address()),
                    );
                }
            }
        }

        let total_accounts = match ctx.leg {
            HumaFinanceSwapLegAccounts::Deposit { .. } => DEPOSIT_ACCOUNTS_LEN,
            HumaFinanceSwapLegAccounts::InstantWithdraw { .. } => INSTANT_WITHDRAW_ACCOUNTS_LEN,
        };

        let instruction_accounts =
            unsafe { core::slice::from_raw_parts(instruction_accounts_ptr, total_accounts) };

        let mut account_views = [ctx.base.payer; MAX_ACCOUNTS];
        account_views[1] = ctx.base.huma_config;
        account_views[2] = ctx.base.pool_config;
        account_views[3] = ctx.base.pool_state;

        match ctx.leg {
            HumaFinanceSwapLegAccounts::Deposit {
                mode_config,
                mode_mint,
                pool_authority,
                underlying_mint,
                pool_underlying_token,
                depositor_underlying_token,
                depositor_mode_token,
                underlying_token_program,
                mode_token_program,
            } => {
                account_views[4] = mode_config;
                account_views[5] = mode_mint;
                account_views[6] = pool_authority;
                account_views[7] = underlying_mint;
                account_views[8] = pool_underlying_token;
                account_views[9] = depositor_underlying_token;
                account_views[10] = depositor_mode_token;
                account_views[11] = underlying_token_program;
                account_views[12] = mode_token_program;
            }
            HumaFinanceSwapLegAccounts::InstantWithdraw {
                instant_withdrawal_lender_config,
                mode_config,
                mode_mint,
                lender_state,
                underlying_mint,
                pool_authority,
                pool_underlying_token,
                lender_underlying_token,
                lender_mode_token,
                underlying_token_program,
                mode_token_program,
            } => {
                account_views[4] = instant_withdrawal_lender_config;
                account_views[5] = mode_config;
                account_views[6] = mode_mint;
                account_views[7] = lender_state;
                account_views[8] = underlying_mint;
                account_views[9] = pool_authority;
                account_views[10] = pool_underlying_token;
                account_views[11] = lender_underlying_token;
                account_views[12] = lender_mode_token;
                account_views[13] = underlying_token_program;
                account_views[14] = mode_token_program;
            }
        }

        let account_views = &account_views[..total_accounts];

        let mut data = MaybeUninit::<[u8; MAX_DATA_LEN]>::uninit();

        let discriminator = match ctx.leg {
            HumaFinanceSwapLegAccounts::Deposit { .. } => &DEPOSIT_DISCRIMINATOR,
            HumaFinanceSwapLegAccounts::InstantWithdraw { .. } => &INSTANT_WITHDRAW_DISCRIMINATOR,
        };

        unsafe {
            let ptr = data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(discriminator.as_ptr(), ptr, 8);

            match ctx.leg {
                HumaFinanceSwapLegAccounts::Deposit { .. } => {
                    core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
                    core::ptr::copy_nonoverlapping(NO_COMMITMENT_LEN.as_ptr(), ptr.add(16), 4);
                    core::ptr::copy_nonoverlapping(NO_COMMITMENT_BYTES.as_ptr(), ptr.add(20), 13);
                    // commitment_auto_renewal = false
                    core::ptr::write(ptr.add(33), 0);
                }
                HumaFinanceSwapLegAccounts::InstantWithdraw { .. } => {
                    core::ptr::copy_nonoverlapping(
                        (in_amount as u128).to_le_bytes().as_ptr(),
                        ptr.add(8),
                        16,
                    );
                }
            }
        }

        let data_len = match ctx.leg {
            HumaFinanceSwapLegAccounts::Deposit { .. } => DEPOSIT_DATA_LEN,
            HumaFinanceSwapLegAccounts::InstantWithdraw { .. } => INSTANT_WITHDRAW_DATA_LEN,
        };

        let instruction = InstructionView {
            program_id: &HUMA_FINANCE_PROGRAM_ID,
            accounts: instruction_accounts,
            data: unsafe { core::slice::from_raw_parts(data.as_ptr() as *const u8, data_len) },
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
