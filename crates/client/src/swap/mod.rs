#[cfg(feature = "manifest")]
pub mod manifest;

#[cfg(feature = "aldrin")]
pub mod aldrin;

#[cfg(feature = "futarchy")]
pub mod futarchy;

#[cfg(feature = "gamma")]
pub mod gamma;

#[cfg(feature = "omnipair")]
pub mod omnipair;

use solana_pubkey::Pubkey;
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
    Gamma { pool: Option<Pubkey> },

    #[cfg(feature = "aldrin")]
    Aldrin { pool: Option<Pubkey>, side: u8 },

    #[cfg(feature = "futarchy")]
    Futarchy { dao: Option<Pubkey>, swap_type: u8 },

    #[cfg(feature = "manifest")]
    Manifest {
        market: Option<Pubkey>,
        is_exact_in: bool,
    },

    #[cfg(feature = "omnipair")]
    Omnipair { pair: Option<Pubkey> },
}

/// A single step in a multi-swap composition.
///
/// Each step specifies a protocol resolver and the token pair for that leg.
/// This enables both single-pair multi-protocol resolution (same mints,
/// different protocols) and multi-hop routing (A→B, B→C, C→D).
pub struct SwapStep {
    pub protocol: SwapProtocol,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
}

/// Resolve accounts and data for a swap protocol.
///
/// Returns `(remaining_accounts, instruction_data)` ready for
/// the Beethoven on-chain program.
#[cfg(feature = "resolve")]
pub async fn resolve_swap(
    rpc: &RpcClient,
    protocol: &SwapProtocol,
    mint_a: &Pubkey,
    mint_b: &Pubkey,
    user: &Pubkey,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    match protocol {
        #[cfg(feature = "gamma")]
        SwapProtocol::Gamma { pool } => {
            gamma::resolve(rpc, pool.as_ref(), mint_a, mint_b, user).await
        }

        #[cfg(feature = "aldrin")]
        SwapProtocol::Aldrin { pool, side } => {
            aldrin::resolve(rpc, pool.as_ref(), *side, mint_a, mint_b, user).await
        }

        #[cfg(feature = "futarchy")]
        SwapProtocol::Futarchy { dao, swap_type } => {
            futarchy::resolve(rpc, dao.as_ref(), *swap_type, mint_a, mint_b, user).await
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
    user: &Pubkey,
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
