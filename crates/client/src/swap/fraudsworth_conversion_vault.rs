#[cfg(feature = "resolve")]
use solana_rpc_client::nonblocking::rpc_client::RpcClient;
use {
    crate::{get_associated_token_address, ClientError, TOKEN_2022_PROGRAM_ID},
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const FRAUDSWORTH_CONVERSION_VAULT_PROGRAM_ID: Address =
    address!("5uawA6ehYTu69Ggvm3LSK84qFawPKxbWgfngwj15NRJ");
pub const FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID: Address =
    address!("CiQPQrmQh6BPhb9k7dFnsEs5gKPgdrvNKFc5xie5xVGd");

pub const VAULT_CONFIG: Address = address!("8vFpSBnCVt8dfX57FKrsGwy39TEo1TjVzrj9QYGxCkcD");
pub const VAULT_CRIME: Address = address!("Gh9QHMY3J2NGyaHFH2XQCWxedf4G7kBfyu7Jonwn1bHA");
pub const VAULT_FRAUD: Address = address!("DLciB9t3qEuRcndGyjRmu1Z34NCwTPvNwbv7eUsFxTZG");
pub const VAULT_PROFIT: Address = address!("DBMaWgfUW8WBb8VVvqDFkrMpEkPkCPTcLpSpyzHAiwp3");

pub const EXTRA_ACCOUNT_META_LIST_CRIME: Address =
    address!("CStTzemevJvk8vnjw57Wjzk5EFwN12Nmniz6R7qXWykr");
pub const EXTRA_ACCOUNT_META_LIST_FRAUD: Address =
    address!("7QGodnZAYGgastQMXcitcQjraYCMMNDgbp2uL73qjGkd");
pub const EXTRA_ACCOUNT_META_LIST_PROFIT: Address =
    address!("J4dubfKw7vnZLhpPfMHqz8PcYWaChugnnSGUgGDzQ9AB");

pub const CRIME_MINT: Address = address!("cRiMEhAxoDhcEuh3Yf7Z2QkXUXUMKbakhcVqmDsqPXc");
pub const FRAUD_MINT: Address = address!("FraUdp6YhtVJYPxC2w255yAbpTsPqd8Bfhy9rC56jau5");
pub const PROFIT_MINT: Address = address!("pRoFiTj36haRD5sG2Neqib9KoSrtdYMGrM7SEkZetfR");

pub struct FraudsworthConversionVaultSwapInput {
    pub user: Address,
    pub user_input_account: Address,
    pub user_output_account: Address,
    pub input_mint: Address,
    pub output_mint: Address,
}

fn select_vault_for_mint(mint: &Address) -> Result<Address, ClientError> {
    let vault = match *mint {
        CRIME_MINT => VAULT_CRIME,
        FRAUD_MINT => VAULT_FRAUD,
        PROFIT_MINT => VAULT_PROFIT,
        _ => {
            return Err(ClientError::MintMismatch {
                expected: format!("{} or {} or {}", CRIME_MINT, FRAUD_MINT, PROFIT_MINT),
                got: mint.to_string(),
            })
        }
    };

    Ok(vault)
}

fn extra_account_meta_list_for_mint(mint: &Address) -> Result<Address, ClientError> {
    let extra_account_meta_list = match *mint {
        CRIME_MINT => EXTRA_ACCOUNT_META_LIST_CRIME,
        FRAUD_MINT => EXTRA_ACCOUNT_META_LIST_FRAUD,
        PROFIT_MINT => EXTRA_ACCOUNT_META_LIST_PROFIT,
        _ => {
            return Err(ClientError::MintMismatch {
                expected: format!("{} or {} or {}", CRIME_MINT, FRAUD_MINT, PROFIT_MINT),
                got: mint.to_string(),
            })
        }
    };

    Ok(extra_account_meta_list)
}

fn build_hook_accounts(
    mint: &Address,
    source: &Address,
    destination: &Address,
) -> Result<[AccountMeta; 4], ClientError> {
    let extra_account_meta_list = extra_account_meta_list_for_mint(mint)?;

    let whitelist_source = Address::find_program_address(
        &[b"whitelist", source.as_ref()],
        &FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
    )
    .0;

    let whitelist_destination = Address::find_program_address(
        &[b"whitelist", destination.as_ref()],
        &FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID,
    )
    .0;

    Ok([
        AccountMeta::new_readonly(extra_account_meta_list, false),
        AccountMeta::new_readonly(whitelist_source, false),
        AccountMeta::new_readonly(whitelist_destination, false),
        AccountMeta::new_readonly(FRAUDSWORTH_TRANSFER_HOOK_PROGRAM_ID, false),
    ])
}

/// Build Fraudsworth Conversion Vault swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(
    input: &FraudsworthConversionVaultSwapInput,
) -> Result<Vec<AccountMeta>, ClientError> {
    let vault_input = select_vault_for_mint(&input.input_mint)?;
    let vault_output = select_vault_for_mint(&input.output_mint)?;
    let input_hook_accounts =
        build_hook_accounts(&input.input_mint, &input.user_input_account, &vault_input)?;
    let output_hook_accounts = build_hook_accounts(
        &input.output_mint,
        &vault_output,
        &input.user_output_account,
    )?;

    let mut accounts = vec![
        AccountMeta::new_readonly(FRAUDSWORTH_CONVERSION_VAULT_PROGRAM_ID, false),
        AccountMeta::new(input.user, true),
        AccountMeta::new_readonly(VAULT_CONFIG, false),
        AccountMeta::new(input.user_input_account, false),
        AccountMeta::new(input.user_output_account, false),
        AccountMeta::new_readonly(input.input_mint, false),
        AccountMeta::new_readonly(input.output_mint, false),
        AccountMeta::new(vault_input, false),
        AccountMeta::new(vault_output, false),
        AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
    ];

    accounts.extend_from_slice(&input_hook_accounts);
    accounts.extend_from_slice(&output_hook_accounts);

    Ok(accounts)
}

/// Build Fraudsworth Conversion Vault swap extra data: [pre_balance].
pub fn build_extra_data(pre_balance: u64) -> Vec<u8> {
    pre_balance.to_le_bytes().to_vec()
}

/// Resolve accounts and data for a Fraudsworth Conversion Vault swap via RPC.
#[cfg(feature = "resolve")]
pub async fn resolve(
    _rpc: &RpcClient,
    mint_in: &Address,
    mint_out: &Address,
    user: &Address,
    pre_balance: u64,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    if mint_in == mint_out {
        return Err(ClientError::InvalidAccountData(
            "Conversion mint_in and mint_out must be different".to_string(),
        ));
    }

    let user_input_account = get_associated_token_address(user, mint_in, &TOKEN_2022_PROGRAM_ID);
    let user_output_account = get_associated_token_address(user, mint_out, &TOKEN_2022_PROGRAM_ID);

    let input = FraudsworthConversionVaultSwapInput {
        user: *user,
        input_mint: *mint_in,
        output_mint: *mint_out,
        user_input_account,
        user_output_account,
    };

    Ok((build_accounts(&input)?, build_extra_data(pre_balance)))
}
