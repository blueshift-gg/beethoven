use {
    crate::TOKEN_PROGRAM_ID,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};
#[cfg(feature = "resolve")]
use {
    crate::{
        discover_pool_with_flip, get_associated_token_address, get_token_program_for_mint,
        read_pubkey, ClientError,
    },
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

pub const OBRIC_V2_PROGRAM_ID: Address = address!("obriQD1zbpyLz95G5n7nJe6a4DPjpFwa5XYPoNm113y");

// Market account layout offsets
// Layout: ... [32 ref_oracle]
// ... [32 reserve_x] [32 reserve_y]
// ... [32 second_ref_oracle]
// ... [32 mint_x] [32 mint_y]
// ... [32 third_ref_oracle] [32 x_price_feed] [32 y_price_feed] ...
#[cfg(feature = "resolve")]
const OFFSET_REF_ORACLE: usize = 9;
#[cfg(feature = "resolve")]
const OFFSET_RESERVE_X: usize = 73;
#[cfg(feature = "resolve")]
const OFFSET_RESERVE_Y: usize = 105;
#[cfg(feature = "resolve")]
const OFFSET_SECOND_REF_ORACLE: usize = 169;
#[cfg(feature = "resolve")]
const OFFSET_MINT_X: usize = 202;
#[cfg(feature = "resolve")]
const OFFSET_MINT_Y: usize = 234;
#[cfg(feature = "resolve")]
const OFFSET_THIRD_REF_ORACLE: usize = 482;
#[cfg(feature = "resolve")]
const OFFSET_X_PRICE_FEED: usize = 514;
#[cfg(feature = "resolve")]
const OFFSET_Y_PRICE_FEED: usize = 546;

/// Pre-resolved addresses for building an Obric V2 swap instruction offline.
pub struct ObricV2SwapInput {
    pub market: Address,
    pub second_ref_oracle: Address,
    pub third_ref_oracle: Address,
    pub reserve_x: Address,
    pub reserve_y: Address,
    pub user_ta_x: Address,
    pub user_ta_y: Address,
    pub ref_oracle: Address,
    pub x_price_feed: Address,
    pub y_price_feed: Address,
    pub user: Address,
}

/// Build Obric V2 swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &ObricV2SwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(OBRIC_V2_PROGRAM_ID, false),
        AccountMeta::new(input.market, false),
        AccountMeta::new_readonly(input.second_ref_oracle, false),
        AccountMeta::new_readonly(input.third_ref_oracle, false),
        AccountMeta::new(input.reserve_x, false),
        AccountMeta::new(input.reserve_y, false),
        AccountMeta::new(input.user_ta_x, false),
        AccountMeta::new(input.user_ta_y, false),
        AccountMeta::new(input.ref_oracle, false),
        AccountMeta::new_readonly(input.x_price_feed, false),
        AccountMeta::new_readonly(input.y_price_feed, false),
        AccountMeta::new(input.user, true),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
    ]
}

/// Build Obric V2 swap extra data: [x_to_y].
pub fn build_extra_data(x_to_y: bool) -> Vec<u8> {
    vec![x_to_y as u8]
}

#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    market: Option<&Address>,
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
            let (pubkey, account) = discover_pool_with_flip(
                rpc,
                &OBRIC_V2_PROGRAM_ID,
                OFFSET_MINT_X,
                OFFSET_MINT_Y,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let mint_x = read_pubkey(&market_data, OFFSET_MINT_X)?;
    let mint_y = read_pubkey(&market_data, OFFSET_MINT_Y)?;
    let reserve_x = read_pubkey(&market_data, OFFSET_RESERVE_X)?;
    let reserve_y = read_pubkey(&market_data, OFFSET_RESERVE_Y)?;
    let ref_oracle = read_pubkey(&market_data, OFFSET_REF_ORACLE)?;
    let second_ref_oracle = read_pubkey(&market_data, OFFSET_SECOND_REF_ORACLE)?;
    let third_ref_oracle = read_pubkey(&market_data, OFFSET_THIRD_REF_ORACLE)?;
    let x_price_feed = read_pubkey(&market_data, OFFSET_X_PRICE_FEED)?;
    let y_price_feed = read_pubkey(&market_data, OFFSET_Y_PRICE_FEED)?;

    let x_to_y = if *mint_a == mint_x && *mint_b == mint_y {
        true
    } else if *mint_a == mint_y && *mint_b == mint_x {
        false
    } else {
        return Err(ClientError::MintMismatch {
            expected: format!("({}, {}) or ({}, {})", mint_x, mint_y, mint_y, mint_x),
            got: format!("({}, {})", mint_a, mint_b),
        });
    };

    let token_program_x = get_token_program_for_mint(rpc, &mint_x).await?;
    let token_program_y = get_token_program_for_mint(rpc, &mint_y).await?;

    let user_ta_x = get_associated_token_address(user, &mint_x, &token_program_x);
    let user_ta_y = get_associated_token_address(user, &mint_y, &token_program_y);

    let input = ObricV2SwapInput {
        market: market_pubkey,
        second_ref_oracle,
        third_ref_oracle,
        reserve_x,
        reserve_y,
        user_ta_x,
        user_ta_y,
        ref_oracle,
        x_price_feed,
        y_price_feed,
        user: *user,
    };

    Ok((build_accounts(&input), build_extra_data(x_to_y)))
}
