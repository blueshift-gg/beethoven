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

pub const PHOENIX_LEGACY_PROGRAM_ID: Address =
    address!("PhoeNiXZ8ByJGLkxNfZRnkUfjvmuYqLR89jjFHGqdXY");

const SWAP_INSTRUCTION_DISCRIMINANT: u8 = 0;

/// Max CPI payload: Phoenix swap discriminant (1 byte) + Borsh IOC order packet (up to 88 bytes).
/// Borsh `Option` encodes `None` as a single `0u8`, not tag + 8 zero bytes.
const MAX_DATA_LEN: usize = 89;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Side {
    Bid = 0,
    Ask = 1,
}

#[repr(u8)]
pub enum OrderPacketType {
    PostOnly = 0,
    Limit = 1,
    ImmediateOrCancel = 2,
}

pub struct PhoenixLegacy;

/// Extra swap data after the Beethoven 16-byte header (`in_amount`, `minimum_out_amount`).
///
/// Layout is fixed-size; option-like fields use a tag byte (`0` = none, `1` = some) followed by
/// eight bytes (ignored when tag is `0`).
///
/// `in_amount` maps to `num_base_lots` when [`Side::Ask`] and to `num_quote_lots` when [`Side::Bid`].
/// `minimum_out_amount` maps to `min_quote_lots_to_fill` when [`Side::Ask`] and to
/// `min_base_lots_to_fill` when [`Side::Bid`]. The other min-* field is set to `0`.
///
/// `max_counterpart_lots` is `num_quote_lots` for [`Side::Ask`] and `num_base_lots` for [`Side::Bid`]
/// (the IOC cap on the asset that is not driven by `in_amount`).
pub struct PhoenixLegacySwapData {
    pub side: Side,
    pub price_in_ticks: Option<u64>,
    pub max_counterpart_lots: u64,
    pub self_trade_behavior: u8,
    pub match_limit: Option<u64>,
    pub client_order_id: u128,
    pub use_only_deposited_funds: bool,
    pub last_valid_slot: Option<u64>,
    pub last_valid_unix_timestamp_in_seconds: Option<u64>,
}

impl PhoenixLegacySwapData {
    pub const DATA_LEN: usize = 63;
}

fn read_opt_u64(data: &[u8], off: &mut usize) -> Result<Option<u64>, ProgramError> {
    let tag = *data.get(*off).ok_or(ProgramError::InvalidInstructionData)?;
    *off += 1;
    let raw = data
        .get(*off..*off + 8)
        .ok_or(ProgramError::InvalidInstructionData)?;
    *off += 8;
    let v = u64::from_le_bytes(raw.try_into().unwrap());
    Ok(if tag == 0 { None } else { Some(v) })
}

#[inline]
unsafe fn write_u8(ptr: *mut u8, len: &mut usize, byte: u8) {
    core::ptr::write(ptr.add(*len), byte);
    *len += 1;
}

#[inline]
unsafe fn write_u64_le(ptr: *mut u8, len: &mut usize, value: u64) {
    core::ptr::copy_nonoverlapping(value.to_le_bytes().as_ptr(), ptr.add(*len), 8);
    *len += 8;
}

#[inline]
unsafe fn write_borsh_opt_u64(ptr: *mut u8, len: &mut usize, v: Option<u64>) {
    let p = ptr.add(*len);
    match v {
        None => {
            core::ptr::write(p, 0);
            *len += 1;
        }
        Some(x) => {
            core::ptr::write(p, 1);
            core::ptr::copy_nonoverlapping(x.to_le_bytes().as_ptr(), p.add(1), 8);
            *len += 9;
        }
    }
}

impl TryFrom<&[u8]> for PhoenixLegacySwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }
        let mut off = 0usize;
        let side = match *data.get(off).ok_or(ProgramError::InvalidInstructionData)? {
            0 => Side::Bid,
            1 => Side::Ask,
            _ => return Err(ProgramError::InvalidInstructionData),
        };
        off += 1;
        let price_in_ticks = read_opt_u64(data, &mut off)?;
        let max_counterpart_lots = u64::from_le_bytes(
            data.get(off..off + 8)
                .ok_or(ProgramError::InvalidInstructionData)?
                .try_into()
                .unwrap(),
        );
        off += 8;
        let self_trade_behavior = *data.get(off).ok_or(ProgramError::InvalidInstructionData)?;
        off += 1;
        let match_limit = read_opt_u64(data, &mut off)?;
        let lo = u64::from_le_bytes(
            data.get(off..off + 8)
                .ok_or(ProgramError::InvalidInstructionData)?
                .try_into()
                .unwrap(),
        );
        off += 8;
        let hi = u64::from_le_bytes(
            data.get(off..off + 8)
                .ok_or(ProgramError::InvalidInstructionData)?
                .try_into()
                .unwrap(),
        );
        off += 8;
        let client_order_id = ((hi as u128) << 64) | (lo as u128);
        let use_only_deposited_funds =
            *data.get(off).ok_or(ProgramError::InvalidInstructionData)? != 0;
        off += 1;
        let last_valid_slot = read_opt_u64(data, &mut off)?;
        let last_valid_unix_timestamp_in_seconds = read_opt_u64(data, &mut off)?;
        Ok(Self {
            side,
            price_in_ticks,
            max_counterpart_lots,
            self_trade_behavior,
            match_limit,
            client_order_id,
            use_only_deposited_funds,
            last_valid_slot,
            last_valid_unix_timestamp_in_seconds,
        })
    }
}

impl PhoenixLegacySwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 9;
}

pub struct PhoenixLegacySwapAccounts<'info> {
    pub phoenix_program: &'info AccountView,
    pub log_authority: &'info AccountView,
    pub market: &'info AccountView,
    pub trader: &'info AccountView,
    pub base_account: &'info AccountView,
    pub quote_account: &'info AccountView,
    pub base_vault: &'info AccountView,
    pub quote_vault: &'info AccountView,
    pub token_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for PhoenixLegacySwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [phoenix_program, log_authority, market, trader, base_account, quote_account, base_vault, quote_vault, token_program, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(PhoenixLegacySwapAccounts {
            phoenix_program,
            log_authority,
            market,
            trader,
            base_account,
            quote_account,
            base_vault,
            quote_vault,
            token_program,
        })
    }
}

impl<'info> Swap<'info> for PhoenixLegacy {
    type Accounts = PhoenixLegacySwapAccounts<'info>;
    type Data = PhoenixLegacySwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::readonly(ctx.phoenix_program.address()),
            InstructionAccount::readonly(ctx.log_authority.address()),
            InstructionAccount::writable(ctx.market.address()),
            InstructionAccount::readonly_signer(ctx.trader.address()),
            InstructionAccount::writable(ctx.base_account.address()),
            InstructionAccount::writable(ctx.quote_account.address()),
            InstructionAccount::writable(ctx.base_vault.address()),
            InstructionAccount::writable(ctx.quote_vault.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
        ];

        let account_infos = [
            ctx.phoenix_program,
            ctx.log_authority,
            ctx.market,
            ctx.trader,
            ctx.base_account,
            ctx.quote_account,
            ctx.base_vault,
            ctx.quote_vault,
            ctx.token_program,
        ];

        let (num_base_lots, num_quote_lots, min_base_lots_to_fill, min_quote_lots_to_fill) =
            match data.side {
                Side::Ask => (
                    in_amount,
                    data.max_counterpart_lots,
                    0u64,
                    minimum_out_amount,
                ),
                Side::Bid => (
                    data.max_counterpart_lots,
                    in_amount,
                    minimum_out_amount,
                    0u64,
                ),
            };

        let mut instruction_data = MaybeUninit::<[u8; MAX_DATA_LEN]>::uninit();
        let mut len = 0usize;

        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            write_u8(ptr, &mut len, SWAP_INSTRUCTION_DISCRIMINANT);
            write_u8(ptr, &mut len, OrderPacketType::ImmediateOrCancel as u8);
            write_u8(ptr, &mut len, data.side as u8);
            write_borsh_opt_u64(ptr, &mut len, data.price_in_ticks);
            write_u64_le(ptr, &mut len, num_base_lots);
            write_u64_le(ptr, &mut len, num_quote_lots);
            write_u64_le(ptr, &mut len, min_base_lots_to_fill);
            write_u64_le(ptr, &mut len, min_quote_lots_to_fill);
            write_u8(ptr, &mut len, data.self_trade_behavior);
            write_borsh_opt_u64(ptr, &mut len, data.match_limit);
            let cid = data.client_order_id;
            write_u64_le(ptr, &mut len, cid as u64);
            write_u64_le(ptr, &mut len, (cid >> 64) as u64);
            write_u8(ptr, &mut len, data.use_only_deposited_funds as u8);
            write_borsh_opt_u64(ptr, &mut len, data.last_valid_slot);
            write_borsh_opt_u64(ptr, &mut len, data.last_valid_unix_timestamp_in_seconds);
        }

        let instruction = InstructionView {
            program_id: &PHOENIX_LEGACY_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, len)
            },
        };

        invoke_signed(&instruction, &account_infos, signer_seeds)
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
