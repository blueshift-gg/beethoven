use {
    crate::Swap,
    solana_account_view::AccountView,
    solana_address::address_eq,
    solana_instruction_view::cpi::Signer,
    solana_program_error::{ProgramError, ProgramResult},
};

/// Typed context for swap operations, discriminated by protocol.
pub enum SwapContext<'info> {
    #[cfg(feature = "perena-swap")]
    Perena(crate::perena::PerenaSwapAccounts<'info>),

    #[cfg(feature = "solfi-swap")]
    SolFi(crate::solfi::SolFiSwapAccounts<'info>),

    #[cfg(feature = "solfi_v2-swap")]
    SolFiV2(crate::solfi_v2::SolFiV2SwapAccounts<'info>),

    #[cfg(feature = "manifest-swap")]
    Manifest(crate::manifest::ManifestSwapAccounts<'info>),

    #[cfg(feature = "heaven-swap")]
    Heaven(crate::heaven::HeavenSwapAccounts<'info>),

    #[cfg(feature = "aldrin-swap")]
    Aldrin(crate::aldrin::AldrinSwapAccounts<'info>),

    #[cfg(feature = "aldrin_v2-swap")]
    AldrinV2(crate::aldrin_v2::AldrinV2SwapAccounts<'info>),

    #[cfg(feature = "futarchy-swap")]
    Futarchy(crate::futarchy::FutarchySwapAccounts<'info>),

    #[cfg(feature = "gamma-swap")]
    Gamma(crate::gamma::GammaSwapAccounts<'info>),
}

/// Protocol-specific swap data enum for use with SwapContext
pub enum SwapData<'a> {
    #[cfg(feature = "perena-swap")]
    Perena(crate::perena::PerenaSwapData),

    #[cfg(feature = "solfi-swap")]
    SolFi(crate::solfi::SolFiSwapData),

    #[cfg(feature = "solfi_v2-swap")]
    SolFiV2(crate::solfi_v2::SolFiV2SwapData),

    #[cfg(feature = "manifest-swap")]
    Manifest(crate::manifest::ManifestSwapData),

    #[cfg(feature = "heaven-swap")]
    Heaven(crate::heaven::HeavenSwapData<'a>),

    #[cfg(feature = "aldrin-swap")]
    Aldrin(crate::aldrin::AldrinSwapData),

    #[cfg(feature = "aldrin_v2-swap")]
    AldrinV2(crate::aldrin_v2::AldrinV2SwapData),

    #[cfg(feature = "futarchy-swap")]
    Futarchy(crate::futarchy::FutarchySwapData),

    #[cfg(feature = "gamma-swap")]
    Gamma(()),
}

impl<'a> SwapContext<'a> {
    pub fn try_from_swap_data(&self, data: &'a [u8]) -> Result<SwapData<'a>, ProgramError> {
        match self {
            #[cfg(feature = "perena-swap")]
            SwapContext::Perena(_) => Ok(SwapData::Perena(
                crate::perena::PerenaSwapData::try_from(data)?,
            )),

            #[cfg(feature = "solfi-swap")]
            SwapContext::SolFi(_) => Ok(SwapData::SolFi(crate::solfi::SolFiSwapData::try_from(
                data,
            )?)),

            #[cfg(feature = "solfi_v2-swap")]
            SwapContext::SolFiV2(_) => Ok(SwapData::SolFiV2(
                crate::solfi_v2::SolFiV2SwapData::try_from(data)?,
            )),

            #[cfg(feature = "manifest-swap")]
            SwapContext::Manifest(_) => Ok(SwapData::Manifest(
                crate::manifest::ManifestSwapData::try_from(data)?,
            )),

            #[cfg(feature = "heaven-swap")]
            SwapContext::Heaven(_) => Ok(SwapData::Heaven(
                crate::heaven::HeavenSwapData::try_from(data)?,
            )),

            #[cfg(feature = "aldrin-swap")]
            SwapContext::Aldrin(_) => Ok(SwapData::Aldrin(
                crate::aldrin::AldrinSwapData::try_from(data)?,
            )),

            #[cfg(feature = "aldrin_v2-swap")]
            SwapContext::AldrinV2(_) => Ok(SwapData::AldrinV2(
                crate::aldrin_v2::AldrinV2SwapData::try_from(data)?,
            )),

            #[cfg(feature = "futarchy-swap")]
            SwapContext::Futarchy(_) => Ok(SwapData::Futarchy(
                crate::futarchy::FutarchySwapData::try_from(data)?,
            )),

            #[cfg(feature = "gamma-swap")]
            SwapContext::Gamma(_) => Ok(SwapData::Gamma(())),

            #[allow(unreachable_patterns)]
            _ => Err(ProgramError::InvalidAccountData),
        }
    }
}

impl<'a> Swap<'a> for SwapContext<'a> {
    type Accounts = Self;
    type Data = SwapData<'a>;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        match (ctx, data) {
            #[cfg(feature = "perena-swap")]
            (SwapContext::Perena(accounts), SwapData::Perena(d)) => {
                crate::perena::Perena::swap_signed(
                    accounts,
                    in_amount,
                    minimum_out_amount,
                    d,
                    signer_seeds,
                )
            }

            #[cfg(feature = "solfi-swap")]
            (SwapContext::SolFi(accounts), SwapData::SolFi(d)) => crate::solfi::SolFi::swap_signed(
                accounts,
                in_amount,
                minimum_out_amount,
                d,
                signer_seeds,
            ),

            #[cfg(feature = "solfi_v2-swap")]
            (SwapContext::SolFiV2(accounts), SwapData::SolFiV2(d)) => {
                crate::solfi_v2::SolFiV2::swap_signed(
                    accounts,
                    in_amount,
                    minimum_out_amount,
                    d,
                    signer_seeds,
                )
            }

            #[cfg(feature = "manifest-swap")]
            (SwapContext::Manifest(accounts), SwapData::Manifest(d)) => {
                crate::manifest::Manifest::swap_signed(
                    accounts,
                    in_amount,
                    minimum_out_amount,
                    d,
                    signer_seeds,
                )
            }

            #[cfg(feature = "heaven-swap")]
            (SwapContext::Heaven(accounts), SwapData::Heaven(d)) => {
                crate::heaven::Heaven::swap_signed(
                    accounts,
                    in_amount,
                    minimum_out_amount,
                    d,
                    signer_seeds,
                )
            }

            #[cfg(feature = "aldrin-swap")]
            (SwapContext::Aldrin(accounts), SwapData::Aldrin(d)) => {
                crate::aldrin::Aldrin::swap_signed(
                    accounts,
                    in_amount,
                    minimum_out_amount,
                    d,
                    signer_seeds,
                )
            }

            #[cfg(feature = "aldrin_v2-swap")]
            (SwapContext::AldrinV2(accounts), SwapData::AldrinV2(d)) => {
                crate::aldrin_v2::AldrinV2::swap_signed(
                    accounts,
                    in_amount,
                    minimum_out_amount,
                    d,
                    signer_seeds,
                )
            }

            #[cfg(feature = "futarchy-swap")]
            (SwapContext::Futarchy(accounts), SwapData::Futarchy(d)) => {
                crate::futarchy::Futarchy::swap_signed(
                    accounts,
                    in_amount,
                    minimum_out_amount,
                    d,
                    signer_seeds,
                )
            }

            #[cfg(feature = "gamma-swap")]
            (SwapContext::Gamma(accounts), SwapData::Gamma(())) => {
                crate::gamma::Gamma::swap_signed(
                    accounts,
                    in_amount,
                    minimum_out_amount,
                    &(),
                    signer_seeds,
                )
            }

            #[allow(unreachable_patterns)]
            _ => Err(ProgramError::InvalidAccountData),
        }
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

pub fn try_from_swap_context<'info>(
    accounts: &'info [AccountView],
) -> Result<SwapContext<'info>, ProgramError> {
    let detector_account = accounts.first().ok_or(ProgramError::NotEnoughAccountKeys)?;

    #[cfg(feature = "perena-swap")]
    if address_eq(
        detector_account.address(),
        &crate::perena::PERENA_PROGRAM_ID,
    ) {
        let ctx = crate::perena::PerenaSwapAccounts::try_from(accounts)?;
        return Ok(SwapContext::Perena(ctx));
    }

    #[cfg(feature = "solfi-swap")]
    if address_eq(detector_account.address(), &crate::solfi::SOLFI_PROGRAM_ID) {
        let ctx = crate::solfi::SolFiSwapAccounts::try_from(accounts)?;
        return Ok(SwapContext::SolFi(ctx));
    }

    #[cfg(feature = "solfi_v2-swap")]
    if address_eq(
        detector_account.address(),
        &crate::solfi_v2::SOLFI_V2_PROGRAM_ID,
    ) {
        let ctx = crate::solfi_v2::SolFiV2SwapAccounts::try_from(accounts)?;
        return Ok(SwapContext::SolFiV2(ctx));
    }

    #[cfg(feature = "manifest-swap")]
    if address_eq(
        detector_account.address(),
        &crate::manifest::MANIFEST_PROGRAM_ID,
    ) {
        let ctx = crate::manifest::ManifestSwapAccounts::try_from(accounts)?;
        return Ok(SwapContext::Manifest(ctx));
    }

    #[cfg(feature = "heaven-swap")]
    if address_eq(
        detector_account.address(),
        &crate::heaven::HEAVEN_PROGRAM_ID,
    ) {
        let ctx = crate::heaven::HeavenSwapAccounts::try_from(accounts)?;
        return Ok(SwapContext::Heaven(ctx));
    }

    #[cfg(feature = "aldrin-swap")]
    if address_eq(
        detector_account.address(),
        &crate::aldrin::ALDRIN_PROGRAM_ID,
    ) {
        let ctx = crate::aldrin::AldrinSwapAccounts::try_from(accounts)?;
        return Ok(SwapContext::Aldrin(ctx));
    }

    #[cfg(feature = "aldrin_v2-swap")]
    if address_eq(
        detector_account.address(),
        &crate::aldrin_v2::ALDRIN_V2_PROGRAM_ID,
    ) {
        let ctx = crate::aldrin_v2::AldrinV2SwapAccounts::try_from(accounts)?;
        return Ok(SwapContext::AldrinV2(ctx));
    }

    #[cfg(feature = "futarchy-swap")]
    if address_eq(
        detector_account.address(),
        &crate::futarchy::FUTARCHY_PROGRAM_ID,
    ) {
        let ctx = crate::futarchy::FutarchySwapAccounts::try_from(accounts)?;
        return Ok(SwapContext::Futarchy(ctx));
    }

    #[cfg(feature = "gamma-swap")]
    if address_eq(detector_account.address(), &crate::gamma::GAMMA_PROGRAM_ID) {
        let ctx = crate::gamma::GammaSwapAccounts::try_from(accounts)?;
        return Ok(SwapContext::Gamma(ctx));
    }

    Err(ProgramError::InvalidAccountData)
}

pub fn swap_signed(
    accounts: &[AccountView],
    in_amount: u64,
    minimum_out_amount: u64,
    data: &SwapData<'_>,
    signer_seeds: &[Signer],
) -> ProgramResult {
    let ctx = try_from_swap_context(accounts)?;
    SwapContext::swap_signed(&ctx, in_amount, minimum_out_amount, data, signer_seeds)
}

pub fn swap(
    accounts: &[AccountView],
    in_amount: u64,
    minimum_out_amount: u64,
    data: &SwapData<'_>,
) -> ProgramResult {
    swap_signed(accounts, in_amount, minimum_out_amount, data, &[])
}

// Deposit context - similar pattern
use crate::Deposit;

pub enum DepositContext<'info> {
    #[cfg(feature = "kamino-deposit")]
    Kamino(crate::kamino::KaminoDepositAccounts<'info>),

    #[cfg(feature = "jupiter-deposit")]
    Jupiter(crate::jupiter::JupiterEarnDepositAccounts<'info>),
}

impl<'info> Deposit<'info> for DepositContext<'info> {
    type Accounts = Self;

    fn deposit_signed(ctx: &Self::Accounts, amount: u64, signer_seeds: &[Signer]) -> ProgramResult {
        match ctx {
            #[cfg(feature = "kamino-deposit")]
            DepositContext::Kamino(accounts) => {
                crate::kamino::Kamino::deposit_signed(accounts, amount, signer_seeds)
            }

            #[cfg(feature = "jupiter-deposit")]
            DepositContext::Jupiter(accounts) => {
                crate::jupiter::JupiterEarn::deposit_signed(accounts, amount, signer_seeds)
            }

            #[allow(unreachable_patterns)]
            _ => Err(ProgramError::InvalidAccountData),
        }
    }

    fn deposit(ctx: &Self::Accounts, amount: u64) -> ProgramResult {
        Self::deposit_signed(ctx, amount, &[])
    }
}

pub fn try_from_deposit_context<'info>(
    accounts: &'info [AccountView],
) -> Result<DepositContext<'info>, ProgramError> {
    let detector_account = accounts.first().ok_or(ProgramError::NotEnoughAccountKeys)?;

    #[cfg(feature = "kamino-deposit")]
    if address_eq(
        detector_account.address(),
        &crate::kamino::KAMINO_LEND_PROGRAM_ID,
    ) {
        let ctx = crate::kamino::KaminoDepositAccounts::try_from(accounts)?;
        return Ok(DepositContext::Kamino(ctx));
    }

    #[cfg(feature = "jupiter-deposit")]
    if address_eq(
        detector_account.address(),
        &crate::jupiter::JUPITER_EARN_PROGRAM_ID,
    ) {
        let ctx = crate::jupiter::JupiterEarnDepositAccounts::try_from(accounts)?;
        return Ok(DepositContext::Jupiter(ctx));
    }

    Err(ProgramError::InvalidAccountData)
}
