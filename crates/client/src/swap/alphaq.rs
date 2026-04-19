#[cfg(feature = "resolve")]
use {
    crate::{discover_pool_with_flip, get_associated_token_address, read_pubkey, ClientError},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    crate::{SYSVAR_INSTRUCTIONS_ID, TOKEN_PROGRAM_ID},
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const ALPHAQ_PROGRAM_ID: Address = address!("ALPHAQmeA7bjrVuccPsYPiCvsi428SNwte66Srvs4pHA");

// Market account layout offsets
// Layout: ... [32 vault_token_account_a] @ 112 [32 vault_token_account_b] ... [32 mint_a] [32 mint_b] ...
#[cfg(feature = "resolve")]
const OFFSET_VAULT_TOKEN_ACCOUNT_A: usize = 112;
#[cfg(feature = "resolve")]
const OFFSET_VAULT_TOKEN_ACCOUNT_B: usize = 144;
#[cfg(feature = "resolve")]
const OFFSET_MINT_A: usize = 240;
#[cfg(feature = "resolve")]
const OFFSET_MINT_B: usize = 272;

/// Pre-resolved addresses for building an AlphaQ swap instruction offline.
pub struct AlphaqSwapInput {
    pub user: Address,
    pub market: Address,
    pub market_state: Address,
    pub user_ata_a: Address,
    pub user_ata_b: Address,
    pub vault_ta_a: Address,
    pub vault_ta_b: Address,
}

/// Build AlphaQ swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &AlphaqSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(ALPHAQ_PROGRAM_ID, false), // alphaq_program
        AccountMeta::new(input.user, true),                  // user
        AccountMeta::new(input.market, false),               // market
        AccountMeta::new(input.market_state, false),         // market_state
        AccountMeta::new(input.user_ata_a, false),           // user_ata_a
        AccountMeta::new(input.user_ata_b, false),           // user_ata_b
        AccountMeta::new(input.vault_ta_a, false),           // vault_ta_a
        AccountMeta::new(input.vault_ta_b, false),           // vault_ta_b
        AccountMeta::new(input.vault_ta_a, false),           // token_authority_a
        AccountMeta::new(input.vault_ta_b, false),           // token_authority_b
        AccountMeta::new(input.vault_ta_b, false),           // vendor_key
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),  // token_program
        AccountMeta::new_readonly(SYSVAR_INSTRUCTIONS_ID, false), // instructions_sysvar
    ]
}

/// Build AlphaQ extra data: [a_to_b].
pub fn build_extra_data(a_to_b: bool) -> Vec<u8> {
    vec![a_to_b as u8]
}

/// Resolve accounts and data for an AlphaQ swap via RPC.
///
/// `mint_a` is the input mint (what you're selling). Direction is inferred
/// by comparing `mint_a` against the market's token_a mint.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    market: Option<&Address>,
    market_state: &Address,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let (market_pubkey, market_data) = match market {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = discover_pool_with_flip(
                rpc,
                &ALPHAQ_PROGRAM_ID,
                OFFSET_MINT_A,
                OFFSET_MINT_B,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let market_mint_a = read_pubkey(&market_data, OFFSET_MINT_A)?;
    let market_mint_b = read_pubkey(&market_data, OFFSET_MINT_B)?;
    let vault_token_account_a = read_pubkey(&market_data, OFFSET_VAULT_TOKEN_ACCOUNT_A)?;
    let vault_token_account_b = read_pubkey(&market_data, OFFSET_VAULT_TOKEN_ACCOUNT_B)?;

    let a_to_b = if *mint_a == market_mint_a && *mint_b == market_mint_b {
        true
    } else if *mint_a == market_mint_b && *mint_b == market_mint_a {
        false
    } else {
        return Err(ClientError::MintMismatch {
            expected: format!("{} or {}", market_mint_a, market_mint_b),
            got: mint_a.to_string(),
        });
    };

    let user_token_account_a =
        get_associated_token_address(user, &market_mint_a, &TOKEN_PROGRAM_ID);
    let user_token_account_b =
        get_associated_token_address(user, &market_mint_b, &TOKEN_PROGRAM_ID);

    let input = AlphaqSwapInput {
        user: *user,
        market: market_pubkey,
        market_state: *market_state,
        user_ata_a: user_token_account_a,
        user_ata_b: user_token_account_b,
        vault_ta_a: vault_token_account_a,
        vault_ta_b: vault_token_account_b,
    };

    Ok((build_accounts(&input), build_extra_data(a_to_b)))
}
