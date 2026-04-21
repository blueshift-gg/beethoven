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

pub const SYNATRA_PROGRAM_ID: Address = address!("synatfE5AvWtbDT9sSvDsF9gmeqR9qeq3FA84bhxWur");
const YSOL_MINT: Address = address!("yso11zxLbHA3wBJ9HAtVu6wnesqz9A2qxnhxanasZ4N");

const STAKE_SOL_DISCRIMINATOR: [u8; 8] = [200, 38, 157, 155, 245, 57, 236, 168];
const STAKE_TOKEN_DISCRIMINATOR: [u8; 8] = [191, 127, 193, 101, 37, 96, 87, 211];
const STAKE_SOL_ACCOUNTS_LEN: usize = 8;
const STAKE_TOKEN_ACCOUNTS_LEN: usize = 11;
const MAX_ACCOUNTS: usize = 11;

pub struct Synatra;

impl SynatraSwapAccounts<'_> {
    pub const MIN_NUM_ACCOUNTS: usize = 9;
}

pub struct SynatraSwapBaseAccounts<'info> {
    pub synatra_program: &'info AccountView,
    pub signer: &'info AccountView,
    pub payer: &'info AccountView,
    pub pool: &'info AccountView,
}

pub struct SynatraStakeLegAccounts<'info> {
    pub associated_token_program: &'info AccountView,
    pub token_program: &'info AccountView,
    pub system_program: &'info AccountView,
}

pub enum SynatraSwapType<'info> {
    StakeSol {
        receipt_token: &'info AccountView,
        user_receipt_ata: &'info AccountView,
    },
    StakeToken {
        stake_token: &'info AccountView,
        receipt_token: &'info AccountView,
        user_token_ata: &'info AccountView,
        user_receipt_ata: &'info AccountView,
        pool_token_ata: &'info AccountView,
    },
}

pub struct SynatraSwapAccounts<'info> {
    pub base: SynatraSwapBaseAccounts<'info>,
    pub swap_type: SynatraSwapType<'info>,
    pub leg: SynatraStakeLegAccounts<'info>,
}

impl<'info> TryFrom<&'info [AccountView]> for SynatraSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [synatra_program, signer, payer, pool, remaining_accounts @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if remaining_accounts.len() < 5 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let base = SynatraSwapBaseAccounts {
            synatra_program,
            signer,
            payer,
            pool,
        };

        let mut index = 0;

        // only the stake_sol instruction requires ySOL at index 3
        let swap_type = if remaining_accounts[0].address().eq(&YSOL_MINT) {
            index += 2;

            SynatraSwapType::StakeSol {
                receipt_token: &remaining_accounts[0],
                user_receipt_ata: &remaining_accounts[1],
            }
        } else {
            index += 5;

            SynatraSwapType::StakeToken {
                stake_token: &remaining_accounts[0],
                receipt_token: &remaining_accounts[1],
                user_receipt_ata: &remaining_accounts[2],
                user_token_ata: &remaining_accounts[3],
                pool_token_ata: &remaining_accounts[4],
            }
        };

        let leg = SynatraStakeLegAccounts {
            associated_token_program: &remaining_accounts[index],
            token_program: &remaining_accounts[index + 1],
            system_program: &remaining_accounts[index + 2],
        };

        Ok(SynatraSwapAccounts {
            base,
            swap_type,
            leg,
        })
    }
}

impl<'info> Swap<'info> for Synatra {
    type Accounts = SynatraSwapAccounts<'info>;
    type Data = ();

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        _minimum_out_amount: u64,
        _data: &(),
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let total_accounts = match ctx.swap_type {
            SynatraSwapType::StakeSol { .. } => STAKE_SOL_ACCOUNTS_LEN,
            SynatraSwapType::StakeToken { .. } => STAKE_TOKEN_ACCOUNTS_LEN,
        };

        let mut instruction_accounts = MaybeUninit::<[InstructionAccount; MAX_ACCOUNTS]>::uninit();
        let instruction_accounts_ptr = instruction_accounts.as_mut_ptr() as *mut InstructionAccount;
        let mut index = 0;

        unsafe {
            core::ptr::write(
                instruction_accounts_ptr,
                InstructionAccount::writable(ctx.base.signer.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(index + 1),
                InstructionAccount::writable_signer(ctx.base.payer.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(index + 2),
                InstructionAccount::writable(ctx.base.pool.address()),
            );

            match ctx.swap_type {
                SynatraSwapType::StakeSol {
                    receipt_token,
                    user_receipt_ata,
                } => {
                    core::ptr::write(
                        instruction_accounts_ptr.add(index + 3),
                        InstructionAccount::writable(receipt_token.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(index + 4),
                        InstructionAccount::writable(user_receipt_ata.address()),
                    );
                    index = 5;
                }
                SynatraSwapType::StakeToken {
                    stake_token,
                    receipt_token,
                    user_receipt_ata,
                    user_token_ata,
                    pool_token_ata,
                } => {
                    core::ptr::write(
                        instruction_accounts_ptr.add(index + 3),
                        InstructionAccount::writable(stake_token.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(index + 4),
                        InstructionAccount::writable(receipt_token.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(index + 5),
                        InstructionAccount::writable(user_receipt_ata.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(index + 6),
                        InstructionAccount::writable(user_token_ata.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(index + 7),
                        InstructionAccount::writable(pool_token_ata.address()),
                    );
                    index = 8;
                }
            }

            core::ptr::write(
                instruction_accounts_ptr.add(index),
                InstructionAccount::readonly(ctx.leg.associated_token_program.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(index + 1),
                InstructionAccount::readonly(ctx.leg.token_program.address()),
            );
            core::ptr::write(
                instruction_accounts_ptr.add(index + 2),
                InstructionAccount::readonly(ctx.leg.system_program.address()),
            );
        }

        let instruction_accounts =
            unsafe { core::slice::from_raw_parts(instruction_accounts_ptr, total_accounts) };

        let mut account_views = [ctx.base.signer; MAX_ACCOUNTS];

        account_views[1] = ctx.base.payer;
        account_views[2] = ctx.base.pool;

        let index = match ctx.swap_type {
            SynatraSwapType::StakeSol {
                receipt_token,
                user_receipt_ata,
            } => {
                account_views[3] = receipt_token;
                account_views[4] = user_receipt_ata;
                5
            }
            SynatraSwapType::StakeToken {
                stake_token,
                receipt_token,
                user_receipt_ata,
                user_token_ata,
                pool_token_ata,
            } => {
                account_views[3] = stake_token;
                account_views[4] = receipt_token;
                account_views[5] = user_receipt_ata;
                account_views[6] = user_token_ata;
                account_views[7] = pool_token_ata;
                8
            }
        };

        account_views[index] = ctx.leg.associated_token_program;
        account_views[index + 1] = ctx.leg.token_program;
        account_views[index + 2] = ctx.leg.system_program;

        let account_views = &account_views[..total_accounts];

        let mut instruction_data = MaybeUninit::<[u8; 16]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            let discriminator = match ctx.swap_type {
                SynatraSwapType::StakeSol { .. } => STAKE_SOL_DISCRIMINATOR,
                SynatraSwapType::StakeToken { .. } => STAKE_TOKEN_DISCRIMINATOR,
            };
            core::ptr::copy_nonoverlapping(discriminator.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
        }

        let instruction = InstructionView {
            program_id: &SYNATRA_PROGRAM_ID,
            accounts: instruction_accounts,
            data: unsafe { instruction_data.assume_init_ref() },
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
