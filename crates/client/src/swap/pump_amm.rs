use {solana_address::Address, solana_instruction::AccountMeta};

pub const PUMP_AMM_PROGRAM_ID: Address =
    Address::from_str_const("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");
pub const FEE_PROGRAM_ID: Address =
    Address::from_str_const("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");
pub const BONDING_CURVE_PROGRAM_ID: Address =
    Address::from_str_const("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");

// Pool account layout offsets
// Layout: [8-byte discriminator] [1 bump] [2 index] [32 creator] [32 base_mint] [32 quote_mint] [32 lp_mint] [32 pool_base_token_account] [32 pool_quote_token_account] [8 lp_supply] [32 coin_creator]
#[cfg(feature = "resolve")]
const OFFSET_BASE_MINT: usize = 43;
#[cfg(feature = "resolve")]
const OFFSET_QUOTE_MINT: usize = OFFSET_BASE_MINT + 32;
#[cfg(feature = "resolve")]
const OFFSET_POOL_BASE_TOKEN_ACCOUNT: usize = OFFSET_QUOTE_MINT + 32 + 32;
#[cfg(feature = "resolve")]
const OFFSET_POOL_QUOTE_TOKEN_ACCOUNT: usize = OFFSET_POOL_BASE_TOKEN_ACCOUNT + 32;
#[cfg(feature = "resolve")]
const OFFSET_COIN_CREATOR: usize = OFFSET_POOL_QUOTE_TOKEN_ACCOUNT + 32 + 8;

// Global Config account layout offsets
// Layout: [8-byte discriminator] [32 admin] [8 lp_fee_basis_points] [8 protocol_fee_basis_points] [1 disable_flags] [4 + 8 * 32 protocol_fee_recipients]
#[cfg(feature = "resolve")]
pub const OFFSET_FIRST_PROTOCOL_FEE_RECIPIENT: usize = 61;

/// Pre-resolved addresses for building a Pump AMM swap instruction offline.
pub struct PumpAmmSwapInput {
    pub pool: Address,
    pub user: Address,
    pub global_config: Address,
    pub base_mint: Address,
    pub quote_mint: Address,
    pub user_base_token_account: Address,
    pub user_quote_token_account: Address,
    pub pool_base_token_account: Address,
    pub pool_quote_token_account: Address,
    pub protocol_fee_recipient: Address,
    pub protocol_fee_recipient_token_account: Address,
    pub base_token_program: Address,
    pub quote_token_program: Address,
    pub event_authority: Address,
    pub coin_creator_vault_ata: Address,
    pub coin_creator_vault_authority: Address,
    pub global_volume_accumulator: Address,
    pub user_volume_accumulator: Address,
    pub fee_config: Address,
}

pub fn build_accounts(input: &PumpAmmSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(PUMP_AMM_PROGRAM_ID, false),
        AccountMeta::new(input.pool, false),
        AccountMeta::new(input.user, true),
        AccountMeta::new_readonly(input.global_config, false),
        AccountMeta::new_readonly(input.base_mint, false),
        AccountMeta::new_readonly(input.quote_mint, false),
        AccountMeta::new(input.user_base_token_account, false),
        AccountMeta::new(input.user_quote_token_account, false),
        AccountMeta::new(input.pool_base_token_account, false),
        AccountMeta::new(input.pool_quote_token_account, false),
        AccountMeta::new_readonly(input.protocol_fee_recipient, false),
        AccountMeta::new(input.protocol_fee_recipient_token_account, false),
        AccountMeta::new_readonly(input.base_token_program, false),
        AccountMeta::new_readonly(input.quote_token_program, false),
        AccountMeta::new_readonly(crate::SYSTEM_PROGRAM_ID, false),
        AccountMeta::new_readonly(crate::ASSOCIATED_TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(input.event_authority, false),
        AccountMeta::new_readonly(PUMP_AMM_PROGRAM_ID, false),
        AccountMeta::new(input.coin_creator_vault_ata, false),
        AccountMeta::new_readonly(input.coin_creator_vault_authority, false),
        AccountMeta::new_readonly(input.global_volume_accumulator, false),
        AccountMeta::new(input.user_volume_accumulator, false),
        AccountMeta::new_readonly(input.fee_config, false),
        AccountMeta::new_readonly(FEE_PROGRAM_ID, false),
    ]
}

/// Build Pump AMM extra data: `[Option<bool>]`.
pub fn build_extra_data(track_volume: Option<bool>) -> Vec<u8> {
    let mut data = Vec::with_capacity(2);
    match track_volume {
        Some(track_volume) => {
            data.push(1);
            data.push(if track_volume { 1 } else { 0 });
        }
        None => {
            data.push(0);
            data.push(0);
        }
    }
    data
}

#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &solana_rpc_client::nonblocking::rpc_client::RpcClient,
    pool: Option<&Address>,
    track_volume: Option<bool>,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), crate::error::ClientError> {
    let (pool_pk, pool_data) = match pool {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pk, account) = crate::discover_pool_with_flip(
                rpc,
                &PUMP_AMM_PROGRAM_ID,
                OFFSET_BASE_MINT,
                OFFSET_QUOTE_MINT,
                // the usual order is inverted for Pump AMM: mint_a is quote, mint_b is base
                mint_b,
                mint_a,
            )
            .await?;
            (pk, account.data)
        }
    };

    let base_mint = crate::read_pubkey(&pool_data, OFFSET_BASE_MINT)?;
    let quote_mint = crate::read_pubkey(&pool_data, OFFSET_QUOTE_MINT)?;
    let pool_base_token_account = crate::read_pubkey(&pool_data, OFFSET_POOL_BASE_TOKEN_ACCOUNT)?;
    let pool_quote_token_account = crate::read_pubkey(&pool_data, OFFSET_POOL_QUOTE_TOKEN_ACCOUNT)?;
    let coin_creator = crate::read_pubkey(&pool_data, OFFSET_COIN_CREATOR)?;

    if (*mint_a != quote_mint || *mint_b != base_mint)
        && (*mint_a != base_mint || *mint_b != quote_mint)
    {
        return Err(crate::error::ClientError::MintMismatch {
            expected: format!(
                "quote {} and base {} (buy direction: mint_a=quote, mint_b=base)",
                quote_mint, base_mint
            ),
            got: format!("{} / {}", mint_a, mint_b),
        });
    }

    // `buy`: spend quote (`mint_a`) to receive base (`mint_b`).
    if *mint_a != quote_mint {
        return Err(crate::error::ClientError::InvalidAccountData(
            "Pump AMM `buy` expects mint_a == pool quote mint (input)".into(),
        ));
    }
    if *mint_b != base_mint {
        return Err(crate::error::ClientError::InvalidAccountData(
            "Pump AMM `buy` expects mint_b == pool base mint (output)".into(),
        ));
    }

    let base_token_program = crate::get_token_program_for_mint(rpc, &base_mint).await?;
    let quote_token_program = crate::get_token_program_for_mint(rpc, &quote_mint).await?;

    let (global_config, _) =
        Address::find_program_address(&[b"global_config"], &PUMP_AMM_PROGRAM_ID);
    let gc_data = rpc.get_account(&global_config).await?.data;
    let protocol_fee_recipient = crate::read_pubkey(&gc_data, OFFSET_FIRST_PROTOCOL_FEE_RECIPIENT)?;

    let protocol_fee_recipient_token_account = crate::get_associated_token_address(
        &protocol_fee_recipient,
        &quote_mint,
        &quote_token_program,
    );

    let user_base_token_account =
        crate::get_associated_token_address(user, &base_mint, &base_token_program);
    let user_quote_token_account =
        crate::get_associated_token_address(user, &quote_mint, &quote_token_program);

    let (coin_creator_vault_authority, _) = Address::find_program_address(
        &[b"creator_vault", coin_creator.as_ref()],
        &PUMP_AMM_PROGRAM_ID,
    );
    let coin_creator_vault_ata = crate::get_associated_token_address(
        &coin_creator_vault_authority,
        &quote_mint,
        &quote_token_program,
    );

    let (global_volume_accumulator, _) =
        Address::find_program_address(&[b"global_volume_accumulator"], &PUMP_AMM_PROGRAM_ID);
    let (user_volume_accumulator, _) = Address::find_program_address(
        &[b"user_volume_accumulator", user.as_ref()],
        &PUMP_AMM_PROGRAM_ID,
    );

    let (event_authority, _) =
        Address::find_program_address(&[b"__event_authority"], &PUMP_AMM_PROGRAM_ID);
    let (fee_config, _) = Address::find_program_address(
        &[b"fee_config", BONDING_CURVE_PROGRAM_ID.as_ref()],
        &FEE_PROGRAM_ID,
    );

    let input = PumpAmmSwapInput {
        pool: pool_pk,
        user: *user,
        global_config,
        base_mint,
        quote_mint,
        user_base_token_account,
        user_quote_token_account,
        pool_base_token_account,
        pool_quote_token_account,
        protocol_fee_recipient,
        protocol_fee_recipient_token_account,
        base_token_program,
        quote_token_program,
        event_authority,
        coin_creator_vault_ata,
        coin_creator_vault_authority,
        global_volume_accumulator,
        user_volume_accumulator,
        fee_config,
    };

    Ok((build_accounts(&input), build_extra_data(track_volume)))
}
