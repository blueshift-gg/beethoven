#[cfg(feature = "resolve")]
use {
    crate::{discover_pool_with_flip, get_associated_token_address, read_pubkey, ClientError},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    crate::{
        ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID, SYSVAR_INSTRUCTIONS_ID,
        TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::Address,
    solana_instruction::AccountMeta,
};

pub const HEAVEN_PROGRAM_ID: Address =
    Address::from_str_const("HEAVENoP2qxoeuF8Dj2oT1GHEnu49U5mJYkdeC8BAX2o");
pub const CHAINLINK_STORE_PROGRAM_ID: Address =
    Address::from_str_const("HEvSKofvBgfaexv23kMabbYqxasxU3mQ4ibBMEmJWHny");
pub const CHAINLINK_SOL_USD_FEED: Address =
    Address::from_str_const("CH31Xns5z3M1cTAbKW34jcxPPciazARpijcHj9rxtemt");

// LiquidityPoolState account layout offsets
// Layout: [8-byte discriminator] [88 info] [360 market_cap_based_fees] [72 reserve] [48 lp_token]
//         [8 protocol_trading_fees] [8 creator_trading_fees]
//         [8 creator_trading_fees_claimed_by_creator] [8 creator_trading_fees_claimed_by_others]
//         [8 liquidity_provider_trading_fees] [8 creator_trading_fee_protocol_fees]
//         [8 reflection_trading_fees] [8 created_at_slot] [8 trading_volume_usd]
//         [8 creator_trading_fee_trading_volume_threshold]
//         [8 creator_trading_fee_trading_volume_threshold_reached_unix_timestamp]
//         [32 token_a_vault] [32 token_b_vault]
//         [32 protocol_config] [32 key]
//         [65 token_a] [65 token_b]

// LiquidityPoolInfo struct layout offsets (88)
// Layout: [32 creator] [32 update_authority] [8 open_at] [8 created_at] [2 protocol_config_version] [1 type] [1 pool_authority_bump] [1 temp_sol_holder_bump] [3 _pad]

// FeeBrackets struct layout offsets (72)
// Layout: [4 * (16) fee_bracket] [1 count] [7 _padding]

// FeeBracket struct layout offsets (16)
// Layout: [8 market_cap_upper_bound] [4 buy_fee_bps] [4 sell_fee_bps]

// LiquidityPoolMarketCapBasedFees struct layout offsets (360)
// Layout: [5 * (72) fee_brackets]

// LiquidityPoolReserve struct layout offsets (72)
// Layout: [8 token_a] [8 token_b] [8 snapshot_slot] [8 snapshot_a] [8 snapshot_b] [8 padding] [8 initial_a] [8 initial_b] [1 leader_slot_window] [7 _pad]

// LiquidityPoolLpTokenInfo struct layout offsets (48)
// Layout: [40 liquidity_pool_lp_token_supply] [1 decimals] [7 _pad]

// LiquidityPoolLpTokenSupply struct layout offsets (40)
// Layout: [8 initial] [8 total] [8 unlocked] [8 locked] [8 burnt]

// LiquidityPoolTokenInfo struct layout offsets (65)
// Layout: [32 mint] [1 decimals] [32 owner]
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_A_VAULT: usize = 664;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_B_VAULT: usize = 696;
#[cfg(feature = "resolve")]
const OFFSET_PROTOCOL_CONFIG: usize = 728;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_A_MINT: usize = 792;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_A_OWNER: usize = 825;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_B_MINT: usize = 857;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_B_OWNER: usize = 890;

pub struct HeavenSwapInput {
    pub liquidity_pool_state: Address,
    pub user: Address,
    pub token_a_program: Address,
    pub token_b_program: Address,
    pub token_a_mint: Address,
    pub token_b_mint: Address,
    pub user_token_a_vault: Address,
    pub user_token_b_vault: Address,
    pub token_a_vault: Address,
    pub token_b_vault: Address,
    pub protocol_config: Address,
}

pub fn build_accounts(input: &HeavenSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(HEAVEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(input.token_a_program, false),
        AccountMeta::new_readonly(input.token_b_program, false),
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new(input.liquidity_pool_state, false),
        AccountMeta::new(input.user, true),
        AccountMeta::new_readonly(input.token_a_mint, false),
        AccountMeta::new_readonly(input.token_b_mint, false),
        AccountMeta::new(input.user_token_a_vault, false),
        AccountMeta::new(input.user_token_b_vault, false),
        AccountMeta::new(input.token_a_vault, false),
        AccountMeta::new(input.token_b_vault, false),
        AccountMeta::new(input.protocol_config, false),
        AccountMeta::new_readonly(SYSVAR_INSTRUCTIONS_ID, false),
        AccountMeta::new_readonly(CHAINLINK_STORE_PROGRAM_ID, false),
        AccountMeta::new_readonly(CHAINLINK_SOL_USD_FEED, false),
    ]
}

/// Build Heaven extra data: [direction, ...encoded_user_defined_event_data].
pub fn build_extra_data(direction: u8, encoded_user_defined_event_data: &[u8]) -> Vec<u8> {
    let mut data = vec![direction];
    data.extend_from_slice(encoded_user_defined_event_data);
    data
}

#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    pool: Option<&Address>,
    direction: u8,
    encoded_user_defined_event_data: &[u8],
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    if direction > 1 {
        return Err(ClientError::InvalidAccountData(format!(
            "Invalid heaven direction: {} (expected 0=Buy or 1=Sell)",
            direction
        )));
    }

    let (pool_pubkey, pool_data) = match pool {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = discover_pool_with_flip(
                rpc,
                &HEAVEN_PROGRAM_ID,
                OFFSET_TOKEN_A_MINT,
                OFFSET_TOKEN_B_MINT,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let token_a_mint = read_pubkey(&pool_data, OFFSET_TOKEN_A_MINT)?;
    let token_b_mint = read_pubkey(&pool_data, OFFSET_TOKEN_B_MINT)?;
    let token_a_owner = read_pubkey(&pool_data, OFFSET_TOKEN_A_OWNER)?;
    let token_b_owner = read_pubkey(&pool_data, OFFSET_TOKEN_B_OWNER)?;
    let token_a_vault = read_pubkey(&pool_data, OFFSET_TOKEN_A_VAULT)?;
    let token_b_vault = read_pubkey(&pool_data, OFFSET_TOKEN_B_VAULT)?;
    let protocol_config = read_pubkey(&pool_data, OFFSET_PROTOCOL_CONFIG)?;

    let pair_matches = (*mint_a == token_a_mint && *mint_b == token_b_mint)
        || (*mint_a == token_b_mint && *mint_b == token_a_mint);
    if !pair_matches {
        return Err(ClientError::MintMismatch {
            expected: format!("{}/{}", token_a_mint, token_b_mint),
            got: format!("{}/{}", mint_a, mint_b),
        });
    }

    if token_a_owner != TOKEN_PROGRAM_ID && token_a_owner != TOKEN_2022_PROGRAM_ID {
        return Err(ClientError::InvalidAccountData(format!(
            "token_a owner {} is not a supported token program",
            token_a_owner
        )));
    }
    if token_b_owner != TOKEN_PROGRAM_ID && token_b_owner != TOKEN_2022_PROGRAM_ID {
        return Err(ClientError::InvalidAccountData(format!(
            "token_b owner {} is not a supported token program",
            token_b_owner
        )));
    }

    let user_token_a_vault = get_associated_token_address(user, &token_a_mint, &token_a_owner);
    let user_token_b_vault = get_associated_token_address(user, &token_b_mint, &token_b_owner);

    let input = HeavenSwapInput {
        liquidity_pool_state: pool_pubkey,
        user: *user,
        token_a_program: token_a_owner,
        token_b_program: token_b_owner,
        token_a_mint,
        token_b_mint,
        user_token_a_vault,
        user_token_b_vault,
        token_a_vault,
        token_b_vault,
        protocol_config,
    };

    Ok((
        build_accounts(&input),
        build_extra_data(direction, encoded_user_defined_event_data),
    ))
}
