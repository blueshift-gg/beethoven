#![no_std]

use {
    beethoven_core::Deposit,
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const DRIFT_PROGRAM_ID: Address =
    Address::from_str_const("dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH");
const DEPOSIT_DISCRIMINATOR: [u8; 8] = [242, 35, 198, 137, 82, 225, 242, 182];
const DEPOSIT_DATA_LEN: usize = 19;
// balanced estimation without blowing the stack
const MAX_DEPOSIT_ACCOUNTS: usize = 16;

pub struct Drift;

pub struct DriftDepositData {
    pub market_index: u16,
    pub reduce_only: u8,
}

impl DriftDepositData {
    // 2 - market index
    // 1 - reduce only
    pub const DATA_LEN: usize = 3;
}

impl TryFrom<&[u8]> for DriftDepositData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            market_index: u16::from_le_bytes(data[0..2].try_into().unwrap()),
            reduce_only: data[2],
        })
    }
}

pub struct DriftDepositAccounts<'info> {
    pub drift_program: &'info AccountView,
    pub state: &'info AccountView,
    pub user: &'info AccountView,
    pub user_stats: &'info AccountView,
    pub authority: &'info AccountView,
    pub spot_market_vault: &'info AccountView,
    pub user_token_account: &'info AccountView,
    pub token_program: &'info AccountView,
    pub remaining_accounts: &'info [AccountView],
}

impl<'info> TryFrom<&'info [AccountView]> for DriftDepositAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [drift_program, state, user, user_stats, authority, spot_market_vault, user_token_account, token_program, remaining_accounts @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(Self {
            drift_program,
            state,
            user,
            user_stats,
            authority,
            spot_market_vault,
            user_token_account,
            token_program,
            remaining_accounts,
        })
    }
}

impl<'info> Deposit<'info> for Drift {
    type Accounts = DriftDepositAccounts<'info>;
    type Data = DriftDepositData;

    fn deposit_signed(
        ctx: &DriftDepositAccounts<'info>,
        amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let total_accounts = 7 + ctx.remaining_accounts.len();
        if total_accounts > MAX_DEPOSIT_ACCOUNTS {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let mut account_metas = MaybeUninit::<[InstructionAccount; MAX_DEPOSIT_ACCOUNTS]>::uninit();
        let account_metas_ptr = account_metas.as_mut_ptr() as *mut InstructionAccount;

        unsafe {
            core::ptr::write(
                account_metas_ptr,
                InstructionAccount::readonly(ctx.state.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(1),
                InstructionAccount::writable(ctx.user.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(2),
                InstructionAccount::writable(ctx.user_stats.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(3),
                InstructionAccount::writable_signer(ctx.authority.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(4),
                InstructionAccount::writable(ctx.spot_market_vault.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(5),
                InstructionAccount::writable(ctx.user_token_account.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(6),
                InstructionAccount::readonly(ctx.token_program.address()),
            );

            for (index, account) in ctx.remaining_accounts.iter().enumerate() {
                core::ptr::write(
                    account_metas_ptr.add(7 + index),
                    InstructionAccount::from(account),
                );
            }
        }

        let account_metas =
            unsafe { core::slice::from_raw_parts(account_metas_ptr, total_accounts) };

        let mut account_infos = [ctx.state; MAX_DEPOSIT_ACCOUNTS];
        account_infos[1] = ctx.user;
        account_infos[2] = ctx.user_stats;
        account_infos[3] = ctx.authority;
        account_infos[4] = ctx.spot_market_vault;
        account_infos[5] = ctx.user_token_account;
        account_infos[6] = ctx.token_program;
        for (index, account) in ctx.remaining_accounts.iter().enumerate() {
            account_infos[7 + index] = account;
        }
        let account_infos = &account_infos[..total_accounts];

        let mut instruction_data = MaybeUninit::<[u8; DEPOSIT_DATA_LEN]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(DEPOSIT_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(data.market_index.to_le_bytes().as_ptr(), ptr.add(8), 2);
            core::ptr::copy_nonoverlapping(amount.to_le_bytes().as_ptr(), ptr.add(10), 8);
            *ptr.add(18) = data.reduce_only;
        }

        let deposit_ix = InstructionView {
            program_id: &DRIFT_PROGRAM_ID,
            accounts: account_metas,
            data: unsafe { instruction_data.assume_init_ref() },
        };

        invoke_signed_with_bounds::<MAX_DEPOSIT_ACCOUNTS>(
            &deposit_ix,
            account_infos,
            signer_seeds,
        )?;

        Ok(())
    }

    fn deposit(ctx: &DriftDepositAccounts<'info>, amount: u64, data: &Self::Data) -> ProgramResult {
        Self::deposit_signed(ctx, amount, data, &[])
    }
}
