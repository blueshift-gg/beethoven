use {
    crate::TOKEN_PROGRAM_ID,
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};
#[cfg(feature = "resolve")]
use {
    crate::{get_associated_token_address, get_token_program_for_mint, read_pubkey, ClientError},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};

pub const CARROT_BOOST_PROGRAM_ID: Address =
    address!("C73nDAFn23RYwiFa6vtHshSbcg8x6BLYjw3bERJ3vHxf");

// CLendGroup account layout offsets
// Layout: [8 discriminator] [32 group] ...
#[cfg(feature = "resolve")]
const OFFSET_CLEND_GROUP: usize = 8;
// Bank account layout offsets
// Layout: [8 discriminator] [32 mint] ...
#[cfg(feature = "resolve")]
const OFFSET_BANK_MINT: usize = 8;

/// Pre-resolved addresses for building an Carrot Boost deposit instruction offline.
pub struct CarrotBoostDepositInput {
    pub clend_group: Address,
    pub clend_account: Address,
    pub signer: Address,
    pub bank: Address,
    pub signer_token_account: Address,
    pub bank_liquidity_vault: Address,
}

/// Build Carrot Boost deposit AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: &CarrotBoostDepositInput) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new_readonly(CARROT_BOOST_PROGRAM_ID, false),
        AccountMeta::new_readonly(input.clend_group, false),
        AccountMeta::new(input.clend_account, false),
        AccountMeta::new(input.signer, true),
        AccountMeta::new(input.bank, false),
        AccountMeta::new(input.signer_token_account, false),
        AccountMeta::new(input.bank_liquidity_vault, false),
        AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
    ]
}

/// Build Carrot Boost extra data: [deposit_up_to_amount].
pub fn build_extra_data(deposit_up_to_amount: u8) -> Vec<u8> {
    vec![deposit_up_to_amount]
}

/// Resolve accounts from a known vault; checks mint pair and PDAs against on-chain data.
#[cfg(feature = "resolve")]
pub async fn resolve(
    rpc: &RpcClient,
    clend_account: &Address,
    bank: &Address,
    deposit_up_to_amount: u8,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let clend_group_data = rpc.get_account(clend_account).await?.data;
    let clend_group = read_pubkey(&clend_group_data, OFFSET_CLEND_GROUP)?;
    let bank_data = rpc.get_account(bank).await?.data;
    let bank_mint = read_pubkey(&bank_data, OFFSET_BANK_MINT)?;
    let bank_mint_token_program = get_token_program_for_mint(rpc, &bank_mint).await?;
    let signer_token_account =
        get_associated_token_address(user, &bank_mint, &bank_mint_token_program);
    let bank_liquidity_vault = Address::find_program_address(
        &[b"liquidity_vault", bank.as_ref()],
        &CARROT_BOOST_PROGRAM_ID,
    )
    .0;

    let input = CarrotBoostDepositInput {
        clend_group,
        clend_account: *clend_account,
        signer: *user,
        bank: *bank,
        signer_token_account,
        bank_liquidity_vault,
    };

    Ok((
        build_accounts(&input),
        build_extra_data(deposit_up_to_amount),
    ))
}
