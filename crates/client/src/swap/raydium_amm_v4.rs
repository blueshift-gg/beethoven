use {
    crate::{error::ClientError, TOKEN_PROGRAM_ID},
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const RAYDIUM_AMM_V4_PROGRAM_ID: Address =
    address!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");

// AMM pool account layout offsets
// Layout: [8 status] [8 nonce] [8 order_num] [8 depth] [8 coin_decimals] [8 pc_decimals] [8 state] [8 reset_flag] [8 min_size] [8 vol_max_cut_ratio] [8 amount_wave] [8 coin_lot_size] [8 pc_lot_size] [8 min_price_multiplier] [8 max_price_multiplier] [8 sys_decimal_value] [64 fees] [144 state_data] [32 coin_vault] [32 pc_vault] [32 coin_vault_mint] [32 pc_vault_mint]
#[cfg(feature = "resolve")]
const OFFSET_COIN_VAULT: usize = 336;
#[cfg(feature = "resolve")]
const OFFSET_PC_VAULT: usize = OFFSET_COIN_VAULT + 32;
#[cfg(feature = "resolve")]
const OFFSET_COIN_VAULT_MINT: usize = OFFSET_PC_VAULT + 32;
#[cfg(feature = "resolve")]
const OFFSET_PC_VAULT_MINT: usize = OFFSET_COIN_VAULT_MINT + 32;

pub struct RaydiumAmmV4SwapInput {
    pub amm: Address,
    pub amm_authority: Address,
    pub amm_coin_vault: Address,
    pub amm_pc_vault: Address,
    pub user_source_token_account: Address,
    pub user_destination_token_account: Address,
    pub user_wallet: Address,
}

pub fn build_accounts(input: &RaydiumAmmV4SwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(RAYDIUM_AMM_V4_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new(input.amm, false),
        AccountMeta::new_readonly(input.amm_authority, false),
        AccountMeta::new(input.amm_coin_vault, false),
        AccountMeta::new(input.amm_pc_vault, false),
        AccountMeta::new(input.user_source_token_account, false),
        AccountMeta::new(input.user_destination_token_account, false),
        AccountMeta::new(input.user_wallet, true),
    ]
}

pub fn build_extra_data() -> Vec<u8> {
    vec![]
}

#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &solana_rpc_client::nonblocking::rpc_client::RpcClient,
    amm: Option<&Address>,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), crate::error::ClientError> {
    use crate::{get_associated_token_address, get_token_program_for_mint};

    let (amm_pubkey, pool_data) = match amm {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pk, acct) = crate::discover_pool_with_flip(
                rpc,
                &RAYDIUM_AMM_V4_PROGRAM_ID,
                OFFSET_COIN_VAULT_MINT,
                OFFSET_PC_VAULT_MINT,
                mint_a,
                mint_b,
            )
            .await?;
            (pk, acct.data)
        }
    };

    let coin_vault = crate::read_pubkey(&pool_data, OFFSET_COIN_VAULT)?;
    let pc_vault = crate::read_pubkey(&pool_data, OFFSET_PC_VAULT)?;
    let coin_mint = crate::read_pubkey(&pool_data, OFFSET_COIN_VAULT_MINT)?;
    let pc_mint = crate::read_pubkey(&pool_data, OFFSET_PC_VAULT_MINT)?;

    if !(*mint_a == coin_mint || *mint_a == pc_mint) {
        return Err(ClientError::MintMismatch {
            expected: format!("{} or {}", coin_mint, pc_mint),
            got: mint_a.to_string(),
        });
    }
    if !(*mint_b == coin_mint || *mint_b == pc_mint) {
        return Err(ClientError::MintMismatch {
            expected: format!("{} or {}", coin_mint, pc_mint),
            got: mint_b.to_string(),
        });
    }

    let (input_mint, output_mint) = if *mint_a == coin_mint {
        (coin_mint, pc_mint)
    } else {
        (pc_mint, coin_mint)
    };

    let nonce = u64::from_le_bytes(pool_data[8..16].try_into().map_err(|_| {
        ClientError::InvalidAccountData("AMM account too short for nonce".to_string())
    })?);
    let bump = nonce as u8;
    let amm_authority = Address::create_program_address(
        &[b"amm authority".as_ref(), &[bump]],
        &RAYDIUM_AMM_V4_PROGRAM_ID,
    )
    .map_err(|e| ClientError::InvalidAccountData(format!("create_program_address: {:?}", e)))?;

    let input_token_program = get_token_program_for_mint(rpc, &input_mint).await?;
    let output_token_program = get_token_program_for_mint(rpc, &output_mint).await?;

    let user_source_token_account =
        get_associated_token_address(user, &input_mint, &input_token_program);
    let user_destination_token_account =
        get_associated_token_address(user, &output_mint, &output_token_program);

    let input = RaydiumAmmV4SwapInput {
        amm: amm_pubkey,
        amm_authority,
        amm_coin_vault: coin_vault,
        amm_pc_vault: pc_vault,
        user_source_token_account,
        user_destination_token_account,
        user_wallet: *user,
    };

    Ok((build_accounts(&input), build_extra_data()))
}
