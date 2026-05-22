#[cfg(feature = "resolve")]
use {
    crate::{
        discover_pool_with_flip, get_associated_token_address, get_token_program_for_mint,
        read_pubkey, ClientError,
    },
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const PHOENIX_LEGACY_PROGRAM_ID: Address =
    address!("PhoeNiXZ8ByJGLkxNfZRnkUfjvmuYqLR89jjFHGqdXY");

const DATA_LEN: usize = 63;

pub enum Side {
    Bid,
    Ask,
}

pub enum SelfTradeBehavior {
    Abort,
    CancelProvide,
    DecrementTake,
}

// Market account layout offsets
#[cfg(feature = "resolve")]
const OFFSET_BASE_MINT: usize = 48;
#[cfg(feature = "resolve")]
const OFFSET_QUOTE_MINT: usize = 128;

pub struct PhoenixLegacySwapInput {
    pub log_authority: Address,
    pub market: Address,
    pub trader: Address,
    pub base_account: Address,
    pub quote_account: Address,
    pub base_vault: Address,
    pub quote_vault: Address,
    pub token_program: Address,
}

/// Pre-resolved addresses for building a Phoenix Legacy swap instruction offline.
pub fn build_accounts(input: PhoenixLegacySwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(PHOENIX_LEGACY_PROGRAM_ID, false),
        AccountMeta::new_readonly(input.log_authority, false),
        AccountMeta::new(input.market, false),
        AccountMeta::new(input.trader, true),
        AccountMeta::new(input.base_account, false),
        AccountMeta::new(input.quote_account, false),
        AccountMeta::new(input.base_vault, false),
        AccountMeta::new(input.quote_vault, false),
        AccountMeta::new_readonly(input.token_program, false),
    ]
}

fn push_opt_u64(buf: &mut Vec<u8>, v: Option<u64>) {
    match v {
        None => {
            buf.push(0);
            buf.extend_from_slice(&[0u8; 8]);
        }
        Some(x) => {
            buf.push(1);
            buf.extend_from_slice(&x.to_le_bytes());
        }
    }
}

/// Build Phoenix Legacy extra data: [side, price_in_ticks, max_counterpart_lots, self_trade_behavior, match_limit, client_order_id, use_only_deposited_funds, last_valid_slot, last_valid_unix_timestamp_in_seconds].
#[allow(clippy::too_many_arguments)]
pub fn build_extra_data(
    side: Side,
    price_in_ticks: Option<u64>,
    max_counterpart_lots: u64,
    self_trade_behavior: SelfTradeBehavior,
    match_limit: Option<u64>,
    client_order_id: u128,
    use_only_deposited_funds: bool,
    last_valid_slot: Option<u64>,
    last_valid_unix_timestamp_in_seconds: Option<u64>,
) -> Vec<u8> {
    let mut data = Vec::with_capacity(DATA_LEN);
    data.push(side as u8);
    push_opt_u64(&mut data, price_in_ticks);
    data.extend_from_slice(&max_counterpart_lots.to_le_bytes());
    data.push(self_trade_behavior as u8);
    push_opt_u64(&mut data, match_limit);
    data.extend_from_slice(&(client_order_id as u64).to_le_bytes());
    data.extend_from_slice(&((client_order_id >> 64) as u64).to_le_bytes());
    data.push(use_only_deposited_funds as u8);
    push_opt_u64(&mut data, last_valid_slot);
    push_opt_u64(&mut data, last_valid_unix_timestamp_in_seconds);
    data
}

/// Resolve accounts and data for a Phoenix Legacy swap via RPC.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    market: Option<&Address>,
    side: u8,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let (market_pk, market_data) = match market {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = discover_pool_with_flip(
                rpc,
                &PHOENIX_LEGACY_PROGRAM_ID,
                OFFSET_BASE_MINT,
                OFFSET_QUOTE_MINT,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let base_mint = read_pubkey(&market_data, OFFSET_BASE_MINT)?;
    let quote_mint = read_pubkey(&market_data, OFFSET_QUOTE_MINT)?;

    if *mint_b != base_mint && *mint_b != quote_mint {
        return Err(ClientError::MintMismatch {
            expected: format!("{} or {}", base_mint, quote_mint),
            got: mint_b.to_string(),
        });
    }

    let side = match side {
        0 => Side::Bid,
        1 => Side::Ask,
        _ => return Err(ClientError::InvalidAccountData("Invalid side".into())),
    };

    let max_counterpart_lots = match side {
        Side::Bid => u64::MAX,
        Side::Ask => 0u64,
    };

    let base_token_program = get_token_program_for_mint(rpc, &base_mint).await?;
    let quote_token_program = get_token_program_for_mint(rpc, &quote_mint).await?;
    if base_token_program != quote_token_program {
        return Err(ClientError::InvalidAccountData(
            "Phoenix legacy swap requires base and quote mints to use the same token program"
                .into(),
        ));
    }

    let log_authority = Address::find_program_address(&[b"log"], &PHOENIX_LEGACY_PROGRAM_ID).0;
    let base_vault = Address::find_program_address(
        &[b"vault", market_pk.as_ref(), base_mint.as_ref()],
        &PHOENIX_LEGACY_PROGRAM_ID,
    )
    .0;
    let quote_vault = Address::find_program_address(
        &[b"vault", market_pk.as_ref(), quote_mint.as_ref()],
        &PHOENIX_LEGACY_PROGRAM_ID,
    )
    .0;

    let base_account = get_associated_token_address(user, &base_mint, &base_token_program);
    let quote_account = get_associated_token_address(user, &quote_mint, &quote_token_program);

    let input = PhoenixLegacySwapInput {
        log_authority,
        market: market_pk,
        trader: *user,
        base_account,
        quote_account,
        base_vault,
        quote_vault,
        token_program: base_token_program,
    };

    Ok((
        build_accounts(input),
        build_extra_data(
            side,
            None,
            max_counterpart_lots,
            SelfTradeBehavior::CancelProvide,
            None,
            0,
            false,
            None,
            None,
        )
        .to_vec(),
    ))
}
