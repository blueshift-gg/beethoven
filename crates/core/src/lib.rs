#![no_std]

use {
    solana_account_view::AccountView,
    solana_instruction_view::cpi::Signer,
    solana_program_error::{ProgramError, ProgramResult},
};

/// Core trait for swap operations across different DEX protocols.
///
/// Each protocol implements this trait with its specific account requirements,
/// instruction data format, and CPI logic.
pub trait Swap<'info> {
    /// Protocol-specific accounts required for the swap CPI
    type Accounts;

    /// Protocol-specific instruction data beyond in_amount and minimum_out_amount
    type Data;

    /// Execute a swap with PDA signing capability
    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult;

    /// Execute a swap without signing (user is direct signer)
    fn swap(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
    ) -> ProgramResult;
}

/// Minimum size of an SPL Token account data buffer.
const TOKEN_ACCOUNT_MIN_LEN: usize = 165;

/// Offset of the mint field in an SPL Token account.
const TOKEN_ACCOUNT_MINT_OFFSET: usize = 0;

/// Length of an address (pubkey) in bytes.
const ADDRESS_LEN: usize = 32;

/// Number of root accounts in a `SwapWithParameters` instruction.
pub const SWAP_PARAMETERS_LEN: usize = 3;

/// Root parameters for `SwapWithParameters`.
///
/// These occupy fixed positions 0-2 in the account list:
///   [0] user_wallet — signer/authority
///   [1] in_ata      — user's input token account
///   [2] out_ata     — user's output token account
///
/// Mints, token programs, and all other accounts live in the
/// protocol-specific remaining accounts (positions 3+).
/// First remaining account is the DEX program ID (detector).
pub struct SwapParameters<'info> {
    pub user_wallet: &'info AccountView,
    pub in_ata: &'info AccountView,
    pub out_ata: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for SwapParameters<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < SWAP_PARAMETERS_LEN {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [user_wallet, in_ata, out_ata, ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(SwapParameters {
            user_wallet,
            in_ata,
            out_ata,
        })
    }
}

/// Read the mint address stored in a token account (SPL Token / Token-2022).
///
/// The mint is always at bytes 0..32 of the account data.
///
/// # Safety
///
/// Caller must ensure `account` is a valid SPL Token or Token-2022 account.
pub unsafe fn token_account_mint(account: &AccountView) -> &[u8] {
    let data = account.borrow_unchecked();
    debug_assert!(data.len() >= TOKEN_ACCOUNT_MIN_LEN);
    &data[TOKEN_ACCOUNT_MINT_OFFSET..TOKEN_ACCOUNT_MINT_OFFSET + ADDRESS_LEN]
}

/// Core trait for swap operations with standardized root parameters.
///
/// Unlike `Swap` where each protocol defines all accounts, this trait
/// splits accounts into root parameters (user_wallet, in_ata, out_ata,
/// mint_in, mint_out) at positions 0-4 and protocol-specific remaining
/// accounts at positions 5+.
///
/// Direction (buy/sell) is derived on-chain by comparing `mint_in`
/// against vault data — the client never passes direction flags.
pub trait SwapWithParameters<'info> {
    /// Protocol-specific accounts parsed from remaining (positions 5+).
    /// First remaining account is the DEX program ID (detector).
    type Remaining;

    /// Extra instruction data beyond in_amount/min_out_amount.
    /// Most protocols: ().
    type Extra;

    /// Parse protocol-specific accounts from the remaining account slice.
    fn try_parse_remaining(
        remaining: &'info [AccountView],
    ) -> Result<Self::Remaining, ProgramError>;

    /// Execute the swap via CPI with PDA signing capability.
    ///
    /// The implementation derives direction from root parameters and
    /// remaining account data, then builds the full CPI.
    fn swap_with_parameters_signed(
        params: &SwapParameters<'info>,
        remaining: &Self::Remaining,
        in_amount: u64,
        minimum_out_amount: u64,
        extra: &Self::Extra,
        signer_seeds: &[Signer],
    ) -> ProgramResult;

    /// Execute the swap without signing (user is direct signer).
    fn swap_with_parameters(
        params: &SwapParameters<'info>,
        remaining: &Self::Remaining,
        in_amount: u64,
        minimum_out_amount: u64,
        extra: &Self::Extra,
    ) -> ProgramResult {
        Self::swap_with_parameters_signed(
            params,
            remaining,
            in_amount,
            minimum_out_amount,
            extra,
            &[],
        )
    }
}

/// Core trait for deposit operations across different protocols.
///
/// Each protocol implements this trait with its specific account requirements and CPI logic.
pub trait Deposit<'info> {
    /// Protocol-specific accounts required for the deposit CPI
    type Accounts;

    /// Execute a deposit with PDA signing capability
    fn deposit_signed(ctx: &Self::Accounts, amount: u64, signer_seeds: &[Signer]) -> ProgramResult;

    /// Execute a deposit without signing (user is direct signer)
    fn deposit(ctx: &Self::Accounts, amount: u64) -> ProgramResult;
}
