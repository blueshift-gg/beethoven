#[cfg(feature = "resolve")]
use {
    crate::{discover_pool_with_flip, get_associated_token_address, read_pubkey, ClientError},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const SAROS_AMM_PROGRAM_ID: Address = address!("SSwapUtytfBdBn1b9NUGG6foMVPtcWgpRU32HToDUZr");

// Pair account layout offsets
// Layout: [1 version] [1 is_initialized] [1 bump_seed] [32 token_program_id] [32 token_a] [32 token_b] [32 pool_mint] [32 token_a_mint] [32 token_b_mint] [32 pool_fee_account_info] ...
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_PROGRAM_ID: usize = 3;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_A: usize = 35;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_B: usize = 67;
#[cfg(feature = "resolve")]
const OFFSET_POOL_MINT: usize = 99;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_A_MINT: usize = 131;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_B_MINT: usize = 163;
#[cfg(feature = "resolve")]
const OFFSET_POOL_FEE_ACCOUNT_INFO: usize = 195;

pub struct SarosAmmSwapInput {
    pub swap_info: Address,
    pub authority_info: Address,
    pub user_transfer_authority_info: Address,
    pub source_info: Address,
    pub swap_source_info: Address,
    pub swap_destination_info: Address,
    pub destination_info: Address,
    pub pool_mint_info: Address,
    pub pool_fee_account_info: Address,
    pub token_program_info: Address,
}

pub fn build_accounts(input: &SarosAmmSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(SAROS_AMM_PROGRAM_ID, false),
        AccountMeta::new_readonly(input.swap_info, false),
        AccountMeta::new_readonly(input.authority_info, false),
        AccountMeta::new(input.user_transfer_authority_info, true),
        AccountMeta::new(input.source_info, false),
        AccountMeta::new(input.swap_source_info, false),
        AccountMeta::new(input.swap_destination_info, false),
        AccountMeta::new(input.destination_info, false),
        AccountMeta::new(input.pool_mint_info, false),
        AccountMeta::new(input.pool_fee_account_info, false),
        AccountMeta::new_readonly(input.token_program_info, false),
    ]
}

pub fn build_extra_data() -> Vec<u8> {
    vec![]
}

#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    swap_info: Option<&Address>,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let (swap_info_pubkey, swap_info_data) = match swap_info {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = discover_pool_with_flip(
                rpc,
                &SAROS_AMM_PROGRAM_ID,
                OFFSET_TOKEN_A_MINT,
                OFFSET_TOKEN_B_MINT,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let token_program = read_pubkey(&swap_info_data, OFFSET_TOKEN_PROGRAM_ID)?;
    let token_a = read_pubkey(&swap_info_data, OFFSET_TOKEN_A)?;
    let token_b = read_pubkey(&swap_info_data, OFFSET_TOKEN_B)?;
    let pool_mint = read_pubkey(&swap_info_data, OFFSET_POOL_MINT)?;
    let mint_a_pool = read_pubkey(&swap_info_data, OFFSET_TOKEN_A_MINT)?;
    let mint_b_pool = read_pubkey(&swap_info_data, OFFSET_TOKEN_B_MINT)?;
    let pool_fee_account_info = read_pubkey(&swap_info_data, OFFSET_POOL_FEE_ACCOUNT_INFO)?;

    let (swap_source, swap_destination, source_ata, dest_ata) =
        if *mint_a == mint_a_pool && *mint_b == mint_b_pool {
            let s = get_associated_token_address(user, mint_a, &token_program);
            let d = get_associated_token_address(user, mint_b, &token_program);
            (token_a, token_b, s, d)
        } else if *mint_a == mint_b_pool && *mint_b == mint_a_pool {
            let s = get_associated_token_address(user, mint_a, &token_program);
            let d = get_associated_token_address(user, mint_b, &token_program);
            (token_b, token_a, s, d)
        } else {
            return Err(ClientError::MintMismatch {
                expected: format!("{} and {}", mint_a_pool, mint_b_pool),
                got: format!("{} / {}", mint_a, mint_b),
            });
        };

    let (authority, _) =
        Address::find_program_address(&[swap_info_pubkey.as_ref()], &SAROS_AMM_PROGRAM_ID);

    let input = SarosAmmSwapInput {
        swap_info: swap_info_pubkey,
        authority_info: authority,
        user_transfer_authority_info: *user,
        source_info: source_ata,
        swap_source_info: swap_source,
        swap_destination_info: swap_destination,
        destination_info: dest_ata,
        pool_mint_info: pool_mint,
        pool_fee_account_info,
        token_program_info: token_program,
    };

    Ok((build_accounts(&input), build_extra_data()))
}
