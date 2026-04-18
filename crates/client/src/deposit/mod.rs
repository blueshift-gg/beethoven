#[cfg(feature = "resolve")]
use crate::ClientError;
#[cfg(feature = "resolve")]
use solana_address::Address;
#[cfg(feature = "resolve")]
use solana_instruction::AccountMeta;
#[cfg(feature = "resolve")]
use solana_rpc_client::nonblocking::rpc_client::RpcClient;

/// Top-level deposit protocol selector.
///
/// Each variant carries the protocol-specific config and data needed
/// to resolve accounts. When `pool`/`market` is `None`, the resolver
/// discovers it via `getProgramAccounts` with memcmp filters on the mints.
pub enum DepositProtocol {}

/// Resolve accounts and data for a deposit protocol.
///
/// Returns `(remaining_accounts, instruction_data)` ready for
/// the Beethoven on-chain program.
/// #[cfg(feature = "resolve")]
pub async fn resolve_deposit(
    _rpc: &RpcClient,
    _protocol: &DepositProtocol,
    _user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    todo!()
}
