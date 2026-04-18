#[cfg(feature = "resolve")]
use {
    crate::{
        discover_pool, get_associated_token_address, get_token_program_for_mint, read_pubkey,
        ClientError,
    },
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const METEORA_DYNAMIC_BONDING_CURVE_PROGRAM_ID: Address =
    address!("dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN");
pub const POOL_AUTHORITY: Address = address!("FhVo3mqL8PW5pH5U2CN4XE33DokiyZnUwuGpH2hmHLuM");
pub const EVENT_AUTHORITY: Address = address!("8Ks12pbrD6PXxfty1hVQiE9sc289zgU1zHkvXhrSdriF");

#[derive(Copy, Clone)]
pub enum SwapMode {
    ExactIn,
    PartialFill,
    ExactOut,
}

// VirtualPool account layout offsets
// Layout: [8 discriminator] [64 volatility_tracker] [32 config] [32 creator] [32 base_mint] [32 base_vault] [32 quote_vault] ...
#[cfg(feature = "resolve")]
pub const OFFSET_VIRTUAL_POOL_CONFIG: usize = 72;
#[cfg(feature = "resolve")]
pub const OFFSET_VIRTUAL_POOL_BASE_MINT: usize = 136;
#[cfg(feature = "resolve")]
pub const OFFSET_VIRTUAL_POOL_BASE_VAULT: usize = 168;
#[cfg(feature = "resolve")]
pub const OFFSET_VIRTUAL_POOL_QUOTE_VAULT: usize = 200;

// Config account layout offsets
// Layout: [8 discriminator] [32 quote_mint] ...
#[cfg(feature = "resolve")]
pub const OFFSET_CONFIG_QUOTE_MINT: usize = 8;

/// Pre-resolved addresses for building a Meteora Dynamic Bonding Curve instruction offline.
pub struct MeteoraDynamicBondingCurveSwapInput {
    pub config: Address,
    pub pool: Address,
    pub input_token_account: Address,
    pub output_token_account: Address,
    pub base_vault: Address,
    pub quote_vault: Address,
    pub base_mint: Address,
    pub quote_mint: Address,
    pub payer: Address,
    pub token_base_program: Address,
    pub token_quote_program: Address,
    pub referral_token_account: Option<Address>,
}

/// Build Meteora Dynamic Bonding Curve swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &MeteoraDynamicBondingCurveSwapInput) -> Vec<AccountMeta> {
    let referral_token_account = if let Some(addr) = input.referral_token_account {
        AccountMeta::new(addr, false)
    } else {
        AccountMeta::new_readonly(METEORA_DYNAMIC_BONDING_CURVE_PROGRAM_ID, false)
    };

    vec![
        AccountMeta::new_readonly(METEORA_DYNAMIC_BONDING_CURVE_PROGRAM_ID, false),
        AccountMeta::new_readonly(POOL_AUTHORITY, false),
        AccountMeta::new_readonly(input.config, false),
        AccountMeta::new(input.pool, false),
        AccountMeta::new(input.input_token_account, false),
        AccountMeta::new(input.output_token_account, false),
        AccountMeta::new(input.base_vault, false),
        AccountMeta::new(input.quote_vault, false),
        AccountMeta::new_readonly(input.base_mint, false),
        AccountMeta::new_readonly(input.quote_mint, false),
        AccountMeta::new_readonly(input.payer, true),
        AccountMeta::new_readonly(input.token_base_program, false),
        AccountMeta::new_readonly(input.token_quote_program, false),
        referral_token_account,
        AccountMeta::new_readonly(EVENT_AUTHORITY, false),
        AccountMeta::new_readonly(METEORA_DYNAMIC_BONDING_CURVE_PROGRAM_ID, false),
    ]
}

/// Build Meteora Dynamic Bonding Curve extra data: [swap_mode].
pub fn build_extra_data(swap_mode: SwapMode) -> Vec<u8> {
    vec![swap_mode as u8]
}

/// Resolve accounts and extra data for Meteora Dynamic Bonding Curve swap.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    pool: Option<&Address>,
    referral_token_account: Option<&Address>,
    swap_mode: SwapMode,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let quote_vault = Address::find_program_address(
        &[b"token_vault", mint_b.as_ref(), pool.unwrap().as_ref()],
        &METEORA_DYNAMIC_BONDING_CURVE_PROGRAM_ID,
    )
    .0;

    let (pool_pubkey, pool_data) = match pool {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = discover_pool(
                rpc,
                &METEORA_DYNAMIC_BONDING_CURVE_PROGRAM_ID,
                &[
                    (OFFSET_VIRTUAL_POOL_BASE_MINT, mint_a),
                    (OFFSET_VIRTUAL_POOL_QUOTE_VAULT, &quote_vault),
                ],
            )
            .await?;

            (pubkey, account.data)
        }
    };

    let config_pk = read_pubkey(&pool_data, OFFSET_VIRTUAL_POOL_CONFIG)?;
    let base_mint = read_pubkey(&pool_data, OFFSET_VIRTUAL_POOL_BASE_MINT)?;
    let base_vault = read_pubkey(&pool_data, OFFSET_VIRTUAL_POOL_BASE_VAULT)?;
    let quote_vault = read_pubkey(&pool_data, OFFSET_VIRTUAL_POOL_QUOTE_VAULT)?;

    if *mint_a != base_mint {
        return Err(ClientError::MintMismatch {
            expected: base_mint.to_string(),
            got: mint_a.to_string(),
        });
    }

    let cfg = rpc.get_account(&config_pk).await?;
    let quote_mint = read_pubkey(&cfg.data, OFFSET_CONFIG_QUOTE_MINT)?;

    if *mint_b != quote_mint {
        return Err(ClientError::MintMismatch {
            expected: quote_mint.to_string(),
            got: mint_b.to_string(),
        });
    }

    let token_base_program = get_token_program_for_mint(rpc, &base_mint).await?;
    let token_quote_program = get_token_program_for_mint(rpc, &quote_mint).await?;

    let input_token_account = get_associated_token_address(user, mint_a, &token_base_program);
    let output_token_account = get_associated_token_address(user, mint_b, &token_quote_program);

    let input = MeteoraDynamicBondingCurveSwapInput {
        config: config_pk,
        pool: pool_pubkey,
        input_token_account,
        output_token_account,
        base_vault,
        quote_vault,
        base_mint,
        quote_mint,
        payer: *user,
        token_base_program,
        token_quote_program,
        referral_token_account: referral_token_account.copied(),
    };

    Ok((build_accounts(&input), build_extra_data(swap_mode)))
}
