#![no_std]

use {
    beethoven_core::Deposit,
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const KAMINO_LEND_PROGRAM_ID: Address = Address::new_from_array([0; 32]);
const REFRESH_RESERVE_DISCRIMINATOR: [u8; 8] = [2, 218, 138, 235, 79, 201, 25, 102];
const REFRESH_OBLIGATION_DISCRIMINATOR: [u8; 8] = [33, 132, 147, 228, 151, 192, 72, 89];
const DEPOSIT_RESERVE_LIQUIDITY_AND_OBLIGATION_COLLATERAL_V2_DISCRIMINATOR: [u8; 8] =
    [216, 224, 191, 27, 204, 151, 102, 175];

pub struct Kamino;

pub struct KaminoDepositAccounts<'info> {
    pub kamino_lending_program: &'info AccountView,
    pub owner: &'info AccountView,
    pub obligation: &'info AccountView,
    pub lending_market: &'info AccountView,
    pub lending_market_authority: &'info AccountView,
    pub reserve: &'info AccountView,
    pub reserve_liquidity_mint: &'info AccountView,
    pub reserve_liquidity_supply: &'info AccountView,
    pub reserve_collateral_mint: &'info AccountView,
    pub reserve_destination_deposit_collateral: &'info AccountView,
    pub user_source_liquidity: &'info AccountView,
    pub placeholder_user_destination_collateral: &'info AccountView,
    pub collateral_token_program: &'info AccountView,
    pub liquidity_token_program: &'info AccountView,
    pub instruction_sysvar_account: &'info AccountView,
    pub obligation_farm_user_state: &'info AccountView,
    pub reserve_farm_state: &'info AccountView,
    pub farms_program: &'info AccountView,
    pub scope_oracle: &'info AccountView,
    pub reserve_accounts: &'info [AccountView],
}

impl<'info> TryFrom<&'info [AccountView]> for KaminoDepositAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < 19 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [kamino_lending_program, owner, obligation, lending_market, lending_market_authority, reserve, reserve_liquidity_mint, reserve_liquidity_supply, reserve_collateral_mint, reserve_destination_deposit_collateral, user_source_liquidity, placeholder_user_destination_collateral, collateral_token_program, liquidity_token_program, instruction_sysvar_account, obligation_farm_user_state, reserve_farm_state, farms_program, scope_oracle, remaining_accounts @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        let mut total_reserve_accounts = 0;
        for reserve in remaining_accounts {
            if reserve.owned_by(&KAMINO_LEND_PROGRAM_ID) && total_reserve_accounts < 13 {
                total_reserve_accounts += 1;
            } else {
                break;
            }
        }

        Ok(KaminoDepositAccounts {
            owner,
            obligation,
            lending_market,
            lending_market_authority,
            reserve,
            reserve_liquidity_mint,
            reserve_liquidity_supply,
            reserve_collateral_mint,
            reserve_destination_deposit_collateral,
            user_source_liquidity,
            placeholder_user_destination_collateral,
            collateral_token_program,
            liquidity_token_program,
            instruction_sysvar_account,
            obligation_farm_user_state,
            reserve_farm_state,
            farms_program,
            scope_oracle,
            kamino_lending_program,
            reserve_accounts: &remaining_accounts[..total_reserve_accounts],
        })
    }
}

impl<'info> Deposit<'info> for Kamino {
    type Accounts = KaminoDepositAccounts<'info>;

    fn deposit_signed(
        ctx: &KaminoDepositAccounts<'info>,
        amount: u64,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        // Refresh reserves
        let accounts = [
            InstructionAccount::writable(ctx.reserve.address()),
            InstructionAccount::readonly(ctx.kamino_lending_program.address()),
            InstructionAccount::readonly(ctx.kamino_lending_program.address()),
            InstructionAccount::readonly(ctx.kamino_lending_program.address()),
            InstructionAccount::readonly(ctx.scope_oracle.address()),
        ];

        let account_infos = [
            ctx.reserve,
            ctx.kamino_lending_program,
            ctx.kamino_lending_program,
            ctx.kamino_lending_program,
            ctx.scope_oracle,
        ];

        let instruction = InstructionView {
            program_id: &KAMINO_LEND_PROGRAM_ID,
            accounts: &accounts,
            data: &REFRESH_RESERVE_DISCRIMINATOR,
        };

        invoke_signed(&instruction, &account_infos, signer_seeds)?;

        for reserve in ctx.reserve_accounts {
            let accounts = [
                InstructionAccount::writable(reserve.address()),
                InstructionAccount::readonly(ctx.kamino_lending_program.address()),
                InstructionAccount::readonly(ctx.kamino_lending_program.address()),
                InstructionAccount::readonly(ctx.kamino_lending_program.address()),
                InstructionAccount::readonly(ctx.scope_oracle.address()),
            ];

            let account_infos = [
                ctx.reserve,
                ctx.kamino_lending_program,
                ctx.kamino_lending_program,
                ctx.kamino_lending_program,
                ctx.scope_oracle,
            ];

            let instruction = InstructionView {
                program_id: &KAMINO_LEND_PROGRAM_ID,
                accounts: &accounts,
                data: &REFRESH_RESERVE_DISCRIMINATOR,
            };

            invoke_signed(&instruction, &account_infos, signer_seeds)?;
        }

        // Refresh obligation
        const MAX_REFRESH_OBLIGATION_ACCOUNTS: usize = 15;

        let mut obligation_accounts =
            MaybeUninit::<[InstructionAccount; MAX_REFRESH_OBLIGATION_ACCOUNTS]>::uninit();
        let obligation_accounts_ptr = obligation_accounts.as_mut_ptr() as *mut InstructionAccount;

        unsafe {
            core::ptr::write(
                obligation_accounts_ptr,
                InstructionAccount::writable(ctx.obligation.address()),
            );
            core::ptr::write(
                obligation_accounts_ptr.add(1),
                InstructionAccount::readonly(ctx.lending_market.address()),
            );

            for (i, reserve) in ctx.reserve_accounts.iter().enumerate() {
                core::ptr::write(
                    obligation_accounts_ptr.add(2 + i),
                    InstructionAccount::readonly(reserve.address()),
                );
            }
        }

        let obligation_accounts_len = 2 + ctx.reserve_accounts.len();
        let obligation_accounts_slice = unsafe {
            core::slice::from_raw_parts(obligation_accounts_ptr, obligation_accounts_len)
        };

        let mut obligation_account_infos = [ctx.obligation; MAX_REFRESH_OBLIGATION_ACCOUNTS];
        obligation_account_infos[1] = ctx.lending_market;

        for (i, reserve) in ctx.reserve_accounts.iter().enumerate() {
            obligation_account_infos[2 + i] = reserve;
        }

        let instruction = InstructionView {
            program_id: &KAMINO_LEND_PROGRAM_ID,
            accounts: obligation_accounts_slice,
            data: &REFRESH_OBLIGATION_DISCRIMINATOR,
        };

        invoke_signed(&instruction, &obligation_account_infos, signer_seeds)?;

        // Deposit CPI
        let accounts = [
            InstructionAccount::writable_signer(ctx.owner.address()),
            InstructionAccount::writable(ctx.obligation.address()),
            InstructionAccount::readonly(ctx.lending_market.address()),
            InstructionAccount::readonly(ctx.lending_market_authority.address()),
            InstructionAccount::writable(ctx.reserve.address()),
            InstructionAccount::readonly(ctx.reserve_liquidity_mint.address()),
            InstructionAccount::writable(ctx.reserve_liquidity_supply.address()),
            InstructionAccount::writable(ctx.reserve_collateral_mint.address()),
            InstructionAccount::writable(ctx.reserve_destination_deposit_collateral.address()),
            InstructionAccount::writable(ctx.user_source_liquidity.address()),
            InstructionAccount::readonly(ctx.placeholder_user_destination_collateral.address()),
            InstructionAccount::readonly(ctx.collateral_token_program.address()),
            InstructionAccount::readonly(ctx.liquidity_token_program.address()),
            InstructionAccount::readonly(ctx.instruction_sysvar_account.address()),
            InstructionAccount::writable(ctx.obligation_farm_user_state.address()),
            InstructionAccount::writable(ctx.reserve_farm_state.address()),
            InstructionAccount::readonly(ctx.farms_program.address()),
        ];

        let account_infos = [
            ctx.owner,
            ctx.obligation,
            ctx.lending_market,
            ctx.lending_market_authority,
            ctx.reserve,
            ctx.reserve_liquidity_mint,
            ctx.reserve_liquidity_supply,
            ctx.reserve_collateral_mint,
            ctx.reserve_destination_deposit_collateral,
            ctx.user_source_liquidity,
            ctx.placeholder_user_destination_collateral,
            ctx.collateral_token_program,
            ctx.liquidity_token_program,
            ctx.instruction_sysvar_account,
            ctx.obligation_farm_user_state,
            ctx.reserve_farm_state,
            ctx.farms_program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 16]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(
                DEPOSIT_RESERVE_LIQUIDITY_AND_OBLIGATION_COLLATERAL_V2_DISCRIMINATOR.as_ptr(),
                ptr,
                8,
            );
            core::ptr::copy_nonoverlapping(amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
        }

        let deposit_ix = InstructionView {
            program_id: &KAMINO_LEND_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 16)
            },
        };

        invoke_signed(&deposit_ix, &account_infos, signer_seeds)?;

        Ok(())
    }

    fn deposit(ctx: &KaminoDepositAccounts<'info>, amount: u64) -> ProgramResult {
        Self::deposit_signed(ctx, amount, &[])
    }
}
