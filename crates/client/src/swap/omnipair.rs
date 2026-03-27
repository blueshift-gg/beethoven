use {solana_address::Address, solana_instruction::AccountMeta};

pub const OMNIPAIR_PROGRAM_ID: Address =
    Address::from_str_const("omnixgS8fnqHfCcTGKWj6JtKjzpJZ1Y5y9pyFkQDkYE");

// Pair account layout offsets (after 8-byte discriminator)
// Layout: [8-byte discriminator] [32 token0] [32 token1] [32 lp_mint] [32 rate_model] ...
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_0: usize = 8;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_1: usize = 40;
#[cfg(feature = "resolve")]
const OFFSET_RATE_MODEL: usize = 104;

/// Pre-resolved addresses for building an Omnipair swap instruction offline.
pub struct OmnipairSwapInput {
    pub pair: Address,
    pub rate_model: Address,
    pub futarchy_authority: Address,
    pub token_in_vault: Address,
    pub token_out_vault: Address,
    pub user_token_in_account: Address,
    pub user_token_out_account: Address,
    pub token_in_mint: Address,
    pub token_out_mint: Address,
    pub user: Address,
    pub event_authority: Address,
}

/// Build Omnipair swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &OmnipairSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(OMNIPAIR_PROGRAM_ID, false),
        AccountMeta::new(input.pair, false),
        AccountMeta::new(input.rate_model, false),
        AccountMeta::new_readonly(input.futarchy_authority, false),
        AccountMeta::new(input.token_in_vault, false),
        AccountMeta::new(input.token_out_vault, false),
        AccountMeta::new(input.user_token_in_account, false),
        AccountMeta::new(input.user_token_out_account, false),
        AccountMeta::new_readonly(input.token_in_mint, false),
        AccountMeta::new_readonly(input.token_out_mint, false),
        AccountMeta::new_readonly(input.user, true),
        AccountMeta::new_readonly(crate::TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(crate::TOKEN_2022_PROGRAM_ID, false),
        AccountMeta::new_readonly(input.event_authority, false),
        AccountMeta::new_readonly(OMNIPAIR_PROGRAM_ID, false),
    ]
}

/// Resolve accounts and data for an Omnipair swap via RPC.
///
/// `mint_a` is the input mint (what you're selling). Direction is inferred
/// by comparing `mint_a` against the pair's token0.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &solana_rpc_client::nonblocking::rpc_client::RpcClient,
    pair: Option<&Address>,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), crate::error::ClientError> {
    let (pair_pubkey, pair_data) = match pair {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = crate::discover_pool_with_flip(
                rpc,
                &OMNIPAIR_PROGRAM_ID,
                OFFSET_TOKEN_0,
                OFFSET_TOKEN_1,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let token_0 = crate::read_pubkey(&pair_data, OFFSET_TOKEN_0)?;
    let token_1 = crate::read_pubkey(&pair_data, OFFSET_TOKEN_1)?;
    let rate_model = crate::read_pubkey(&pair_data, OFFSET_RATE_MODEL)?;

    let (token_in_mint, token_out_mint) = if *mint_a == token_0 {
        (token_0, token_1)
    } else if *mint_a == token_1 {
        (token_1, token_0)
    } else {
        return Err(crate::error::ClientError::MintMismatch {
            expected: format!("{} or {}", token_0, token_1),
            got: mint_a.to_string(),
        });
    };

    // Vaults are PDAs derived from ["reserve_vault", pair, mint]
    let (token_in_vault, _) = Address::find_program_address(
        &[
            b"reserve_vault",
            pair_pubkey.as_ref(),
            token_in_mint.as_ref(),
        ],
        &OMNIPAIR_PROGRAM_ID,
    );
    let (token_out_vault, _) = Address::find_program_address(
        &[
            b"reserve_vault",
            pair_pubkey.as_ref(),
            token_out_mint.as_ref(),
        ],
        &OMNIPAIR_PROGRAM_ID,
    );

    let (futarchy_authority, _) =
        Address::find_program_address(&[b"futarchy_authority"], &OMNIPAIR_PROGRAM_ID);
    let (event_authority, _) =
        Address::find_program_address(&[b"__event_authority"], &OMNIPAIR_PROGRAM_ID);

    let token_in_program = crate::get_token_program_for_mint(rpc, &token_in_mint).await?;
    let token_out_program = crate::get_token_program_for_mint(rpc, &token_out_mint).await?;

    let user_token_in_account =
        crate::get_associated_token_address(user, &token_in_mint, &token_in_program);
    let user_token_out_account =
        crate::get_associated_token_address(user, &token_out_mint, &token_out_program);

    let input = OmnipairSwapInput {
        pair: pair_pubkey,
        rate_model,
        futarchy_authority,
        token_in_vault,
        token_out_vault,
        user_token_in_account,
        user_token_out_account,
        token_in_mint,
        token_out_mint,
        user: *user,
        event_authority,
    };

    Ok((build_accounts(&input), vec![]))
}
