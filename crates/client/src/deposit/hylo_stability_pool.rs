#[cfg(feature = "resolve")]
use crate::{get_associated_token_address, ClientError};
use {
    crate::{ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID},
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const HYLO_STABILITY_PROGRAM_ID: Address =
    address!("HysTabVUfmQBFcmzu1ctRd1Y1fxd66RBpboy1bmtDSQQ");
pub const POOL_CONFIG: Address = address!("2jk7miWrsTbt5hUSaCXPkEQPvuUMgbFLpgMzMQw3Z6ar");
pub const HYLO: Address = address!("9cd2sAfbBvKs4SX9YKo4dcjwP3TgTVQ8dT5koshGcDND");
pub const HYUSD_MINT: Address = address!("5YMkXAYccHSGnHn9nob9xEvv6Pvka9DZWH7nTbotTu9E");
pub const XSOL_MINT: Address = address!("4sWNB8zGWHkh6UnmwiEtzNxL4XrN7uK9tosbESbJFfVs");
pub const POOL_AUTH: Address = address!("5YrRAQag9BbJkauDtJkd1vsTquXT6N46oU8rJ66GDxHd");
pub const STABLECOIN_POOL: Address = address!("EqozKyMj7FVnLHc2cJj3VC25aBr4AhVh1cGM2WDajGe9");
pub const LEVERCOIN_POOL: Address = address!("4GPXVXuzk8ABAUkoXeBJg8r9kccEXQjoi5vqSxE9rhk1");
pub const LP_TOKEN_AUTH: Address = address!("5YWerkcqAXTSCzKC1X52BXtfv2aoNCB6wzv7wEXuGWpq");
pub const SHYUSD_MINT: Address = address!("HnnGv3HrSqjRpgdFmx7vQGjntNEoex1SU4e9Lxcxuihz");
pub const SOL_PRICE_UPDATE_V2: Address = address!("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");
pub const EVENT_AUTHORITY: Address = address!("8fjUWoZTb8ox8JFRJTb7WznL1V8oJT9o21kQKHJzbTS8");

/// Pre-resolved addresses for building an Hylo Stability Pool deposit instruction offline.
pub struct HyloStabilityPoolDepositInput {
    pub user: Address,
    pub user_stablecoin_ta: Address,
    pub user_lp_token_ta: Address,
}

/// Build Hylo Stability Pool deposit AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &HyloStabilityPoolDepositInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(HYLO_STABILITY_PROGRAM_ID, false),
        AccountMeta::new(input.user, true),
        AccountMeta::new_readonly(POOL_CONFIG, false),
        AccountMeta::new_readonly(HYLO, false),
        AccountMeta::new_readonly(HYUSD_MINT, false),
        AccountMeta::new_readonly(XSOL_MINT, false),
        AccountMeta::new(input.user_stablecoin_ta, false),
        AccountMeta::new(input.user_lp_token_ta, false),
        AccountMeta::new_readonly(POOL_AUTH, false),
        AccountMeta::new(STABLECOIN_POOL, false),
        AccountMeta::new_readonly(LEVERCOIN_POOL, false),
        AccountMeta::new_readonly(LP_TOKEN_AUTH, false),
        AccountMeta::new(SHYUSD_MINT, false),
        AccountMeta::new_readonly(SOL_PRICE_UPDATE_V2, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(EVENT_AUTHORITY, false),
        AccountMeta::new_readonly(HYLO_STABILITY_PROGRAM_ID, false),
    ]
}

/// Hylo Stability Pool deposit has no extra data.
pub fn build_extra_data() -> Vec<u8> {
    vec![]
}

/// Resolve accounts.
#[cfg(feature = "resolve")]
pub async fn resolve(user: &Address) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let user_stablecoin_ta = get_associated_token_address(user, &HYUSD_MINT, &TOKEN_PROGRAM_ID);
    let user_lp_token_ta = get_associated_token_address(user, &SHYUSD_MINT, &TOKEN_PROGRAM_ID);

    let input = HyloStabilityPoolDepositInput {
        user: *user,
        user_stablecoin_ta,
        user_lp_token_ta,
    };

    Ok((build_accounts(&input), build_extra_data()))
}
