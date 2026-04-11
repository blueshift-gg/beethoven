use {
    crate::{error::ClientError, TOKEN_PROGRAM_ID},
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const RISE_PROGRAM_ID: Address = address!("RiseZSHaLdj7pfn1tisUoSdG2i3QcVz9sQKuaRG9rar");
pub const MAYFLOWER_PROGRAM_ID: Address = address!("AVMmmRzwc2kETQNhPiFVnyu62HrgsQXTD6D7SnSfEz7v");
pub const MAY_LOG_ACCOUNT: Address = address!("EKVkmuwDKRKHw85NPTbKSKuS75EY4NLcxe1qzSPixLdy");

// Market account layout offsets
// Layout: [8 discriminator] [32 tenant] [32 market_data] [32 mint_token] [32 mint_main] ...
#[cfg(feature = "resolve")]
const OFFSET_MARKET_TENANT: usize = 8;
#[cfg(feature = "resolve")]
const OFFSET_MARKET_MARKET_META: usize = 40;
#[cfg(feature = "resolve")]
const OFFSET_MARKET_MINT_TOKEN: usize = 72;
#[cfg(feature = "resolve")]
const OFFSET_MARKET_MINT_MAIN: usize = 104;

// Tenant account layout offsets
// Layout: [8 discriminator] [32 admin] [4 tally_cooldown_seconds] [8 last_tally_timestamp] [32 seed] ...
#[cfg(feature = "resolve")]
const OFFSET_TENANT_SEED: usize = 52;

pub enum RiseInstruction {
    BuyWithExactCashIn {
        new_shoulder_end: u64,
        floor_increase_ratio: [u8; 16],
        max_new_floor: [u8; 16],
        max_area_shrinkage_tolerance_units: u64,
        min_liq_ratio: [u8; 16],
    },
    SellWithExactTokenIn,
}

pub struct RiseBaseAccounts {
    signer: Address,
    tenant: Address,
    market: Address,
    cash_escrow: Address,
    may_tenant: Address,
    may_market_group: Address,
    market_meta: Address,
    may_market: Address,
}

pub struct RiseLegAccounts {
    mint_token: Address,
    mint_main: Address,
    mint_token_token_account: Address,
    mint_main_token_account: Address,
    liq_vault_main: Address,
    rev_escrow_group: Address,
    rev_escrow_tenant: Address,
    token_program_main: Address,
    creator_escrow: Address,
    team_escrow: Address,
}

/// Pre-resolved addresses for building an Rise swap instruction offline.
pub struct RiseSwapInput {
    base: RiseBaseAccounts,
    tenant_seed: Option<Address>,
    leg: RiseLegAccounts,
}

/// Build Rise swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &RiseSwapInput) -> Vec<AccountMeta> {
    let mut accounts = vec![
        AccountMeta::new_readonly(RISE_PROGRAM_ID, false),
        AccountMeta::new(input.base.signer, true),
        AccountMeta::new(input.base.tenant, false),
        AccountMeta::new(input.base.market, false),
        AccountMeta::new(input.base.cash_escrow, false),
        AccountMeta::new_readonly(input.base.may_tenant, false),
        AccountMeta::new_readonly(input.base.may_market_group, false),
        AccountMeta::new(input.base.market_meta, false),
        AccountMeta::new(input.base.may_market, false),
        AccountMeta::new(input.leg.mint_token, false),
        AccountMeta::new_readonly(input.leg.mint_main, false),
        AccountMeta::new(input.leg.mint_token_token_account, false),
        AccountMeta::new(input.leg.mint_main_token_account, false),
        AccountMeta::new(input.leg.liq_vault_main, false),
        AccountMeta::new(input.leg.rev_escrow_group, false),
        AccountMeta::new(input.leg.rev_escrow_tenant, false),
        AccountMeta::new_readonly(input.leg.token_program_main, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(MAYFLOWER_PROGRAM_ID, false),
        AccountMeta::new(MAY_LOG_ACCOUNT, false),
        AccountMeta::new(input.leg.creator_escrow, false),
        AccountMeta::new(input.leg.team_escrow, false),
    ];

    // insert after may_market
    if let Some(tenant_seed) = input.tenant_seed {
        accounts.insert(9, AccountMeta::new_readonly(tenant_seed, false));
    }

    accounts
}

/// Build Rise swap extra data: [new_shoulder_end, floor_increase_ratio, max_new_floor, max_area_shrinkage_tolerance_units, min_liq_ratio].
pub fn build_extra_data(
    new_shoulder_end: u64,
    floor_increase_ratio: [u8; 16],
    max_new_floor: [u8; 16],
    max_area_shrinkage_tolerance_units: u64,
    min_liq_ratio: [u8; 16],
) -> Vec<u8> {
    let mut out = Vec::with_capacity(64);
    out.extend_from_slice(&new_shoulder_end.to_le_bytes());
    out.extend_from_slice(&floor_increase_ratio);
    out.extend_from_slice(&max_new_floor);
    out.extend_from_slice(&max_area_shrinkage_tolerance_units.to_le_bytes());
    out.extend_from_slice(&min_liq_ratio);
    out
}

/// Resolve accounts and data for an Rise swap via RPC.
#[cfg(feature = "resolve")]
#[allow(clippy::too_many_arguments)]
pub async fn resolve(
    rpc: &solana_rpc_client::nonblocking::rpc_client::RpcClient,
    market: Option<&Address>,
    new_shoulder_end: Option<u64>,
    floor_increase_ratio: Option<[u8; 16]>,
    max_new_floor: Option<[u8; 16]>,
    max_area_shrinkage_tolerance_units: Option<u64>,
    min_liq_ratio: Option<[u8; 16]>,
    mint_token: &Address,
    mint_main: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    use crate::{get_associated_token_address, get_token_program_for_mint, read_pubkey};

    let (market_pubkey, market_data) = match market {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = crate::discover_pool_with_flip(
                rpc,
                &RISE_PROGRAM_ID,
                OFFSET_MARKET_MINT_TOKEN,
                OFFSET_MARKET_MINT_MAIN,
                mint_token,
                mint_main,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    if mint_token != &read_pubkey(&market_data, OFFSET_MARKET_MINT_TOKEN)? {
        return Err(ClientError::MintMismatch {
            expected: mint_token.to_string(),
            got: mint_token.to_string(),
        });
    }
    if mint_main != &read_pubkey(&market_data, OFFSET_MARKET_MINT_MAIN)? {
        return Err(ClientError::MintMismatch {
            expected: mint_main.to_string(),
            got: mint_main.to_string(),
        });
    }

    let tenant = read_pubkey(&market_data, OFFSET_MARKET_TENANT)?;
    let tenant_data = rpc.get_account(&tenant).await?.data;
    let tenant_seed = read_pubkey(&tenant_data, OFFSET_TENANT_SEED)?;
    let cash_escrow =
        Address::find_program_address(&[b"cash_escrow", market_pubkey.as_ref()], &RISE_PROGRAM_ID)
            .0;
    let may_tenant =
        Address::find_program_address(&[b"tenant", tenant_seed.as_ref()], &MAYFLOWER_PROGRAM_ID).0;
    let may_market_group = Address::find_program_address(
        &[b"market_group", tenant_seed.as_ref()],
        &MAYFLOWER_PROGRAM_ID,
    )
    .0;
    let may_market =
        Address::find_program_address(&[b"market", tenant_seed.as_ref()], &MAYFLOWER_PROGRAM_ID).0;
    let market_meta = read_pubkey(&market_data, OFFSET_MARKET_MARKET_META)?;
    let mint_main_token_program = get_token_program_for_mint(rpc, mint_main).await?;
    let token_dst = get_associated_token_address(user, mint_token, &TOKEN_PROGRAM_ID);
    let main_src = get_associated_token_address(user, mint_main, &mint_main_token_program);
    let liq_vault_main = Address::find_program_address(
        &[b"liq_vault_main", market_meta.as_ref()],
        &MAYFLOWER_PROGRAM_ID,
    )
    .0;
    let rev_escrow_group = Address::find_program_address(
        &[b"rev_escrow_group", market_meta.as_ref()],
        &MAYFLOWER_PROGRAM_ID,
    )
    .0;
    let rev_escrow_tenant = Address::find_program_address(
        &[b"rev_escrow_tenant", market_meta.as_ref()],
        &MAYFLOWER_PROGRAM_ID,
    )
    .0;
    let creator_escrow = Address::find_program_address(
        &[b"creator_escrow", market_pubkey.as_ref()],
        &RISE_PROGRAM_ID,
    )
    .0;
    let team_escrow =
        Address::find_program_address(&[b"team_escrow", mint_main.as_ref()], &RISE_PROGRAM_ID).0;

    let input = RiseSwapInput {
        base: RiseBaseAccounts {
            signer: *user,
            tenant,
            market: market_pubkey,
            cash_escrow,
            may_tenant,
            may_market_group,
            market_meta,
            may_market,
        },
        tenant_seed: Some(tenant_seed),
        leg: RiseLegAccounts {
            mint_token: *mint_token,
            mint_main: *mint_main,
            mint_token_token_account: token_dst,
            mint_main_token_account: main_src,
            liq_vault_main,
            rev_escrow_group,
            rev_escrow_tenant,
            token_program_main: mint_main_token_program,
            creator_escrow,
            team_escrow,
        },
    };

    let data = match (
        new_shoulder_end,
        floor_increase_ratio,
        max_new_floor,
        max_area_shrinkage_tolerance_units,
        min_liq_ratio,
    ) {
        (
            Some(new_shoulder_end),
            Some(floor_increase_ratio),
            Some(max_new_floor),
            Some(max_area_shrinkage_tolerance_units),
            Some(min_liq_ratio),
        ) => build_extra_data(
            new_shoulder_end,
            floor_increase_ratio,
            max_new_floor,
            max_area_shrinkage_tolerance_units,
            min_liq_ratio,
        ),
        _ => vec![],
    };

    Ok((build_accounts(&input), data))
}
