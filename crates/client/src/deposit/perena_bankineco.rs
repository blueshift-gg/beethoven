#[cfg(feature = "resolve")]
use {
    crate::{get_associated_token_address, get_token_program_for_mint, read_pubkey, ClientError},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    crate::{ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID},
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const BANKINECO_PROGRAM_ID: Address = address!("save8RQVPMWNTzU18t3GBvBkN9hT7jsGjiCQ28FpD9H");

// BankState account layout offsets
// Layout: [8 discriminator] [1 bump] [1 bank_index] [6 padding] [136 BankConfig] [40 BankStatus] [128 BankAccounting] [112 BankMint] ...

// BankConfig struct layout offsets
// Layout: [32 creator] [32 bank_manager] [32 risk_manager] [40 padding]

// BankStatus struct layout offsets
// Layout: [1 is_halted] [1 is_halted_deposit] [1 is_halted_withdrawal] [5 padding] [32 padding]

// BankAccounting struct layout offsets
// Layout: [8 yielding_tvl] [8 total_issued_supply] [8 max_yielding_tvl] [104 padding]

// BankMint struct layout offsets
// Layout: [32 pubkey] [8 price] [1 decimals] [7 padding] [64 padding]

// VaultGenState account layout offsets
// Layout: [8 discriminator] [1 vault_type] [1 vault_index] [1 yielding_token_index] [1 bump] [4 padding] [464 VaultConfig]

// VaultConfig struct layout offsets
// Layout: [32 creator] [32 bank] [32 team_account] [32 oracle_state] [32 risk_manager]
//         [32 yielding_token_mint] [32 yielding_vault_ata] [1 yielding_mint_decimals] [7 padding]
//         [32 fee_controller] [96 permissioned_users]
//         [2 padding] [2 performance_fee_bps] [2 minting_fee_bps] [2 burning_fee_bps] [2 lp_account_index] [2 lp_third_party_id]
//         [1 lending_platform] [1 lending_type] [2 padding] [88 padding]

// OracleGenState account layout offsets
// Layout: [8 discriminator] [1 bump] [1 veto] [6 padding] [32 bank] [32 vault]
#[cfg(feature = "resolve")]
const OFFSET_BANK_MINT: usize = 320;
#[cfg(feature = "resolve")]
const OFFSET_VAULT_BANK: usize = 48;
#[cfg(feature = "resolve")]
const OFFSET_VAULT_TEAM: usize = 80;
#[cfg(feature = "resolve")]
const OFFSET_VAULT_ORACLE: usize = 112;
#[cfg(feature = "resolve")]
const OFFSET_VAULT_YIELDING_TOKEN_MINT: usize = 176;
#[cfg(feature = "resolve")]
const OFFSET_VAULT_YIELDING_VAULT_ATA: usize = 208;

/// Pre-resolved addresses for building an Bankineco deposit instruction offline.
pub struct PerenaBankinecoSwapInput {
    pub user: Address,
    pub bank_state: Address,
    pub vault_state: Address,
    pub oracle_state: Address,
    pub yielding_mint: Address,
    pub bank_mint: Address,
    pub yielding_user_ta: Address,
    pub bank_mint_user_ta: Address,
    pub yielding_vault_ata: Address,
    pub team_state: Address,
    pub fee_team_ata: Address,
    pub token_program: Address,
    pub yielding_mint_program: Address,
}

/// Build Bankineco deposit AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &PerenaBankinecoSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(BANKINECO_PROGRAM_ID, false),
        AccountMeta::new(input.user, true),
        AccountMeta::new(input.bank_state, false),
        AccountMeta::new(input.vault_state, false),
        AccountMeta::new_readonly(input.oracle_state, false),
        AccountMeta::new_readonly(input.yielding_mint, false),
        AccountMeta::new(input.bank_mint, false),
        AccountMeta::new(input.yielding_user_ta, false),
        AccountMeta::new(input.bank_mint_user_ta, false),
        AccountMeta::new(input.yielding_vault_ata, false),
        AccountMeta::new(input.team_state, false),
        AccountMeta::new(input.fee_team_ata, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new_readonly(input.token_program, false),
        AccountMeta::new_readonly(input.yielding_mint_program, false),
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
    ]
}

/// Build Bankineco extra data: [min_bank_mint_minted].
pub fn build_extra_data(min_bank_mint_minted: u64) -> Vec<u8> {
    min_bank_mint_minted.to_le_bytes().to_vec()
}

/// Resolve accounts from a known vault; checks mint pair and PDAs against on-chain data.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    vault: &Address,
    min_bank_mint_minted: u64,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let vault_data = rpc.get_account(vault).await?.data;

    let bank = read_pubkey(&vault_data, OFFSET_VAULT_BANK)?;
    let bank_data = rpc.get_account(&bank).await?.data;
    let oracle = read_pubkey(&vault_data, OFFSET_VAULT_ORACLE)?;
    let yielding_mint = read_pubkey(&vault_data, OFFSET_VAULT_YIELDING_TOKEN_MINT)?;
    let bank_mint = read_pubkey(&bank_data, OFFSET_BANK_MINT)?;

    let yielding_mint_token_program = get_token_program_for_mint(rpc, &yielding_mint).await?;
    let yielding_user_ta =
        get_associated_token_address(user, &yielding_mint, &yielding_mint_token_program);
    let bank_token_program = get_token_program_for_mint(rpc, &bank_mint).await?;
    let bank_mint_user_ta = get_associated_token_address(user, &bank_mint, &bank_token_program);
    let yielding_vault_ata = read_pubkey(&vault_data, OFFSET_VAULT_YIELDING_VAULT_ATA)?;
    let team = read_pubkey(&vault_data, OFFSET_VAULT_TEAM)?;
    let fee_team_ata =
        get_associated_token_address(&team, &yielding_mint, &yielding_mint_token_program);

    let input = PerenaBankinecoSwapInput {
        user: *user,
        bank_state: bank,
        vault_state: *vault,
        oracle_state: oracle,
        yielding_mint,
        bank_mint,
        yielding_user_ta,
        bank_mint_user_ta,
        yielding_vault_ata,
        team_state: team,
        fee_team_ata,
        token_program: bank_token_program,
        yielding_mint_program: yielding_mint_token_program,
    };

    Ok((
        build_accounts(&input),
        build_extra_data(min_bank_mint_minted),
    ))
}
