#[cfg(feature = "resolve")]
use {crate::get_associated_token_address, solana_rpc_client::nonblocking::rpc_client::RpcClient};
use {
    crate::{SYSTEM_PROGRAM_ID, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID},
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const FRAUDSWORTH_TAX_PROGRAM_ID: Address =
    address!("43fZGRtmEsP7ExnJE1dbTbNjaP1ncvVmMPusSeksWGEj");
pub const FRAUDSWORTH_AMM_PROGRAM_ID: Address =
    address!("5JsSAL3kJDUWD4ZveYXYZmgm1eVqueesTZVdAvtZg8cR");
pub const FRAUDSWORTH_STAKING_PROGRAM_ID: Address =
    address!("12b3t1cNiAUoYLiWFEnFa4w6qYxVAiqCWU7KZuzLPYtH");
pub const FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID: Address =
    address!("CiQPQrmQh6BPhb9k7dFnsEs5gKPgdrvNKFc5xie5xVGd");
pub const EPOCH_STATE: Address = address!("FjJrLcmDjA8FtavGWdhJq3pdirAH889oWXc2bhEAMbDU");
pub const SWAP_AUTHORITY: Address = address!("CoCdbornGtiZ8tLxF5HD2TdGidfgfwbbiDX79BaZGJ2D");
pub const TAX_AUTHORITY: Address = address!("8zijSBnoiGQzwccQkdNuAwbZCieDZsxdn2GgKDErCemQ");
pub const STAKE_POOL: Address = address!("5BdRPPwEDpHEtRgdp4MfywbwmZnrf6u23bXMnG1w8ViN");
pub const STAKING_ESCROW: Address = address!("E68zPDgzMqnycj23g9T74ioHbDdvq3Npj5tT2yPd1SY");
pub const CARNAGE_VAULT: Address = address!("5988CYMcvJpNtGbtCDnAMxrjrLxRCq3qPME7w2v36aNT");
pub const TREASURY: Address = address!("3ihhwLnEJ2duwPSLYxhLbFrdhhxXLcvcrV9rAHqMgzCv");
pub const WSOL_INTERMEDIARY: Address = address!("2HPNULWVVdTcRiAm2DkghLA6frXxA2Nsu4VRu8a4qQ1s");

pub const POOL_WSOL_CRIME: Address = address!("ZWUZ3PzGk6bg6g3BS3WdXKbdAecUgZxnruKXQkte7wf");
pub const POOL_WSOL_CRIME_VAULT_A: Address =
    address!("14rFLiXzXk7aXLnwAz2kwQUjG9vauS84AQLu6LH9idUM");
pub const POOL_WSOL_CRIME_VAULT_B: Address =
    address!("6s6cprCGxTAYCk9LiwCpCsdHzReW7CLZKqy3ZSCtmV1b");

pub const POOL_WSOL_FRAUD: Address = address!("AngvViTVGd2zxP8KoFUjGU3TyrQjqeM1idRWiKM8p3mq");
pub const POOL_WSOL_FRAUD_VAULT_A: Address =
    address!("3sUDyw1k61NSKgn2EA9CaS3FbSZAApGeCRNwNFQPwg8o");
pub const POOL_WSOL_FRAUD_VAULT_B: Address =
    address!("2nzqXn6FivXjPSgrUGTA58eeVUDjGhvn4QLfhXK1jbjP");

pub const EXTRA_ACCOUNT_META_LIST_CRIME: Address =
    address!("7QGodnZAYGgastQMXcitcQjraYCMMNDgbp2uL73qjGkd");
pub const EXTRA_ACCOUNT_META_LIST_FRAUD: Address =
    address!("CStTzemevJvk8vnjw57Wjzk5EFwN12Nmniz6R7qXWykr");

pub const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
pub const CRIME_MINT: Address = address!("cRiMEhAxoDhcEuh3Yf7Z2QkXUXUMKbakhcVqmDsqPXc");
pub const FRAUD_MINT: Address = address!("FraUdp6YhtVJYPxC2w255yAbpTsPqd8Bfhy9rC56jau5");

/// Pre-resolved addresses for building a Fraudsworth Tax swap instruction offline.
pub struct FraudsworthTaxSwapInput {
    pub user: Address,
    pub user_token_a: Address,
    pub user_token_b: Address,
    pub is_buy: bool,
    pub is_crime: bool,
}

/// Build Fraudsworth Tax swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &FraudsworthTaxSwapInput) -> Vec<AccountMeta> {
    let mut accounts = vec![
        AccountMeta::new_readonly(FRAUDSWORTH_TAX_PROGRAM_ID, false),
        AccountMeta::new(input.user, true),
        AccountMeta::new_readonly(EPOCH_STATE, false),
        if input.is_buy {
            AccountMeta::new_readonly(SWAP_AUTHORITY, false)
        } else {
            AccountMeta::new(SWAP_AUTHORITY, false)
        },
        AccountMeta::new_readonly(TAX_AUTHORITY, false),
        AccountMeta::new(
            if input.is_crime {
                POOL_WSOL_CRIME
            } else {
                POOL_WSOL_FRAUD
            },
            false,
        ),
        AccountMeta::new(
            if input.is_crime {
                POOL_WSOL_CRIME_VAULT_A
            } else {
                POOL_WSOL_FRAUD_VAULT_A
            },
            false,
        ),
        AccountMeta::new(
            if input.is_crime {
                POOL_WSOL_CRIME_VAULT_B
            } else {
                POOL_WSOL_FRAUD_VAULT_B
            },
            false,
        ),
        AccountMeta::new_readonly(WSOL_MINT, false),
        AccountMeta::new_readonly(
            if input.is_crime {
                CRIME_MINT
            } else {
                FRAUD_MINT
            },
            false,
        ),
        AccountMeta::new(input.user_token_a, false),
        AccountMeta::new(input.user_token_b, false),
        AccountMeta::new(STAKE_POOL, false),
        AccountMeta::new(STAKING_ESCROW, false),
        AccountMeta::new(CARNAGE_VAULT, false),
        AccountMeta::new(TREASURY, false),
    ];

    if !input.is_buy {
        accounts.push(AccountMeta::new(WSOL_INTERMEDIARY, false));
    }

    let (source_token_account, destination_token_account) = if input.is_crime {
        let user_crime_ata =
            get_associated_token_address(&input.user, &CRIME_MINT, &TOKEN_2022_PROGRAM_ID);

        if input.is_buy {
            (POOL_WSOL_CRIME_VAULT_B, user_crime_ata)
        } else {
            (user_crime_ata, POOL_WSOL_CRIME_VAULT_B)
        }
    } else {
        let user_fraud_ata =
            get_associated_token_address(&input.user, &FRAUD_MINT, &TOKEN_2022_PROGRAM_ID);

        if input.is_buy {
            (POOL_WSOL_FRAUD_VAULT_B, user_fraud_ata)
        } else {
            (user_fraud_ata, POOL_WSOL_FRAUD_VAULT_B)
        }
    };

    let whitelist_source = Address::find_program_address(
        &[b"whitelist", source_token_account.as_ref()],
        &FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
    )
    .0;
    let whitelist_destination = Address::find_program_address(
        &[b"whitelist", destination_token_account.as_ref()],
        &FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
    )
    .0;

    accounts.extend_from_slice(&[
        AccountMeta::new_readonly(FRAUDSWORTH_AMM_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new_readonly(FRAUDSWORTH_STAKING_PROGRAM_ID, false),
        AccountMeta::new_readonly(
            if input.is_crime {
                EXTRA_ACCOUNT_META_LIST_CRIME
            } else {
                EXTRA_ACCOUNT_META_LIST_FRAUD
            },
            false,
        ),
        AccountMeta::new_readonly(whitelist_source, false),
        AccountMeta::new_readonly(whitelist_destination, false),
        AccountMeta::new_readonly(FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID, false),
    ]);

    accounts
}

/// Build Fraudsworth Tax extra data: [is_buy].
pub fn build_extra_data(is_buy: bool) -> Vec<u8> {
    vec![is_buy as u8]
}

/// Resolve accounts and data for a Fraudsworth Tax swap via RPC.
#[cfg(feature = "resolve")]
pub async fn resolve(
    _rpc: &RpcClient,
    user: &Address,
    is_buy: bool,
    is_crime: bool,
) -> Result<(Vec<AccountMeta>, Vec<u8>), crate::error::ClientError> {
    let user_token_a = get_associated_token_address(user, &WSOL_MINT, &TOKEN_PROGRAM_ID);
    let token_b = if is_crime { CRIME_MINT } else { FRAUD_MINT };
    let user_token_b = get_associated_token_address(user, &token_b, &TOKEN_2022_PROGRAM_ID);

    let input = FraudsworthTaxSwapInput {
        user: *user,
        user_token_a,
        user_token_b,
        is_buy,
        is_crime,
    };

    Ok((build_accounts(&input), build_extra_data(is_buy)))
}
