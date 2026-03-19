#![no_std]

use {
    solana_account_view::AccountView, solana_instruction_view::cpi::Signer,
    solana_program_error::ProgramResult,
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

/// Core trait for aggregator route operations (Jupiter, Titan, OKX, etc.).
///
/// Each aggregator implements this trait with its specific account requirements,
/// instruction data validation, and CPI logic. Unlike `Swap`, route operations
/// receive raw instruction data.
pub trait Route<'info> {
    /// Protocol-specific accounts required for the route CPI
    type Accounts;

    /// Validate that swap_data encodes the expected amount and slippage_bps
    fn check_amount_and_slippage(
        ctx: &Self::Accounts,
        swap_data: &[u8],
        amount: u64,
        slippage_bps: u16,
    ) -> ProgramResult;

    /// Execute a route with PDA signing capability
    fn route_signed(
        ctx: &Self::Accounts,
        swap_data: &[u8],
        remaining_accounts: &[AccountView],
        signer_seeds: &[Signer],
    ) -> ProgramResult;

    /// Execute a route without signing (user is direct signer)
    fn route(
        ctx: &Self::Accounts,
        swap_data: &[u8],
        remaining_accounts: &[AccountView],
    ) -> ProgramResult;
}
