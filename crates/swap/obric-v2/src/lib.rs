#![no_std]

use {
    beethoven_core::Swap,
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const OBRIC_V2_PROGRAM_ID: Address =
    Address::from_str_const("obriQD1zbpyLz95G5n7nJe6a4DPjpFwa5XYPoNm113y");

const SWAP2_DISCRIMINATOR: [u8; 8] = [65, 75, 63, 76, 235, 91, 91, 136];

pub struct ObricV2;

pub struct ObricV2SwapData {
    pub x_to_y: u8,
}

impl ObricV2SwapData {
    pub const DATA_LEN: usize = 1;
}

impl TryFrom<&[u8]> for ObricV2SwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self { x_to_y: data[0] })
    }
}

impl ObricV2SwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 13;
}

pub struct ObricV2SwapAccounts<'info> {
    pub obric_v2_program: &'info AccountView,
    pub market: &'info AccountView,
    pub second_ref_oracle: &'info AccountView,
    pub third_ref_oracle: &'info AccountView,
    pub reserve_x: &'info AccountView,
    pub reserve_y: &'info AccountView,
    pub user_ta_x: &'info AccountView,
    pub user_ta_y: &'info AccountView,
    pub ref_oracle: &'info AccountView,
    pub x_price_feed: &'info AccountView,
    pub y_price_feed: &'info AccountView,
    pub user: &'info AccountView,
    pub token_program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for ObricV2SwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [obric_v2_program, market, second_ref_oracle, third_ref_oracle, reserve_x, reserve_y, user_ta_x, user_ta_y, ref_oracle, x_price_feed, y_price_feed, user, token_program, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(ObricV2SwapAccounts {
            obric_v2_program,
            market,
            second_ref_oracle,
            third_ref_oracle,
            reserve_x,
            reserve_y,
            user_ta_x,
            user_ta_y,
            ref_oracle,
            x_price_feed,
            y_price_feed,
            user,
            token_program,
        })
    }
}

impl<'info> Swap<'info> for ObricV2 {
    type Accounts = ObricV2SwapAccounts<'info>;
    type Data = ObricV2SwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::writable(ctx.market.address()),
            InstructionAccount::readonly(ctx.second_ref_oracle.address()),
            InstructionAccount::readonly(ctx.third_ref_oracle.address()),
            InstructionAccount::writable(ctx.reserve_x.address()),
            InstructionAccount::writable(ctx.reserve_y.address()),
            InstructionAccount::writable(ctx.user_ta_x.address()),
            InstructionAccount::writable(ctx.user_ta_y.address()),
            InstructionAccount::writable(ctx.ref_oracle.address()),
            InstructionAccount::readonly(ctx.x_price_feed.address()),
            InstructionAccount::readonly(ctx.y_price_feed.address()),
            InstructionAccount::writable_signer(ctx.user.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
        ];

        let account_infos = [
            ctx.market,
            ctx.second_ref_oracle,
            ctx.third_ref_oracle,
            ctx.reserve_x,
            ctx.reserve_y,
            ctx.user_ta_x,
            ctx.user_ta_y,
            ctx.ref_oracle,
            ctx.x_price_feed,
            ctx.y_price_feed,
            ctx.user,
            ctx.token_program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 25]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(SWAP2_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::write(ptr.add(8), data.x_to_y);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(9), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(17),
                8,
            );
        }

        let instruction = InstructionView {
            program_id: &OBRIC_V2_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe { instruction_data.assume_init_ref() },
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
