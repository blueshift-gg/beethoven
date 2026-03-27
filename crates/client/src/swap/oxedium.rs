use {solana_instruction::AccountMeta, solana_pubkey::Pubkey};

pub const OXEDIUM_PROGRAM_ID: Pubkey =
    Pubkey::from_str_const("oV3SkLhiXSG946FaqDf1yNocFMhE1ZvomGsoWF8Mzap");

const VAULT_SEED: &[u8] = b"vault-seed";
const OXEDIUM_SEED: &[u8] = b"oxedium-seed";
const OXE_GLOBAL_SEED: &[u8] = b"oxe-global-seed";

// Vault account layout (after 8-byte Anchor discriminator):
// [8]  base_fee_bps     u64
// [8]  protocol_fee_bps u64
// [8]  max_exit_fee_bps u64
// [32] token_mint       Pubkey
// [32] pyth_price_account Pubkey  ← offset 64
const OFFSET_PYTH_PRICE_ACCOUNT: usize = 64;

pub struct OxediumSwapInput {
    pub mint_in: Pubkey,
    pub mint_out: Pubkey,
    pub pyth_price_in: Pubkey,
    pub pyth_price_out: Pubkey,
    pub vault_pda_in: Pubkey,
    pub vault_pda_out: Pubkey,
    pub vault_ata_in: Pubkey,
    pub vault_ata_out: Pubkey,
    pub oxe_global_pda: Pubkey,
    pub user: Pubkey,
    pub user_ata_in: Pubkey,
    pub user_ata_out: Pubkey,
}

pub fn build_accounts(input: &OxediumSwapInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(OXEDIUM_PROGRAM_ID, false),
        AccountMeta::new(input.user, true),
        AccountMeta::new_readonly(input.mint_in, false),
        AccountMeta::new_readonly(input.mint_out, false),
        AccountMeta::new_readonly(input.pyth_price_in, false),
        AccountMeta::new_readonly(input.pyth_price_out, false),
        AccountMeta::new(input.user_ata_in, false),
        AccountMeta::new(input.user_ata_out, false),
        AccountMeta::new(input.vault_pda_in, false),
        AccountMeta::new(input.vault_pda_out, false),
        AccountMeta::new(input.vault_ata_in, false),
        AccountMeta::new(input.vault_ata_out, false),
        AccountMeta::new_readonly(input.oxe_global_pda, false),
        AccountMeta::new_readonly(crate::ASSOCIATED_TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(crate::TOKEN_PROGRAM_ID, false),
        AccountMeta::new_readonly(crate::SYSTEM_PROGRAM_ID, false),
    ]
}

#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &solana_rpc_client::nonblocking::rpc_client::RpcClient,
    mint_a: &Pubkey,
    mint_b: &Pubkey,
    user: &Pubkey,
) -> Result<(Vec<AccountMeta>, Vec<u8>), crate::error::ClientError> {
    let (vault_pda_in, _) =
        Pubkey::find_program_address(&[VAULT_SEED, mint_a.as_ref()], &OXEDIUM_PROGRAM_ID);
    let (vault_pda_out, _) =
        Pubkey::find_program_address(&[VAULT_SEED, mint_b.as_ref()], &OXEDIUM_PROGRAM_ID);

    let vault_in_data = rpc.get_account_data(&vault_pda_in).await?;
    let vault_out_data = rpc.get_account_data(&vault_pda_out).await?;

    let pyth_price_in = crate::read_pubkey(&vault_in_data, OFFSET_PYTH_PRICE_ACCOUNT)?;
    let pyth_price_out = crate::read_pubkey(&vault_out_data, OFFSET_PYTH_PRICE_ACCOUNT)?;

    let token_in_program = crate::get_token_program_for_mint(rpc, mint_a).await?;
    let token_out_program = crate::get_token_program_for_mint(rpc, mint_b).await?;

    let vault_ata_in =
        crate::get_associated_token_address(&vault_pda_in, mint_a, &token_in_program);
    let vault_ata_out =
        crate::get_associated_token_address(&vault_pda_out, mint_b, &token_out_program);

    let (oxe_global_pda, _) = Pubkey::find_program_address(
        &[OXEDIUM_SEED, OXE_GLOBAL_SEED],
        &OXEDIUM_PROGRAM_ID,
    );

    let user_ata_in = crate::get_associated_token_address(user, mint_a, &token_in_program);
    let user_ata_out = crate::get_associated_token_address(user, mint_b, &token_out_program);

    let input = OxediumSwapInput {
        mint_in: *mint_a,
        mint_out: *mint_b,
        pyth_price_in,
        pyth_price_out,
        vault_pda_in,
        vault_pda_out,
        vault_ata_in,
        vault_ata_out,
        oxe_global_pda,
        user: *user,
        user_ata_in,
        user_ata_out,
    };

    Ok((build_accounts(&input), vec![]))
}
