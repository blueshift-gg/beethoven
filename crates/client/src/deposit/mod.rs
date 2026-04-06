#[cfg(feature = "resolve")]
use crate::ClientError;
#[cfg(feature = "resolve")]
use solana_address::Address;
#[cfg(feature = "resolve")]
use solana_instruction::AccountMeta;
#[cfg(feature = "resolve")]
use solana_rpc_client::nonblocking::rpc_client::RpcClient;

#[cfg(feature = "perena_bankineco")]
pub mod perena_bankineco;

/// Top-level deposit protocol selector.
///
/// Each variant carries the protocol-specific config and data needed
/// to resolve accounts. When `pool`/`market` is `None`, the resolver
/// discovers it via `getProgramAccounts` with memcmp filters on the mints.
pub enum DepositProtocol {
    #[cfg(feature = "perena_bankineco")]
    PerenaBankineco {
        vault: Address,
        min_bank_mint_minted: u64,
    },
}

/// Resolve accounts and data for a deposit protocol.
///
/// Returns `(remaining_accounts, instruction_data)` ready for
/// the Beethoven on-chain program.
/// #[cfg(feature = "resolve")]
pub async fn resolve_deposit(
    rpc: &RpcClient,
    protocol: &DepositProtocol,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    match protocol {
        #[cfg(feature = "perena_bankineco")]
        DepositProtocol::PerenaBankineco {
            vault,
            min_bank_mint_minted,
        } => perena_bankineco::resolve(rpc, vault, *min_bank_mint_minted, user).await,
    }
}
