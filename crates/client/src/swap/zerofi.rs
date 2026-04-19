#[cfg(feature = "resolve")]
use {
    crate::{get_associated_token_address, read_pubkey, ClientError},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    crate::{SYSVAR_INSTRUCTIONS_ID, TOKEN_PROGRAM_ID},
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const ZEROFI_PROGRAM_ID: Address = address!("ZERor4xhbUycZ6gb9ntrhqscUcZmAbQDjEAtCf4hbZY");

// Market account layout offsets
// Layout: ... [32 mint_in] @ 72 [32 mint_out] ...
#[cfg(feature = "resolve")]
const OFFSET_MARKET_MINT_IN: usize = 72;
#[cfg(feature = "resolve")]
const OFFSET_MARKET_MINT_OUT: usize = 104;

// Cfg account layout offsets
// Layout: ... [32 ta] @ 40 ...
#[cfg(feature = "resolve")]
const OFFSET_CFG_TA: usize = 40;

/// Pre-resolved addresses for building an ZeroFi swap instruction offline.
pub struct ZerofiSwapInput {
    pub market: Address,
    pub cfg_in: Address,
    pub ta_in: Address,
    pub cfg_out: Address,
    pub ta_out: Address,
    pub usr_ta_in: Address,
    pub usr_ta_out: Address,
}

/// Build ZeroFi swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &ZerofiSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(ZEROFI_PROGRAM_ID, false),
        AccountMeta::new(input.market, false),
        AccountMeta::new(input.cfg_in, false),
        AccountMeta::new(input.ta_in, false),
        AccountMeta::new(input.cfg_out, false),
        AccountMeta::new(input.ta_out, false),
        AccountMeta::new(input.usr_ta_in, false),
        AccountMeta::new(input.usr_ta_out, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(SYSVAR_INSTRUCTIONS_ID, false),
    ]
}

/// Resolve ZeroFi `swap` accounts and empty Beethoven extra data.
///
/// `mint_a` is the token in; `mint_b` must match the output mint implied by the pool orientation.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    market: Option<&Address>,
    cfg_in: &Address,
    cfg_out: &Address,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let (market_pubkey, market_data) = match market {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = crate::discover_pool_with_flip(
                rpc,
                &ZEROFI_PROGRAM_ID,
                OFFSET_MARKET_MINT_IN,
                OFFSET_MARKET_MINT_OUT,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let cfg_in_data = rpc.get_account(cfg_in).await?.data;
    let ta_in = read_pubkey(&cfg_in_data, OFFSET_CFG_TA)?;
    let cfg_out_data = rpc.get_account(cfg_out).await?.data;
    let ta_out = read_pubkey(&cfg_out_data, OFFSET_CFG_TA)?;
    let market_mint_in = read_pubkey(&market_data, OFFSET_MARKET_MINT_IN)?;
    let market_mint_out = read_pubkey(&market_data, OFFSET_MARKET_MINT_OUT)?;

    let (mint_in, mint_out) = if *mint_a == market_mint_in {
        (market_mint_in, market_mint_out)
    } else if *mint_a == market_mint_out {
        (market_mint_out, market_mint_in)
    } else {
        return Err(ClientError::MintMismatch {
            expected: market_mint_in.to_string(),
            got: mint_a.to_string(),
        });
    };

    if *mint_b != mint_out {
        return Err(ClientError::MintMismatch {
            expected: market_mint_out.to_string(),
            got: mint_b.to_string(),
        });
    }

    let usr_ta_in = get_associated_token_address(user, &mint_in, &TOKEN_PROGRAM_ID);
    let usr_ta_out = get_associated_token_address(user, &mint_out, &TOKEN_PROGRAM_ID);

    let input = ZerofiSwapInput {
        market: market_pubkey,
        cfg_in: *cfg_in,
        ta_in,
        cfg_out: *cfg_out,
        ta_out,
        usr_ta_in,
        usr_ta_out,
    };

    Ok((build_accounts(&input), vec![]))
}
