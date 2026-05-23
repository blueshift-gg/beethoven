#[cfg(feature = "resolve")]
use {
    crate::{
        discover_pool_with_flip, get_associated_token_address, get_token_program_for_mint,
        read_pubkey, ClientError, TOKEN_PROGRAM_ID,
    },
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    crate::{ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID},
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const PUMP_AMM_PROGRAM_ID: Address = address!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");
pub const FEE_PROGRAM_ID: Address = address!("pfeeUxB6jkeY1Hxd7CsFCAjcbHA9rWtchMGdZ6VojVZ");
pub const BONDING_CURVE_PROGRAM_ID: Address =
    address!("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");
pub const GLOBAL_CONFIG: Address = address!("ADyA8hdefvWN2dbGGWFotbzWxrAvLW83WG6QCVXvJKqw");
pub const EVENT_AUTHORITY: Address = address!("GS4CU59F31iL7aR2Q8zVS8DRrcRnXX1yjQ66TqNVQnaR");
pub const GLOBAL_VOLUME_ACCUMULATOR: Address =
    address!("C2aFPdENg4A2HQsmrd5rTw5TaYBX5Ku887cWjbFKtZpw");
pub const FEE_CONFIG: Address = address!("5PHirr8joyTMp9JMm6nW7hNDVyEYdkzDqazxPD7RaTjx");
const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
pub const FEE_RECIPIENT: Address = address!("EHAAiTxcdDwQ3U4bU6YcMsQGaekdzLS3B5SmYo46kJtL");

// Pool account layout offsets
// Layout: [8 discriminator] [1 bump] [2 index] [32 creator] [32 base_mint] [32 quote_mint] [32 lp_mint] [32 pool_base_token_account] [32 pool_quote_token_account] [8 lp_supply] [32 coin_creator] [1 is_mayhem_mode] [1 is_cashback_coin]
#[cfg(feature = "resolve")]
const OFFSET_BASE_MINT: usize = 43;
#[cfg(feature = "resolve")]
const OFFSET_QUOTE_MINT: usize = 75;
#[cfg(feature = "resolve")]
const OFFSET_POOL_BASE_TOKEN_ACCOUNT: usize = 139;
#[cfg(feature = "resolve")]
const OFFSET_POOL_QUOTE_TOKEN_ACCOUNT: usize = 171;
#[cfg(feature = "resolve")]
const OFFSET_COIN_CREATOR: usize = 211;
#[cfg(feature = "resolve")]
const OFFSET_IS_CASHBACK_COIN: usize = 244;

// Global Config account layout offsets
// Layout: [8 discriminator] [32 admin] [8 lp_fee_basis_points] [8 protocol_fee_basis_points] [1 disable_flags] [8 * 32 protocol_fee_recipients]
#[cfg(feature = "resolve")]
pub const OFFSET_FIRST_PROTOCOL_FEE_RECIPIENT: usize = 57;

pub struct PumpAmmSwapBase {
    pub pool: Address,
    pub user: Address,
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
    pub coin_creator_vault_ata: Address,
    pub coin_creator_vault_authority: Address,
}

pub enum PumpAmmSwapLeg {
    Buy { user_volume_accumulator: Address },
    Sell,
}

pub struct PumpAmmSwapTail {
    pub fee_recipient_quote_mint_ata: Address,
    pub remaining_accounts: Vec<AccountMeta>,
}

/// Pre-resolved addresses for building a Pump AMM swap instruction offline.
pub struct PumpAmmSwapInput {
    pub base: PumpAmmSwapBase,
    pub leg: PumpAmmSwapLeg,
    pub tail: PumpAmmSwapTail,
}

pub fn build_accounts(input: &PumpAmmSwapInput) -> Vec<AccountMeta> {
    let mut meta = vec![
        AccountMeta::new_readonly(PUMP_AMM_PROGRAM_ID, false),
        AccountMeta::new(input.base.pool, false),
        AccountMeta::new(input.base.user, true),
        AccountMeta::new_readonly(GLOBAL_CONFIG, false),
        AccountMeta::new_readonly(input.base.base_mint, false),
        AccountMeta::new_readonly(input.base.quote_mint, false),
        AccountMeta::new(input.base.user_base_token_account, false),
        AccountMeta::new(input.base.user_quote_token_account, false),
        AccountMeta::new(input.base.pool_base_token_account, false),
        AccountMeta::new(input.base.pool_quote_token_account, false),
        AccountMeta::new_readonly(input.base.protocol_fee_recipient, false),
        AccountMeta::new(input.base.protocol_fee_recipient_token_account, false),
        AccountMeta::new_readonly(input.base.base_token_program, false),
        AccountMeta::new_readonly(input.base.quote_token_program, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(EVENT_AUTHORITY, false),
        AccountMeta::new_readonly(PUMP_AMM_PROGRAM_ID, false),
        AccountMeta::new(input.base.coin_creator_vault_ata, false),
        AccountMeta::new_readonly(input.base.coin_creator_vault_authority, false),
    ];

    match input.leg {
        PumpAmmSwapLeg::Buy {
            user_volume_accumulator,
        } => {
            meta.extend([
                AccountMeta::new(GLOBAL_VOLUME_ACCUMULATOR, false),
                AccountMeta::new(user_volume_accumulator, false),
            ]);
        }
        PumpAmmSwapLeg::Sell => {}
    }

    meta.extend(
        [
            AccountMeta::new_readonly(FEE_CONFIG, false),
            AccountMeta::new_readonly(FEE_PROGRAM_ID, false),
        ]
        .into_iter()
        .chain(input.tail.remaining_accounts.iter().cloned()),
    );

    meta.extend([
        AccountMeta::new_readonly(FEE_RECIPIENT, false),
        AccountMeta::new(input.tail.fee_recipient_quote_mint_ata, false),
    ]);

    meta
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
    rpc: &RpcClient,
    pool: Option<&Address>,
    track_volume: Option<bool>,
    is_buy: bool,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let (pool_pk, pool_data) = match pool {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pk, account) = discover_pool_with_flip(
                rpc,
                &PUMP_AMM_PROGRAM_ID,
                OFFSET_BASE_MINT,
                OFFSET_QUOTE_MINT,
                mint_a,
                mint_b,
            )
            .await?;
            (pk, account.data)
        }
    };

    let base_mint = read_pubkey(&pool_data, OFFSET_BASE_MINT)?;
    let quote_mint = read_pubkey(&pool_data, OFFSET_QUOTE_MINT)?;
    let pool_base_token_account = read_pubkey(&pool_data, OFFSET_POOL_BASE_TOKEN_ACCOUNT)?;
    let pool_quote_token_account = read_pubkey(&pool_data, OFFSET_POOL_QUOTE_TOKEN_ACCOUNT)?;
    let coin_creator = read_pubkey(&pool_data, OFFSET_COIN_CREATOR)?;

    // mint_a is base_mint, mint_b is quote_mint
    if *mint_a != base_mint {
        return Err(ClientError::MintMismatch {
            expected: format!("base {}", base_mint),
            got: mint_a.to_string(),
        });
    }
    if *mint_b != quote_mint {
        return Err(ClientError::MintMismatch {
            expected: format!("quote {}", quote_mint),
            got: mint_b.to_string(),
        });
    }

    let base_token_program = get_token_program_for_mint(rpc, &base_mint).await?;
    let quote_token_program = get_token_program_for_mint(rpc, &quote_mint).await?;

    let gc_data = rpc.get_account(&GLOBAL_CONFIG).await?.data;
    let protocol_fee_recipient = read_pubkey(&gc_data, OFFSET_FIRST_PROTOCOL_FEE_RECIPIENT)?;

    let protocol_fee_recipient_token_account =
        get_associated_token_address(&protocol_fee_recipient, &quote_mint, &quote_token_program);

    let user_base_token_account =
        get_associated_token_address(user, &base_mint, &base_token_program);
    let user_quote_token_account =
        get_associated_token_address(user, &quote_mint, &quote_token_program);

    let (coin_creator_vault_authority, _) = Address::find_program_address(
        &[b"creator_vault", coin_creator.as_ref()],
        &PUMP_AMM_PROGRAM_ID,
    );
    let coin_creator_vault_ata = get_associated_token_address(
        &coin_creator_vault_authority,
        &quote_mint,
        &quote_token_program,
    );

    let (user_volume_accumulator, _) = Address::find_program_address(
        &[b"user_volume_accumulator", user.as_ref()],
        &PUMP_AMM_PROGRAM_ID,
    );

    let mut remaining_accounts = vec![];

    let is_cashback = pool_data[OFFSET_IS_CASHBACK_COIN] == 1;

    if is_cashback {
        let cashback_receiver =
            get_associated_token_address(&user_volume_accumulator, &WSOL_MINT, &TOKEN_PROGRAM_ID);

        remaining_accounts.push(AccountMeta::new(cashback_receiver, false));

        if !is_buy {
            remaining_accounts.push(AccountMeta::new(user_volume_accumulator, false));
        }
    }

    // let pool_v2 =
    //     Address::find_program_address(&[b"pool-v2", base_mint.as_ref()], &PUMP_AMM_PROGRAM_ID).0;

    // remaining_accounts.push(AccountMeta::new_readonly(pool_v2, false));

    let fee_recipient_quote_mint_ata =
        get_associated_token_address(&FEE_RECIPIENT, &quote_mint, &quote_token_program);

    let input = PumpAmmSwapInput {
        base: PumpAmmSwapBase {
            pool: pool_pk,
            user: *user,
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
            coin_creator_vault_ata,
            coin_creator_vault_authority,
        },
        leg: if is_buy {
            PumpAmmSwapLeg::Buy {
                user_volume_accumulator,
            }
        } else {
            PumpAmmSwapLeg::Sell
        },
        tail: PumpAmmSwapTail {
            fee_recipient_quote_mint_ata,
            remaining_accounts,
        },
    };

    Ok((build_accounts(&input), build_extra_data(track_volume)))
}
