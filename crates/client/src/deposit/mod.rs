#[cfg(feature = "resolve")]
use crate::ClientError;
#[cfg(feature = "resolve")]
use solana_address::Address;
#[cfg(feature = "resolve")]
use solana_instruction::AccountMeta;
#[cfg(feature = "resolve")]
use solana_rpc_client::nonblocking::rpc_client::RpcClient;

#[cfg(feature = "hylo_stability_pool")]
pub mod hylo_stability_pool;

/// Top-level deposit protocol selector.
///
/// Each variant carries the protocol-specific config and data needed
/// to resolve accounts. When `pool`/`market` is `None`, the resolver
/// discovers it via `getProgramAccounts` with memcmp filters on the mints.
pub enum DepositProtocol {
    #[cfg(feature = "hylo_stability_pool")]
    HyloStabilityPool,
}

/// Resolve accounts and data for a deposit protocol.
///
/// Returns `(remaining_accounts, instruction_data)` ready for
/// the Beethoven on-chain program.
/// #[cfg(feature = "resolve")]
pub async fn resolve_deposit(
    _rpc: &RpcClient,
    protocol: &DepositProtocol,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    match protocol {
        #[cfg(feature = "hylo_stability_pool")]
        DepositProtocol::HyloStabilityPool => hylo_stability_pool::resolve(user).await,
    }
}
