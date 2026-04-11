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

pub const RISE_PROGRAM_ID: Address = address!("RiseZSHaLdj7pfn1tisUoSdG2i3QcVz9sQKuaRG9rar");

const BUY_WITH_EXACT_CASH_IN_DISCRIMINATOR: [u8; 8] = [53, 248, 95, 20, 54, 162, 146, 247];
const SELL_WITH_EXACT_TOKEN_IN_DISCRIMINATOR: [u8; 8] = [27, 141, 98, 109, 197, 168, 104, 84];

pub const NUM_ACCOUNTS_BUY_WITH_EXACT_CASH_IN: usize = 23;
pub const NUM_ACCOUNTS_SELL_WITH_EXACT_TOKEN_IN: usize = 22;
pub const MAX_NUM_ACCOUNTS: usize = NUM_ACCOUNTS_BUY_WITH_EXACT_CASH_IN;

pub const BUY_DATA_LEN: usize = 88;
pub const SELL_DATA_LEN: usize = 24;
pub const MAX_DATA_LEN: usize = BUY_DATA_LEN;

pub struct Rise;

#[repr(u8)]
pub enum RiseSwapType {
    BuyWithExactCashIn,
    SellWithExactTokenIn,
}

pub struct DecimalSerialized([u8; 16]);

impl TryFrom<&[u8]> for DecimalSerialized {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() != 16 {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self(data.try_into().unwrap()))
    }
}

impl From<&DecimalSerialized> for [u8; 16] {
    fn from(data: &DecimalSerialized) -> Self {
        data.0
    }
}

pub enum RiseSwapData {
    BuyWithExactCashIn {
        new_shoulder_end: u64,
        floor_increase_ratio: DecimalSerialized,
        max_new_floor: DecimalSerialized,
        max_area_shrinkage_tolerance_units: u64,
        min_liq_ratio: DecimalSerialized,
    },
    SellWithExactTokenIn,
}

impl RiseSwapData {
    // 8 - new_shoulder_end
    // 16 - floor_increase_ratio
    // 16 - max_new_floor
    // 8 - max_area_shrinkage_tolerance_units
    // 16 - min_liq_ratio
    pub const DATA_LEN: usize = 64;
}

impl TryFrom<&[u8]> for RiseSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        // if data len is DATA_LEN, it's assumed to be buy
        if data.len() == Self::DATA_LEN {
            let mut offset = 0;
            let new_shoulder_end = u64::from_le_bytes(
                data[offset..offset + 8]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
            );
            offset += 8;
            let floor_increase_ratio = data[offset..offset + 16]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?;
            offset += 16;
            let max_new_floor = data[offset..offset + 16]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?;

            let max_area_shrinkage_tolerance_units = u64::from_le_bytes(
                data[offset..offset + 8]
                    .try_into()
                    .map_err(|_| ProgramError::InvalidInstructionData)?,
            );
            offset += 8;
            let min_liq_ratio = data[offset..offset + 16]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?;

            Ok(Self::BuyWithExactCashIn {
                new_shoulder_end,
                floor_increase_ratio,
                max_new_floor,
                max_area_shrinkage_tolerance_units,
                min_liq_ratio,
            })
        } else {
            Ok(Self::SellWithExactTokenIn)
        }
    }
}

pub struct RiseSwapBaseAccounts<'info> {
    // buyer or seller
    pub signer: &'info AccountView,
    pub tenant: &'info AccountView,
    pub market: &'info AccountView,
    pub cash_escrow: &'info AccountView,
    pub may_tenant: &'info AccountView,
    pub may_market_group: &'info AccountView,
    pub market_meta: &'info AccountView,
    pub may_market: &'info AccountView,
}

pub struct RiseSwapLegAccounts<'info> {
    pub mint_token: &'info AccountView,
    pub mint_main: &'info AccountView,
    pub token_dst: &'info AccountView,
    pub main_src: &'info AccountView,
    pub liq_vault_main: &'info AccountView,
    pub rev_escrow_group: &'info AccountView,
    pub rev_escrow_tenant: &'info AccountView,
    pub token_program_main: &'info AccountView,
    pub token_program: &'info AccountView,
    pub mayflower_program: &'info AccountView,
    pub may_log_account: &'info AccountView,
    pub creator_escrow: &'info AccountView,
    pub team_escrow: &'info AccountView,
}

pub struct RiseSwapAccounts<'info> {
    pub rise_program: &'info AccountView,
    pub base: RiseSwapBaseAccounts<'info>,
    pub tenant_seed: Option<&'info AccountView>,
    pub leg: RiseSwapLegAccounts<'info>,
}

impl RiseSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 23;
}

impl<'info> TryFrom<&'info [AccountView]> for RiseSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let rise_program = accounts.first().ok_or(ProgramError::NotEnoughAccountKeys)?;

        // check if account len is at least the minimum
        if accounts.len() < NUM_ACCOUNTS_SELL_WITH_EXACT_TOKEN_IN {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let i = accounts
            .get(1..)
            .ok_or(ProgramError::NotEnoughAccountKeys)?;

        let base = RiseSwapBaseAccounts {
            signer: &i[0],
            tenant: &i[1],
            market: &i[2],
            cash_escrow: &i[3],
            may_tenant: &i[4],
            may_market_group: &i[5],
            market_meta: &i[6],
            may_market: &i[7],
        };

        match accounts.len() {
            NUM_ACCOUNTS_BUY_WITH_EXACT_CASH_IN => Ok(Self {
                rise_program,
                base,
                tenant_seed: Some(&i[8]),
                leg: RiseSwapLegAccounts {
                    mint_token: &i[9],
                    mint_main: &i[10],
                    token_dst: &i[11],
                    main_src: &i[12],
                    liq_vault_main: &i[13],
                    rev_escrow_group: &i[14],
                    rev_escrow_tenant: &i[15],
                    token_program_main: &i[16],
                    token_program: &i[17],
                    mayflower_program: &i[18],
                    may_log_account: &i[19],
                    creator_escrow: &i[20],
                    team_escrow: &i[21],
                },
            }),
            NUM_ACCOUNTS_SELL_WITH_EXACT_TOKEN_IN => Ok(Self {
                rise_program,
                base,
                tenant_seed: None,
                leg: RiseSwapLegAccounts {
                    mint_token: &i[8],
                    mint_main: &i[9],
                    token_dst: &i[10],
                    main_src: &i[11],
                    liq_vault_main: &i[12],
                    rev_escrow_group: &i[13],
                    rev_escrow_tenant: &i[14],
                    token_program_main: &i[15],
                    token_program: &i[16],
                    mayflower_program: &i[17],
                    may_log_account: &i[18],
                    creator_escrow: &i[19],
                    team_escrow: &i[20],
                },
            }),
            _ => Err(ProgramError::NotEnoughAccountKeys),
        }
    }
}

impl<'info> Swap<'info> for Rise {
    type Accounts = RiseSwapAccounts<'info>;
    type Data = RiseSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let mut account_metas = MaybeUninit::<[InstructionAccount; MAX_NUM_ACCOUNTS]>::uninit();
        let account_metas_ptr = account_metas.as_mut_ptr() as *mut InstructionAccount;
        let mut len = 0;

        unsafe {
            core::ptr::write(
                account_metas_ptr,
                InstructionAccount::writable_signer(ctx.base.signer.address()),
            );
            len += 1;
            core::ptr::write(
                account_metas_ptr.add(len),
                InstructionAccount::writable(ctx.base.tenant.address()),
            );
            len += 1;
            core::ptr::write(
                account_metas_ptr.add(len),
                InstructionAccount::writable(ctx.base.market.address()),
            );
            len += 1;
            core::ptr::write(
                account_metas_ptr.add(len),
                InstructionAccount::writable(ctx.base.cash_escrow.address()),
            );
            len += 1;
            core::ptr::write(
                account_metas_ptr.add(len),
                InstructionAccount::readonly(ctx.base.may_tenant.address()),
            );
            len += 1;
            core::ptr::write(
                account_metas_ptr.add(len),
                InstructionAccount::writable(ctx.base.may_market_group.address()),
            );
            len += 1;
            core::ptr::write(
                account_metas_ptr.add(len),
                InstructionAccount::writable(ctx.base.market_meta.address()),
            );
            len += 1;
            core::ptr::write(
                account_metas_ptr.add(len),
                InstructionAccount::writable(ctx.base.may_market.address()),
            );
            len += 1;

            if let Some(tenant_seed) = ctx.tenant_seed {
                core::ptr::write(
                    account_metas_ptr.add(len),
                    InstructionAccount::readonly(tenant_seed.address()),
                );
                len += 1;
            }

            core::ptr::write(
                account_metas_ptr.add(len),
                InstructionAccount::writable(ctx.leg.mint_token.address()),
            );
            len += 1;
            core::ptr::write(
                account_metas_ptr.add(len),
                InstructionAccount::readonly(ctx.leg.mint_main.address()),
            );
            len += 1;
            core::ptr::write(
                account_metas_ptr.add(len),
                InstructionAccount::writable(ctx.leg.token_dst.address()),
            );
            len += 1;
            core::ptr::write(
                account_metas_ptr.add(len),
                InstructionAccount::writable(ctx.leg.main_src.address()),
            );
            len += 1;
            core::ptr::write(
                account_metas_ptr.add(len),
                InstructionAccount::writable(ctx.leg.liq_vault_main.address()),
            );
            len += 1;
            core::ptr::write(
                account_metas_ptr.add(len),
                InstructionAccount::writable(ctx.leg.rev_escrow_group.address()),
            );
            len += 1;
            core::ptr::write(
                account_metas_ptr.add(len),
                InstructionAccount::writable(ctx.leg.rev_escrow_tenant.address()),
            );
            len += 1;
            core::ptr::write(
                account_metas_ptr.add(len),
                InstructionAccount::readonly(ctx.leg.token_program_main.address()),
            );
            len += 1;
            core::ptr::write(
                account_metas_ptr.add(len),
                InstructionAccount::readonly(ctx.leg.token_program.address()),
            );
            len += 1;
            core::ptr::write(
                account_metas_ptr.add(len),
                InstructionAccount::readonly(ctx.leg.mayflower_program.address()),
            );
            len += 1;
            core::ptr::write(
                account_metas_ptr.add(len),
                InstructionAccount::writable(ctx.leg.may_log_account.address()),
            );
            len += 1;
            core::ptr::write(
                account_metas_ptr.add(len),
                InstructionAccount::writable(ctx.leg.creator_escrow.address()),
            );
            len += 1;
            core::ptr::write(
                account_metas_ptr.add(len),
                InstructionAccount::writable(ctx.leg.team_escrow.address()),
            );
            len += 1;
        }

        let account_metas = unsafe { core::slice::from_raw_parts(account_metas_ptr, len) };

        let mut account_infos = [ctx.base.signer; MAX_NUM_ACCOUNTS];
        let mut len = 1;
        account_infos[len] = ctx.base.tenant;
        len += 1;
        account_infos[len] = ctx.base.market;
        len += 1;
        account_infos[len] = ctx.base.cash_escrow;
        len += 1;
        account_infos[len] = ctx.base.may_tenant;
        len += 1;
        account_infos[len] = ctx.base.may_market_group;
        len += 1;
        account_infos[len] = ctx.base.market_meta;
        len += 1;
        account_infos[len] = ctx.base.may_market;
        len += 1;

        if let Some(tenant_seed) = ctx.tenant_seed {
            account_infos[len] = tenant_seed;
            len += 1;
        }

        account_infos[len] = ctx.leg.mint_token;
        len += 1;
        account_infos[len] = ctx.leg.mint_main;
        len += 1;
        account_infos[len] = ctx.leg.token_dst;
        len += 1;
        account_infos[len] = ctx.leg.main_src;
        len += 1;
        account_infos[len] = ctx.leg.liq_vault_main;
        len += 1;
        account_infos[len] = ctx.leg.rev_escrow_group;
        len += 1;
        account_infos[len] = ctx.leg.rev_escrow_tenant;
        len += 1;
        account_infos[len] = ctx.leg.token_program_main;
        len += 1;
        account_infos[len] = ctx.leg.token_program;
        len += 1;
        account_infos[len] = ctx.leg.mayflower_program;
        len += 1;
        account_infos[len] = ctx.leg.may_log_account;
        len += 1;
        account_infos[len] = ctx.leg.creator_escrow;
        len += 1;
        account_infos[len] = ctx.leg.team_escrow;
        len += 1;

        let account_infos = &account_infos[..len];

        let mut instruction_data = MaybeUninit::<[u8; MAX_DATA_LEN]>::uninit();
        let ix_len = match data {
            RiseSwapData::BuyWithExactCashIn { .. } => BUY_DATA_LEN,
            RiseSwapData::SellWithExactTokenIn => SELL_DATA_LEN,
        };

        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            let discriminator = match data {
                RiseSwapData::BuyWithExactCashIn { .. } => &BUY_WITH_EXACT_CASH_IN_DISCRIMINATOR,
                RiseSwapData::SellWithExactTokenIn => &SELL_WITH_EXACT_TOKEN_IN_DISCRIMINATOR,
            };
            core::ptr::copy_nonoverlapping(discriminator.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(16),
                8,
            );
            if let RiseSwapData::BuyWithExactCashIn {
                new_shoulder_end,
                floor_increase_ratio,
                max_new_floor,
                max_area_shrinkage_tolerance_units,
                min_liq_ratio,
            } = data
            {
                core::ptr::copy_nonoverlapping(
                    new_shoulder_end.to_le_bytes().as_ptr(),
                    ptr.add(24),
                    8,
                );
                core::ptr::copy_nonoverlapping(floor_increase_ratio.0.as_ptr(), ptr.add(32), 16);
                core::ptr::copy_nonoverlapping(max_new_floor.0.as_ptr(), ptr.add(48), 16);
                core::ptr::copy_nonoverlapping(
                    max_area_shrinkage_tolerance_units.to_le_bytes().as_ptr(),
                    ptr.add(64),
                    8,
                );
                core::ptr::copy_nonoverlapping(min_liq_ratio.0.as_ptr(), ptr.add(72), 16);
            }
        }

        let instruction = InstructionView {
            program_id: &RISE_PROGRAM_ID,
            accounts: account_metas,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, ix_len)
            },
        };

        invoke_signed_with_bounds::<MAX_NUM_ACCOUNTS, _>(&instruction, account_infos, signer_seeds)
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
