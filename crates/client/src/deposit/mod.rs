#[cfg(feature = "resolve")]
use crate::ClientError;
#[cfg(feature = "resolve")]
use solana_address::Address;
#[cfg(feature = "resolve")]
use solana_instruction::AccountMeta;
#[cfg(feature = "resolve")]
use solana_rpc_client::nonblocking::rpc_client::RpcClient;

#[cfg(feature = "carrot-boost")]
pub mod carrot_boost;

/// Top-level deposit protocol selector.
///
/// Each variant carries the protocol-specific config and data needed
/// to resolve accounts. When `pool`/`market` is `None`, the resolver
/// discovers it via `getProgramAccounts` with memcmp filters on the mints.
pub enum DepositProtocol {
    #[cfg(feature = "carrot-boost")]
    CarrotBoost {
        clend_account: Address,
        bank: Address,
        deposit_up_to_amount: u8,
    },
}

/// Resolve accounts and data for a deposit protocol.
///
/// Returns `(accounts, instruction_data)` ready for the Beethoven on-chain program
/// (outer deposit layout: 8-byte amount + protocol extras).
#[cfg(feature = "resolve")]
pub async fn resolve_deposit(
    rpc: &RpcClient,
    protocol: &DepositProtocol,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    match protocol {
        #[cfg(feature = "carrot-boost")]
        DepositProtocol::CarrotBoost {
            clend_account,
            bank,
            deposit_up_to_amount,
        } => carrot_boost::resolve(rpc, clend_account, bank, *deposit_up_to_amount, user).await,
    }
}
