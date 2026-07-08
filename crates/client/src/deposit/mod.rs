#[cfg(feature = "resolve")]
use crate::ClientError;
#[cfg(feature = "resolve")]
use solana_address::Address;
#[cfg(feature = "resolve")]
use solana_instruction::AccountMeta;
#[cfg(feature = "resolve")]
use solana_rpc_client::nonblocking::rpc_client::RpcClient;

#[cfg(feature = "hylo")]
pub mod hylo;

/// Top-level deposit protocol selector.
///
/// Each variant carries the protocol-specific config needed
/// to resolve accounts.
pub enum DepositProtocol {
    #[cfg(feature = "hylo")]
    Hylo {
        lst_mint: Address,
        mint_type: u8,
        expected_token_out: u64,
        slippage_tolerance: u64,
    },
}

/// Resolve accounts and data for a deposit protocol.
///
/// Returns `(remaining_accounts, instruction_data)` ready for
/// the Beethoven on-chain program.
#[cfg(feature = "resolve")]
pub async fn resolve_deposit(
    rpc: &RpcClient,
    protocol: &DepositProtocol,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    match protocol {
        #[cfg(feature = "hylo")]
        DepositProtocol::Hylo {
            lst_mint,
            mint_type,
            expected_token_out,
            slippage_tolerance,
        } => {
            hylo::resolve(
                rpc,
                lst_mint,
                *mint_type,
                *expected_token_out,
                *slippage_tolerance,
                user,
            )
            .await
        }
    }
}
