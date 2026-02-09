use {
    beethoven_core::{SwapParameters, SwapWithParameters, SWAP_PARAMETERS_LEN},
    solana_account_view::AccountView,
    solana_address::address_eq,
    solana_instruction_view::cpi::Signer,
    solana_program_error::{ProgramError, ProgramResult},
};

pub enum SwapWithParametersContext<'info> {
    #[cfg(feature = "perena-swap-with-parameters")]
    Perena(crate::perena::PerenaSwapRemaining<'info>),

    #[cfg(feature = "solfi-swap-with-parameters")]
    SolFi(crate::solfi::SolFiSwapRemaining<'info>),

    #[cfg(feature = "solfi_v2-swap-with-parameters")]
    SolFiV2(crate::solfi_v2::SolFiV2SwapRemaining<'info>),

    #[cfg(feature = "manifest-swap-with-parameters")]
    Manifest(crate::manifest::ManifestSwapRemaining<'info>),

    #[cfg(feature = "heaven-swap-with-parameters")]
    Heaven(crate::heaven::HeavenSwapRemaining<'info>),

    #[cfg(feature = "aldrin-swap-with-parameters")]
    Aldrin(crate::aldrin::AldrinSwapRemaining<'info>),

    #[cfg(feature = "aldrin_v2-swap-with-parameters")]
    AldrinV2(crate::aldrin_v2::AldrinV2SwapRemaining<'info>),

    #[cfg(feature = "futarchy-swap-with-parameters")]
    Futarchy(crate::futarchy::FutarchySwapRemaining<'info>),

    #[cfg(feature = "gamma-swap-with-parameters")]
    Gamma(crate::gamma::GammaSwapRemaining<'info>),
}

pub enum SwapWithParametersExtra<'a> {
    #[cfg(feature = "perena-swap-with-parameters")]
    Perena(crate::perena::PerenaSwapExtra),

    #[cfg(feature = "solfi-swap-with-parameters")]
    SolFi(()),

    #[cfg(feature = "solfi_v2-swap-with-parameters")]
    SolFiV2(()),

    #[cfg(feature = "manifest-swap-with-parameters")]
    Manifest(crate::manifest::ManifestSwapExtra),

    #[cfg(feature = "heaven-swap-with-parameters")]
    Heaven(crate::heaven::HeavenSwapExtra<'a>),

    #[cfg(feature = "aldrin-swap-with-parameters")]
    Aldrin(()),

    #[cfg(feature = "aldrin_v2-swap-with-parameters")]
    AldrinV2(()),

    #[cfg(feature = "futarchy-swap-with-parameters")]
    Futarchy(()),

    #[cfg(feature = "gamma-swap-with-parameters")]
    Gamma(()),
}

impl<'info> SwapWithParametersContext<'info> {
    pub fn try_from_extra<'a>(
        &self,
        data: &'a [u8],
    ) -> Result<SwapWithParametersExtra<'a>, ProgramError> {
        match self {
            #[cfg(feature = "perena-swap-with-parameters")]
            SwapWithParametersContext::Perena(_) => Ok(SwapWithParametersExtra::Perena(
                crate::perena::PerenaSwapExtra::try_from(data)?,
            )),

            #[cfg(feature = "solfi-swap-with-parameters")]
            SwapWithParametersContext::SolFi(_) => Ok(SwapWithParametersExtra::SolFi(())),

            #[cfg(feature = "solfi_v2-swap-with-parameters")]
            SwapWithParametersContext::SolFiV2(_) => Ok(SwapWithParametersExtra::SolFiV2(())),

            #[cfg(feature = "manifest-swap-with-parameters")]
            SwapWithParametersContext::Manifest(_) => Ok(SwapWithParametersExtra::Manifest(
                crate::manifest::ManifestSwapExtra::try_from(data)?,
            )),

            #[cfg(feature = "heaven-swap-with-parameters")]
            SwapWithParametersContext::Heaven(_) => Ok(SwapWithParametersExtra::Heaven(
                crate::heaven::HeavenSwapExtra::try_from(data)?,
            )),

            #[cfg(feature = "aldrin-swap-with-parameters")]
            SwapWithParametersContext::Aldrin(_) => Ok(SwapWithParametersExtra::Aldrin(())),

            #[cfg(feature = "aldrin_v2-swap-with-parameters")]
            SwapWithParametersContext::AldrinV2(_) => Ok(SwapWithParametersExtra::AldrinV2(())),

            #[cfg(feature = "futarchy-swap-with-parameters")]
            SwapWithParametersContext::Futarchy(_) => Ok(SwapWithParametersExtra::Futarchy(())),

            #[cfg(feature = "gamma-swap-with-parameters")]
            SwapWithParametersContext::Gamma(_) => Ok(SwapWithParametersExtra::Gamma(())),

            #[allow(unreachable_patterns)]
            _ => Err(ProgramError::InvalidAccountData),
        }
    }
}

pub fn try_from_swap_with_parameters_context<'info>(
    accounts: &'info [AccountView],
) -> Result<(SwapParameters<'info>, SwapWithParametersContext<'info>), ProgramError> {
    let params = SwapParameters::try_from(accounts)?;
    let remaining = &accounts[SWAP_PARAMETERS_LEN..];
    let detector = remaining.first().ok_or(ProgramError::NotEnoughAccountKeys)?;

    #[cfg(feature = "perena-swap-with-parameters")]
    if address_eq(
        detector.address(),
        &crate::perena::PERENA_PROGRAM_ID,
    ) {
        let ctx = crate::perena::Perena::try_parse_remaining(remaining)?;
        return Ok((params, SwapWithParametersContext::Perena(ctx)));
    }

    #[cfg(feature = "solfi-swap-with-parameters")]
    if address_eq(detector.address(), &crate::solfi::SOLFI_PROGRAM_ID) {
        let ctx = crate::solfi::SolFi::try_parse_remaining(remaining)?;
        return Ok((params, SwapWithParametersContext::SolFi(ctx)));
    }

    #[cfg(feature = "solfi_v2-swap-with-parameters")]
    if address_eq(
        detector.address(),
        &crate::solfi_v2::SOLFI_V2_PROGRAM_ID,
    ) {
        let ctx = crate::solfi_v2::SolFiV2::try_parse_remaining(remaining)?;
        return Ok((params, SwapWithParametersContext::SolFiV2(ctx)));
    }

    #[cfg(feature = "manifest-swap-with-parameters")]
    if address_eq(
        detector.address(),
        &crate::manifest::MANIFEST_PROGRAM_ID,
    ) {
        let ctx = crate::manifest::Manifest::try_parse_remaining(remaining)?;
        return Ok((params, SwapWithParametersContext::Manifest(ctx)));
    }

    #[cfg(feature = "heaven-swap-with-parameters")]
    if address_eq(
        detector.address(),
        &crate::heaven::HEAVEN_PROGRAM_ID,
    ) {
        let ctx = crate::heaven::Heaven::try_parse_remaining(remaining)?;
        return Ok((params, SwapWithParametersContext::Heaven(ctx)));
    }

    #[cfg(feature = "aldrin-swap-with-parameters")]
    if address_eq(
        detector.address(),
        &crate::aldrin::ALDRIN_PROGRAM_ID,
    ) {
        let ctx = crate::aldrin::Aldrin::try_parse_remaining(remaining)?;
        return Ok((params, SwapWithParametersContext::Aldrin(ctx)));
    }

    #[cfg(feature = "aldrin_v2-swap-with-parameters")]
    if address_eq(
        detector.address(),
        &crate::aldrin_v2::ALDRIN_V2_PROGRAM_ID,
    ) {
        let ctx = crate::aldrin_v2::AldrinV2::try_parse_remaining(remaining)?;
        return Ok((params, SwapWithParametersContext::AldrinV2(ctx)));
    }

    #[cfg(feature = "futarchy-swap-with-parameters")]
    if address_eq(
        detector.address(),
        &crate::futarchy::FUTARCHY_PROGRAM_ID,
    ) {
        let ctx = crate::futarchy::Futarchy::try_parse_remaining(remaining)?;
        return Ok((params, SwapWithParametersContext::Futarchy(ctx)));
    }

    #[cfg(feature = "gamma-swap-with-parameters")]
    if address_eq(detector.address(), &crate::gamma::GAMMA_PROGRAM_ID) {
        let ctx = crate::gamma::Gamma::try_parse_remaining(remaining)?;
        return Ok((params, SwapWithParametersContext::Gamma(ctx)));
    }

    Err(ProgramError::InvalidAccountData)
}

pub fn swap_with_parameters_signed<'a>(
    accounts: &'a [AccountView],
    in_amount: u64,
    minimum_out_amount: u64,
    extra: &SwapWithParametersExtra<'a>,
    signer_seeds: &[Signer],
) -> ProgramResult {
    let (params, ctx) = try_from_swap_with_parameters_context(accounts)?;

    match (&ctx, extra) {
        #[cfg(feature = "perena-swap-with-parameters")]
        (SwapWithParametersContext::Perena(remaining), SwapWithParametersExtra::Perena(e)) => {
            crate::perena::Perena::swap_with_parameters_signed(
                &params, remaining, in_amount, minimum_out_amount, e, signer_seeds,
            )
        }

        #[cfg(feature = "solfi-swap-with-parameters")]
        (SwapWithParametersContext::SolFi(remaining), SwapWithParametersExtra::SolFi(())) => {
            crate::solfi::SolFi::swap_with_parameters_signed(
                &params, remaining, in_amount, minimum_out_amount, &(), signer_seeds,
            )
        }

        #[cfg(feature = "solfi_v2-swap-with-parameters")]
        (SwapWithParametersContext::SolFiV2(remaining), SwapWithParametersExtra::SolFiV2(())) => {
            crate::solfi_v2::SolFiV2::swap_with_parameters_signed(
                &params, remaining, in_amount, minimum_out_amount, &(), signer_seeds,
            )
        }

        #[cfg(feature = "manifest-swap-with-parameters")]
        (SwapWithParametersContext::Manifest(remaining), SwapWithParametersExtra::Manifest(e)) => {
            crate::manifest::Manifest::swap_with_parameters_signed(
                &params, remaining, in_amount, minimum_out_amount, e, signer_seeds,
            )
        }

        #[cfg(feature = "heaven-swap-with-parameters")]
        (SwapWithParametersContext::Heaven(remaining), SwapWithParametersExtra::Heaven(e)) => {
            crate::heaven::Heaven::swap_with_parameters_signed(
                &params, remaining, in_amount, minimum_out_amount, e, signer_seeds,
            )
        }

        #[cfg(feature = "aldrin-swap-with-parameters")]
        (SwapWithParametersContext::Aldrin(remaining), SwapWithParametersExtra::Aldrin(())) => {
            crate::aldrin::Aldrin::swap_with_parameters_signed(
                &params, remaining, in_amount, minimum_out_amount, &(), signer_seeds,
            )
        }

        #[cfg(feature = "aldrin_v2-swap-with-parameters")]
        (SwapWithParametersContext::AldrinV2(remaining), SwapWithParametersExtra::AldrinV2(())) => {
            crate::aldrin_v2::AldrinV2::swap_with_parameters_signed(
                &params, remaining, in_amount, minimum_out_amount, &(), signer_seeds,
            )
        }

        #[cfg(feature = "futarchy-swap-with-parameters")]
        (SwapWithParametersContext::Futarchy(remaining), SwapWithParametersExtra::Futarchy(())) => {
            crate::futarchy::Futarchy::swap_with_parameters_signed(
                &params, remaining, in_amount, minimum_out_amount, &(), signer_seeds,
            )
        }

        #[cfg(feature = "gamma-swap-with-parameters")]
        (SwapWithParametersContext::Gamma(remaining), SwapWithParametersExtra::Gamma(())) => {
            crate::gamma::Gamma::swap_with_parameters_signed(
                &params, remaining, in_amount, minimum_out_amount, &(), signer_seeds,
            )
        }

        #[allow(unreachable_patterns)]
        _ => Err(ProgramError::InvalidAccountData),
    }
}

pub fn swap_with_parameters(
    accounts: &[AccountView],
    in_amount: u64,
    minimum_out_amount: u64,
    extra: &SwapWithParametersExtra<'_>,
) -> ProgramResult {
    swap_with_parameters_signed(accounts, in_amount, minimum_out_amount, extra, &[])
}
