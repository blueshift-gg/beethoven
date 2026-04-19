#[cfg(feature = "resolve")]
use {
    crate::{get_associated_token_address, get_token_program_for_mint, read_pubkey, ClientError},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const BISONFI_PROGRAM_ID: Address = address!("BiSoNHVpsVZW2F7rx2eQ59yQwKxzU5NvBcmKshCSUypi");

// Market account layout
// Layout: ... [32 market_ta_a] @ 120 [32 market_ta_b] [32 mint_a] [32 mint_b] ...
#[cfg(feature = "resolve")]
const OFFSET_MARKET_TA_A: usize = 120;
#[cfg(feature = "resolve")]
const OFFSET_MARKET_TA_B: usize = 152;
#[cfg(feature = "resolve")]
const OFFSET_MINT_A: usize = 184;
#[cfg(feature = "resolve")]
const OFFSET_MINT_B: usize = 216;

/// Pre-resolved addresses for building a BisonFi swap instruction offline.
pub struct BisonfiSwapInput {
    pub user: Address,
    pub market: Address,
    pub market_ta_a: Address,
    pub market_ta_b: Address,
    pub user_ata_a: Address,
    pub user_ata_b: Address,
    pub token_prog_a: Address,
    pub token_prog_b: Address,
    pub logger: Address,
}

/// Build BisonFi swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &BisonfiSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(BISONFI_PROGRAM_ID, false),
        AccountMeta::new(input.user, true),
        AccountMeta::new(input.market, false),
        AccountMeta::new(input.market_ta_a, false),
        AccountMeta::new(input.market_ta_b, false),
        AccountMeta::new(input.user_ata_a, false),
        AccountMeta::new(input.user_ata_b, false),
        AccountMeta::new_readonly(input.token_prog_a, false),
        AccountMeta::new_readonly(input.token_prog_b, false),
        AccountMeta::new_readonly(input.logger, false),
    ]
}

/// Build BisonFi extra data: [b_to_a, exact_out].
pub fn build_extra_data(b_to_a: bool, exact_out: bool) -> Vec<u8> {
    vec![b_to_a as u8, exact_out as u8]
}

/// Resolve accounts and `extra_data` for BisonFi swap via RPC.
///
/// `mint_a` is the input mint (sell). `b_to_a` is `true` when swapping from mint B to mint A.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    market: Option<&Address>,
    exact_out: bool,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
    logger: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let (market_pubkey, market_data) = match market {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = crate::discover_pool_with_flip(
                rpc,
                &BISONFI_PROGRAM_ID,
                OFFSET_MINT_A,
                OFFSET_MINT_B,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let mint_side_a = read_pubkey(&market_data, OFFSET_MINT_A)?;
    let mint_side_b = read_pubkey(&market_data, OFFSET_MINT_B)?;
    let market_ta_a = read_pubkey(&market_data, OFFSET_MARKET_TA_A)?;
    let market_ta_b = read_pubkey(&market_data, OFFSET_MARKET_TA_B)?;

    let b_to_a = if *mint_a == mint_side_a && *mint_b == mint_side_b {
        false
    } else if *mint_a == mint_side_b && *mint_b == mint_side_a {
        true
    } else {
        return Err(ClientError::MintMismatch {
            expected: format!("{} and {}", mint_side_a, mint_side_b),
            got: format!("{} -> {}", mint_a, mint_b),
        });
    };

    let token_prog_a = get_token_program_for_mint(rpc, &mint_side_a).await?;
    let token_prog_b = get_token_program_for_mint(rpc, &mint_side_b).await?;

    let user_ata_a = get_associated_token_address(user, &mint_side_a, &token_prog_a);
    let user_ata_b = get_associated_token_address(user, &mint_side_b, &token_prog_b);

    let input = BisonfiSwapInput {
        user: *user,
        market: market_pubkey,
        market_ta_a,
        market_ta_b,
        user_ata_a,
        user_ata_b,
        token_prog_a,
        token_prog_b,
        logger: *logger,
    };

    Ok((build_accounts(&input), build_extra_data(b_to_a, exact_out)))
}
