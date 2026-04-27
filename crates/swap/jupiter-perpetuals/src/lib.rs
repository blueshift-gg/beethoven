#![no_std]

use {
    beethoven_core::Swap,
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::{address, Address},
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const JUPITER_PERPETUALS_PROGRAM_ID: Address =
    address!("PERPHjGBqRHArX4DySjwM6UJHiR3sWAatqfdBS2qQJu");

const SWAP2_DISCRIMINATOR: [u8; 8] = [65, 75, 63, 76, 235, 91, 91, 136];
const ADD_LIQUIDITY_2_DISCRIMINATOR: [u8; 8] = [228, 162, 78, 28, 70, 219, 116, 115];
const REMOVE_LIQUIDITY_2_DISCRIMINATOR: [u8; 8] = [230, 215, 82, 127, 241, 101, 227, 146];

const SWAP2_NUM_ACCOUNTS: usize = 17;
const ADD_LIQUIDITY_2_NUM_ACCOUNTS: usize = 24;
const REMOVE_LIQUIDITY_2_NUM_ACCOUNTS: usize = 24;
const MIN_NUM_ACCOUNTS: usize = 18;
const MAX_NUM_ACCOUNTS: usize = 24;

const SWAP2_DATA_LEN: usize = 24;
const ADD_LIQUIDITY_2_DATA_LEN: usize = 32;
const REMOVE_LIQUIDITY_2_DATA_LEN: usize = 24;
const MAX_DATA_LEN: usize = 32;

pub struct JupiterPerpetuals;

pub struct JupiterPerpetualsSwapData {
    pub is_add_liquidity: bool,
}

impl TryFrom<&[u8]> for JupiterPerpetualsSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self {
            // defaults to false if no data is provided (swap2)
            is_add_liquidity: if data.is_empty() { false } else { data[0] != 0 },
        })
    }
}

pub struct JupiterPerpetualsSwap2Accounts<'info> {
    pub owner: &'info AccountView,
    pub funding_account: &'info AccountView,
    pub receiving_account: &'info AccountView,
    pub transfer_authority: &'info AccountView,
    pub perpetuals: &'info AccountView,
    pub pool: &'info AccountView,
    pub receiving_custody: &'info AccountView,
    pub receiving_custody_doves_price_account: &'info AccountView,
    pub receiving_custody_pythnet_price_account: &'info AccountView,
    pub receiving_custody_token_account: &'info AccountView,
    pub dispensing_custody: &'info AccountView,
    pub dispensing_custody_doves_price_account: &'info AccountView,
    pub dispensing_custody_pythnet_price_account: &'info AccountView,
    pub dispensing_custody_token_account: &'info AccountView,
    pub token_program: &'info AccountView,
    pub event_authority: &'info AccountView,
    pub program: &'info AccountView,
}

pub struct JupiterPerpetualsLiquidity2Accounts<'info> {
    pub owner: &'info AccountView,
    pub funding_or_receiving_account: &'info AccountView,
    pub lp_token_account: &'info AccountView,
    pub transfer_authority: &'info AccountView,
    pub perpetuals: &'info AccountView,
    pub pool: &'info AccountView,
    pub custody: &'info AccountView,
    pub custody_doves_price_account: &'info AccountView,
    pub custody_pythnet_price_account: &'info AccountView,
    pub custody_token_account: &'info AccountView,
    pub lp_token_mint: &'info AccountView,
    pub token_program: &'info AccountView,
    pub event_authority: &'info AccountView,
    pub program: &'info AccountView,
    pub remaining_accounts: &'info [AccountView],
}

pub enum JupiterPerpetualsSwapAccounts<'info> {
    Swap2(JupiterPerpetualsSwap2Accounts<'info>),
    Liquidity2(JupiterPerpetualsLiquidity2Accounts<'info>),
}

impl<'info> TryFrom<&'info [AccountView]> for JupiterPerpetualsSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [jupiter_perpetuals_program, owner, remaining_accounts @ ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if accounts.len() < MIN_NUM_ACCOUNTS {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        match accounts.len() - 1 {
            SWAP2_NUM_ACCOUNTS => Ok(JupiterPerpetualsSwapAccounts::Swap2(
                JupiterPerpetualsSwap2Accounts {
                    owner,
                    funding_account: &remaining_accounts[0],
                    receiving_account: &remaining_accounts[1],
                    transfer_authority: &remaining_accounts[2],
                    perpetuals: &remaining_accounts[3],
                    pool: &remaining_accounts[4],
                    receiving_custody: &remaining_accounts[5],
                    receiving_custody_doves_price_account: &remaining_accounts[6],
                    receiving_custody_pythnet_price_account: &remaining_accounts[7],
                    receiving_custody_token_account: &remaining_accounts[8],
                    dispensing_custody: &remaining_accounts[9],
                    dispensing_custody_doves_price_account: &remaining_accounts[10],
                    dispensing_custody_pythnet_price_account: &remaining_accounts[11],
                    dispensing_custody_token_account: &remaining_accounts[12],
                    token_program: &remaining_accounts[13],
                    event_authority: &remaining_accounts[14],
                    program: jupiter_perpetuals_program,
                },
            )),
            #[allow(unreachable_patterns)]
            ADD_LIQUIDITY_2_NUM_ACCOUNTS | REMOVE_LIQUIDITY_2_NUM_ACCOUNTS => Ok(
                JupiterPerpetualsSwapAccounts::Liquidity2(JupiterPerpetualsLiquidity2Accounts {
                    owner,
                    funding_or_receiving_account: &remaining_accounts[0],
                    lp_token_account: &remaining_accounts[1],
                    transfer_authority: &remaining_accounts[2],
                    perpetuals: &remaining_accounts[3],
                    pool: &remaining_accounts[4],
                    custody: &remaining_accounts[5],
                    custody_doves_price_account: &remaining_accounts[6],
                    custody_pythnet_price_account: &remaining_accounts[7],
                    custody_token_account: &remaining_accounts[8],
                    lp_token_mint: &remaining_accounts[9],
                    token_program: &remaining_accounts[10],
                    event_authority: &remaining_accounts[11],
                    program: jupiter_perpetuals_program,
                    remaining_accounts: &remaining_accounts[13..],
                }),
            ),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

impl<'info> Swap<'info> for JupiterPerpetuals {
    type Accounts = JupiterPerpetualsSwapAccounts<'info>;
    type Data = JupiterPerpetualsSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let mut instruction_accounts =
            MaybeUninit::<[InstructionAccount; MAX_NUM_ACCOUNTS]>::uninit();
        let instruction_accounts_ptr = instruction_accounts.as_mut_ptr() as *mut InstructionAccount;

        unsafe {
            match ctx {
                JupiterPerpetualsSwapAccounts::Swap2(ctx) => {
                    core::ptr::write(
                        instruction_accounts_ptr,
                        InstructionAccount::writable_signer(ctx.owner.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(1),
                        InstructionAccount::writable(ctx.funding_account.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(2),
                        InstructionAccount::writable(ctx.receiving_account.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(3),
                        InstructionAccount::readonly(ctx.transfer_authority.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(4),
                        InstructionAccount::readonly(ctx.perpetuals.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(5),
                        InstructionAccount::writable(ctx.pool.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(6),
                        InstructionAccount::writable(ctx.receiving_custody.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(7),
                        InstructionAccount::readonly(
                            ctx.receiving_custody_doves_price_account.address(),
                        ),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(8),
                        InstructionAccount::readonly(
                            ctx.receiving_custody_pythnet_price_account.address(),
                        ),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(9),
                        InstructionAccount::writable(ctx.receiving_custody_token_account.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(10),
                        InstructionAccount::writable(ctx.dispensing_custody.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(11),
                        InstructionAccount::readonly(
                            ctx.dispensing_custody_doves_price_account.address(),
                        ),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(12),
                        InstructionAccount::readonly(
                            ctx.dispensing_custody_pythnet_price_account.address(),
                        ),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(13),
                        InstructionAccount::writable(
                            ctx.dispensing_custody_token_account.address(),
                        ),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(14),
                        InstructionAccount::readonly(ctx.token_program.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(15),
                        InstructionAccount::readonly(ctx.event_authority.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(16),
                        InstructionAccount::readonly(ctx.program.address()),
                    );
                }
                JupiterPerpetualsSwapAccounts::Liquidity2(ctx) => {
                    core::ptr::write(
                        instruction_accounts_ptr,
                        InstructionAccount::writable_signer(ctx.owner.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(1),
                        InstructionAccount::writable(ctx.funding_or_receiving_account.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(2),
                        InstructionAccount::writable(ctx.lp_token_account.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(3),
                        InstructionAccount::readonly(ctx.transfer_authority.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(4),
                        InstructionAccount::readonly(ctx.perpetuals.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(5),
                        InstructionAccount::writable(ctx.pool.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(6),
                        InstructionAccount::writable(ctx.custody.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(7),
                        InstructionAccount::readonly(ctx.custody_doves_price_account.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(8),
                        InstructionAccount::readonly(ctx.custody_pythnet_price_account.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(9),
                        InstructionAccount::writable(ctx.custody_token_account.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(10),
                        InstructionAccount::writable(ctx.lp_token_mint.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(11),
                        InstructionAccount::readonly(ctx.token_program.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(12),
                        InstructionAccount::readonly(ctx.event_authority.address()),
                    );
                    core::ptr::write(
                        instruction_accounts_ptr.add(13),
                        InstructionAccount::readonly(ctx.program.address()),
                    );
                    for (index, account) in ctx.remaining_accounts.iter().enumerate() {
                        core::ptr::write(
                            instruction_accounts_ptr.add(14 + index),
                            InstructionAccount::from(account),
                        );
                    }
                }
            }
        }

        let total_accounts = match ctx {
            JupiterPerpetualsSwapAccounts::Swap2(_) => SWAP2_NUM_ACCOUNTS,
            JupiterPerpetualsSwapAccounts::Liquidity2(_) => {
                if data.is_add_liquidity {
                    ADD_LIQUIDITY_2_NUM_ACCOUNTS
                } else {
                    REMOVE_LIQUIDITY_2_NUM_ACCOUNTS
                }
            }
        };

        let instruction_accounts =
            unsafe { core::slice::from_raw_parts(instruction_accounts_ptr, total_accounts) };

        let account_views = {
            match ctx {
                JupiterPerpetualsSwapAccounts::Swap2(ctx) => {
                    let mut account_views = [ctx.owner; MAX_NUM_ACCOUNTS];
                    account_views[1] = ctx.funding_account;
                    account_views[2] = ctx.receiving_account;
                    account_views[3] = ctx.transfer_authority;
                    account_views[4] = ctx.perpetuals;
                    account_views[5] = ctx.pool;
                    account_views[6] = ctx.receiving_custody;
                    account_views[7] = ctx.receiving_custody_doves_price_account;
                    account_views[8] = ctx.receiving_custody_pythnet_price_account;
                    account_views[9] = ctx.receiving_custody_token_account;
                    account_views[10] = ctx.dispensing_custody;
                    account_views[11] = ctx.dispensing_custody_doves_price_account;
                    account_views[12] = ctx.dispensing_custody_pythnet_price_account;
                    account_views[13] = ctx.dispensing_custody_token_account;
                    account_views[14] = ctx.token_program;
                    account_views[15] = ctx.event_authority;
                    account_views[16] = ctx.program;
                    account_views
                }
                JupiterPerpetualsSwapAccounts::Liquidity2(ctx) => {
                    let mut account_views = [ctx.owner; MAX_NUM_ACCOUNTS];
                    account_views[1] = ctx.funding_or_receiving_account;
                    account_views[2] = ctx.lp_token_account;
                    account_views[3] = ctx.transfer_authority;
                    account_views[4] = ctx.perpetuals;
                    account_views[5] = ctx.pool;
                    account_views[6] = ctx.custody;
                    account_views[7] = ctx.custody_doves_price_account;
                    account_views[8] = ctx.custody_pythnet_price_account;
                    account_views[9] = ctx.custody_token_account;
                    account_views[10] = ctx.lp_token_mint;
                    account_views[11] = ctx.token_program;
                    account_views[12] = ctx.event_authority;
                    account_views[13] = ctx.program;
                    for (index, account) in ctx.remaining_accounts.iter().enumerate() {
                        account_views[14 + index] = account;
                    }
                    account_views
                }
            }
        };

        let total_accounts = match ctx {
            JupiterPerpetualsSwapAccounts::Swap2(_) => SWAP2_NUM_ACCOUNTS,
            JupiterPerpetualsSwapAccounts::Liquidity2(_) => {
                if data.is_add_liquidity {
                    ADD_LIQUIDITY_2_NUM_ACCOUNTS
                } else {
                    REMOVE_LIQUIDITY_2_NUM_ACCOUNTS
                }
            }
        };

        let account_views = &account_views[..total_accounts];

        let mut instruction_data = MaybeUninit::<[u8; MAX_DATA_LEN]>::uninit();
        let instruction_data_ptr = instruction_data.as_mut_ptr() as *mut u8;

        let discriminator = match ctx {
            JupiterPerpetualsSwapAccounts::Swap2(_) => &SWAP2_DISCRIMINATOR,
            JupiterPerpetualsSwapAccounts::Liquidity2(_) => {
                if data.is_add_liquidity {
                    &ADD_LIQUIDITY_2_DISCRIMINATOR
                } else {
                    &REMOVE_LIQUIDITY_2_DISCRIMINATOR
                }
            }
        };

        unsafe {
            core::ptr::copy_nonoverlapping(discriminator.as_ptr(), instruction_data_ptr, 8);
            core::ptr::copy_nonoverlapping(
                in_amount.to_le_bytes().as_ptr(),
                instruction_data_ptr.add(8),
                8,
            );
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                instruction_data_ptr.add(16),
                8,
            );
            if let JupiterPerpetualsSwapAccounts::Liquidity2(_) = ctx {
                if data.is_add_liquidity {
                    // token_amount_pre_swap hardcoded to 0
                    core::ptr::copy_nonoverlapping(
                        0u64.to_le_bytes().as_ptr(),
                        instruction_data_ptr.add(24),
                        8,
                    );
                }
            }
        }

        let data_len = match ctx {
            JupiterPerpetualsSwapAccounts::Swap2(_) => SWAP2_DATA_LEN,
            JupiterPerpetualsSwapAccounts::Liquidity2(_) => {
                if data.is_add_liquidity {
                    ADD_LIQUIDITY_2_DATA_LEN
                } else {
                    REMOVE_LIQUIDITY_2_DATA_LEN
                }
            }
        };

        let instruction = InstructionView {
            program_id: &JUPITER_PERPETUALS_PROGRAM_ID,
            accounts: instruction_accounts,
            data: unsafe { core::slice::from_raw_parts(instruction_data_ptr, data_len) },
        };

        invoke_signed_with_bounds::<MAX_NUM_ACCOUNTS, _>(&instruction, account_views, signer_seeds)
    }

    fn swap(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
    ) -> ProgramResult {
        Self::swap_signed(ctx, in_amount, minimum_out_amount, data, &[])
    }
}
