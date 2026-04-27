#[cfg(feature = "resolve")]
use {
    crate::{get_associated_token_address, ClientError, TOKEN_PROGRAM_ID},
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
};
use {
    solana_address::{address, Address},
    solana_instruction::AccountMeta,
};

pub const JUPITER_PERPETUALS_PROGRAM_ID: Address =
    address!("PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu");
pub const TRANSFER_AUTHORITY: Address = address!("AVzP2GeRmqGphJsMxWoqjpUifPpCret7LqWhD8NWQK49");
pub const PERPETUALS: Address = address!("H4ND9aYttUVLFmNypZqLjZ52FYiGvdEB45GmwNoKEjTj");
pub const POOL: Address = address!("5BUwFW4nRbftYTDMbgxykoFWqWHPzahFSNAaaaJtVKsq");
pub const SOL_VAULT: Address = address!("BUvduFTd2sWFagCunBPLupG8fBTJqweLw9DuhruNFSCm");
pub const WETH_VAULT: Address = address!("Bgarxg65CEjN3kosjCW5Du3wEqvV3dpCGDR3a2HRQsYJ");
pub const WBTC_VAULT: Address = address!("FgpXg2J3TzSs7w3WGYYE7aWePdrxBVLCXSxmAKnCZNtZ");
pub const USDC_VAULT: Address = address!("WzWUoCmtVv7eqAbU3BfKPU3fhLP6CXR8NCJH78UK9VS");
pub const USDT_VAULT: Address = address!("Gex24YznvguMad1mBzTQ7a64U1CJy59gvsStQmNnnwAd");
pub const JLP_MINT: Address = address!("27G8MtK7VtTcCHkpASjSDdkWWYfoqT6ggEuKidVJidD4");
pub const EVENT_AUTHORITY: Address = address!("37hJBDnntwqhGbK7L6M1bLyvccj4u55CCUiLPdYkiqBN");
pub const SOL_CUSTODY: Address = address!("7xS2gz2bTp3fwCC7knJvUWTEU9Tycczu6VhJYKgi1wdz");
pub const WETH_CUSTODY: Address = address!("AQCGyheWPLeo6Qp9WpYS9m3Qj479t7R636N9ey1rEjEn");
pub const WBTC_CUSTODY: Address = address!("5Pv3gM9JrFFH883SWAhvJC9RPYmo8UNxuFtv5bMMALkm");
pub const USDC_CUSTODY: Address = address!("G18jKKXQwBbrHeiK3C9MRXhkHsLHf7XgCSisykV46EZa");
pub const USDT_CUSTODY: Address = address!("4vkNeXiYEUizLdrpdPS1eC2mccyM4NUPRtERrk6ZETkk");
pub const SOL_AG_PRICE_FEED: Address = address!("FYq2BWQ1V5P1WFBqr3qB2Kb5yHVvSv7upzKodgQE5zXh");
pub const WETH_AG_PRICE_FEED: Address = address!("AFZnHPzy4mvVCffrVwhewHbFc93uTHvDSFrVH7GtfXF1");
pub const WBTC_AG_PRICE_FEED: Address = address!("hUqAT1KQ7eW1i6Csp9CXYtpPfSAvi835V7wKi5fRfmC");
pub const USDC_AG_PRICE_FEED: Address = address!("6Jp2xZUTWdDD2ZyUPRzeMdc6AFQ5K3pFgZxk2EijfjnM");
pub const USDT_AG_PRICE_FEED: Address = address!("Fgc93D641F8N2d1xLjQ4jmShuD3GE3BsCXA56KBQbF5u");

pub const WSOL_MINT: Address = address!("So11111111111111111111111111111111111111112");
pub const WETH_MINT: Address = address!("7vfCXTUXx5WJV5JADk17DUJ4ksgau7utNKj4b963voxs");
pub const WBTC_MINT: Address = address!("3NZ9JMVBmGAqocybic2c7LQCJScmgsAZ6vQqTDzcqmJh");
pub const USDC_MINT: Address = address!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
pub const USDT_MINT: Address = address!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");

const MINT_CUSTODY_PAIRS: [(Address, Address); 5] = [
    (WSOL_MINT, SOL_CUSTODY),
    (WETH_MINT, WETH_CUSTODY),
    (WBTC_MINT, WBTC_CUSTODY),
    (USDC_MINT, USDC_CUSTODY),
    (USDT_MINT, USDT_CUSTODY),
];

pub struct JupiterPerpetualsSwap2Input {
    pub owner: Address,
    pub funding_account: Address,
    pub receiving_account: Address,
    pub input_mint: Address,
    pub output_mint: Address,
}

pub struct JupiterPerpetualsLiquidity2Input {
    pub owner: Address,
    pub funding_or_receiving_account: Address,
    pub lp_token_account: Address,
    pub mint: Address,
}

/// Pre-resolved addresses for building an Jupiter Perpetuals swap instruction offline.
pub enum JupiterPerpetualsSwapInput {
    Swap2(JupiterPerpetualsSwap2Input),
    Liquidity2(JupiterPerpetualsLiquidity2Input),
}

fn get_mint_accounts(mint: Address) -> Result<[AccountMeta; 4], ClientError> {
    let accounts = match mint {
        WSOL_MINT => [
            AccountMeta::new_readonly(SOL_CUSTODY, false),
            AccountMeta::new_readonly(SOL_AG_PRICE_FEED, false),
            AccountMeta::new_readonly(SOL_AG_PRICE_FEED, false),
            AccountMeta::new_readonly(SOL_VAULT, false),
        ],
        WETH_MINT => [
            AccountMeta::new_readonly(WETH_CUSTODY, false),
            AccountMeta::new_readonly(WETH_AG_PRICE_FEED, false),
            AccountMeta::new_readonly(WETH_AG_PRICE_FEED, false),
            AccountMeta::new_readonly(WETH_VAULT, false),
        ],
        WBTC_MINT => [
            AccountMeta::new_readonly(WBTC_CUSTODY, false),
            AccountMeta::new_readonly(WBTC_AG_PRICE_FEED, false),
            AccountMeta::new_readonly(WBTC_AG_PRICE_FEED, false),
            AccountMeta::new_readonly(WBTC_VAULT, false),
        ],
        USDC_MINT => [
            AccountMeta::new_readonly(USDC_CUSTODY, false),
            AccountMeta::new_readonly(USDC_AG_PRICE_FEED, false),
            AccountMeta::new_readonly(USDC_AG_PRICE_FEED, false),
            AccountMeta::new_readonly(USDC_VAULT, false),
        ],
        USDT_MINT => [
            AccountMeta::new_readonly(USDT_CUSTODY, false),
            AccountMeta::new_readonly(USDT_AG_PRICE_FEED, false),
            AccountMeta::new_readonly(USDT_AG_PRICE_FEED, false),
            AccountMeta::new_readonly(USDT_VAULT, false),
        ],
        _ => {
            return Err(ClientError::MintMismatch {
                expected: format!(
                    "{} or {} or {} or {} or {}",
                    WSOL_MINT, WETH_MINT, WBTC_MINT, USDC_MINT, USDT_MINT
                ),
                got: mint.to_string(),
            })
        }
    };

    Ok(accounts)
}

/// Build Jupiter Perpetuals swap AccountMeta list from pre-resolved addresses (no RPC needed).
pub fn build_accounts(input: JupiterPerpetualsSwapInput) -> Result<Vec<AccountMeta>, ClientError> {
    let metas = match input {
        JupiterPerpetualsSwapInput::Swap2(input) => {
            let mut metas = vec![
                AccountMeta::new(input.owner, true),
                AccountMeta::new(input.funding_account, false),
                AccountMeta::new(input.receiving_account, false),
                AccountMeta::new_readonly(TRANSFER_AUTHORITY, false),
                AccountMeta::new_readonly(PERPETUALS, false),
                AccountMeta::new(POOL, false),
            ];

            metas.extend(get_mint_accounts(input.input_mint)?);
            metas.extend(get_mint_accounts(input.output_mint)?);
            metas.extend([
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(EVENT_AUTHORITY, false),
                AccountMeta::new_readonly(JUPITER_PERPETUALS_PROGRAM_ID, false),
            ]);

            metas
        }
        JupiterPerpetualsSwapInput::Liquidity2(input) => {
            let mut metas = vec![
                AccountMeta::new(input.owner, true),
                AccountMeta::new(input.funding_or_receiving_account, false),
                AccountMeta::new(input.lp_token_account, false),
                AccountMeta::new_readonly(TRANSFER_AUTHORITY, false),
                AccountMeta::new_readonly(PERPETUALS, false),
                AccountMeta::new(POOL, false),
            ];

            metas.extend(get_mint_accounts(input.mint)?);
            metas.extend([
                AccountMeta::new(JLP_MINT, false),
                AccountMeta::new_readonly(TOKEN_PROGRAM_ID, false),
                AccountMeta::new_readonly(EVENT_AUTHORITY, false),
                AccountMeta::new_readonly(JUPITER_PERPETUALS_PROGRAM_ID, false),
            ]);
            metas.extend(MINT_CUSTODY_PAIRS.into_iter().map(|(mint, custody)| {
                if input.mint == mint {
                    AccountMeta::new(custody, false)
                } else {
                    AccountMeta::new_readonly(custody, false)
                }
            }));
            metas.extend([
                AccountMeta::new_readonly(SOL_AG_PRICE_FEED, false),
                AccountMeta::new_readonly(WETH_AG_PRICE_FEED, false),
                AccountMeta::new_readonly(WBTC_AG_PRICE_FEED, false),
                AccountMeta::new_readonly(USDC_AG_PRICE_FEED, false),
                AccountMeta::new_readonly(USDT_AG_PRICE_FEED, false),
            ]);

            metas
        }
    };

    Ok(metas)
}

#[cfg(feature = "resolve")]
pub async fn resolve(
    _rpc: &RpcClient,
    mint_a: &Address,
    mint_b: &Address,
    user: &Address,
) -> Result<(Vec<AccountMeta>, Vec<u8>), ClientError> {
    let input = if [*mint_a, *mint_b].contains(&JLP_MINT) {
        let mint = if mint_a == &JLP_MINT { mint_b } else { mint_a };
        let funding_or_receiving_account =
            get_associated_token_address(user, mint, &TOKEN_PROGRAM_ID);
        let lp_token_account = get_associated_token_address(user, &JLP_MINT, &TOKEN_PROGRAM_ID);

        JupiterPerpetualsSwapInput::Liquidity2(JupiterPerpetualsLiquidity2Input {
            owner: *user,
            funding_or_receiving_account,
            lp_token_account,
            mint: *mint,
        })
    } else {
        JupiterPerpetualsSwapInput::Swap2(JupiterPerpetualsSwap2Input {
            owner: *user,
            funding_account: get_associated_token_address(user, mint_a, &TOKEN_PROGRAM_ID),
            receiving_account: get_associated_token_address(user, mint_b, &TOKEN_PROGRAM_ID),
            input_mint: *mint_a,
            output_mint: *mint_b,
        })
    };

    Ok((build_accounts(input)?, vec![]))
}
