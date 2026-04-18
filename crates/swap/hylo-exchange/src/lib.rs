#![no_std]

use {
    beethoven_core::Swap,
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::{address, Address},
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const HYLO_EXCHANGE_PROGRAM_ID: Address =
    address!("HYEXCHtHkBagdStcJCp3xbbb9B7sdMdWXFNj6mdsG4hn");

const MINT_STABLECOIN_DISCRIMINATOR: [u8; 8] = [196, 235, 215, 70, 211, 5, 214, 238];
const REDEEM_STABLECOIN_DISCRIMINATOR: [u8; 8] = [69, 46, 6, 97, 170, 130, 160, 237];
const MINT_LEVERCOIN_DISCRIMINATOR: [u8; 8] = [91, 156, 221, 157, 151, 186, 223, 231];
const REDEEM_LEVERCOIN_DISCRIMINATOR: [u8; 8] = [132, 166, 215, 32, 46, 131, 174, 44];
const SWAP_STABLE_TO_LEVER_DISCRIMINATOR: [u8; 8] = [123, 194, 84, 140, 192, 193, 193, 161];
const SWAP_LEVER_TO_STABLE_DISCRIMINATOR: [u8; 8] = [167, 111, 84, 179, 69, 7, 135, 48];
// hardcoded to 20
const SLIPPAGE_TOLERANCE: u64 = 20;
const HYUSD_STABLECOIN_AUTH: Address = address!("CfuSViqf6wvUKEprLhtuCsSanvfAsMbDmkAW92FP95qe");

pub struct HyloExchange;

#[repr(u8)]
pub enum SwapType {
    MintStablecoin = 0,
    RedeemStablecoin = 1,
    MintLevercoin = 2,
    RedeemLevercoin = 3,
    SwapStableToLever = 4,
    SwapLeverToStable = 5,
}

pub struct HyloExchangeSwapData {
    pub swap_type: SwapType,
}

impl HyloExchangeSwapData {
    pub const DATA_LEN: usize = 1;
}

impl TryFrom<&[u8]> for HyloExchangeSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }
        let swap_type = match data[0] {
            0 => SwapType::MintStablecoin,
            1 => SwapType::RedeemStablecoin,
            2 => SwapType::MintLevercoin,
            3 => SwapType::RedeemLevercoin,
            4 => SwapType::SwapStableToLever,
            5 => SwapType::SwapLeverToStable,
            _ => return Err(ProgramError::InvalidInstructionData),
        };
        Ok(Self { swap_type })
    }
}

pub struct HyloExchangeSwapAccounts<'info> {
    pub hylo_program: &'info AccountView,
    pub leg: HyloExchangeLeg<'info>,
}

pub enum HyloExchangeLeg<'info> {
    MintStablecoin {
        user: &'info AccountView,
        hylo: &'info AccountView,
        fee_auth: &'info AccountView,
        vault_auth: &'info AccountView,
        stablecoin_auth: &'info AccountView,
        fee_vault: &'info AccountView,
        lst_vault: &'info AccountView,
        lst_header: &'info AccountView,
        user_lst_ta: &'info AccountView,
        user_stablecoin_ta: &'info AccountView,
        lst_mint: &'info AccountView,
        stablecoin_mint: &'info AccountView,
        sol_usd_pyth_feed: &'info AccountView,
        token_program: &'info AccountView,
        associated_token_program: &'info AccountView,
        system_program: &'info AccountView,
        event_authority: &'info AccountView,
        program: &'info AccountView,
    },
    RedeemStablecoin {
        user: &'info AccountView,
        hylo: &'info AccountView,
        fee_auth: &'info AccountView,
        vault_auth: &'info AccountView,
        fee_vault: &'info AccountView,
        lst_vault: &'info AccountView,
        lst_header: &'info AccountView,
        user_stablecoin_ta: &'info AccountView,
        user_lst_ta: &'info AccountView,
        stablecoin_mint: &'info AccountView,
        lst_mint: &'info AccountView,
        sol_usd_pyth_feed: &'info AccountView,
        system_program: &'info AccountView,
        token_program: &'info AccountView,
        associated_token_program: &'info AccountView,
        event_authority: &'info AccountView,
        program: &'info AccountView,
    },
    MintLevercoin {
        user: &'info AccountView,
        hylo: &'info AccountView,
        fee_auth: &'info AccountView,
        vault_auth: &'info AccountView,
        levercoin_auth: &'info AccountView,
        fee_vault: &'info AccountView,
        lst_vault: &'info AccountView,
        lst_header: &'info AccountView,
        user_lst_ta: &'info AccountView,
        user_levercoin_ta: &'info AccountView,
        lst_mint: &'info AccountView,
        levercoin_mint: &'info AccountView,
        stablecoin_mint: &'info AccountView,
        sol_usd_pyth_feed: &'info AccountView,
        token_program: &'info AccountView,
        associated_token_program: &'info AccountView,
        system_program: &'info AccountView,
        event_authority: &'info AccountView,
        program: &'info AccountView,
    },
    RedeemLevercoin {
        user: &'info AccountView,
        hylo: &'info AccountView,
        fee_auth: &'info AccountView,
        vault_auth: &'info AccountView,
        fee_vault: &'info AccountView,
        lst_vault: &'info AccountView,
        lst_header: &'info AccountView,
        user_levercoin_ta: &'info AccountView,
        user_lst_ta: &'info AccountView,
        levercoin_mint: &'info AccountView,
        stablecoin_mint: &'info AccountView,
        lst_mint: &'info AccountView,
        sol_usd_pyth_feed: &'info AccountView,
        system_program: &'info AccountView,
        token_program: &'info AccountView,
        associated_token_program: &'info AccountView,
        event_authority: &'info AccountView,
        program: &'info AccountView,
    },
    // swap_lever_to_stable, swap_stable_to_lever
    Rebalance {
        user: &'info AccountView,
        hylo: &'info AccountView,
        sol_usd_pyth_feed: &'info AccountView,
        stablecoin_mint: &'info AccountView,
        stablecoin_auth: &'info AccountView,
        fee_auth: &'info AccountView,
        fee_vault: &'info AccountView,
        user_stablecoin_ta: &'info AccountView,
        levercoin_mint: &'info AccountView,
        levercoin_auth: &'info AccountView,
        user_levercoin_ta: &'info AccountView,
        token_program: &'info AccountView,
        event_authority: &'info AccountView,
        program: &'info AccountView,
    },
}

impl HyloExchangeSwapAccounts<'_> {
    pub const NUM_ACCOUNTS_MINT_STABLECOIN: usize = 19;
    pub const NUM_ACCOUNTS_REDEEM_STABLECOIN: usize = 18;
    pub const NUM_ACCOUNTS_MINT_LEVERCOIN: usize = 20;
    pub const NUM_ACCOUNTS_REDEEM_LEVERCOIN: usize = 19;
    pub const NUM_ACCOUNTS_REBALANCE: usize = 15;
}

fn mint_decimals_exponent(mint: &AccountView) -> Result<u8, ProgramError> {
    // SPL Mint layout stores decimals at byte offset 44.
    let data_len = mint.data_len();
    let data = unsafe { core::slice::from_raw_parts(mint.data_ptr() as *const u8, data_len) };
    let decimals = *data.get(44).ok_or(ProgramError::InvalidAccountData)?;
    Ok(decimals)
}

fn build_instruction_data(
    discriminator: [u8; 8],
    amount_in: u64,
    min_amount_out: u64,
    token_out_decimals_exponent: u8,
) -> [u8; 35] {
    let mut instruction_data = MaybeUninit::<[u8; 35]>::uninit();

    unsafe {
        let ptr = instruction_data.as_mut_ptr() as *mut u8;
        core::ptr::copy_nonoverlapping(discriminator.as_ptr(), ptr, 8);
        core::ptr::copy_nonoverlapping(amount_in.to_le_bytes().as_ptr(), ptr.add(8), 8);

        // set Option<SlippageConfig> to Some
        core::ptr::write(ptr.add(16), 1);

        // https://github.com/hylo-so/sdk/blob/29ae4f3576345f53fed230766c7c44a3a9b59db6/hylo-core/src/slippage_config.rs#L36
        // min_amount_out is already product of floor(expected_token_out * (10_000 - slippage_tolerance) / 10_000)
        // calculate expected_token_out from min_amount_out

        let expected_token_out = min_amount_out * 10_000 / (10_000 - SLIPPAGE_TOLERANCE);
        core::ptr::copy_nonoverlapping(expected_token_out.to_le_bytes().as_ptr(), ptr.add(17), 8);

        core::ptr::write(ptr.add(25) as *mut i8, -(token_out_decimals_exponent as i8));

        core::ptr::copy_nonoverlapping(SLIPPAGE_TOLERANCE.to_le_bytes().as_ptr(), ptr.add(26), 8);

        // 100% in basis points (10_000) has 4 zeroes
        core::ptr::write(ptr.add(34) as *mut i8, -4);

        instruction_data.assume_init()
    }
}

impl<'info> TryFrom<&'info [AccountView]> for HyloExchangeSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let hylo_program = accounts.first().ok_or(ProgramError::NotEnoughAccountKeys)?;

        let len = accounts.len();
        let num_mint = HyloExchangeSwapAccounts::NUM_ACCOUNTS_MINT_STABLECOIN;
        let num_redeem = HyloExchangeSwapAccounts::NUM_ACCOUNTS_REDEEM_STABLECOIN;
        let num_mint_levercoin = HyloExchangeSwapAccounts::NUM_ACCOUNTS_MINT_LEVERCOIN;
        let num_redeem_levercoin = HyloExchangeSwapAccounts::NUM_ACCOUNTS_REDEEM_LEVERCOIN;
        let num_rebalance = HyloExchangeSwapAccounts::NUM_ACCOUNTS_REBALANCE;

        // num_rebalance is the minimum number of accounts required
        if accounts.len() < num_rebalance {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        // CPI accounts start at index 1 (index 0 is protocol program id).
        let i = accounts
            .get(1..)
            .ok_or(ProgramError::NotEnoughAccountKeys)?;

        // MintStablecoin and RedeemLevercoin share the same total length (19).
        // Disambiguate by probing the 5th CPI account (stablecoin_auth).
        let stablecoin_auth_probe = i.get(4).ok_or(ProgramError::NotEnoughAccountKeys)?;

        match len {
            n if n == num_mint && stablecoin_auth_probe.address().eq(&HYUSD_STABLECOIN_AUTH) => {
                Ok(HyloExchangeSwapAccounts {
                    hylo_program,
                    leg: HyloExchangeLeg::MintStablecoin {
                        user: &i[0],
                        hylo: &i[1],
                        fee_auth: &i[2],
                        vault_auth: &i[3],
                        stablecoin_auth: &i[4],
                        fee_vault: &i[5],
                        lst_vault: &i[6],
                        lst_header: &i[7],
                        user_lst_ta: &i[8],
                        user_stablecoin_ta: &i[9],
                        lst_mint: &i[10],
                        stablecoin_mint: &i[11],
                        sol_usd_pyth_feed: &i[12],
                        token_program: &i[13],
                        associated_token_program: &i[14],
                        system_program: &i[15],
                        event_authority: &i[16],
                        program: &i[17],
                    },
                })
            }
            n if n == num_redeem => Ok(HyloExchangeSwapAccounts {
                hylo_program,
                leg: HyloExchangeLeg::RedeemStablecoin {
                    user: &i[0],
                    hylo: &i[1],
                    fee_auth: &i[2],
                    vault_auth: &i[3],
                    fee_vault: &i[4],
                    lst_vault: &i[5],
                    lst_header: &i[6],
                    user_stablecoin_ta: &i[7],
                    user_lst_ta: &i[8],
                    stablecoin_mint: &i[9],
                    lst_mint: &i[10],
                    sol_usd_pyth_feed: &i[11],
                    system_program: &i[12],
                    token_program: &i[13],
                    associated_token_program: &i[14],
                    event_authority: &i[15],
                    program: &i[16],
                },
            }),
            n if n == num_mint_levercoin => Ok(HyloExchangeSwapAccounts {
                hylo_program,
                leg: HyloExchangeLeg::MintLevercoin {
                    user: &i[0],
                    hylo: &i[1],
                    fee_auth: &i[2],
                    vault_auth: &i[3],
                    levercoin_auth: &i[4],
                    fee_vault: &i[5],
                    lst_vault: &i[6],
                    lst_header: &i[7],
                    user_lst_ta: &i[8],
                    user_levercoin_ta: &i[9],
                    lst_mint: &i[10],
                    levercoin_mint: &i[11],
                    stablecoin_mint: &i[12],
                    sol_usd_pyth_feed: &i[13],
                    token_program: &i[14],
                    associated_token_program: &i[15],
                    system_program: &i[16],
                    event_authority: &i[17],
                    program: &i[18],
                },
            }),
            n if n == num_redeem_levercoin => Ok(HyloExchangeSwapAccounts {
                hylo_program,
                leg: HyloExchangeLeg::RedeemLevercoin {
                    user: &i[0],
                    hylo: &i[1],
                    fee_auth: &i[2],
                    vault_auth: &i[3],
                    fee_vault: &i[4],
                    lst_vault: &i[5],
                    lst_header: &i[6],
                    user_levercoin_ta: &i[7],
                    user_lst_ta: &i[8],
                    levercoin_mint: &i[9],
                    stablecoin_mint: &i[10],
                    lst_mint: &i[11],
                    sol_usd_pyth_feed: &i[12],
                    system_program: &i[13],
                    token_program: &i[14],
                    associated_token_program: &i[15],
                    event_authority: &i[16],
                    program: &i[17],
                },
            }),
            n if n == num_rebalance => Ok(HyloExchangeSwapAccounts {
                hylo_program,
                leg: HyloExchangeLeg::Rebalance {
                    user: &i[0],
                    hylo: &i[1],
                    sol_usd_pyth_feed: &i[2],
                    stablecoin_mint: &i[3],
                    stablecoin_auth: &i[4],
                    fee_auth: &i[5],
                    fee_vault: &i[6],
                    user_stablecoin_ta: &i[7],
                    levercoin_mint: &i[8],
                    levercoin_auth: &i[9],
                    user_levercoin_ta: &i[10],
                    token_program: &i[11],
                    event_authority: &i[12],
                    program: &i[13],
                },
            }),
            // more accurately its the accounts passed do not match the expected number of accounts
            _ => Err(ProgramError::NotEnoughAccountKeys),
        }
    }
}

impl<'info> Swap<'info> for HyloExchange {
    type Accounts = HyloExchangeSwapAccounts<'info>;
    type Data = HyloExchangeSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        // Each Hylo leg has a different account count, so we keep fixed-size stack
        // arrays per-leg (no allocation) and share the common invoke logic.
        macro_rules! invoke_hylo {
            (
                $discriminator:expr,
                $token_out_mint:expr,
                $accounts:expr,
                $account_infos:expr
            ) => {{
                let instruction_data = build_instruction_data(
                    $discriminator,
                    in_amount,
                    minimum_out_amount,
                    mint_decimals_exponent($token_out_mint)?,
                );

                let instruction = InstructionView {
                    program_id: &HYLO_EXCHANGE_PROGRAM_ID,
                    accounts: &$accounts,
                    data: &instruction_data,
                };

                invoke_signed(&instruction, &$account_infos, signer_seeds)
            }};
        }

        match &ctx.leg {
            HyloExchangeLeg::MintStablecoin {
                user,
                hylo,
                fee_auth,
                vault_auth,
                stablecoin_auth,
                fee_vault,
                lst_vault,
                lst_header,
                user_lst_ta,
                user_stablecoin_ta,
                lst_mint,
                stablecoin_mint,
                sol_usd_pyth_feed,
                token_program,
                associated_token_program,
                system_program,
                event_authority,
                program,
            } => {
                let accounts = [
                    InstructionAccount::writable_signer(user.address()),
                    InstructionAccount::writable(hylo.address()),
                    InstructionAccount::readonly(fee_auth.address()),
                    InstructionAccount::readonly(vault_auth.address()),
                    InstructionAccount::readonly(stablecoin_auth.address()),
                    InstructionAccount::writable(fee_vault.address()),
                    InstructionAccount::writable(lst_vault.address()),
                    InstructionAccount::readonly(lst_header.address()),
                    InstructionAccount::writable(user_lst_ta.address()),
                    InstructionAccount::writable(user_stablecoin_ta.address()),
                    InstructionAccount::readonly(lst_mint.address()),
                    InstructionAccount::writable(stablecoin_mint.address()),
                    InstructionAccount::readonly(sol_usd_pyth_feed.address()),
                    InstructionAccount::readonly(token_program.address()),
                    InstructionAccount::readonly(associated_token_program.address()),
                    InstructionAccount::readonly(system_program.address()),
                    InstructionAccount::readonly(event_authority.address()),
                    InstructionAccount::readonly(program.address()),
                ];

                let account_infos = [
                    *user,
                    *hylo,
                    *fee_auth,
                    *vault_auth,
                    *stablecoin_auth,
                    *fee_vault,
                    *lst_vault,
                    *lst_header,
                    *user_lst_ta,
                    *user_stablecoin_ta,
                    *lst_mint,
                    *stablecoin_mint,
                    *sol_usd_pyth_feed,
                    *token_program,
                    *associated_token_program,
                    *system_program,
                    *event_authority,
                    *program,
                ];

                let discriminator = MINT_STABLECOIN_DISCRIMINATOR;
                let token_out = stablecoin_mint;

                invoke_hylo!(discriminator, token_out, accounts, account_infos)
            }
            HyloExchangeLeg::RedeemStablecoin {
                user,
                hylo,
                fee_auth,
                vault_auth,
                fee_vault,
                lst_vault,
                lst_header,
                user_stablecoin_ta,
                user_lst_ta,
                stablecoin_mint,
                lst_mint,
                sol_usd_pyth_feed,
                system_program,
                token_program,
                associated_token_program,
                event_authority,
                program,
            } => {
                let accounts = [
                    InstructionAccount::writable_signer(user.address()),
                    InstructionAccount::writable(hylo.address()),
                    InstructionAccount::readonly(fee_auth.address()),
                    InstructionAccount::readonly(vault_auth.address()),
                    InstructionAccount::writable(fee_vault.address()),
                    InstructionAccount::writable(lst_vault.address()),
                    InstructionAccount::readonly(lst_header.address()),
                    InstructionAccount::writable(user_stablecoin_ta.address()),
                    InstructionAccount::writable(user_lst_ta.address()),
                    InstructionAccount::writable(stablecoin_mint.address()),
                    InstructionAccount::readonly(lst_mint.address()),
                    InstructionAccount::readonly(sol_usd_pyth_feed.address()),
                    InstructionAccount::readonly(system_program.address()),
                    InstructionAccount::readonly(token_program.address()),
                    InstructionAccount::readonly(associated_token_program.address()),
                    InstructionAccount::readonly(event_authority.address()),
                    InstructionAccount::readonly(program.address()),
                ];

                let account_infos = [
                    *user,
                    *hylo,
                    *fee_auth,
                    *vault_auth,
                    *fee_vault,
                    *lst_vault,
                    *lst_header,
                    *user_stablecoin_ta,
                    *user_lst_ta,
                    *stablecoin_mint,
                    *lst_mint,
                    *sol_usd_pyth_feed,
                    *system_program,
                    *token_program,
                    *associated_token_program,
                    *event_authority,
                    *program,
                ];

                let discriminator = REDEEM_STABLECOIN_DISCRIMINATOR;
                let token_out = lst_mint;

                invoke_hylo!(discriminator, token_out, accounts, account_infos)
            }
            HyloExchangeLeg::MintLevercoin {
                user,
                hylo,
                fee_auth,
                vault_auth,
                levercoin_auth,
                fee_vault,
                lst_vault,
                lst_header,
                user_lst_ta,
                user_levercoin_ta,
                lst_mint,
                levercoin_mint,
                stablecoin_mint,
                sol_usd_pyth_feed,
                token_program,
                associated_token_program,
                system_program,
                event_authority,
                program,
            } => {
                let accounts = [
                    InstructionAccount::writable_signer(user.address()),
                    InstructionAccount::writable(hylo.address()),
                    InstructionAccount::readonly(fee_auth.address()),
                    InstructionAccount::readonly(vault_auth.address()),
                    InstructionAccount::readonly(levercoin_auth.address()),
                    InstructionAccount::writable(fee_vault.address()),
                    InstructionAccount::writable(lst_vault.address()),
                    InstructionAccount::readonly(lst_header.address()),
                    InstructionAccount::writable(user_lst_ta.address()),
                    InstructionAccount::writable(user_levercoin_ta.address()),
                    InstructionAccount::readonly(lst_mint.address()),
                    InstructionAccount::writable(levercoin_mint.address()),
                    InstructionAccount::readonly(stablecoin_mint.address()),
                    InstructionAccount::readonly(sol_usd_pyth_feed.address()),
                    InstructionAccount::readonly(token_program.address()),
                    InstructionAccount::readonly(associated_token_program.address()),
                    InstructionAccount::readonly(system_program.address()),
                    InstructionAccount::readonly(event_authority.address()),
                    InstructionAccount::readonly(program.address()),
                ];

                let account_infos = [
                    *user,
                    *hylo,
                    *fee_auth,
                    *vault_auth,
                    *levercoin_auth,
                    *fee_vault,
                    *lst_vault,
                    *lst_header,
                    *user_lst_ta,
                    *user_levercoin_ta,
                    *lst_mint,
                    *levercoin_mint,
                    *stablecoin_mint,
                    *sol_usd_pyth_feed,
                    *token_program,
                    *associated_token_program,
                    *system_program,
                    *event_authority,
                    *program,
                ];

                let discriminator = MINT_LEVERCOIN_DISCRIMINATOR;
                let token_out = levercoin_mint;

                invoke_hylo!(discriminator, token_out, accounts, account_infos)
            }
            HyloExchangeLeg::RedeemLevercoin {
                user,
                hylo,
                fee_auth,
                vault_auth,
                fee_vault,
                lst_vault,
                lst_header,
                user_levercoin_ta,
                user_lst_ta,
                levercoin_mint,
                stablecoin_mint,
                lst_mint,
                sol_usd_pyth_feed,
                system_program,
                token_program,
                associated_token_program,
                event_authority,
                program,
            } => {
                let accounts = [
                    InstructionAccount::writable_signer(user.address()),
                    InstructionAccount::writable(hylo.address()),
                    InstructionAccount::readonly(fee_auth.address()),
                    InstructionAccount::readonly(vault_auth.address()),
                    InstructionAccount::writable(fee_vault.address()),
                    InstructionAccount::writable(lst_vault.address()),
                    InstructionAccount::readonly(lst_header.address()),
                    InstructionAccount::writable(user_levercoin_ta.address()),
                    InstructionAccount::writable(user_lst_ta.address()),
                    InstructionAccount::writable(levercoin_mint.address()),
                    InstructionAccount::readonly(stablecoin_mint.address()),
                    InstructionAccount::readonly(lst_mint.address()),
                    InstructionAccount::readonly(sol_usd_pyth_feed.address()),
                    InstructionAccount::readonly(system_program.address()),
                    InstructionAccount::readonly(token_program.address()),
                    InstructionAccount::readonly(associated_token_program.address()),
                    InstructionAccount::readonly(event_authority.address()),
                    InstructionAccount::readonly(program.address()),
                ];

                let account_infos = [
                    *user,
                    *hylo,
                    *fee_auth,
                    *vault_auth,
                    *fee_vault,
                    *lst_vault,
                    *lst_header,
                    *user_levercoin_ta,
                    *user_lst_ta,
                    *levercoin_mint,
                    *stablecoin_mint,
                    *lst_mint,
                    *sol_usd_pyth_feed,
                    *system_program,
                    *token_program,
                    *associated_token_program,
                    *event_authority,
                    *program,
                ];

                let discriminator = REDEEM_LEVERCOIN_DISCRIMINATOR;
                let token_out = lst_mint;

                invoke_hylo!(discriminator, token_out, accounts, account_infos)
            }
            HyloExchangeLeg::Rebalance {
                user,
                hylo,
                sol_usd_pyth_feed,
                stablecoin_mint,
                stablecoin_auth,
                fee_auth,
                fee_vault,
                user_stablecoin_ta,
                levercoin_mint,
                levercoin_auth,
                user_levercoin_ta,
                token_program,
                event_authority,
                program,
            } => {
                let accounts = [
                    InstructionAccount::writable_signer(user.address()),
                    InstructionAccount::writable(hylo.address()),
                    InstructionAccount::readonly(sol_usd_pyth_feed.address()),
                    InstructionAccount::writable(stablecoin_mint.address()),
                    InstructionAccount::readonly(stablecoin_auth.address()),
                    InstructionAccount::readonly(fee_auth.address()),
                    InstructionAccount::writable(fee_vault.address()),
                    InstructionAccount::writable(user_stablecoin_ta.address()),
                    InstructionAccount::writable(levercoin_mint.address()),
                    InstructionAccount::readonly(levercoin_auth.address()),
                    InstructionAccount::writable(user_levercoin_ta.address()),
                    InstructionAccount::readonly(token_program.address()),
                    InstructionAccount::readonly(event_authority.address()),
                    InstructionAccount::readonly(program.address()),
                ];

                let account_infos = [
                    *user,
                    *hylo,
                    *sol_usd_pyth_feed,
                    *stablecoin_mint,
                    *stablecoin_auth,
                    *fee_auth,
                    *fee_vault,
                    *user_stablecoin_ta,
                    *levercoin_mint,
                    *levercoin_auth,
                    *user_levercoin_ta,
                    *token_program,
                    *event_authority,
                    *program,
                ];

                let discriminator = match data.swap_type {
                    SwapType::SwapStableToLever => SWAP_STABLE_TO_LEVER_DISCRIMINATOR,
                    SwapType::SwapLeverToStable => SWAP_LEVER_TO_STABLE_DISCRIMINATOR,
                    _ => return Err(ProgramError::InvalidInstructionData),
                };

                let token_out = match data.swap_type {
                    SwapType::SwapStableToLever => levercoin_mint,
                    SwapType::SwapLeverToStable => stablecoin_mint,
                    _ => return Err(ProgramError::InvalidInstructionData),
                };

                invoke_hylo!(discriminator, token_out, accounts, account_infos)
            }
        }
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
