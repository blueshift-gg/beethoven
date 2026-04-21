#[cfg(feature = "resolve")]
use {
    crate::{
        discover_pool_with_flip, get_associated_token_address, get_token_program_for_mint,
        read_pubkey, ClientError,
    },
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    crate::{ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID},
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const SYNATRA_PROGRAM_ID: Address = address!("synatfE5AvWtbDT9sSvDsF9gmeqR9qeq3FA84bhxWur");
const YSOL_MINT: Address = address!("yso11zxLbHA3wBJ9HAtVu6wnesqz9A2qxnhxanasZ4N");

// Pool account layout offsets
// Layout: [8 discriminator] ... [32 stake_token] @ 80 [32 receipt_token] ...
#[cfg(feature = "resolve")]
const OFFSET_STAKE_TOKEN: usize = 80;
#[cfg(feature = "resolve")]
const OFFSET_RECEIPT_TOKEN: usize = 112;

pub struct SynatraSwapBaseAccounts {
    pub signer: Address,
    pub payer: Address,
    pub pool: Address,
}

pub enum SynatraSwapType {
    StakeSol {
        user_receipt_ata: Address,
    },
    StakeToken {
        stake_token: Address,
        receipt_token: Address,
        user_token_ata: Address,
        user_receipt_ata: Address,
        pool_token_ata: Address,
    },
}

/// Pre-resolved addresses for building a Synatra swap instruction offline.
pub struct SynatraSwapInput {
    pub base: SynatraSwapBaseAccounts,
    pub swap_type: SynatraSwapType,
}

/// Build Synatra swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &SynatraSwapInput) -> Vec<AccountMeta> {
    let mut meta = vec![
        AccountMeta::new_readonly(SYNATRA_PROGRAM_ID, false),
        AccountMeta::new_readonly(input.base.signer, true),
        AccountMeta::new(input.base.payer, true),
        AccountMeta::new(input.base.pool, false),
    ];

    match &input.swap_type {
        SynatraSwapType::StakeSol { user_receipt_ata } => {
            meta.extend_from_slice(&[
                AccountMeta::new(YSOL_MINT, false),
                AccountMeta::new(*user_receipt_ata, false),
            ]);
        }
        SynatraSwapType::StakeToken {
            stake_token,
            receipt_token,
            user_token_ata,
            user_receipt_ata,
            pool_token_ata,
        } => {
            meta.extend_from_slice(&[
                AccountMeta::new(*stake_token, false),
                AccountMeta::new(*receipt_token, false),
                AccountMeta::new(*user_token_ata, false),
                AccountMeta::new(*user_receipt_ata, false),
                AccountMeta::new(*pool_token_ata, false),
            ]);
        }
    }

    meta.extend_from_slice(&[
        AccountMeta::new_readonly(ASSOCIATED_TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
    ]);

    meta
}

#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    pool: Option<&Address>,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let (pool_pubkey, pool_data) = match pool {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = discover_pool_with_flip(
                rpc,
                &SYNATRA_PROGRAM_ID,
                OFFSET_STAKE_TOKEN,
                OFFSET_RECEIPT_TOKEN,
                mint_a,
                mint_b,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let pool_stake_token = read_pubkey(&pool_data, OFFSET_STAKE_TOKEN)?;
    let pool_receipt_token = read_pubkey(&pool_data, OFFSET_RECEIPT_TOKEN)?;

    if *mint_a != pool_stake_token {
        return Err(ClientError::MintMismatch {
            expected: pool_stake_token.to_string(),
            got: mint_a.to_string(),
        });
    }
    if *mint_b != pool_receipt_token {
        return Err(ClientError::MintMismatch {
            expected: pool_receipt_token.to_string(),
            got: mint_b.to_string(),
        });
    }

    let base = SynatraSwapBaseAccounts {
        signer: *user,
        payer: *user,
        pool: pool_pubkey,
    };

    let pool_receipt_token_token_program =
        get_token_program_for_mint(rpc, &pool_receipt_token).await?;
    let user_receipt_ata =
        get_associated_token_address(user, &pool_receipt_token, &pool_receipt_token_token_program);

    let swap_type = match pool_receipt_token.eq(&YSOL_MINT) {
        true => SynatraSwapType::StakeSol { user_receipt_ata },
        false => {
            let pool_token_token_program =
                get_token_program_for_mint(rpc, &pool_stake_token).await?;

            SynatraSwapType::StakeToken {
                stake_token: pool_stake_token,
                receipt_token: pool_receipt_token,
                user_token_ata: get_associated_token_address(
                    user,
                    &pool_stake_token,
                    &pool_token_token_program,
                ),
                user_receipt_ata,
                pool_token_ata: get_associated_token_address(
                    &pool_pubkey,
                    &pool_stake_token,
                    &pool_receipt_token_token_program,
                ),
            }
        }
    };

    let input = SynatraSwapInput { base, swap_type };

    Ok((build_accounts(&input), vec![]))
}
