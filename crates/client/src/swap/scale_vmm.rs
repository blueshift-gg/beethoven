use {crate::SYSTEM_PROGRAM_ID, solana_address::Address, solana_instruction::AccountMeta};
#[cfg(feature = "resolve")]
use {
    crate::{
        discover_pool_with_flip, get_associated_token_address, get_token_program_for_mint,
        read_pubkey, read_u8, ClientError,
    },
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

pub const SCALE_VMM_PROGRAM_ID: Address =
    Address::from_str_const("SCALEWoRSpVZpMRqHEcDfNvBh3nUSe34jDr9r689gLa");
pub const SCALE_AMM_PROGRAM_ID: Address =
    Address::from_str_const("SCALEwAvEK5gtkdHiFzXfPgtk2YwJxPDzaV3aDmR7tA");

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Side {
    Buy = 0,
    Sell = 1,
}

// FeeBeneficiary struct layout offsets
// Layout: [32 wallet] [2 share_bps]
#[cfg(feature = "resolve")]
const FEE_BENEFICIARY_LEN: usize = 34;

// PairState account layout offsets
// Layout: [8 discriminator] [1 enabled] [1 graduated] [32 mint_a] [32 mint_b] [16 token_a_reserves] [16 token_b_reserves] [16 shift] [1 curve_type] [1 fee_beneficiary_count] [170 fee_beneficiaries]...
#[cfg(feature = "resolve")]
const OFFSET_PAIR_MINT_A: usize = 10;
#[cfg(feature = "resolve")]
const OFFSET_PAIR_MINT_B: usize = 42;
#[cfg(feature = "resolve")]
const OFFSET_PAIR_FEE_BENEFICIARY_COUNT: usize = 123;
#[cfg(feature = "resolve")]
const OFFSET_PAIR_FEE_BENEFICIARIES: usize = 124;
#[cfg(feature = "resolve")]
const OFFSET_PAIR_AMM_POOL: usize = 294;
#[cfg(feature = "resolve")]
const MAX_BENEFICIARIES: usize = 5;

// Scale AMM Pool account layout offsets
// Layout: [8 discriminator] [1 enabled] [32 owner] [32 mint_a] [32 mint_b] [16 token_a_reserves] [16 token_b_reserves] [16 shift] [1 curve_type] [1 fee_beneficiary_count] [170 fee_beneficiaries]...
#[cfg(feature = "resolve")]
const OFFSET_AMM_POOL_MINT_A: usize = 41;
#[cfg(feature = "resolve")]
const OFFSET_AMM_POOL_MINT_B: usize = 73;

// PlatformConfig account layout offsets
// Layout: [8 discriminator] [32 authority] [32 fee_beneficiary] ...
#[cfg(feature = "resolve")]
const OFFSET_PLATFORM_CONFIG_FEE_BENEFICIARY: usize = 40;

pub struct ScaleVmmSwapInput {
    pub pair: Address,
    pub user: Address,
    pub mint_a: Address,
    pub mint_b: Address,
    pub user_ta_a: Address,
    pub user_ta_b: Address,
    pub vault_a: Address,
    pub vault_b: Address,
    pub platform_fee_ta_a: Address,
    pub token_program_a: Address,
    pub token_program_b: Address,
    pub config: Address,
    pub amm_program: Address,
    pub amm_pool: Address,
    pub amm_vault_a: Address,
    pub amm_vault_b: Address,
    pub amm_config: Address,
    pub beneficiary_accounts: Vec<Address>,
}

/// Build Scale VMM swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &ScaleVmmSwapInput) -> Vec<AccountMeta> {
    let mut metas = vec![
        AccountMeta::new_readonly(SCALE_VMM_PROGRAM_ID, false),
        AccountMeta::new(input.pair, false),
        AccountMeta::new(input.user, true),
        AccountMeta::new_readonly(input.mint_a, false),
        AccountMeta::new_readonly(input.mint_b, false),
        AccountMeta::new(input.user_ta_a, false),
        AccountMeta::new(input.user_ta_b, false),
        AccountMeta::new(input.vault_a, false),
        AccountMeta::new(input.vault_b, false),
        AccountMeta::new(input.platform_fee_ta_a, false),
        AccountMeta::new_readonly(input.token_program_a, false),
        AccountMeta::new_readonly(input.token_program_b, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new_readonly(input.config, false),
        AccountMeta::new_readonly(input.amm_program, false),
        AccountMeta::new(input.amm_pool, false),
        AccountMeta::new(input.amm_vault_a, false),
        AccountMeta::new(input.amm_vault_b, false),
        AccountMeta::new_readonly(input.amm_config, false),
    ];

    metas.extend(
        input
            .beneficiary_accounts
            .iter()
            .copied()
            .map(|a| AccountMeta::new(a, false)),
    );

    metas
}

/// Scale VMM extra data: [side]
pub fn build_extra_data(side: Side) -> Vec<u8> {
    vec![side as u8]
}

#[cfg(feature = "resolve")]
fn config_pda() -> Address {
    Address::find_program_address(&[b"config"], &SCALE_VMM_PROGRAM_ID).0
}

#[cfg(feature = "resolve")]
fn vmm_vault_pda(pair: &Address, mint: &Address) -> Address {
    Address::find_program_address(&[pair.as_ref(), mint.as_ref()], &SCALE_VMM_PROGRAM_ID).0
}

#[cfg(feature = "resolve")]
fn amm_vault_pda(pool: &Address, mint: &Address) -> Address {
    Address::find_program_address(&[pool.as_ref(), mint.as_ref()], &SCALE_AMM_PROGRAM_ID).0
}

#[cfg(feature = "resolve")]
fn amm_config_pda() -> Address {
    Address::find_program_address(&[b"config"], &SCALE_AMM_PROGRAM_ID).0
}

#[cfg(feature = "resolve")]
fn read_fee_beneficiary_wallets(pair_data: &[u8]) -> Result<Vec<Address>, ClientError> {
    let count = read_u8(pair_data, OFFSET_PAIR_FEE_BENEFICIARY_COUNT)? as usize;
    if count > MAX_BENEFICIARIES {
        return Err(ClientError::InvalidAccountData(format!(
            "Invalid fee_beneficiary_count: {}",
            count
        )));
    }

    let mut wallets = Vec::with_capacity(count);
    for i in 0..count {
        let off = OFFSET_PAIR_FEE_BENEFICIARIES + i * FEE_BENEFICIARY_LEN;
        wallets.push(read_pubkey(pair_data, off)?);
    }
    Ok(wallets)
}

#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    pair: Option<&Address>,
    side: &Side,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let (pair_pubkey, pair_data) = match pair {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = discover_pool_with_flip(
                rpc,
                &SCALE_VMM_PROGRAM_ID,
                OFFSET_PAIR_MINT_A,
                OFFSET_PAIR_MINT_B,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let pair_mint_a = read_pubkey(&pair_data, OFFSET_PAIR_MINT_A)?;
    let pair_mint_b = read_pubkey(&pair_data, OFFSET_PAIR_MINT_B)?;

    let pair_matches = (*mint_a == pair_mint_a && *mint_b == pair_mint_b)
        || (*mint_a == pair_mint_b && *mint_b == pair_mint_a);
    if !pair_matches {
        return Err(ClientError::MintMismatch {
            expected: format!("{}/{}", pair_mint_a, pair_mint_b),
            got: format!("{}/{}", mint_a, mint_b),
        });
    }

    let token_program_a = get_token_program_for_mint(rpc, &pair_mint_a).await?;
    let token_program_b = get_token_program_for_mint(rpc, &pair_mint_b).await?;

    let user_ta_a = get_associated_token_address(user, &pair_mint_a, &token_program_a);
    let user_ta_b = get_associated_token_address(user, &pair_mint_b, &token_program_b);

    let vault_a = vmm_vault_pda(&pair_pubkey, &pair_mint_a);
    let vault_b = vmm_vault_pda(&pair_pubkey, &pair_mint_b);

    let config = config_pda();
    let config_account = rpc.get_account(&config).await?;
    let fee_beneficiary_wallet =
        read_pubkey(&config_account.data, OFFSET_PLATFORM_CONFIG_FEE_BENEFICIARY)?;
    let platform_fee_ta_a =
        get_associated_token_address(&fee_beneficiary_wallet, &pair_mint_a, &token_program_a);

    let amm_pool = read_pubkey(&pair_data, OFFSET_PAIR_AMM_POOL)?;
    let amm_pool_account = rpc.get_account(&amm_pool).await?;
    let amm_pool_mint_a = read_pubkey(&amm_pool_account.data, OFFSET_AMM_POOL_MINT_A)?;
    let amm_pool_mint_b = read_pubkey(&amm_pool_account.data, OFFSET_AMM_POOL_MINT_B)?;
    let amm_pair_matches = (*mint_a == amm_pool_mint_a && *mint_b == amm_pool_mint_b)
        || (*mint_a == amm_pool_mint_b && *mint_b == amm_pool_mint_a);
    if !amm_pair_matches {
        return Err(ClientError::MintMismatch {
            expected: format!("{}/{}", amm_pool_mint_a, amm_pool_mint_b),
            got: format!("{}/{}", mint_a, mint_b),
        });
    }

    let amm_vault_a = amm_vault_pda(&amm_pool, &amm_pool_mint_a);
    let amm_vault_b = amm_vault_pda(&amm_pool, &amm_pool_mint_b);
    let amm_config = amm_config_pda();

    let beneficiary_wallets = read_fee_beneficiary_wallets(&pair_data)?;
    let beneficiary_accounts = beneficiary_wallets
        .into_iter()
        .map(|w| get_associated_token_address(&w, &pair_mint_a, &token_program_a))
        .collect::<Vec<_>>();

    let input = ScaleVmmSwapInput {
        pair: pair_pubkey,
        user: *user,
        mint_a: pair_mint_a,
        mint_b: pair_mint_b,
        user_ta_a,
        user_ta_b,
        vault_a,
        vault_b,
        platform_fee_ta_a,
        token_program_a,
        token_program_b,
        config,
        amm_program: SCALE_AMM_PROGRAM_ID,
        amm_pool,
        amm_vault_a,
        amm_vault_b,
        amm_config,
        beneficiary_accounts,
    };

    Ok((build_accounts(&input), build_extra_data(*side)))
}
