#[cfg(feature = "resolve")]
use {
    crate::{get_associated_token_address, get_token_program_for_mint, ClientError},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    crate::{ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID},
    solana_address::Address,
    solana_instruction::AccountMeta,
};

pub const HYLO_PROGRAM_ID: Address =
    Address::from_str_const("HYEXCHtHkBagdStcJCp3xbbb9B7sdMdWXFNj6mdsG4hn");

pub const HYUSD_MINT: Address =
    Address::from_str_const("5YMkXAYccHSGnHn9nob9xEvv6Pvka9DZWH7nTbotTu9E");

pub const XSOL_MINT: Address =
    Address::from_str_const("4sWNB8zGWHkh6UnmwiEtzNxL4XrN7uK9tosbESbJFfVs");

pub const SOL_USD_PYTH_FEED: Address =
    Address::from_str_const("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");

// PDA seeds
const SEED_HYLO: &[u8] = b"hylo";
const SEED_FEE_AUTH: &[u8] = b"fee_auth";
const SEED_VAULT_AUTH: &[u8] = b"vault_auth";
const SEED_MINT_AUTH: &[u8] = b"mint_auth";
const SEED_LST_HEADER: &[u8] = b"lst_header";
const SEED_EVENT_AUTH: &[u8] = b"__event_authority";

fn derive_pda(seeds: &[&[u8]]) -> Address {
    let (addr, _) = Address::find_program_address(seeds, &HYLO_PROGRAM_ID);
    addr
}

/// Pre-resolved addresses for building a Hylo deposit instruction offline.
pub struct HyloDepositInput {
    pub user: Address,
    pub hylo_state: Address,
    pub fee_auth: Address,
    pub vault_auth: Address,
    pub coin_auth: Address,
    pub fee_vault: Address,
    pub lst_vault: Address,
    pub lst_header: Address,
    pub user_lst_ta: Address,
    pub user_output_ta: Address,
    pub lst_mint: Address,
    pub output_mint: Address,
    pub sol_usd_pyth_feed: Address,
    pub token_program: Address,
    pub event_authority: Address,
}

/// Build account metas for a Hylo deposit instruction.
/// mint_type: 0 = stablecoin (hyUSD), 1 = levercoin (xSOL)
pub fn build_accounts(input: &HyloDepositInput, mint_type: u8) -> Vec<AccountMeta> {
    let mut accounts = vec![
        AccountMeta::new_readonly(HYLO_PROGRAM_ID, false), // detection prefix
        AccountMeta::new(input.user, true),                // user (signer)
        AccountMeta::new(input.hylo_state, false),         // hylo state
        AccountMeta::new_readonly(input.fee_auth, false),  // fee_auth
        AccountMeta::new_readonly(input.vault_auth, false), // vault_auth
        AccountMeta::new_readonly(input.coin_auth, false), // stablecoin_auth or levercoin_auth
        AccountMeta::new(input.fee_vault, false),          // fee_vault
        AccountMeta::new(input.lst_vault, false),          // lst_vault
        AccountMeta::new_readonly(input.lst_header, false), // lst_header
        AccountMeta::new(input.user_lst_ta, false),        // user_lst_ta
        AccountMeta::new(input.user_output_ta, false),     // user_output_ta
        AccountMeta::new_readonly(input.lst_mint, false),  // lst_mint
        AccountMeta::new(input.output_mint, false),        // output_mint (hyUSD or xSOL)
    ];

    // levercoin has an extra stablecoin_mint account at this position
    if mint_type == 1 {
        accounts.push(AccountMeta::new_readonly(HYUSD_MINT, false));
    }

    accounts.extend_from_slice(&[
        AccountMeta::new_readonly(input.sol_usd_pyth_feed, false),
        AccountMeta::new_readonly(input.token_program, false),
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new_readonly(input.event_authority, false),
        AccountMeta::new_readonly(HYLO_PROGRAM_ID, false), // program itself (Anchor CPI)
    ]);

    accounts
}

/// Build Hylo extra data: [mint_type, expected_token_out, slippage_tolerance].
pub fn build_extra_data(
    mint_type: u8,
    expected_token_out: u64,
    slippage_tolerance: u64,
) -> Vec<u8> {
    let mut data = Vec::with_capacity(17);
    data.push(mint_type);
    data.extend_from_slice(&expected_token_out.to_le_bytes());
    data.extend_from_slice(&slippage_tolerance.to_le_bytes());
    data
}

/// Resolve accounts for a Hylo deposit. All accounts are deterministic
/// PDA/ATA derivations; the only RPC call checks the LST's token program.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    lst_mint: &Address,
    mint_type: u8,
    expected_token_out: u64,
    slippage_tolerance: u64,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let output_mint = if mint_type == 0 {
        HYUSD_MINT
    } else {
        XSOL_MINT
    };

    // Derive PDAs
    let hylo_state = derive_pda(&[SEED_HYLO]);
    let fee_auth = derive_pda(&[SEED_FEE_AUTH, lst_mint.as_ref()]);
    let vault_auth = derive_pda(&[SEED_VAULT_AUTH, lst_mint.as_ref()]);
    let coin_auth = derive_pda(&[SEED_MINT_AUTH, output_mint.as_ref()]);
    let lst_header = derive_pda(&[SEED_LST_HEADER, lst_mint.as_ref()]);
    let event_authority = derive_pda(&[SEED_EVENT_AUTH]);

    // Derive ATAs
    let lst_token_program = get_token_program_for_mint(rpc, lst_mint).await?;
    let fee_vault = get_associated_token_address(&fee_auth, lst_mint, &lst_token_program);
    let lst_vault = get_associated_token_address(&vault_auth, lst_mint, &lst_token_program);
    let user_lst_ta = get_associated_token_address(user, lst_mint, &lst_token_program);
    // hyUSD and xSOL use standard SPL Token
    let user_output_ta = get_associated_token_address(user, &output_mint, &TOKEN_PROGRAM_ID);

    let input = HyloDepositInput {
        user: *user,
        hylo_state,
        fee_auth,
        vault_auth,
        coin_auth,
        fee_vault,
        lst_vault,
        lst_header,
        user_lst_ta,
        user_output_ta,
        lst_mint: *lst_mint,
        output_mint,
        sol_usd_pyth_feed: SOL_USD_PYTH_FEED,
        token_program: lst_token_program,
        event_authority,
    };

    Ok((
        build_accounts(&input, mint_type),
        build_extra_data(mint_type, expected_token_out, slippage_tolerance),
    ))
}
