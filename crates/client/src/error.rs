use thiserror::Error;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Invalid account data: {0}")]
    InvalidAccountData(String),

    #[error("Pool not found for the given mints")]
    PoolNotFound,

    #[error("Mint mismatch: expected {expected}, got {got}")]
    MintMismatch { expected: String, got: String },

    #[error("Invalid swap type: {0}")]
    InvalidSwapType(String),
}

#[cfg(feature = "resolve")]
impl From<solana_rpc_client_api::client_error::Error> for ClientError {
    fn from(e: solana_rpc_client_api::client_error::Error) -> Self {
        ClientError::Rpc(e.to_string())
    }
}
