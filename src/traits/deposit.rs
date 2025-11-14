use pinocchio::{
    ProgramResult, account_info::AccountInfo, instruction::Signer, program_error::ProgramError,
    pubkey::pubkey_eq,
};

/// Core trait for deposit operations across different protocols (Kamino, Jupiter, etc.)
///
/// Each protocol implements this trait with its specific account requirements and CPI logic.
/// The trait provides both signed and unsigned deposit methods.
pub trait Deposit<'info> {
    /// Protocol-specific accounts required for the deposit CPI
    type Accounts;

    /// Execute a deposit with PDA signing capability
    ///
    /// # Arguments
    /// * `ctx` - Protocol-specific account context
    /// * `amount` - Amount to deposit
    /// * `signer_seeds` - Seeds for PDA signing
    fn deposit_signed(ctx: &Self::Accounts, amount: u64, signer_seeds: &[Signer]) -> ProgramResult;

    /// Execute a deposit without signing (user is direct signer)
    ///
    /// # Arguments
    /// * `ctx` - Protocol-specific account context
    /// * `amount` - Amount to deposit
    fn deposit(ctx: &Self::Accounts, amount: u64) -> ProgramResult;
}

/// Typed context for deposit operations, discriminated by protocol.
///
/// This enum contains the protocol-specific account structures after parsing
/// and discrimination. Users can pattern match on this to perform custom
/// validation before executing the deposit.
pub enum DepositContext<'info> {
    #[cfg(feature = "kamino")]
    Kamino(crate::programs::kamino::KaminoDepositAccounts<'info>),

    #[cfg(feature = "jupiter")]
    Jupiter(crate::programs::jupiter::JupiterEarnDepositAccounts<'info>),
}

impl<'info> Deposit<'info> for DepositContext<'info> {
    type Accounts = Self;

    fn deposit_signed(ctx: &Self::Accounts, amount: u64, signer_seeds: &[Signer]) -> ProgramResult {
        match ctx {
            #[cfg(feature = "kamino")]
            DepositContext::Kamino(kamino_ctx) => {
                crate::programs::kamino::Kamino::deposit_signed(kamino_ctx, amount, signer_seeds)
            }

            #[cfg(feature = "jupiter")]
            DepositContext::Jupiter(jupiter_ctx) => {
                crate::programs::jupiter::JupiterEarn::deposit_signed(
                    jupiter_ctx,
                    amount,
                    signer_seeds,
                )
            }
        }
    }

    fn deposit(ctx: &Self::Accounts, amount: u64) -> ProgramResult {
        Self::deposit_signed(ctx, amount, &[])
    }
}

/// Parses accounts and discriminates the protocol based on the first account's owner.
///
/// This function returns a typed `DepositContext` that allows users to:
/// - Pattern match on the protocol type
/// - Access typed account fields for custom validation
/// - Inspect account properties before executing the deposit
///
/// # Arguments
/// * `accounts` - Slice of accounts where the first account's owner determines the protocol
///
/// # Returns
/// * `Ok(DepositContext)` - Typed context for the detected protocol
/// * `Err(ProgramError::NotEnoughAccountKeys)` - Empty account slice provided
/// * `Err(ProgramError::InvalidAccountData)` - No matching protocol found or invalid account structure
///
/// # Example
/// ```ignore
/// let ctx = try_from_deposit_context(remaining_accounts)?;
///
/// match &ctx {
///     DepositContext::Kamino(kamino_accounts) => {
///         // Custom validation for Kamino
///         if kamino_accounts.owner.key() != expected_authority {
///             return Err(ProgramError::InvalidAccountData);
///         }
///     }
///     DepositContext::Jupiter(jupiter_accounts) => {
///         // Custom validation for Jupiter
///     }
/// }
///
/// // Use the trait method to execute
/// DepositContext::deposit(&ctx, amount)?;
/// ```
pub fn try_from_deposit_context<'info>(
    accounts: &'info [AccountInfo],
) -> Result<DepositContext<'info>, ProgramError> {
    let detector_account = accounts.first().ok_or(ProgramError::NotEnoughAccountKeys)?;

    #[cfg(feature = "kamino")]
    if pubkey_eq(
        detector_account.key(),
        &crate::programs::kamino::KAMINO_LEND_PROGRAM_ID,
    ) {
        let ctx = crate::programs::kamino::KaminoDepositAccounts::try_from(accounts)?;
        return Ok(DepositContext::Kamino(ctx));
    }

    #[cfg(feature = "jupiter")]
    if pubkey_eq(
        detector_account.key(),
        &crate::programs::jupiter::JUPITER_EARN_PROGRAM_ID,
    ) {
        let ctx = crate::programs::jupiter::JupiterEarnDepositAccounts::try_from(accounts)?;
        return Ok(DepositContext::Jupiter(ctx));
    }

    Err(ProgramError::InvalidAccountData)
}

/// Convenience function: Parses accounts, discriminates protocol, and executes deposit with PDA signing.
///
/// This is equivalent to calling `try_from_deposit_context` followed by `DepositContext::deposit_signed`.
/// For custom validation, use those functions separately instead.
///
/// # Arguments
/// * `accounts` - Slice of accounts where the first account's owner determines the protocol
/// * `amount` - Amount of tokens to deposit
/// * `signer_seeds` - Seeds for PDA signing
///
/// # Returns
/// * `Ok(())` - Deposit executed successfully
/// * `Err(ProgramError)` - Parsing, discrimination, or CPI failed
pub fn deposit_signed<'info>(
    accounts: &'info [AccountInfo],
    amount: u64,
    signer_seeds: &[Signer],
) -> ProgramResult {
    let ctx = try_from_deposit_context(accounts)?;
    DepositContext::deposit_signed(&ctx, amount, signer_seeds)
}

/// Convenience function: Parses accounts, discriminates protocol, and executes deposit.
///
/// This is equivalent to calling `try_from_deposit_context` followed by `DepositContext::deposit`.
/// For custom validation, use those functions separately instead.
///
/// # Arguments
/// * `accounts` - Slice of accounts where the first account's owner determines the protocol
/// * `amount` - Amount of tokens to deposit
///
/// # Returns
/// * `Ok(())` - Deposit executed successfully
/// * `Err(ProgramError)` - Parsing, discrimination, or CPI failed
pub fn deposit<'info>(accounts: &'info [AccountInfo], amount: u64) -> ProgramResult {
    deposit_signed(accounts, amount, &[])
}
