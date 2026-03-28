use {
    crate::Swap,
    solana_account_view::AccountView,
    solana_address::address_eq,
    solana_instruction_view::cpi::Signer,
    solana_program_error::{ProgramError, ProgramResult},
};

fn split_accounts_checked(
    accounts: &[AccountView],
    count: usize,
) -> Result<(&[AccountView], &[AccountView]), ProgramError> {
    accounts
        .split_at_checked(count)
        .ok_or(ProgramError::NotEnoughAccountKeys)
}

fn split_data_checked(data: &[u8], count: usize) -> Result<(&[u8], &[u8]), ProgramError> {
    data.split_at_checked(count)
        .ok_or(ProgramError::InvalidInstructionData)
}

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

    #[cfg(feature = "scale_amm-swap")]
    ScaleAmm(crate::scale_amm::ScaleAmmSwapAccounts<'info>),

    #[cfg(feature = "scale_vmm-swap")]
    ScaleVmm(crate::scale_vmm::ScaleVmmSwapAccounts<'info>),

    #[cfg(feature = "omnipair-swap")]
    Omnipair(crate::omnipair::OmnipairSwapAccounts<'info>),

    #[cfg(feature = "hadron-swap")]
    Hadron(crate::hadron::HadronSwapAccounts<'info>),
    #[cfg(feature = "raydium-cpmm-swap")]
    RaydiumCpmm(crate::raydium_cpmm::RaydiumCpmmSwapAccounts<'info>),
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

    #[cfg(feature = "scale_amm-swap")]
    ScaleAmm(crate::scale_amm::ScaleAmmSwapData),

    #[cfg(feature = "scale_vmm-swap")]
    ScaleVmm(crate::scale_vmm::ScaleVmmSwapData),

    #[cfg(feature = "omnipair-swap")]
    Omnipair(()),

    #[cfg(feature = "hadron-swap")]
    Hadron(crate::hadron::HadronSwapData),
    #[cfg(feature = "raydium-cpmm-swap")]
    RaydiumCpmm(()),
}

impl<'a> SwapContext<'a> {
    /// Parse protocol-specific swap data, returning the parsed data and remaining bytes.
    pub fn try_from_swap_data(
        &self,
        data: &'a [u8],
    ) -> Result<(SwapData<'a>, &'a [u8]), ProgramError> {
        match self {
            #[cfg(feature = "perena-swap")]
            SwapContext::Perena(_) => {
                let n = crate::perena::PerenaSwapData::DATA_LEN;
                let (mine, rest) = split_data_checked(data, n)?;
                Ok((
                    SwapData::Perena(crate::perena::PerenaSwapData::try_from(mine)?),
                    rest,
                ))
            }

            #[cfg(feature = "solfi-swap")]
            SwapContext::SolFi(_) => {
                let n = crate::solfi::SolFiSwapData::DATA_LEN;
                let (mine, rest) = split_data_checked(data, n)?;
                Ok((
                    SwapData::SolFi(crate::solfi::SolFiSwapData::try_from(mine)?),
                    rest,
                ))
            }

            #[cfg(feature = "solfi_v2-swap")]
            SwapContext::SolFiV2(_) => {
                let n = crate::solfi_v2::SolFiV2SwapData::DATA_LEN;
                let (mine, rest) = split_data_checked(data, n)?;
                Ok((
                    SwapData::SolFiV2(crate::solfi_v2::SolFiV2SwapData::try_from(mine)?),
                    rest,
                ))
            }

            #[cfg(feature = "manifest-swap")]
            SwapContext::Manifest(_) => {
                let n = crate::manifest::ManifestSwapData::DATA_LEN;
                let (mine, rest) = split_data_checked(data, n)?;
                Ok((
                    SwapData::Manifest(crate::manifest::ManifestSwapData::try_from(mine)?),
                    rest,
                ))
            }

            #[cfg(feature = "heaven-swap")]
            SwapContext::Heaven(_) => {
                // Heaven has variable-length data (direction + event).
                // Consumes all remaining data — must be the last leg in multi-swap.
                Ok((
                    SwapData::Heaven(crate::heaven::HeavenSwapData::try_from(data)?),
                    &[],
                ))
            }

            #[cfg(feature = "aldrin-swap")]
            SwapContext::Aldrin(_) => {
                let n = crate::aldrin::AldrinSwapData::DATA_LEN;
                let (mine, rest) = split_data_checked(data, n)?;
                Ok((
                    SwapData::Aldrin(crate::aldrin::AldrinSwapData::try_from(mine)?),
                    rest,
                ))
            }

            #[cfg(feature = "aldrin_v2-swap")]
            SwapContext::AldrinV2(_) => {
                let n = crate::aldrin_v2::AldrinV2SwapData::DATA_LEN;
                let (mine, rest) = split_data_checked(data, n)?;
                Ok((
                    SwapData::AldrinV2(crate::aldrin_v2::AldrinV2SwapData::try_from(mine)?),
                    rest,
                ))
            }

            #[cfg(feature = "futarchy-swap")]
            SwapContext::Futarchy(_) => {
                let n = crate::futarchy::FutarchySwapData::DATA_LEN;
                let (mine, rest) = split_data_checked(data, n)?;
                Ok((
                    SwapData::Futarchy(crate::futarchy::FutarchySwapData::try_from(mine)?),
                    rest,
                ))
            }

            #[cfg(feature = "gamma-swap")]
            SwapContext::Gamma(_) => Ok((SwapData::Gamma(()), data)),

            #[cfg(feature = "scale_amm-swap")]
            SwapContext::ScaleAmm(_) => {
                let n = crate::scale_amm::ScaleAmmSwapData::DATA_LEN;
                let (mine, rest) = split_data_checked(data, n)?;
                Ok((
                    SwapData::ScaleAmm(crate::scale_amm::ScaleAmmSwapData::try_from(mine)?),
                    rest,
                ))
            }

            #[cfg(feature = "scale_vmm-swap")]
            SwapContext::ScaleVmm(_) => {
                let n = crate::scale_vmm::ScaleVmmSwapData::DATA_LEN;
                let (mine, rest) = split_data_checked(data, n)?;
                Ok((
                    SwapData::ScaleVmm(crate::scale_vmm::ScaleVmmSwapData::try_from(mine)?),
                    rest,
                ))
            }

            #[cfg(feature = "omnipair-swap")]
            SwapContext::Omnipair(_) => Ok((SwapData::Omnipair(()), data)),

            #[cfg(feature = "hadron-swap")]
            SwapContext::Hadron(_) => {
                let n = crate::hadron::HadronSwapData::DATA_LEN;
                if data.len() < n {
                    return Err(ProgramError::InvalidInstructionData);
                }
                let (mine, rest) = data.split_at(n);
                Ok((
                    SwapData::Hadron(crate::hadron::HadronSwapData::try_from(mine)?),
                    rest,
                ))
            }

            #[cfg(feature = "raydium-cpmm-swap")]
            SwapContext::RaydiumCpmm(_) => Ok((SwapData::RaydiumCpmm(()), data)),

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

            #[cfg(feature = "scale_amm-swap")]
            (SwapContext::ScaleAmm(accounts), SwapData::ScaleAmm(d)) => {
                crate::scale_amm::ScaleAmm::swap_signed(
                    accounts,
                    in_amount,
                    minimum_out_amount,
                    d,
                    signer_seeds,
                )
            }

            #[cfg(feature = "scale_vmm-swap")]
            (SwapContext::ScaleVmm(accounts), SwapData::ScaleVmm(d)) => {
                crate::scale_vmm::ScaleVmm::swap_signed(
                    accounts,
                    in_amount,
                    minimum_out_amount,
                    d,
                    signer_seeds,
                )
            }

            #[cfg(feature = "omnipair-swap")]
            (SwapContext::Omnipair(accounts), SwapData::Omnipair(())) => {
                crate::omnipair::Omnipair::swap_signed(
                    accounts,
                    in_amount,
                    minimum_out_amount,
                    &(),
                    signer_seeds,
                )
            }

            #[cfg(feature = "hadron-swap")]
            (SwapContext::Hadron(accounts), SwapData::Hadron(d)) => {
                crate::hadron::Hadron::swap_signed(
                    accounts,
                    in_amount,
                    minimum_out_amount,
                    d,
                    signer_seeds,
                )
            }

            #[cfg(feature = "raydium-cpmm-swap")]
            (SwapContext::RaydiumCpmm(accounts), SwapData::RaydiumCpmm(())) => {
                crate::raydium_cpmm::RaydiumCpmm::swap_signed(
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

/// Detect the protocol from the first account, parse the swap context,
/// and return both the context and the remaining (unconsumed) accounts.
pub fn try_from_swap_context<'info>(
    accounts: &'info [AccountView],
) -> Result<(SwapContext<'info>, &'info [AccountView]), ProgramError> {
    let detector_account = accounts.first().ok_or(ProgramError::NotEnoughAccountKeys)?;

    #[cfg(feature = "perena-swap")]
    if address_eq(
        detector_account.address(),
        &crate::perena::PERENA_PROGRAM_ID,
    ) {
        let (mine, rest) =
            split_accounts_checked(accounts, crate::perena::PerenaSwapAccounts::NUM_ACCOUNTS)?;
        let ctx = crate::perena::PerenaSwapAccounts::try_from(mine)?;
        return Ok((SwapContext::Perena(ctx), rest));
    }

    #[cfg(feature = "solfi-swap")]
    if address_eq(detector_account.address(), &crate::solfi::SOLFI_PROGRAM_ID) {
        let (mine, rest) =
            split_accounts_checked(accounts, crate::solfi::SolFiSwapAccounts::NUM_ACCOUNTS)?;
        let ctx = crate::solfi::SolFiSwapAccounts::try_from(mine)?;
        return Ok((SwapContext::SolFi(ctx), rest));
    }

    #[cfg(feature = "solfi_v2-swap")]
    if address_eq(
        detector_account.address(),
        &crate::solfi_v2::SOLFI_V2_PROGRAM_ID,
    ) {
        let (mine, rest) =
            split_accounts_checked(accounts, crate::solfi_v2::SolFiV2SwapAccounts::NUM_ACCOUNTS)?;
        let ctx = crate::solfi_v2::SolFiV2SwapAccounts::try_from(mine)?;
        return Ok((SwapContext::SolFiV2(ctx), rest));
    }

    #[cfg(feature = "manifest-swap")]
    if address_eq(
        detector_account.address(),
        &crate::manifest::MANIFEST_PROGRAM_ID,
    ) {
        let (mine, rest) = split_accounts_checked(
            accounts,
            crate::manifest::ManifestSwapAccounts::NUM_ACCOUNTS,
        )?;
        let ctx = crate::manifest::ManifestSwapAccounts::try_from(mine)?;
        return Ok((SwapContext::Manifest(ctx), rest));
    }

    #[cfg(feature = "heaven-swap")]
    if address_eq(
        detector_account.address(),
        &crate::heaven::HEAVEN_PROGRAM_ID,
    ) {
        let (mine, rest) =
            split_accounts_checked(accounts, crate::heaven::HeavenSwapAccounts::NUM_ACCOUNTS)?;
        let ctx = crate::heaven::HeavenSwapAccounts::try_from(mine)?;
        return Ok((SwapContext::Heaven(ctx), rest));
    }

    #[cfg(feature = "aldrin-swap")]
    if address_eq(
        detector_account.address(),
        &crate::aldrin::ALDRIN_PROGRAM_ID,
    ) {
        let (mine, rest) =
            split_accounts_checked(accounts, crate::aldrin::AldrinSwapAccounts::NUM_ACCOUNTS)?;
        let ctx = crate::aldrin::AldrinSwapAccounts::try_from(mine)?;
        return Ok((SwapContext::Aldrin(ctx), rest));
    }

    #[cfg(feature = "aldrin_v2-swap")]
    if address_eq(
        detector_account.address(),
        &crate::aldrin_v2::ALDRIN_V2_PROGRAM_ID,
    ) {
        let (mine, rest) = split_accounts_checked(
            accounts,
            crate::aldrin_v2::AldrinV2SwapAccounts::NUM_ACCOUNTS,
        )?;
        let ctx = crate::aldrin_v2::AldrinV2SwapAccounts::try_from(mine)?;
        return Ok((SwapContext::AldrinV2(ctx), rest));
    }

    #[cfg(feature = "futarchy-swap")]
    if address_eq(
        detector_account.address(),
        &crate::futarchy::FUTARCHY_PROGRAM_ID,
    ) {
        let (mine, rest) = split_accounts_checked(
            accounts,
            crate::futarchy::FutarchySwapAccounts::NUM_ACCOUNTS,
        )?;
        let ctx = crate::futarchy::FutarchySwapAccounts::try_from(mine)?;
        return Ok((SwapContext::Futarchy(ctx), rest));
    }

    #[cfg(feature = "gamma-swap")]
    if address_eq(detector_account.address(), &crate::gamma::GAMMA_PROGRAM_ID) {
        let (mine, rest) =
            split_accounts_checked(accounts, crate::gamma::GammaSwapAccounts::NUM_ACCOUNTS)?;
        let ctx = crate::gamma::GammaSwapAccounts::try_from(mine)?;
        return Ok((SwapContext::Gamma(ctx), rest));
    }

    #[cfg(feature = "scale_amm-swap")]
    if address_eq(
        detector_account.address(),
        &crate::scale_amm::SCALE_AMM_PROGRAM_ID,
    ) {
        let (mine, rest) = split_accounts_checked(
            accounts,
            crate::scale_amm::ScaleAmmSwapAccounts::NUM_ACCOUNTS,
        )?;
        let ctx = crate::scale_amm::ScaleAmmSwapAccounts::try_from(mine)?;
        return Ok((SwapContext::ScaleAmm(ctx), rest));
    }

    #[cfg(feature = "scale_vmm-swap")]
    if address_eq(
        detector_account.address(),
        &crate::scale_vmm::SCALE_VMM_PROGRAM_ID,
    ) {
        let (mine, rest) = split_accounts_checked(
            accounts,
            crate::scale_vmm::ScaleVmmSwapAccounts::NUM_ACCOUNTS,
        )?;
        let ctx = crate::scale_vmm::ScaleVmmSwapAccounts::try_from(mine)?;
        return Ok((SwapContext::ScaleVmm(ctx), rest));
    }

    #[cfg(feature = "omnipair-swap")]
    if address_eq(
        detector_account.address(),
        &crate::omnipair::OMNIPAIR_PROGRAM_ID,
    ) {
        let (mine, rest) = split_accounts_checked(
            accounts,
            crate::omnipair::OmnipairSwapAccounts::NUM_ACCOUNTS,
        )?;
        let ctx = crate::omnipair::OmnipairSwapAccounts::try_from(mine)?;
        return Ok((SwapContext::Omnipair(ctx), rest));
    }

    #[cfg(feature = "hadron-swap")]
    if address_eq(
        detector_account.address(),
        &crate::hadron::HADRON_PROGRAM_ID,
    ) {
        let ctx = crate::hadron::HadronSwapAccounts::try_from(accounts)?;
        return Ok((SwapContext::Hadron(ctx), &[]));
    }

    #[cfg(feature = "raydium-cpmm-swap")]
    if address_eq(
        detector_account.address(),
        &crate::raydium_cpmm::RAYDIUM_CPMM_PROGRAM_ID,
    ) {
        let (mine, rest) = split_accounts_checked(
            accounts,
            crate::raydium_cpmm::RaydiumCpmmSwapAccounts::NUM_ACCOUNTS,
        )?;
        let ctx = crate::raydium_cpmm::RaydiumCpmmSwapAccounts::try_from(mine)?;
        return Ok((SwapContext::RaydiumCpmm(ctx), rest));
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
    let (ctx, _remaining) = try_from_swap_context(accounts)?;
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

    #[cfg(feature = "drift-deposit")]
    Drift(crate::drift::DriftDepositAccounts<'info>),

    #[cfg(feature = "marginfi-deposit")]
    Marginfi(crate::marginfi::MarginfiDepositAccounts<'info>),
}

/// Protocol-specific deposit data enum for use with DepositContext
pub enum DepositData {
    #[cfg(feature = "kamino-deposit")]
    Kamino(()),
    #[cfg(feature = "jupiter-deposit")]
    Jupiter(()),
    #[cfg(feature = "drift-deposit")]
    Drift(crate::drift::DriftDepositData),
    #[cfg(feature = "marginfi-deposit")]
    Marginfi(crate::marginfi::MarginfiDepositData),
}

impl<'a> DepositContext<'a> {
    pub fn try_from_deposit_data(
        &self,
        data: &'a [u8],
    ) -> Result<(DepositData, &'a [u8]), ProgramError> {
        match self {
            #[cfg(feature = "kamino-deposit")]
            DepositContext::Kamino(_) => Ok((DepositData::Kamino(()), &[])),

            #[cfg(feature = "jupiter-deposit")]
            DepositContext::Jupiter(_) => Ok((DepositData::Jupiter(()), &[])),

            #[cfg(feature = "drift-deposit")]
            DepositContext::Drift(_) => Ok((
                DepositData::Drift(crate::drift::DriftDepositData::try_from(data)?),
                &[],
            )),

            #[cfg(feature = "marginfi-deposit")]
            DepositContext::Marginfi(_) => Ok((
                DepositData::Marginfi(crate::marginfi::MarginfiDepositData::try_from(data)?),
                &[],
            )),

            #[allow(unreachable_patterns)]
            _ => Err(ProgramError::InvalidAccountData),
        }
    }
}

impl<'info> Deposit<'info> for DepositContext<'info> {
    type Accounts = Self;
    type Data = DepositData;

    fn deposit_signed(
        ctx: &Self::Accounts,
        amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        match ctx {
            #[cfg(feature = "kamino-deposit")]
            DepositContext::Kamino(accounts) => {
                crate::kamino::Kamino::deposit_signed(accounts, amount, &(), signer_seeds)
            }

            #[cfg(feature = "jupiter-deposit")]
            DepositContext::Jupiter(accounts) => {
                crate::jupiter::JupiterEarn::deposit_signed(accounts, amount, &(), signer_seeds)
            }

            #[cfg(feature = "drift-deposit")]
            DepositContext::Drift(accounts) => {
                if let DepositData::Drift(data) = data {
                    crate::drift::Drift::deposit_signed(accounts, amount, data, signer_seeds)
                } else {
                    Err(ProgramError::InvalidInstructionData)
                }
            }

            #[cfg(feature = "marginfi-deposit")]
            DepositContext::Marginfi(accounts) => {
                if let DepositData::Marginfi(data) = data {
                    crate::marginfi::Marginfi::deposit_signed(accounts, amount, data, signer_seeds)
                } else {
                    Err(ProgramError::InvalidInstructionData)
                }
            }

            #[allow(unreachable_patterns)]
            _ => Err(ProgramError::InvalidAccountData),
        }
    }

    fn deposit(ctx: &Self::Accounts, amount: u64, data: &Self::Data) -> ProgramResult {
        Self::deposit_signed(ctx, amount, data, &[])
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

    #[cfg(feature = "drift-deposit")]
    if address_eq(detector_account.address(), &crate::drift::DRIFT_PROGRAM_ID) {
        let ctx = crate::drift::DriftDepositAccounts::try_from(accounts)?;
        return Ok(DepositContext::Drift(ctx));
    }

    #[cfg(feature = "marginfi-deposit")]
    if address_eq(
        detector_account.address(),
        &crate::marginfi::MARGINFI_PROGRAM_ID,
    ) {
        let ctx = crate::marginfi::MarginfiDepositAccounts::try_from(accounts)?;
        return Ok(DepositContext::Marginfi(ctx));
    }

    Err(ProgramError::InvalidAccountData)
}
