#[cfg(feature = "manifest")]
pub mod manifest;

#[cfg(feature = "aldrin")]
pub mod aldrin;

#[cfg(feature = "aldrin-v2")]
pub mod aldrin_v2;

#[cfg(feature = "futarchy")]
pub mod futarchy;

#[cfg(feature = "gamma")]
pub mod gamma;

#[cfg(feature = "omnipair")]
pub mod omnipair;

#[cfg(feature = "hadron")]
pub mod hadron;

#[cfg(feature = "raydium-cpmm")]
pub mod raydium_cpmm;

#[cfg(feature = "raydium-clmm")]
pub mod raydium_clmm;

#[cfg(feature = "perena")]
pub mod perena;

#[cfg(feature = "heaven")]
pub mod heaven;

#[cfg(feature = "scale-amm")]
pub mod scale_amm;

#[cfg(feature = "scale-vmm")]
pub mod scale_vmm;

#[cfg(feature = "solfi")]
pub mod solfi;

#[cfg(feature = "solfi-v2")]
pub mod solfi_v2;

use solana_address::Address;
#[cfg(feature = "resolve")]
use {
    crate::error::ClientError, solana_instruction::AccountMeta,
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

/// Top-level swap protocol selector.
///
/// Each variant carries the protocol-specific config and data needed
/// to resolve accounts. When `pool`/`market` is `None`, the resolver
/// discovers it via `getProgramAccounts` with memcmp filters on the mints.
pub enum SwapProtocol {
    #[cfg(feature = "gamma")]
    Gamma { pool: Option<Address> },

    #[cfg(feature = "aldrin")]
    Aldrin {
        pool: Option<Address>,
        side: aldrin::Side,
    },

    #[cfg(feature = "aldrin-v2")]
    AldrinV2 { pool: Option<Address>, side: u8 },

    #[cfg(feature = "futarchy")]
    Futarchy {
        dao: Option<Address>,
        swap_type: futarchy::SwapType,
    },

    #[cfg(feature = "manifest")]
    Manifest {
        market: Option<Address>,
        is_exact_in: bool,
    },

    #[cfg(feature = "omnipair")]
    Omnipair { pair: Option<Address> },

    #[cfg(feature = "hadron")]
    Hadron {
        config: Address,
        fee_recipient: Address,
        expiration: i64,
    },

    #[cfg(feature = "raydium-cpmm")]
    RaydiumCpmm { pool: Option<Address> },

    #[cfg(feature = "raydium-clmm")]
    RaydiumClmm {
        pool: Option<Address>,
        sqrt_price_limit_x64: u128,
        is_base_input: bool,
    },

    #[cfg(feature = "perena")]
    Perena {
        pool: Option<Address>,
        in_index: u8,
        out_index: u8,
    },

    #[cfg(feature = "heaven")]
    Heaven {
        pool: Option<Address>,
        direction: u8,
        encoded_user_defined_event_data: Vec<u8>,
    },

    #[cfg(feature = "scale-amm")]
    ScaleAmm {
        pool: Option<Address>,
        side: scale_amm::Side,
    },

    #[cfg(feature = "scale-vmm")]
    ScaleVmm {
        pair: Option<Address>,
        side: scale_vmm::Side,
    },

    #[cfg(feature = "solfi")]
    SolFi {
        market: Option<Address>,
        is_quote_to_base: bool,
    },

    #[cfg(feature = "solfi-v2")]
    SolFiV2 {
        market: Option<Address>,
        is_quote_to_base: bool,
    },
}

/// A single step in a multi-swap composition.
///
/// Each step specifies a protocol resolver and the token pair for that leg.
/// This enables both single-pair multi-protocol resolution (same mints,
/// different protocols) and multi-hop routing (A→B, B→C, C→D).
pub struct SwapStep {
    pub protocol: SwapProtocol,
    pub mint_a: Address,
    pub mint_b: Address,
}

/// Resolve accounts and data for a swap protocol.
///
/// Returns `(remaining_accounts, instruction_data)` ready for
/// the Beethoven on-chain program.
#[cfg(feature = "resolve")]
pub async fn resolve_swap(
    rpc: &RpcClient,
    protocol: &SwapProtocol,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    match protocol {
        #[cfg(feature = "gamma")]
        SwapProtocol::Gamma { pool } => {
            gamma::resolve(rpc, pool.as_ref(), mint_a, mint_b, user).await
        }

        #[cfg(feature = "aldrin")]
        SwapProtocol::Aldrin { pool, side } => {
            aldrin::resolve(rpc, pool.as_ref(), side, mint_a, mint_b, user).await
        }

        #[cfg(feature = "aldrin-v2")]
        SwapProtocol::AldrinV2 { pool, side } => {
            aldrin_v2::resolve(rpc, pool.as_ref(), *side, mint_a, mint_b, user).await
        }

        #[cfg(feature = "futarchy")]
        SwapProtocol::Futarchy { dao, swap_type } => {
            futarchy::resolve(rpc, dao.as_ref(), swap_type, mint_a, mint_b, user).await
        }

        #[cfg(feature = "manifest")]
        SwapProtocol::Manifest {
            market,
            is_exact_in,
        } => manifest::resolve(rpc, market.as_ref(), *is_exact_in, mint_a, mint_b, user).await,

        #[cfg(feature = "omnipair")]
        SwapProtocol::Omnipair { pair } => {
            omnipair::resolve(rpc, pair.as_ref(), mint_a, mint_b, user).await
        }

        #[cfg(feature = "hadron")]
        SwapProtocol::Hadron {
            config,
            fee_recipient,
            expiration,
        } => {
            hadron::resolve(
                rpc,
                config,
                mint_a,
                mint_b,
                user,
                fee_recipient,
                *expiration,
            )
            .await
        }

        #[cfg(feature = "raydium-cpmm")]
        SwapProtocol::RaydiumCpmm { pool } => {
            raydium_cpmm::resolve(rpc, pool.as_ref(), mint_a, mint_b, user).await
        }

        #[cfg(feature = "raydium-clmm")]
        SwapProtocol::RaydiumClmm {
            pool,
            sqrt_price_limit_x64,
            is_base_input,
        } => {
            raydium_clmm::resolve(
                rpc,
                pool.as_ref(),
                mint_a,
                mint_b,
                user,
                *sqrt_price_limit_x64,
                *is_base_input,
            )
            .await
        }

        #[cfg(feature = "perena")]
        SwapProtocol::Perena {
            pool,
            in_index,
            out_index,
        } => {
            perena::resolve(
                rpc,
                pool.as_ref(),
                *in_index,
                *out_index,
                mint_a,
                mint_b,
                user,
            )
            .await
        }

        #[cfg(feature = "heaven")]
        SwapProtocol::Heaven {
            pool,
            direction,
            encoded_user_defined_event_data,
        } => {
            heaven::resolve(
                rpc,
                pool.as_ref(),
                *direction,
                encoded_user_defined_event_data,
                mint_a,
                mint_b,
                user,
            )
            .await
        }

        #[cfg(all(feature = "resolve", feature = "scale-amm"))]
        SwapProtocol::ScaleAmm { pool, side } => {
            scale_amm::resolve(rpc, pool.as_ref(), side, mint_a, mint_b, user).await
        }

        #[cfg(all(feature = "resolve", feature = "scale-vmm"))]
        SwapProtocol::ScaleVmm { pair, side } => {
            scale_vmm::resolve(rpc, pair.as_ref(), side, mint_a, mint_b, user).await
        }

        #[cfg(feature = "solfi")]
        SwapProtocol::SolFi {
            market,
            is_quote_to_base,
        } => {
            solfi::resolve(
                rpc,
                market.as_ref(),
                *is_quote_to_base,
                mint_a,
                mint_b,
                user,
            )
            .await
        }

        #[cfg(feature = "solfi-v2")]
        SwapProtocol::SolFiV2 {
            market,
            is_quote_to_base,
        } => {
            solfi_v2::resolve(
                rpc,
                market.as_ref(),
                *is_quote_to_base,
                mint_a,
                mint_b,
                user,
            )
            .await
        }
    }
}

/// Resolve accounts and data for multiple swap steps.
///
/// Returns concatenated `(remaining_accounts, instruction_data)`. Each
/// protocol's account block starts with its program ID, so the on-chain
/// program can detect protocol boundaries when iterating.
///
/// # Example
///
/// ```ignore
/// let steps = vec![
///     SwapStep {
///         protocol: SwapProtocol::Manifest {
///             market: Some(market_addr),
///             is_exact_in: true,
///         },
///         mint_a: wsol,
///         mint_b: usdc,
///     },
///     SwapStep {
///         protocol: SwapProtocol::Gamma { pool: None },
///         mint_a: usdc,
///         mint_b: bonk,
///     },
/// ];
///
/// let (accounts, data) = resolve_swaps(&rpc, &steps, &user).await?;
/// ```
#[cfg(feature = "resolve")]
pub async fn resolve_swaps(
    rpc: &RpcClient,
    steps: &[SwapStep],
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let mut all_accounts = Vec::new();
    let mut all_data = Vec::new();

    for step in steps {
        let (accounts, data) =
            resolve_swap(rpc, &step.protocol, &step.mint_a, &step.mint_b, user).await?;
        all_accounts.extend(accounts);
        all_data.extend(data);
    }

    Ok((all_accounts, all_data))
}
