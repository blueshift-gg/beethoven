use {
    crate::Deposit,
    core::mem::MaybeUninit,
    pinocchio::{
        ProgramResult,
        account_info::AccountInfo,
        cpi::invoke_signed,
        instruction::{AccountMeta, Instruction, Signer},
        program_error::ProgramError,
    },
};

pub const KAMINO_LEND_PROGRAM_ID: [u8; 32] = [0u8; 32];
const REFRESH_RESERVE_DISCRIMINATOR: [u8; 8] = [2, 218, 138, 235, 79, 201, 25, 102];
const REFRESH_OBLIGATION_DISCRIMINATOR: [u8; 8] = [33, 132, 147, 228, 151, 192, 72, 89];
const DEPOSIT_RESERVE_LIQUIDITY_AND_OBLIGATION_COLLATERAL_V2_DISCRIMINATOR: [u8; 8] =
    [216, 224, 191, 27, 204, 151, 102, 175];

/// Kamino lending protocol integration
pub struct Kamino;

/// Account context for Kamino's DepositReserveLiquidityAndObligationCollateralV2 instruction.
///
/// This represents all accounts required for depositing liquidity into a Kamino lending reserve
/// and receiving collateral tokens in an obligation.
///
/// # Account Order
/// Accounts must be provided in the exact order listed below. The TryFrom implementation
/// will validate that at least 19 accounts are present.
pub struct KaminoDepositAccounts<'info> {
    /// Kamino Lending Program (used for optional accounts)
    pub kamino_lending_program: &'info AccountInfo,
    /// Owner of the obligation (must be signer and writable)
    pub owner: &'info AccountInfo,
    /// The obligation account to deposit collateral into (writable)
    pub obligation: &'info AccountInfo,
    /// The lending market this operation belongs to
    pub lending_market: &'info AccountInfo,
    /// Lending market authority PDA
    pub lending_market_authority: &'info AccountInfo,
    /// The reserve account being deposited into (writable)
    pub reserve: &'info AccountInfo,
    /// Mint of the reserve's liquidity token
    pub reserve_liquidity_mint: &'info AccountInfo,
    /// Reserve's liquidity supply account (writable)
    pub reserve_liquidity_supply: &'info AccountInfo,
    /// Reserve's collateral token mint (writable)
    pub reserve_collateral_mint: &'info AccountInfo,
    /// Destination for the minted collateral tokens (writable)
    pub reserve_destination_deposit_collateral: &'info AccountInfo,
    /// User's source liquidity token account (writable)
    pub user_source_liquidity: &'info AccountInfo,
    /// Placeholder for user destination collateral (can be program ID if not used)
    pub placeholder_user_destination_collateral: &'info AccountInfo,
    /// Token program for collateral operations
    pub collateral_token_program: &'info AccountInfo,
    /// Token program for liquidity operations
    pub liquidity_token_program: &'info AccountInfo,
    /// Sysvar Instructions account for introspection
    pub instruction_sysvar_account: &'info AccountInfo,
    /// Obligation's farm user state (writable, can be program ID if farms not used)
    pub obligation_farm_user_state: &'info AccountInfo,
    /// Reserve's farm state (writable, can be program ID if farms not used)
    pub reserve_farm_state: &'info AccountInfo,
    /// Farms program
    pub farms_program: &'info AccountInfo,
    /// Scope Oracle
    pub scope_oracle: &'info AccountInfo,
    /// Reserve Accounts
    pub reserve_accounts: &'info [AccountInfo],
}

impl<'info> TryFrom<&'info [AccountInfo]> for KaminoDepositAccounts<'info> {
    type Error = ProgramError;

    /// Converts a slice of `AccountInfo` into validated `KaminoDepositAccounts`.
    ///
    /// # Arguments
    /// * `accounts` - Slice containing at least 17 accounts in the correct order
    ///
    /// # Returns
    /// * `Ok(KaminoDepositAccounts)` - Successfully parsed account context
    /// * `Err(ProgramError::NotEnoughAccountKeys)` - Fewer than 17 accounts provided
    ///
    /// # Notes
    /// * No upper bound is enforced - extra accounts are ignored (useful for `remaining_accounts`)
    /// * Mutability and signer constraints are NOT validated here; Kamino's program will
    ///   enforce them during CPI, providing clearer error messages
    /// * The `..` pattern allows passing more than 17 accounts without error
    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        // Require minimum of 19 accounts to prevent undefined behavior
        if accounts.len() < 19 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [
            kamino_lending_program,
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
            remaining_accounts @ ..,
        ] = accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // Since it doesn't make sense to perform 2 deposit instructions back to back, as convention we will assume
        // that all remaining_accounts that are owned by the Kamino lending program are reserves
        // Note: This is not the most efficient way to do this, but I have some skill issues so this is what you get
        let mut total_reserve_accounts = 0;

        for reserve in remaining_accounts {
            if reserve.is_owned_by(&KAMINO_LEND_PROGRAM_ID) && total_reserve_accounts < 13 {
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

    /// Executes a deposit into Kamino lending protocol via CPI.
    ///
    /// This deposits liquidity tokens into a reserve and mints collateral tokens
    /// to the user's obligation, enabling them to borrow against the deposited assets.
    ///
    /// # Arguments
    /// * `account_infos` - Slice of accounts required for the deposit (see `KaminoDepositAccounts`)
    /// * `amount` - Amount of liquidity tokens to deposit
    ///
    /// # Returns
    /// * `Ok(())` - Deposit completed successfully
    /// * `Err(ProgramError)` - Invalid accounts or CPI failure
    fn deposit_signed(
        ctx: &KaminoDepositAccounts<'info>,
        amount: u64,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        // Refresh reserves
        // - Start by the refreshing the reserve we're depositing into
        let accounts = [
            AccountMeta::writable(ctx.reserve.key()),
            AccountMeta::readonly(ctx.kamino_lending_program.key()),
            AccountMeta::readonly(ctx.kamino_lending_program.key()),
            AccountMeta::readonly(ctx.kamino_lending_program.key()),
            AccountMeta::readonly(ctx.scope_oracle.key()),
        ];

        let account_infos = [
            ctx.reserve,
            ctx.kamino_lending_program,
            ctx.kamino_lending_program,
            ctx.kamino_lending_program,
            ctx.scope_oracle,
        ];

        let instruction = Instruction {
            program_id: &KAMINO_LEND_PROGRAM_ID,
            accounts: &accounts,
            data: &REFRESH_RESERVE_DISCRIMINATOR,
        };

        invoke_signed(&instruction, &account_infos, signer_seeds)?;

        // - Now refresh all the other reserves (if any)
        for reserve in ctx.reserve_accounts {
            let accounts = [
                AccountMeta::writable(reserve.key()),
                AccountMeta::readonly(ctx.kamino_lending_program.key()),
                AccountMeta::readonly(ctx.kamino_lending_program.key()),
                AccountMeta::readonly(ctx.kamino_lending_program.key()),
                AccountMeta::readonly(ctx.scope_oracle.key()),
            ];

            let account_infos = [
                ctx.reserve,
                ctx.kamino_lending_program,
                ctx.kamino_lending_program,
                ctx.kamino_lending_program,
                ctx.scope_oracle,
            ];

            let instruction = Instruction {
                program_id: &KAMINO_LEND_PROGRAM_ID,
                accounts: &accounts,
                data: &REFRESH_RESERVE_DISCRIMINATOR,
            };

            invoke_signed(&instruction, &account_infos, signer_seeds)?;
        }

        // Refresh obligation
        const MAX_REFRESH_OBLIGATION_ACCOUNTS: usize = 15;

        // Build account metas: obligation + lending_market + all reserves (up to 13)
        let mut obligation_accounts =
            MaybeUninit::<[AccountMeta; MAX_REFRESH_OBLIGATION_ACCOUNTS]>::uninit();
        let obligation_accounts_ptr = obligation_accounts.as_mut_ptr() as *mut AccountMeta;

        unsafe {
            // First account: writable obligation
            core::ptr::write(
                obligation_accounts_ptr,
                AccountMeta::writable(ctx.obligation.key()),
            );
            // Second account: readonly lending_market
            core::ptr::write(
                obligation_accounts_ptr.add(1),
                AccountMeta::readonly(ctx.lending_market.key()),
            );

            // Add all reserve accounts (read-only)
            for (i, reserve) in ctx.reserve_accounts.iter().enumerate() {
                core::ptr::write(
                    obligation_accounts_ptr.add(2 + i),
                    AccountMeta::readonly(reserve.key()),
                );
            }
        }

        let obligation_accounts_len = 2 + ctx.reserve_accounts.len();
        let obligation_accounts_slice = unsafe {
            core::slice::from_raw_parts(obligation_accounts_ptr, obligation_accounts_len)
        };

        // Build account infos: obligation + lending_market + all reserves
        // Fill unused slots with obligation to avoid UB (invoke_signed is fine with extra accounts)
        // Note: I know this is retarded, but I have some skill issues so this is what you get
        let mut obligation_account_infos = [ctx.obligation; MAX_REFRESH_OBLIGATION_ACCOUNTS];
        obligation_account_infos[1] = ctx.lending_market;

        for (i, reserve) in ctx.reserve_accounts.iter().enumerate() {
            obligation_account_infos[2 + i] = reserve;
        }

        let instruction = Instruction {
            program_id: &KAMINO_LEND_PROGRAM_ID,
            accounts: obligation_accounts_slice,
            data: &REFRESH_OBLIGATION_DISCRIMINATOR,
        };

        // change to cpi::slice_invoke_signed,
        invoke_signed(&instruction, &obligation_account_infos, signer_seeds)?;

        // Deposit CPI
        let accounts = [
            AccountMeta::writable_signer(ctx.owner.key()),
            AccountMeta::writable(ctx.obligation.key()),
            AccountMeta::readonly(ctx.lending_market.key()),
            AccountMeta::readonly(ctx.lending_market_authority.key()),
            AccountMeta::writable(ctx.reserve.key()),
            AccountMeta::readonly(ctx.reserve_liquidity_mint.key()),
            AccountMeta::writable(ctx.reserve_liquidity_supply.key()),
            AccountMeta::writable(ctx.reserve_collateral_mint.key()),
            AccountMeta::writable(ctx.reserve_destination_deposit_collateral.key()),
            AccountMeta::writable(ctx.user_source_liquidity.key()),
            AccountMeta::readonly(ctx.placeholder_user_destination_collateral.key()),
            AccountMeta::readonly(ctx.collateral_token_program.key()),
            AccountMeta::readonly(ctx.liquidity_token_program.key()),
            AccountMeta::readonly(ctx.instruction_sysvar_account.key()),
            AccountMeta::writable(ctx.obligation_farm_user_state.key()),
            AccountMeta::writable(ctx.reserve_farm_state.key()),
            AccountMeta::readonly(ctx.farms_program.key()),
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

        let deposit_ix = Instruction {
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
