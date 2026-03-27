use {solana_address::Address, solana_instruction::AccountMeta};

pub const CP_AMM_PROGRAM_ID: Address =
    Address::from_str_const("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");

pub const POOL_AUTHORITY: Address =
    Address::from_str_const("HLnpSz9h2S4hiLQ43rnSD9XkcUThA7B8hQMKmDaiTLcC");

// Pair account layout offsets (after 8-byte discriminator)
// Layout: [8-byte discriminator] [160 pool_fees] [32 token_a_mint] [32 token_b_mint] [32 token_a_vault] [32 token_b_vault] ...
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_A_MINT: usize = 8 + 160;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_B_MINT: usize = OFFSET_TOKEN_A_MINT + 32;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_A_VAULT: usize = OFFSET_TOKEN_B_MINT + 32;
#[cfg(feature = "resolve")]
const OFFSET_TOKEN_B_VAULT: usize = OFFSET_TOKEN_A_VAULT + 32;

pub struct MeteoraDammV2SwapInput {
    pub pool: Address,
    pub input_token_account: Address,
    pub output_token_account: Address,
    pub token_a_vault: Address,
    pub token_b_vault: Address,
    pub token_a_mint: Address,
    pub token_b_mint: Address,
    pub payer: Address,
    pub token_a_program: Address,
    pub token_b_program: Address,
    pub referral_token_account: Address,
    pub event_authority: Address,
}

pub fn build_accounts(input: &MeteoraDammV2SwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(CP_AMM_PROGRAM_ID, false),
        AccountMeta::new_readonly(POOL_AUTHORITY, false),
        AccountMeta::new(input.pool, false),
        AccountMeta::new(input.input_token_account, false),
        AccountMeta::new(input.output_token_account, false),
        AccountMeta::new(input.token_a_vault, false),
        AccountMeta::new(input.token_b_vault, false),
        AccountMeta::new_readonly(input.token_a_mint, false),
        AccountMeta::new_readonly(input.token_b_mint, false),
        AccountMeta::new_readonly(input.payer, true),
        AccountMeta::new_readonly(input.token_a_program, false),
        AccountMeta::new_readonly(input.token_b_program, false),
        AccountMeta::new_readonly(input.referral_token_account, false),
        AccountMeta::new_readonly(input.event_authority, false),
        AccountMeta::new_readonly(CP_AMM_PROGRAM_ID, false),
    ]
}

pub fn build_extra_data() -> Vec<u8> {
    vec![]
}

#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &solana_rpc_client::nonblocking::rpc_client::RpcClient,
    pool: Option<&Address>,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), crate::error::ClientError> {
    let (pool_pubkey, pool_data) = match pool {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pk, acc) = crate::discover_pool_with_flip(
                rpc,
                &CP_AMM_PROGRAM_ID,
                OFFSET_TOKEN_A_MINT,
                OFFSET_TOKEN_B_MINT,
                mint_a,
                mint_b,
            )
            .await?;
            (pk, acc.data)
        }
    };

    let token_a_mint = crate::read_pubkey(&pool_data, OFFSET_TOKEN_A_MINT)?;
    let token_b_mint = crate::read_pubkey(&pool_data, OFFSET_TOKEN_B_MINT)?;
    let token_a_vault = crate::read_pubkey(&pool_data, OFFSET_TOKEN_A_VAULT)?;
    let token_b_vault = crate::read_pubkey(&pool_data, OFFSET_TOKEN_B_VAULT)?;

    let (_input_mint, output_mint, user_input_ata, user_output_ata) = if *mint_a == token_a_mint {
        let in_prog = crate::get_token_program_for_mint(rpc, &token_a_mint).await?;
        let out_prog = crate::get_token_program_for_mint(rpc, &token_b_mint).await?;
        (
            token_a_mint,
            token_b_mint,
            crate::get_associated_token_address(user, &token_a_mint, &in_prog),
            crate::get_associated_token_address(user, &token_b_mint, &out_prog),
        )
    } else if *mint_a == token_b_mint {
        let in_prog = crate::get_token_program_for_mint(rpc, &token_b_mint).await?;
        let out_prog = crate::get_token_program_for_mint(rpc, &token_a_mint).await?;
        (
            token_b_mint,
            token_a_mint,
            crate::get_associated_token_address(user, &token_b_mint, &in_prog),
            crate::get_associated_token_address(user, &token_a_mint, &out_prog),
        )
    } else {
        return Err(crate::error::ClientError::MintMismatch {
            expected: format!("{} or {}", token_a_mint, token_b_mint),
            got: mint_a.to_string(),
        });
    };

    if output_mint != *mint_b {
        return Err(crate::error::ClientError::MintMismatch {
            expected: mint_b.to_string(),
            got: output_mint.to_string(),
        });
    }

    let (input_token_account, output_token_account) = (user_input_ata, user_output_ata);

    let token_a_program = crate::get_token_program_for_mint(rpc, &token_a_mint).await?;
    let token_b_program = crate::get_token_program_for_mint(rpc, &token_b_mint).await?;

    let (event_authority, _) =
        Address::find_program_address(&[b"__event_authority"], &CP_AMM_PROGRAM_ID);

    let input = MeteoraDammV2SwapInput {
        pool: pool_pubkey,
        input_token_account,
        output_token_account,
        token_a_vault,
        token_b_vault,
        token_a_mint,
        token_b_mint,
        payer: *user,
        token_a_program,
        token_b_program,
        referral_token_account: CP_AMM_PROGRAM_ID,
        event_authority,
    };

    Ok((build_accounts(&input), build_extra_data()))
}
