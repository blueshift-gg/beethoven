#[cfg(feature = "resolve")]
use {
    crate::{get_associated_token_address, ClientError, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    crate::{SYSVAR_CLOCK_ID, SYSVAR_INSTRUCTIONS_ID},
    solana_address::Address,
    solana_instruction::AccountMeta,
};

pub const HADRON_PROGRAM_ID: Address =
    Address::from_str_const("Q72w4coozA552keKDdeeh2EyQw32qfMFsHPu6cbatom");
pub const FEE_CONFIG_PDA: Address =
    Address::from_str_const("FoDswiGbeRMEm2dEpLfvJTZaQjjZXFcJzCYyhzxiWcRd");

// Config account layout offsets (no discriminator)
// Layout: [1 state] [8 seed] [32 authority] [32 mint_x] [32 mint_y] [1 bump] [32 curve_meta] [1 spread_config_initialized] [1 delta_staleness] [1 oracle_mode] [1 has_pool_fee] [2 padding] [32 pending_authority] [8 nomination_expiry] [32 token_program_x] [32 token_program_y] ...
#[cfg(feature = "resolve")]
const OFFSET_SEED: usize = 1;
#[cfg(feature = "resolve")]
const OFFSET_MINT_X: usize = 41;
#[cfg(feature = "resolve")]
const OFFSET_MINT_Y: usize = 73;
#[cfg(feature = "resolve")]
const OFFSET_CURVE_META: usize = 106;
#[cfg(feature = "resolve")]
const OFFSET_SPREAD_CONFIG_INITIALIZED: usize = 138;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_PROGRAM_X: usize = 184;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_PROGRAM_Y: usize = 216;

/// Pre-resolved addresses for building an Hadron swap instruction offline.
pub struct HadronSwapInput {
    pub token_program_x: Address,
    pub token_program_y: Address,
    pub config: Address,
    pub midprice_oracle: Address,
    pub curve_meta: Address,
    pub curve_prefabs: Address,
    pub user: Address,
    pub user_source: Address,
    pub vault_source: Address,
    pub vault_dest: Address,
    pub user_dest: Address,
    pub fee_recipient_ata: Address,
    pub curve_updates: Address,
    pub spread_config: Option<Address>,
    pub sysvar_instructions: Option<Address>,
}

/// Build Hadron swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &HadronSwapInput) -> Vec<AccountMeta> {
    let mut accounts = vec![
        AccountMeta::new_readonly(HADRON_PROGRAM_ID, false),
        AccountMeta::new_readonly(input.token_program_x, false),
        AccountMeta::new_readonly(input.token_program_y, false),
        AccountMeta::new_readonly(input.config, false),
        AccountMeta::new_readonly(input.midprice_oracle, false),
        AccountMeta::new_readonly(input.curve_meta, false),
        AccountMeta::new(input.curve_prefabs, false),
        AccountMeta::new_readonly(input.user, true),
        AccountMeta::new(input.user_source, false),
        AccountMeta::new(input.vault_source, false),
        AccountMeta::new(input.vault_dest, false),
        AccountMeta::new(input.user_dest, false),
        AccountMeta::new_readonly(FEE_CONFIG_PDA, false),
        AccountMeta::new(input.fee_recipient_ata, false),
        AccountMeta::new_readonly(SYSVAR_CLOCK_ID, false),
        AccountMeta::new(input.curve_updates, false),
    ];

    if let Some(spread_config) = input.spread_config {
        accounts.push(AccountMeta::new_readonly(spread_config, false));
    }

    if let Some(sysvar_instructions) = input.sysvar_instructions {
        accounts.push(AccountMeta::new_readonly(sysvar_instructions, false));
    }

    accounts
}

/// Build Hadron extra data: [is_x, expiration].
pub fn build_extra_data(is_x: bool, expiration: i64) -> Vec<u8> {
    let mut data = vec![is_x as u8];
    data.extend_from_slice(&expiration.to_le_bytes());
    data
}

/// Resolve accounts and data for an Hadron swap via RPC.
///
/// `mint_x` is the input mint (what you're selling). is_x is inferred
/// by comparing `mint_x` against the config's mint_x.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    config: &Address,
    mint_x: &Address,
    mint_y: &Address,
    user: &Address,
    fee_recipient: &Address,
    expiration: i64,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    use crate::read_pubkey;

    let config_account = rpc.get_account(config).await?;

    let config_mint_x = read_pubkey(&config_account.data, OFFSET_MINT_X)?;
    let config_mint_y = read_pubkey(&config_account.data, OFFSET_MINT_Y)?;

    if mint_x == mint_y {
        return Err(ClientError::InvalidAccountData(
            "Hadron swap input and output mint must differ".to_string(),
        ));
    }
    if *mint_x != config_mint_x && *mint_x != config_mint_y {
        return Err(ClientError::MintMismatch {
            expected: format!("{} or {}", config_mint_x, config_mint_y),
            got: mint_x.to_string(),
        });
    }
    if *mint_y != config_mint_x && *mint_y != config_mint_y {
        return Err(ClientError::MintMismatch {
            expected: format!("{} or {}", config_mint_x, config_mint_y),
            got: mint_y.to_string(),
        });
    }

    let token_program_x = read_pubkey(&config_account.data, OFFSET_TOKEN_PROGRAM_X)?;
    let token_program_y = read_pubkey(&config_account.data, OFFSET_TOKEN_PROGRAM_Y)?;

    let is_spl_token = |p: &Address| *p == TOKEN_PROGRAM_ID || *p == TOKEN_2022_PROGRAM_ID;
    if !is_spl_token(&token_program_x) {
        return Err(ClientError::InvalidAccountData(format!(
            "config token_program_x {} is not Token or Token-2022",
            token_program_x
        )));
    }
    if !is_spl_token(&token_program_y) {
        return Err(ClientError::InvalidAccountData(format!(
            "config token_program_y {} is not Token or Token-2022",
            token_program_y
        )));
    }

    let seed = u64::from_le_bytes(
        config_account.data[OFFSET_SEED..OFFSET_SEED + 8]
            .try_into()
            .unwrap(),
    );

    let (midprice_oracle, _) = Address::find_program_address(
        &[
            b"hadron-midprice",
            seed.to_le_bytes().as_ref(),
            config_mint_x.as_ref(),
            config_mint_y.as_ref(),
        ],
        &HADRON_PROGRAM_ID,
    );

    let curve_meta = read_pubkey(&config_account.data, OFFSET_CURVE_META)?;

    let (curve_prefabs, _) = Address::find_program_address(
        &[
            b"hadron-curve-prefabs",
            seed.to_le_bytes().as_ref(),
            config_mint_x.as_ref(),
            config_mint_y.as_ref(),
        ],
        &HADRON_PROGRAM_ID,
    );

    let vault_x = get_associated_token_address(config, &config_mint_x, &token_program_x);
    let vault_y = get_associated_token_address(config, &config_mint_y, &token_program_y);

    // `mint_x` / `mint_y` here are swap leg mints (mint_a in, mint_b out), aligned with
    // `resolve_swap` — not necessarily config mint X / Y order.
    let is_x = *mint_x == config_mint_x;
    let (token_program_in, token_program_out) = if is_x {
        (token_program_x, token_program_y)
    } else {
        (token_program_y, token_program_x)
    };

    let user_source = get_associated_token_address(user, mint_x, &token_program_in);
    let user_dest = get_associated_token_address(user, mint_y, &token_program_out);
    let (vault_source, vault_dest) = if is_x {
        (vault_x, vault_y)
    } else {
        (vault_y, vault_x)
    };
    let fee_recipient_ata = get_associated_token_address(fee_recipient, mint_x, &token_program_in);

    let (curve_updates, _) = Address::find_program_address(
        &[
            b"hadron-curve-updates",
            seed.to_le_bytes().as_ref(),
            config_mint_x.as_ref(),
            config_mint_y.as_ref(),
        ],
        &HADRON_PROGRAM_ID,
    );

    let spread_config_initialized = config_account.data[OFFSET_SPREAD_CONFIG_INITIALIZED] == 1;

    let (spread_config, sysvar_instructions) = if spread_config_initialized {
        let (spread_config, _) =
            Address::find_program_address(&[b"spread_config", config.as_ref()], &HADRON_PROGRAM_ID);

        (Some(spread_config), Some(SYSVAR_INSTRUCTIONS_ID))
    } else {
        (None, None)
    };

    let input = HadronSwapInput {
        token_program_x,
        token_program_y,
        config: *config,
        midprice_oracle,
        curve_meta,
        curve_prefabs,
        user: *user,
        user_source,
        vault_source,
        vault_dest,
        user_dest,
        fee_recipient_ata,
        curve_updates,
        spread_config,
        sysvar_instructions,
    };

    Ok((build_accounts(&input), build_extra_data(is_x, expiration)))
}
