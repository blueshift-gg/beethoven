use {
    crate::SYSTEM_PROGRAM_ID,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};
#[cfg(feature = "resolve")]
use {
    crate::{
        discover_pool_with_flip, get_associated_token_address, get_token_program_for_mint,
        read_pubkey, ClientError,
    },
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

pub const VOLTR_PROGRAM_ID: Address = address!("vVoLTRjQmtFpiYoegx285Ze4gsLJ8ZxgFKVcuvmG1a8");
pub const PROTOCOL: Address = address!("4sycXz9Xwevedo6eiXR8QEhY8yrQrkNS4G1deY9tAD2Y");

// Vault account layout offsets
// Layout: [8 discriminator] ... [32 asset_mint] @ 104 [32 idle_ata] ... [32 lp_mint] @ 272 ...
#[cfg(feature = "resolve")]
const OFFSET_ASSET_MINT: usize = 104;
#[cfg(feature = "resolve")]
const OFFSET_IDLE_ATA: usize = 136;
#[cfg(feature = "resolve")]
const OFFSET_LP_MINT: usize = 272;

pub struct VoltrSwapBaseAccounts {
    pub user_transfer_authority: Address,
    pub vault: Address,
    pub vault_asset_mint: Address,
    pub vault_lp_mint: Address,
}

pub enum VoltrSwapLegAccounts {
    DepositVault {
        user_asset_ata: Address,
        vault_asset_idle_ata: Address,
        vault_asset_idle_auth: Address,
        user_lp_ata: Address,
        vault_lp_mint_auth: Address,
    },
    InstantWithdrawVault {
        user_lp_ata: Address,
        vault_asset_idle_ata: Address,
        vault_asset_idle_auth: Address,
        user_asset_ata: Address,
    },
}

pub struct VoltrSwapTailAccounts {
    pub asset_token_program: Address,
    pub lp_token_program: Address,
}

/// Pre-resolved addresses for building a Voltr swap instruction offline.
pub struct VoltrSwapInput {
    pub base: VoltrSwapBaseAccounts,
    pub leg: VoltrSwapLegAccounts,
    pub tail: VoltrSwapTailAccounts,
}

/// Build Voltr swap AccountMeta list from pre-resolved addresses.
pub fn build_accounts(input: &VoltrSwapInput) -> Vec<AccountMeta> {
    let mut meta = vec![
        AccountMeta::new_readonly(VOLTR_PROGRAM_ID, false),
        AccountMeta::new_readonly(input.base.user_transfer_authority, true),
        AccountMeta::new_readonly(PROTOCOL, false),
        AccountMeta::new(input.base.vault, false),
        AccountMeta::new_readonly(input.base.vault_asset_mint, false),
        AccountMeta::new(input.base.vault_lp_mint, false),
    ];

    match input.leg {
        VoltrSwapLegAccounts::DepositVault {
            user_asset_ata,
            vault_asset_idle_ata,
            vault_asset_idle_auth,
            user_lp_ata,
            vault_lp_mint_auth,
        } => {
            meta.push(AccountMeta::new(user_asset_ata, false));
            meta.push(AccountMeta::new(vault_asset_idle_ata, false));
            meta.push(AccountMeta::new_readonly(vault_asset_idle_auth, false));
            meta.push(AccountMeta::new(user_lp_ata, false));
            meta.push(AccountMeta::new_readonly(vault_lp_mint_auth, false));
        }
        VoltrSwapLegAccounts::InstantWithdrawVault {
            user_lp_ata,
            vault_asset_idle_ata,
            vault_asset_idle_auth,
            user_asset_ata,
        } => {
            meta.push(AccountMeta::new(user_lp_ata, false));
            meta.push(AccountMeta::new(vault_asset_idle_ata, false));
            meta.push(AccountMeta::new(vault_asset_idle_auth, false));
            meta.push(AccountMeta::new(user_asset_ata, false));
        }
    };

    meta.extend_from_slice(&[
        AccountMeta::new_readonly(input.tail.asset_token_program, false),
        AccountMeta::new_readonly(input.tail.lp_token_program, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
    ]);

    meta
}

/// Build Voltr extra data.
///
/// deposit_vault: []
///
/// instant_withdraw_vault: [is_amount_in_lp, is_withdraw_all]
pub fn build_extra_data(is_amount_in_lp: Option<bool>, is_withdraw_all: Option<bool>) -> Vec<u8> {
    // if is_amount_in_lp.is_some() && is_withdraw_all.is_some() {
    if let (Some(is_amount_in_lp), Some(is_withdraw_all)) = (is_amount_in_lp, is_withdraw_all) {
        vec![is_amount_in_lp as u8, is_withdraw_all as u8]
    } else {
        vec![]
    }
}

/// Resolve accounts and extra_data for Voltr (deposit or instant withdraw per `resolve_swap`).
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    vault: Option<&Address>,
    is_amount_in_lp: Option<bool>,
    is_withdraw_all: Option<bool>,
    asset_mint: &Address,
    lp_mint: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let (vault_pubkey, vault_data) = match vault {
        Some(addr) => {
            let account = rpc.get_account(addr).await?;
            (*addr, account.data)
        }
        None => {
            let (pubkey, account) = discover_pool_with_flip(
                rpc,
                &VOLTR_PROGRAM_ID,
                OFFSET_ASSET_MINT,
                OFFSET_LP_MINT,
                asset_mint,
                lp_mint,
            )
            .await?;
            (pubkey, account.data)
        }
    };

    let vault_asset_mint = read_pubkey(&vault_data, OFFSET_ASSET_MINT)?;
    let vault_lp_mint = read_pubkey(&vault_data, OFFSET_LP_MINT)?;

    if vault_asset_mint != *asset_mint {
        return Err(ClientError::InvalidAccountData(format!(
            "vault asset mint {} != asset mint {}",
            vault_asset_mint, asset_mint
        )));
    } else if vault_lp_mint != *lp_mint {
        return Err(ClientError::InvalidAccountData(format!(
            "vault lp mint {} != lp mint {}",
            vault_lp_mint, lp_mint
        )));
    }

    let base = VoltrSwapBaseAccounts {
        user_transfer_authority: *user,
        vault: vault_pubkey,
        vault_asset_mint,
        vault_lp_mint,
    };

    let lp_token_program = get_token_program_for_mint(rpc, &vault_lp_mint).await?;
    let user_lp_ata = get_associated_token_address(user, &vault_lp_mint, &lp_token_program);
    let vault_asset_idle_ata = read_pubkey(&vault_data, OFFSET_IDLE_ATA)?;
    let vault_asset_idle_auth = Address::find_program_address(
        &[b"vault_asset_idle_auth", vault_pubkey.as_ref()],
        &VOLTR_PROGRAM_ID,
    )
    .0;
    let asset_token_program = get_token_program_for_mint(rpc, &vault_asset_mint).await?;
    let user_asset_ata =
        get_associated_token_address(user, &vault_asset_mint, &asset_token_program);

    let leg = if is_amount_in_lp.is_some() && is_withdraw_all.is_some() {
        VoltrSwapLegAccounts::InstantWithdrawVault {
            user_lp_ata,
            vault_asset_idle_ata,
            vault_asset_idle_auth,
            user_asset_ata,
        }
    } else {
        let vault_lp_mint_auth = Address::find_program_address(
            &[b"vault_lp_mint_auth", vault_pubkey.as_ref()],
            &VOLTR_PROGRAM_ID,
        )
        .0;

        VoltrSwapLegAccounts::DepositVault {
            user_asset_ata,
            vault_asset_idle_ata,
            vault_asset_idle_auth,
            user_lp_ata,
            vault_lp_mint_auth,
        }
    };

    let tail = VoltrSwapTailAccounts {
        asset_token_program,
        lp_token_program,
    };

    let input = VoltrSwapInput { base, leg, tail };

    Ok((
        build_accounts(&input),
        build_extra_data(is_amount_in_lp, is_withdraw_all),
    ))
}
