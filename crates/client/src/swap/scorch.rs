#[cfg(feature = "resolve")]
use {
    crate::{
        discover_pool_with_flip, get_associated_token_address, get_token_program_for_mint,
        read_pubkey, ClientError,
    },
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    crate::{MEMO_PROGRAM_ID, SYSVAR_INSTRUCTIONS_ID},
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const SCORCH_PROGRAM_ID: Address = address!("SCoRcH8c2dpjvcJD6FiPbCSQyQgu3PcUAWj2Xxx3mqn");
pub const ORACLE_PROGRAM_ID: Address = address!("ojh19ojaKduoJZuaJADhcVGp4xt1TcdAvZmpVsCorch");

// State a account layout offset
// Layout: ... [32 market_ta_a] @ 48 ...
#[cfg(feature = "resolve")]
const OFFSET_STATE_A_MARKET_TA_A: usize = 48;

// State b account layout offset
// Layout: ... [32 market_ta_b] @ 48 ...
#[cfg(feature = "resolve")]
const OFFSET_STATE_B_MARKET_TA_B: usize = 48;

// State c account layout offset
// Layout: ... [32 mint_a] @ 16 ... [32 mint_b] @ 82 ...
#[cfg(feature = "resolve")]
const OFFSET_STATE_C_MINT_A: usize = 16;
#[cfg(feature = "resolve")]
const OFFSET_STATE_C_MINT_B: usize = 82;

/// Pre-resolved addresses for building an Scorch swap instruction offline.
pub struct ScorchSwapInput {
    pub market: Address,
    pub payer: Address,
    pub user_ata_a: Address,
    pub user_ata_b: Address,
    pub market_ta_a: Address,
    pub market_ta_b: Address,
    pub mint_a: Address,
    pub mint_b: Address,
    pub token_program_a: Address,
    pub token_program_b: Address,
    pub acc1: Address,
    pub state_a: Address,
    pub state_b: Address,
    pub state_c: Address,
}

/// Build Scorch swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &ScorchSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(SCORCH_PROGRAM_ID, false),
        AccountMeta::new(input.market, false),
        AccountMeta::new(input.payer, true),
        AccountMeta::new(input.user_ata_a, false),
        AccountMeta::new(input.user_ata_b, false),
        AccountMeta::new(input.market_ta_a, false),
        AccountMeta::new(input.market_ta_b, false),
        AccountMeta::new_readonly(input.mint_a, false),
        AccountMeta::new_readonly(input.mint_b, false),
        AccountMeta::new_readonly(input.token_program_a, false),
        AccountMeta::new_readonly(input.token_program_b, false),
        AccountMeta::new_readonly(MEMO_PROGRAM_ID, false),
        AccountMeta::new_readonly(ORACLE_PROGRAM_ID, false),
        AccountMeta::new_readonly(input.acc1, false),
        AccountMeta::new(input.state_a, false),
        AccountMeta::new(input.state_b, false),
        AccountMeta::new(input.state_c, false),
        AccountMeta::new_readonly(SYSVAR_INSTRUCTIONS_ID, false),
    ]
}

/// Build Scorch swap extra data: [param].
pub fn build_extra_data(param: &[u8; 17]) -> Vec<u8> {
    param.to_vec()
}

/// Resolve Scorch swap accounts via RPC: reads mints from the two market vault token accounts.
#[cfg(feature = "resolve")]
#[allow(clippy::too_many_arguments)]
pub async fn resolve(
    rpc: &RpcClient,
    market: Option<&Address>,
    acc1: Address,
    state_a: Address,
    state_b: Address,
    state_c: Address,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
    param: &[u8; 17],
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let (market_pubkey, _market_data) = match market {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = discover_pool_with_flip(
                rpc,
                &SCORCH_PROGRAM_ID,
                OFFSET_STATE_C_MINT_A,
                OFFSET_STATE_C_MINT_B,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let state_a_data = rpc.get_account(&state_a).await?.data;
    let state_b_data = rpc.get_account(&state_b).await?.data;
    let state_c_data = rpc.get_account(&state_c).await?.data;
    let market_ta_a = read_pubkey(&state_a_data, OFFSET_STATE_A_MARKET_TA_A)?;
    let market_ta_b = read_pubkey(&state_b_data, OFFSET_STATE_B_MARKET_TA_B)?;
    let market_mint_a = read_pubkey(&state_c_data, OFFSET_STATE_C_MINT_A)?;
    let market_mint_b: Address = read_pubkey(&state_c_data, OFFSET_STATE_C_MINT_B)?;

    let token_program_a = get_token_program_for_mint(rpc, &market_mint_a).await?;
    let token_program_b = get_token_program_for_mint(rpc, &market_mint_b).await?;

    let ((mint_in, token_program_in, market_ta_in), (mint_out, token_program_out, market_ta_out)) =
        match (*mint_a, *mint_b) {
            (a, b) if a == market_mint_a && b == market_mint_b => (
                (market_mint_a, token_program_a, market_ta_a),
                (market_mint_b, token_program_b, market_ta_b),
            ),
            (a, b) if a == market_mint_b && b == market_mint_a => (
                (market_mint_b, token_program_b, market_ta_b),
                (market_mint_a, token_program_a, market_ta_a),
            ),
            _ => {
                return Err(ClientError::MintMismatch {
                    expected: format!("({}, {})", market_mint_a, market_mint_b),
                    got: format!("({}, {})", mint_a, mint_b),
                });
            }
        };

    let user_ata_in = get_associated_token_address(user, &mint_in, &token_program_in);
    let user_ata_out = get_associated_token_address(user, &mint_out, &token_program_out);

    let input = ScorchSwapInput {
        market: market_pubkey,
        payer: *user,
        user_ata_a: user_ata_in,
        user_ata_b: user_ata_out,
        market_ta_a: market_ta_in,
        market_ta_b: market_ta_out,
        mint_a: mint_in,
        mint_b: mint_out,
        token_program_a: token_program_in,
        token_program_b: token_program_out,
        acc1,
        state_a,
        state_b,
        state_c,
    };

    Ok((build_accounts(&input), build_extra_data(param)))
}
