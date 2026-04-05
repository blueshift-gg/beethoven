#[cfg(feature = "resolve")]
use {
    crate::{get_associated_token_address, get_token_program_for_mint, read_pubkey},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    crate::{TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID},
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const STABBLE_STABLE_SWAP_PROGRAM_ID: Address =
    address!("swapNyd8XiQwJ6ianp9snpu4brUqFxadzvHebnAXjJZ");
pub const VAULT_STATE: Address = address!("stab1io8dHvK26KoHmTwwHyYmHRbUWbyEJx6CdrGabC");
pub const VAULT_PROGRAM: Address = address!("vo1tWgqZMjG61Z2T9qUaMYKqZ75CYzMuaZ2LZP1n7HV");
pub const FEE_VAULT_AUTHORITY: Address = address!("8UgoPZAR8ZLoEmV6pJ8SZ6JKESP2X8nbnrZSdSgNtg1y");

// Pool account layout offsets
// Layout: [8 discriminator] [32 owner] [32 vault] ...
#[cfg(feature = "resolve")]
const OFFSET_VAULT_STATE: usize = 40;

// https://github.com/stabbleorg/amm-sdk/blob/main/programs/stable-swap/src/pda.rs#L4
pub fn get_withdraw_authority_address(vault_address: &Address) -> Address {
    Address::find_program_address(
        &[b"withdraw_authority", &vault_address.to_bytes()],
        &STABBLE_STABLE_SWAP_PROGRAM_ID,
    )
    .0
}

// https://github.com/stabbleorg/amm-sdk/blob/main/programs/vault/src/pda.rs#L4
pub fn get_vault_authority_address(pool_address: &Address) -> Address {
    Address::find_program_address(
        &[b"vault_authority", &pool_address.to_bytes()],
        &VAULT_PROGRAM,
    )
    .0
}

/// Pre-resolved addresses for building a Stabble Stable swap instruction offline.
pub struct StabbleStableSwapInput {
    pub user: Address,
    pub mint_in: Address,
    pub mint_out: Address,
    pub user_token_in: Address,
    pub user_token_out: Address,
    pub vault_token_in: Address,
    pub vault_token_out: Address,
    pub beneficiary_token_out: Address,
    pub pool: Address,
    pub withdraw_authority: Address,
    pub vault: Address,
    pub vault_authority: Address,
    pub token_program: Address,
    pub token_2022_program: Address,
}

/// Build Stabble Stable swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &StabbleStableSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(STABBLE_STABLE_SWAP_PROGRAM_ID, false),
        AccountMeta::new(input.user, true),
        AccountMeta::new_readonly(input.mint_in, false),
        AccountMeta::new_readonly(input.mint_out, false),
        AccountMeta::new(input.user_token_in, false),
        AccountMeta::new(input.user_token_out, false),
        AccountMeta::new(input.vault_token_in, false),
        AccountMeta::new(input.vault_token_out, false),
        AccountMeta::new(input.beneficiary_token_out, false),
        AccountMeta::new(input.pool, false),
        AccountMeta::new_readonly(input.withdraw_authority, false),
        AccountMeta::new_readonly(VAULT_STATE, false),
        AccountMeta::new_readonly(input.vault_authority, false),
        AccountMeta::new_readonly(VAULT_PROGRAM, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_2022_PROGRAM_ID, false),
    ]
}

/// Stabble Stable swap has no extra data.
pub fn build_extra_data() -> Vec<u8> {
    vec![]
}

/// Resolve accounts and data for a Stabble Stable swap via RPC.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    // pool is not optional because each pool can have up to 4 mints, which complicates byte comparison
    pool: &Address,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), crate::error::ClientError> {
    let pool_data = rpc.get_account(pool).await?.data;

    // sanity check if pool has mint a and mint b

    let mint_a_token_program = get_token_program_for_mint(rpc, mint_a).await?;
    let mint_b_token_program = get_token_program_for_mint(rpc, mint_b).await?;
    let user_token_in = get_associated_token_address(user, mint_a, &mint_a_token_program);
    let user_token_out = get_associated_token_address(user, mint_b, &mint_b_token_program);
    let vault_state = read_pubkey(&pool_data, OFFSET_VAULT_STATE)?;
    let vault_authority = get_vault_authority_address(&vault_state);
    let vault_token_in =
        get_associated_token_address(&vault_authority, mint_a, &mint_a_token_program);
    let vault_token_out =
        get_associated_token_address(&vault_authority, mint_b, &mint_b_token_program);
    let beneficiary_token_out =
        get_associated_token_address(&FEE_VAULT_AUTHORITY, mint_b, &mint_b_token_program);
    let withdraw_authority = get_withdraw_authority_address(pool);
    let token_program = get_token_program_for_mint(rpc, mint_a).await?;
    let token_2022_program = get_token_program_for_mint(rpc, mint_b).await?;

    let input = StabbleStableSwapInput {
        user: *user,
        mint_in: *mint_a,
        mint_out: *mint_b,
        user_token_in,
        user_token_out,
        vault_token_in,
        vault_token_out,
        beneficiary_token_out,
        pool: *pool,
        withdraw_authority,
        vault: vault_state,
        vault_authority,
        token_program,
        token_2022_program,
    };

    Ok((build_accounts(&input), build_extra_data()))
}
