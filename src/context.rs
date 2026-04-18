use {
    crate::{Swap, SwapTokenAccounts},
    solana_account_view::AccountView,
    solana_address::{address_eq, Address},
    solana_instruction_view::cpi::Signer,
    solana_program_error::{ProgramError, ProgramResult},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SwapProtocolTag {
    Perena = 0,
    SolFi = 1,
    SolFiV2 = 2,
    Manifest = 3,
    Heaven = 4,
    Aldrin = 5,
    AldrinV2 = 6,
    Futarchy = 7,
    Gamma = 8,
    ScaleAmm = 9,
    ScaleVmm = 10,
    Omnipair = 11,
    Hadron = 12,
    RaydiumCpmm = 13,
}

impl SwapProtocolTag {
    pub fn from_byte(value: u8) -> Result<Self, ProgramError> {
        match value {
            0 => Ok(Self::Perena),
            1 => Ok(Self::SolFi),
            2 => Ok(Self::SolFiV2),
            3 => Ok(Self::Manifest),
            4 => Ok(Self::Heaven),
            5 => Ok(Self::Aldrin),
            6 => Ok(Self::AldrinV2),
            7 => Ok(Self::Futarchy),
            8 => Ok(Self::Gamma),
            9 => Ok(Self::ScaleAmm),
            10 => Ok(Self::ScaleVmm),
            11 => Ok(Self::Omnipair),
            12 => Ok(Self::Hadron),
            13 => Ok(Self::RaydiumCpmm),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }

    pub const fn fixed_account_count(self) -> usize {
        match self {
            Self::Perena => 12,
            Self::SolFi => 9,
            Self::SolFiV2 => 14,
            Self::Manifest => 15,
            Self::Heaven => 17,
            Self::Aldrin => 11,
            Self::AldrinV2 => 12,
            Self::Futarchy => 10,
            Self::Gamma => 14,
            Self::ScaleAmm => 15,
            Self::ScaleVmm => 22,
            Self::Omnipair => 15,
            Self::Hadron => 16,
            Self::RaydiumCpmm => 14,
        }
    }
}

impl TryFrom<u8> for SwapProtocolTag {
    type Error = ProgramError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::from_byte(value)
    }
}

fn validate_tagged_program_account(
    program_account: &AccountView,
    expected_program_id: &Address,
) -> Result<(), ProgramError> {
    if address_eq(program_account.address(), expected_program_id) {
        Ok(())
    } else {
        Err(ProgramError::InvalidAccountData)
    }
}

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
                let (mine, rest) = split_data_checked(data, n)?;
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

    pub fn try_from_swap_data_exact(&self, data: &'a [u8]) -> Result<SwapData<'a>, ProgramError> {
        let (parsed, remaining_data) = self.try_from_swap_data(data)?;

        if !remaining_data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(parsed)
    }

    pub fn token_accounts(
        &self,
        data: &SwapData<'a>,
    ) -> Result<(&'a AccountView, &'a AccountView), ProgramError> {
        match (self, data) {
            #[cfg(feature = "perena-swap")]
            (SwapContext::Perena(accounts), SwapData::Perena(d)) => {
                Ok(crate::perena::Perena::token_accounts(accounts, d))
            }

            #[cfg(feature = "solfi-swap")]
            (SwapContext::SolFi(accounts), SwapData::SolFi(d)) => {
                Ok(crate::solfi::SolFi::token_accounts(accounts, d))
            }

            #[cfg(feature = "solfi_v2-swap")]
            (SwapContext::SolFiV2(accounts), SwapData::SolFiV2(d)) => {
                Ok(crate::solfi_v2::SolFiV2::token_accounts(accounts, d))
            }

            #[cfg(feature = "manifest-swap")]
            (SwapContext::Manifest(accounts), SwapData::Manifest(d)) => {
                Ok(crate::manifest::Manifest::token_accounts(accounts, d))
            }

            #[cfg(feature = "heaven-swap")]
            (SwapContext::Heaven(accounts), SwapData::Heaven(d)) => {
                Ok(crate::heaven::Heaven::token_accounts(accounts, d))
            }

            #[cfg(feature = "aldrin-swap")]
            (SwapContext::Aldrin(accounts), SwapData::Aldrin(d)) => {
                Ok(crate::aldrin::Aldrin::token_accounts(accounts, d))
            }

            #[cfg(feature = "aldrin_v2-swap")]
            (SwapContext::AldrinV2(accounts), SwapData::AldrinV2(d)) => {
                Ok(crate::aldrin_v2::AldrinV2::token_accounts(accounts, d))
            }

            #[cfg(feature = "futarchy-swap")]
            (SwapContext::Futarchy(accounts), SwapData::Futarchy(d)) => {
                Ok(crate::futarchy::Futarchy::token_accounts(accounts, d))
            }

            #[cfg(feature = "gamma-swap")]
            (SwapContext::Gamma(accounts), SwapData::Gamma(())) => {
                Ok(crate::gamma::Gamma::token_accounts(accounts, &()))
            }

            #[cfg(feature = "scale_amm-swap")]
            (SwapContext::ScaleAmm(accounts), SwapData::ScaleAmm(d)) => {
                Ok(crate::scale_amm::ScaleAmm::token_accounts(accounts, d))
            }

            #[cfg(feature = "scale_vmm-swap")]
            (SwapContext::ScaleVmm(accounts), SwapData::ScaleVmm(d)) => {
                Ok(crate::scale_vmm::ScaleVmm::token_accounts(accounts, d))
            }

            #[cfg(feature = "omnipair-swap")]
            (SwapContext::Omnipair(accounts), SwapData::Omnipair(())) => {
                Ok(crate::omnipair::Omnipair::token_accounts(accounts, &()))
            }

            #[cfg(feature = "hadron-swap")]
            (SwapContext::Hadron(accounts), SwapData::Hadron(d)) => {
                Ok(crate::hadron::Hadron::token_accounts(accounts, d))
            }

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

pub fn try_from_tagged_swap_context<'info>(
    tag: SwapProtocolTag,
    accounts: &'info [AccountView],
    remaining_accounts_len: usize,
) -> Result<(SwapContext<'info>, &'info [AccountView]), ProgramError> {
    let consumed_accounts = tag
        .fixed_account_count()
        .checked_add(remaining_accounts_len)
        .ok_or(ProgramError::InvalidInstructionData)?;

    let (mine, rest) = split_accounts_checked(accounts, consumed_accounts)?;
    let program_account = mine.first().ok_or(ProgramError::NotEnoughAccountKeys)?;

    match tag {
        SwapProtocolTag::Perena => {
            #[cfg(feature = "perena-swap")]
            {
                validate_tagged_program_account(
                    program_account,
                    &crate::perena::PERENA_PROGRAM_ID,
                )?;
                let ctx = crate::perena::PerenaSwapAccounts::try_from(mine)?;
                Ok((SwapContext::Perena(ctx), rest))
            }
            #[cfg(not(feature = "perena-swap"))]
            {
                Err(ProgramError::InvalidInstructionData)
            }
        }

        SwapProtocolTag::SolFi => {
            #[cfg(feature = "solfi-swap")]
            {
                validate_tagged_program_account(program_account, &crate::solfi::SOLFI_PROGRAM_ID)?;
                let ctx = crate::solfi::SolFiSwapAccounts::try_from(mine)?;
                Ok((SwapContext::SolFi(ctx), rest))
            }
            #[cfg(not(feature = "solfi-swap"))]
            {
                Err(ProgramError::InvalidInstructionData)
            }
        }

        SwapProtocolTag::SolFiV2 => {
            #[cfg(feature = "solfi_v2-swap")]
            {
                validate_tagged_program_account(
                    program_account,
                    &crate::solfi_v2::SOLFI_V2_PROGRAM_ID,
                )?;
                let ctx = crate::solfi_v2::SolFiV2SwapAccounts::try_from(mine)?;
                Ok((SwapContext::SolFiV2(ctx), rest))
            }
            #[cfg(not(feature = "solfi_v2-swap"))]
            {
                Err(ProgramError::InvalidInstructionData)
            }
        }

        SwapProtocolTag::Manifest => {
            #[cfg(feature = "manifest-swap")]
            {
                validate_tagged_program_account(
                    program_account,
                    &crate::manifest::MANIFEST_PROGRAM_ID,
                )?;
                let ctx = crate::manifest::ManifestSwapAccounts::try_from(mine)?;
                Ok((SwapContext::Manifest(ctx), rest))
            }
            #[cfg(not(feature = "manifest-swap"))]
            {
                Err(ProgramError::InvalidInstructionData)
            }
        }

        SwapProtocolTag::Heaven => {
            #[cfg(feature = "heaven-swap")]
            {
                validate_tagged_program_account(
                    program_account,
                    &crate::heaven::HEAVEN_PROGRAM_ID,
                )?;
                let ctx = crate::heaven::HeavenSwapAccounts::try_from(mine)?;
                Ok((SwapContext::Heaven(ctx), rest))
            }
            #[cfg(not(feature = "heaven-swap"))]
            {
                Err(ProgramError::InvalidInstructionData)
            }
        }

        SwapProtocolTag::Aldrin => {
            #[cfg(feature = "aldrin-swap")]
            {
                validate_tagged_program_account(
                    program_account,
                    &crate::aldrin::ALDRIN_PROGRAM_ID,
                )?;
                let ctx = crate::aldrin::AldrinSwapAccounts::try_from(mine)?;
                Ok((SwapContext::Aldrin(ctx), rest))
            }
            #[cfg(not(feature = "aldrin-swap"))]
            {
                Err(ProgramError::InvalidInstructionData)
            }
        }

        SwapProtocolTag::AldrinV2 => {
            #[cfg(feature = "aldrin_v2-swap")]
            {
                validate_tagged_program_account(
                    program_account,
                    &crate::aldrin_v2::ALDRIN_V2_PROGRAM_ID,
                )?;
                let ctx = crate::aldrin_v2::AldrinV2SwapAccounts::try_from(mine)?;
                Ok((SwapContext::AldrinV2(ctx), rest))
            }
            #[cfg(not(feature = "aldrin_v2-swap"))]
            {
                Err(ProgramError::InvalidInstructionData)
            }
        }

        SwapProtocolTag::Futarchy => {
            #[cfg(feature = "futarchy-swap")]
            {
                validate_tagged_program_account(
                    program_account,
                    &crate::futarchy::FUTARCHY_PROGRAM_ID,
                )?;
                let ctx = crate::futarchy::FutarchySwapAccounts::try_from(mine)?;
                Ok((SwapContext::Futarchy(ctx), rest))
            }
            #[cfg(not(feature = "futarchy-swap"))]
            {
                Err(ProgramError::InvalidInstructionData)
            }
        }

        SwapProtocolTag::Gamma => {
            #[cfg(feature = "gamma-swap")]
            {
                validate_tagged_program_account(program_account, &crate::gamma::GAMMA_PROGRAM_ID)?;
                let ctx = crate::gamma::GammaSwapAccounts::try_from(mine)?;
                Ok((SwapContext::Gamma(ctx), rest))
            }
            #[cfg(not(feature = "gamma-swap"))]
            {
                Err(ProgramError::InvalidInstructionData)
            }
        }

        SwapProtocolTag::ScaleAmm => {
            #[cfg(feature = "scale_amm-swap")]
            {
                validate_tagged_program_account(
                    program_account,
                    &crate::scale_amm::SCALE_AMM_PROGRAM_ID,
                )?;
                let ctx = crate::scale_amm::ScaleAmmSwapAccounts::try_from(mine)?;
                Ok((SwapContext::ScaleAmm(ctx), rest))
            }
            #[cfg(not(feature = "scale_amm-swap"))]
            {
                Err(ProgramError::InvalidInstructionData)
            }
        }

        SwapProtocolTag::ScaleVmm => {
            #[cfg(feature = "scale_vmm-swap")]
            {
                validate_tagged_program_account(
                    program_account,
                    &crate::scale_vmm::SCALE_VMM_PROGRAM_ID,
                )?;
                let ctx = crate::scale_vmm::ScaleVmmSwapAccounts::try_from(mine)?;
                Ok((SwapContext::ScaleVmm(ctx), rest))
            }
            #[cfg(not(feature = "scale_vmm-swap"))]
            {
                Err(ProgramError::InvalidInstructionData)
            }
        }

        SwapProtocolTag::Omnipair => {
            #[cfg(feature = "omnipair-swap")]
            {
                validate_tagged_program_account(
                    program_account,
                    &crate::omnipair::OMNIPAIR_PROGRAM_ID,
                )?;
                let ctx = crate::omnipair::OmnipairSwapAccounts::try_from(mine)?;
                Ok((SwapContext::Omnipair(ctx), rest))
            }
            #[cfg(not(feature = "omnipair-swap"))]
            {
                Err(ProgramError::InvalidInstructionData)
            }
        }

        SwapProtocolTag::Hadron => {
            #[cfg(feature = "hadron-swap")]
            {
                validate_tagged_program_account(
                    program_account,
                    &crate::hadron::HADRON_PROGRAM_ID,
                )?;
                let ctx = crate::hadron::HadronSwapAccounts::try_from(mine)?;
                Ok((SwapContext::Hadron(ctx), rest))
            }
            #[cfg(not(feature = "hadron-swap"))]
            {
                Err(ProgramError::InvalidInstructionData)
            }
        }

        SwapProtocolTag::RaydiumCpmm => {
            #[cfg(feature = "raydium-cpmm-swap")]
            {
                validate_tagged_program_account(
                    program_account,
                    &crate::raydium_cpmm::RAYDIUM_CPMM_PROGRAM_ID,
                )?;
                let ctx = crate::raydium_cpmm::RaydiumCpmmSwapAccounts::try_from(mine)?;
                Ok((SwapContext::RaydiumCpmm(ctx), rest))
            }
            #[cfg(not(feature = "raydium-cpmm-swap"))]
            {
                Err(ProgramError::InvalidInstructionData)
            }
        }
    }
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
