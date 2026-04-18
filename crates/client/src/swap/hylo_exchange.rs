#[cfg(feature = "resolve")]
use crate::{get_associated_token_address, get_token_program_for_mint, ClientError};
#[cfg(feature = "resolve")]
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use {
    crate::{ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID},
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const HYLO_EXCHANGE_PROGRAM_ID: Address =
    address!("HYEXCHtHkBagdStcJCp3xbbb9B7sdMdWXFNj6mdsG4hn");
pub const HYLO: Address = address!("9cd2sAfbBvKs4SX9YKo4dcjwP3TgTVQ8dT5koshGcDND");
pub const HYUSD_STABLECOIN_AUTH: Address = address!("CfuSViqf6wvUKEprLhtuCsSanvfAsMbDmkAW92FP95qe");
pub const HYUSD_MINT: Address = address!("5YMkXAYccHSGnHn9nob9xEvv6Pvka9DZWH7nTbotTu9E");
pub const HYUSD_FEE_AUTH: Address = address!("3HT6dD6APJh89XJs9rkn3BmsvkXE9jPG9dWJmUjWu6TS");
pub const HYUSD_FEE_VAULT: Address = address!("Hh8N3Fdauxgq1jjcKdzGBR3D8cdkpLZrFEVumL1tYQLp");
pub const SOL_PRICE_UPDATE_V2: Address = address!("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");
pub const EVENT_AUTHORITY: Address = address!("4VzpNE51Be5vD5Yg8MC3z6TVHq5gGbLJptjv18QbD6WP");

/// Pre-resolved addresses for building a Hylo Exchange swap instruction offline.
pub enum HyloExchangeSwapInput {
    Stablecoin {
        user: Address,
        fee_auth: Address,
        vault_auth: Address,
        fee_vault: Address,
        lst_vault: Address,
        lst_header: Address,
        user_stablecoin_ta: Address,
        user_lst_ta: Address,
        lst_mint: Address,
        is_mint: bool,
    },
    Levercoin {
        user: Address,
        fee_auth: Address,
        vault_auth: Address,
        levercoin_auth: Address,
        fee_vault: Address,
        lst_vault: Address,
        lst_header: Address,
        user_lst_ta: Address,
        user_levercoin_ta: Address,
        lst_mint: Address,
        levercoin_mint: Address,
        is_mint: bool,
    },
    Rebalance {
        user: Address,
        user_stablecoin_ta: Address,
        levercoin_mint: Address,
        levercoin_auth: Address,
        user_levercoin_ta: Address,
    },
}

/// Build Hylo Exchange swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &HyloExchangeSwapInput) -> Vec<AccountMeta> {
    let mut metas = vec![AccountMeta::new_readonly(HYLO_EXCHANGE_PROGRAM_ID, false)];

    match input {
        HyloExchangeSwapInput::Stablecoin {
            user,
            fee_auth,
            vault_auth,
            fee_vault,
            lst_vault,
            lst_header,
            user_stablecoin_ta,
            user_lst_ta,
            lst_mint,
            is_mint,
        } => {
            metas.extend_from_slice(&[
                AccountMeta::new(*user, true),
                AccountMeta::new(HYLO, false),
                AccountMeta::new_readonly(*fee_auth, false),
                AccountMeta::new_readonly(*vault_auth, false),
            ]);

            if *is_mint {
                metas.extend_from_slice(&[
                    AccountMeta::new(HYUSD_STABLECOIN_AUTH, false),
                    AccountMeta::new(*fee_vault, false),
                    AccountMeta::new(*lst_vault, false),
                    AccountMeta::new_readonly(*lst_header, false),
                    AccountMeta::new(*user_lst_ta, false),
                    AccountMeta::new(*user_stablecoin_ta, false),
                    AccountMeta::new_readonly(*lst_mint, false),
                    AccountMeta::new(HYUSD_MINT, false),
                    AccountMeta::new_readonly(SOL_PRICE_UPDATE_V2, false),
                    AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                    AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
                    AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                ]);
            } else {
                metas.extend_from_slice(&[
                    AccountMeta::new(*fee_vault, false),
                    AccountMeta::new(*lst_vault, false),
                    AccountMeta::new_readonly(*lst_header, false),
                    AccountMeta::new(*user_stablecoin_ta, false),
                    AccountMeta::new(*user_lst_ta, false),
                    AccountMeta::new(HYUSD_MINT, false),
                    AccountMeta::new_readonly(*lst_mint, false),
                    AccountMeta::new_readonly(SOL_PRICE_UPDATE_V2, false),
                    AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                    AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                    AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
                ])
            }

            metas.extend_from_slice(&[
                AccountMeta::new_readonly(EVENT_AUTHORITY, false),
                AccountMeta::new_readonly(HYLO_EXCHANGE_PROGRAM_ID, false),
            ]);
        }
        HyloExchangeSwapInput::Levercoin {
            user,
            fee_auth,
            vault_auth,
            levercoin_auth,
            fee_vault,
            lst_vault,
            lst_header,
            user_lst_ta,
            user_levercoin_ta,
            lst_mint,
            levercoin_mint,
            is_mint,
        } => {
            metas.extend_from_slice(&[
                AccountMeta::new(*user, true),
                AccountMeta::new(HYLO, false),
                AccountMeta::new_readonly(*fee_auth, false),
                AccountMeta::new_readonly(*vault_auth, false),
            ]);

            if *is_mint {
                metas.extend_from_slice(&[
                    AccountMeta::new_readonly(*levercoin_auth, false),
                    AccountMeta::new(*fee_vault, false),
                    AccountMeta::new(*lst_vault, false),
                    AccountMeta::new_readonly(*lst_header, false),
                    AccountMeta::new(*user_lst_ta, false),
                    AccountMeta::new(*user_levercoin_ta, false),
                    AccountMeta::new_readonly(*lst_mint, false),
                    AccountMeta::new(*levercoin_mint, false),
                    AccountMeta::new_readonly(HYUSD_MINT, false),
                    AccountMeta::new_readonly(SOL_PRICE_UPDATE_V2, false),
                    AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                    AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
                    AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                ]);
            } else {
                metas.extend_from_slice(&[
                    AccountMeta::new(*fee_vault, false),
                    AccountMeta::new(*lst_vault, false),
                    AccountMeta::new_readonly(*lst_header, false),
                    AccountMeta::new(*user_levercoin_ta, false),
                    AccountMeta::new(*user_lst_ta, false),
                    AccountMeta::new(*levercoin_mint, false),
                    AccountMeta::new_readonly(HYUSD_MINT, false),
                    AccountMeta::new_readonly(*lst_mint, false),
                    AccountMeta::new_readonly(SOL_PRICE_UPDATE_V2, false),
                    AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                    AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                    AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
                ]);
            }

            metas.extend_from_slice(&[
                AccountMeta::new_readonly(EVENT_AUTHORITY, false),
                AccountMeta::new_readonly(HYLO_EXCHANGE_PROGRAM_ID, false),
            ]);
        }
        HyloExchangeSwapInput::Rebalance {
            user,
            user_stablecoin_ta,
            levercoin_mint,
            levercoin_auth,
            user_levercoin_ta,
        } => metas.extend_from_slice(&[
            AccountMeta::new(*user, true),
            AccountMeta::new_readonly(HYLO, false),
            AccountMeta::new_readonly(SOL_PRICE_UPDATE_V2, false),
            AccountMeta::new(HYUSD_MINT, false),
            AccountMeta::new_readonly(HYUSD_STABLECOIN_AUTH, false),
            AccountMeta::new_readonly(HYUSD_FEE_AUTH, false),
            AccountMeta::new(HYUSD_FEE_VAULT, false),
            AccountMeta::new(*user_stablecoin_ta, false),
            AccountMeta::new(*levercoin_mint, false),
            AccountMeta::new_readonly(*levercoin_auth, false),
            AccountMeta::new(*user_levercoin_ta, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
            AccountMeta::new_readonly(EVENT_AUTHORITY, false),
            AccountMeta::new_readonly(HYLO_EXCHANGE_PROGRAM_ID, false),
        ]),
    }

    metas
}

// Hylo Exchange swap: [swap_type]
pub fn build_extra_data(swap_type: u8) -> Vec<u8> {
    vec![swap_type]
}

/// Resolve accounts and data for a Hylo Exchange swap via RPC.
///
/// mint_a: input mint
///
/// mint_b: output mint
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    swap_type: u8,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let input = match swap_type {
        0 | 1 => {
            let is_mint = swap_type == 0;
            let lst_mint = if is_mint { mint_a } else { mint_b };

            let fee_auth = Address::find_program_address(
                &[b"fee_auth", lst_mint.as_ref()],
                &HYLO_EXCHANGE_PROGRAM_ID,
            )
            .0;
            let vault_auth = Address::find_program_address(
                &[b"vault_auth", lst_mint.as_ref()],
                &HYLO_EXCHANGE_PROGRAM_ID,
            )
            .0;
            let mint_token_program = get_token_program_for_mint(rpc, lst_mint).await?;
            let fee_vault = get_associated_token_address(&fee_auth, lst_mint, &mint_token_program);
            let lst_vault =
                get_associated_token_address(&vault_auth, lst_mint, &mint_token_program);
            let lst_header = Address::find_program_address(
                &[b"lst_header", lst_mint.as_ref()],
                &HYLO_EXCHANGE_PROGRAM_ID,
            )
            .0;
            let user_lst_ta = get_associated_token_address(user, lst_mint, &mint_token_program);
            let user_stablecoin_ta =
                get_associated_token_address(user, &HYUSD_MINT, &mint_token_program);

            HyloExchangeSwapInput::Stablecoin {
                user: *user,
                fee_auth,
                vault_auth,
                fee_vault,
                lst_vault,
                lst_header,
                user_stablecoin_ta,
                user_lst_ta,
                lst_mint: *lst_mint,
                is_mint,
            }
        }
        2 | 3 => {
            let is_mint = swap_type == 2;
            let (lst_mint, levercoin_mint) = if is_mint {
                (mint_a, mint_b)
            } else {
                (mint_b, mint_a)
            };

            let fee_auth = Address::find_program_address(
                &[b"fee_auth", lst_mint.as_ref()],
                &HYLO_EXCHANGE_PROGRAM_ID,
            )
            .0;
            let vault_auth = Address::find_program_address(
                &[b"vault_auth", lst_mint.as_ref()],
                &HYLO_EXCHANGE_PROGRAM_ID,
            )
            .0;
            let levercoin_auth = Address::find_program_address(
                &[b"mint_auth", levercoin_mint.as_ref()],
                &HYLO_EXCHANGE_PROGRAM_ID,
            )
            .0;
            let mint_token_program = get_token_program_for_mint(rpc, lst_mint).await?;
            let fee_vault = get_associated_token_address(&fee_auth, lst_mint, &mint_token_program);
            let lst_vault =
                get_associated_token_address(&vault_auth, lst_mint, &mint_token_program);
            let lst_header = Address::find_program_address(
                &[b"lst_header", lst_mint.as_ref()],
                &HYLO_EXCHANGE_PROGRAM_ID,
            )
            .0;
            let user_lst_ta = get_associated_token_address(user, lst_mint, &mint_token_program);
            let levercoin_token_program = get_token_program_for_mint(rpc, levercoin_mint).await?;
            let user_levercoin_ta =
                get_associated_token_address(user, levercoin_mint, &levercoin_token_program);

            HyloExchangeSwapInput::Levercoin {
                user: *user,
                fee_auth,
                vault_auth,
                levercoin_auth,
                fee_vault,
                lst_vault,
                lst_header,
                user_lst_ta,
                user_levercoin_ta,
                lst_mint: *lst_mint,
                levercoin_mint: *levercoin_mint,
                is_mint,
            }
        }
        4 | 5 => {
            let is_mint = swap_type == 4;
            let (stablecoin_mint, levercoin_mint) = if is_mint {
                (mint_a, mint_b)
            } else {
                (mint_b, mint_a)
            };

            let levercoin_auth = Address::find_program_address(
                &[b"mint_auth", levercoin_mint.as_ref()],
                &HYLO_EXCHANGE_PROGRAM_ID,
            )
            .0;
            let mint_token_program = get_token_program_for_mint(rpc, stablecoin_mint).await?;
            let user_stablecoin_ta =
                get_associated_token_address(user, stablecoin_mint, &mint_token_program);
            let user_levercoin_ta =
                get_associated_token_address(user, levercoin_mint, &mint_token_program);

            HyloExchangeSwapInput::Rebalance {
                user: *user,
                user_stablecoin_ta,
                levercoin_mint: *levercoin_mint,
                levercoin_auth,
                user_levercoin_ta,
            }
        }
        _ => return Err(ClientError::InvalidSwapType(swap_type.to_string())),
    };

    Ok((build_accounts(&input), build_extra_data(swap_type)))
}
