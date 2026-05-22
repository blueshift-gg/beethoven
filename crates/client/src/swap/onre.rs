#[cfg(feature = "resolve")]
use {
    crate::{
        discover_pool_with_flip, get_associated_token_address, get_token_program_for_mint,
        read_pubkey, ClientError,
    },
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    crate::{
        ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID, SYSVAR_INSTRUCTIONS_ID, TOKEN_PROGRAM_ID,
    },
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const ONRE_PROGRAM_ID: Address = address!("onreuGhHHgVzMWSkj2oQDLDtvvGvoepBPkqyaubFcwe");
pub const STATE: Address = address!("EL5qwcpKyc2FuQxjWmVLEwpcb4LXXwwWWjMYjf1yi3to");
pub const VAULT_AUTHORITY: Address = address!("Ce3R5ZxvW3cnsGS63ikR8KCdA22nkoXW3PnY83yaLJ78");
pub const VAULT_TOKEN_IN_ACCOUNT: Address =
    address!("BMP8pEkMWHoDYiB2N4VyVUm4Fpv6JYNuSFhpMnzanuHi");
pub const VAULT_TOKEN_OUT_ACCOUNT: Address =
    address!("6zqQk9iDWzCx4NUyKNyfNVyxp8e3od8Br7jwkSDeRz8K");
pub const PERMISSIONLESS_AUTHORITY: Address =
    address!("6MvXFNjBDb7arkEHS68Es6MN2giH7SehdHUvYRPFgbyC");
pub const PERMISSIONLESS_TOKEN_IN_ACCOUNT: Address =
    address!("4iEX62oBnfY9foNH1HjnTHzfbzexHP4xY23h5R7jNppU");
pub const PERMISSIONLESS_TOKEN_OUT_ACCOUNT: Address =
    address!("3vaMSBXYcwEjUGtVExcAxLpUuFQMgDSCxghNgTP1uZ7K");
pub const ONRE_MINT: Address = address!("5Y8NV33Vv7WbnLfq3zBcKSdYPrk7g2KoiQoe7M2tcxp5");
pub const MINT_AUTHORITY: Address = address!("AbpE5YLpdpxj2jRczG9P341Jicf67NvZsaZYrATbMnNX");

// Offer account layout offsets
// Layout: [8 discriminator] [32 token_in_mint] [32 token_out_mint] ...
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_IN_MINT: usize = 8;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_OUT_MINT: usize = 40;

// State account layout offsets
// Layout: [8 discriminator] [32 boss] ...
#[cfg(feature = "resolve")]
const OFFSET_BOSS: usize = 8;

/// Pre-resolved addresses for building an Onre take_offer_permissionless instruction offline.
pub struct OnreSwapInput {
    pub offer: Address,
    pub boss: Address,
    pub token_in_mint: Address,
    pub token_in_program: Address,
    pub user_token_in_account: Address,
    pub user_token_out_account: Address,
    pub boss_token_in_account: Address,
    pub user: Address,
}

/// Build Onre take_offer_permissionless AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &OnreSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(ONRE_PROGRAM_ID, false),
        AccountMeta::new(input.offer, false),
        AccountMeta::new_readonly(STATE, false),
        AccountMeta::new_readonly(input.boss, false),
        AccountMeta::new_readonly(VAULT_AUTHORITY, false),
        AccountMeta::new(VAULT_TOKEN_IN_ACCOUNT, false),
        AccountMeta::new(VAULT_TOKEN_OUT_ACCOUNT, false),
        AccountMeta::new_readonly(PERMISSIONLESS_AUTHORITY, false),
        AccountMeta::new(PERMISSIONLESS_TOKEN_IN_ACCOUNT, false),
        AccountMeta::new(PERMISSIONLESS_TOKEN_OUT_ACCOUNT, false),
        AccountMeta::new(input.token_in_mint, false),
        AccountMeta::new_readonly(input.token_in_program, false),
        AccountMeta::new(ONRE_MINT, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new(input.user_token_in_account, false),
        AccountMeta::new(input.user_token_out_account, false),
        AccountMeta::new(input.boss_token_in_account, false),
        AccountMeta::new_readonly(MINT_AUTHORITY, false),
        AccountMeta::new_readonly(SYSVAR_INSTRUCTIONS_ID, false),
        AccountMeta::new(input.user, true),
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
    ]
}

/// Resolve accounts and data for an Onre take_offer_permissionless swap via RPC.
///
/// Requires `offer` address. The offer account layout must be known to derive
/// vault and permissionless authority addresses; fetch fixtures first.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    offer: Option<&Address>,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let (offer_pubkey, offer_data) = match offer {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = discover_pool_with_flip(
                rpc,
                &ONRE_PROGRAM_ID,
                OFFSET_TOKEN_IN_MINT,
                OFFSET_TOKEN_OUT_MINT,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let state_data = rpc.get_account(&STATE).await?.data;
    let state_boss = read_pubkey(&state_data, OFFSET_BOSS)?;
    let offer_token_in_mint = read_pubkey(&offer_data, OFFSET_TOKEN_IN_MINT)?;
    let offer_token_out_mint = read_pubkey(&offer_data, OFFSET_TOKEN_OUT_MINT)?;

    if *mint_a != offer_token_in_mint || *mint_b != offer_token_out_mint {
        return Err(ClientError::MintMismatch {
            expected: format!("{} or {}", offer_token_in_mint, offer_token_out_mint),
            got: format!("({}, {})", mint_a, mint_b),
        });
    }

    let token_in_mint_token_program = get_token_program_for_mint(rpc, &offer_token_in_mint).await?;
    let token_out_mint_token_program =
        get_token_program_for_mint(rpc, &offer_token_out_mint).await?;
    let user_token_in_account =
        get_associated_token_address(user, &offer_token_in_mint, &token_in_mint_token_program);
    let user_token_out_account =
        get_associated_token_address(user, &offer_token_out_mint, &token_out_mint_token_program);
    let boss_token_in_account = get_associated_token_address(
        &state_boss,
        &offer_token_in_mint,
        &token_in_mint_token_program,
    );

    let input = OnreSwapInput {
        offer: offer_pubkey,
        boss: state_boss,
        token_in_mint: offer_token_in_mint,
        token_in_program: token_in_mint_token_program,
        user_token_in_account,
        user_token_out_account,
        boss_token_in_account,
        user: *user,
    };

    Ok((build_accounts(&input), vec![]))
}
