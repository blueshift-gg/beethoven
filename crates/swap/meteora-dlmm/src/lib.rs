#![no_std]

use {
    beethoven_core::{Swap, SwapTokenAccounts},
    core::{array, mem::MaybeUninit},
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const METEORA_DLMM_PROGRAM_ID: Address =
    Address::from_str_const("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");

const SWAP2_DISCRIMINATOR: [u8; 8] = [65, 75, 63, 76, 235, 91, 91, 136];
const FIXED_ACCOUNT_COUNT: usize = 17;
const CPI_FIXED_ACCOUNT_COUNT: usize = 16;
const MAX_BIN_ARRAY_ACCOUNTS: usize = 5;
const MAX_CPI_ACCOUNTS: usize = CPI_FIXED_ACCOUNT_COUNT + MAX_BIN_ARRAY_ACCOUNTS;
const SWAP2_DATA_LEN: usize = 28;
const EMPTY_REMAINING_ACCOUNTS_INFO_LEN: [u8; 4] = 0u32.to_le_bytes();

pub struct MeteoraDlmm;

pub struct MeteoraDlmmSwapAccounts<'info> {
    pub meteora_dlmm_program: &'info AccountView,
    pub lb_pair: &'info AccountView,
    pub bin_array_bitmap_extension: &'info AccountView,
    pub reserve_x: &'info AccountView,
    pub reserve_y: &'info AccountView,
    pub user_token_in: &'info AccountView,
    pub user_token_out: &'info AccountView,
    pub token_x_mint: &'info AccountView,
    pub token_y_mint: &'info AccountView,
    pub oracle: &'info AccountView,
    pub host_fee_in: &'info AccountView,
    pub user: &'info AccountView,
    pub token_x_program: &'info AccountView,
    pub token_y_program: &'info AccountView,
    pub memo_program: &'info AccountView,
    pub event_authority: &'info AccountView,
    pub program: &'info AccountView,
    pub bin_array_accounts: &'info [AccountView],
}

impl MeteoraDlmmSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = FIXED_ACCOUNT_COUNT;

    pub fn consumed_accounts_len(&self) -> usize {
        FIXED_ACCOUNT_COUNT + self.bin_array_accounts.len()
    }
}

fn optional_instruction_account(account: &AccountView) -> InstructionAccount<'_> {
    InstructionAccount::new(account.address(), account.is_writable(), false)
}

impl<'info> TryFrom<&'info [AccountView]> for MeteoraDlmmSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [meteora_dlmm_program, lb_pair, bin_array_bitmap_extension, reserve_x, reserve_y, user_token_in, user_token_out, token_x_mint, token_y_mint, oracle, host_fee_in, user, token_x_program, token_y_program, memo_program, event_authority, program, bin_array_accounts @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if bin_array_accounts.len() > MAX_BIN_ARRAY_ACCOUNTS {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            meteora_dlmm_program,
            lb_pair,
            bin_array_bitmap_extension,
            reserve_x,
            reserve_y,
            user_token_in,
            user_token_out,
            token_x_mint,
            token_y_mint,
            oracle,
            host_fee_in,
            user,
            token_x_program,
            token_y_program,
            memo_program,
            event_authority,
            program,
            bin_array_accounts,
        })
    }
}

impl<'info> Swap<'info> for MeteoraDlmm {
    type Accounts = MeteoraDlmmSwapAccounts<'info>;
    type Data = ();

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        _data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let instruction_data = build_swap_instruction_data(in_amount, minimum_out_amount);
        if ctx.bin_array_accounts.len() > MAX_BIN_ARRAY_ACCOUNTS {
            return Err(ProgramError::InvalidAccountData);
        }

        let mut instruction_accounts: [InstructionAccount<'_>; MAX_CPI_ACCOUNTS] =
            array::from_fn(|_| InstructionAccount::readonly(ctx.lb_pair.address()));
        instruction_accounts[..CPI_FIXED_ACCOUNT_COUNT]
            .clone_from_slice(&fixed_instruction_accounts(ctx));

        let mut account_views = [ctx.lb_pair; MAX_CPI_ACCOUNTS];
        account_views[..CPI_FIXED_ACCOUNT_COUNT].copy_from_slice(&fixed_account_views(ctx));

        for (index, account) in ctx.bin_array_accounts.iter().enumerate() {
            let slot = CPI_FIXED_ACCOUNT_COUNT + index;
            instruction_accounts[slot] = InstructionAccount::writable(account.address());
            account_views[slot] = account;
        }

        let instruction_account_count = CPI_FIXED_ACCOUNT_COUNT + ctx.bin_array_accounts.len();
        let instruction = InstructionView {
            program_id: &METEORA_DLMM_PROGRAM_ID,
            accounts: &instruction_accounts[..instruction_account_count],
            data: unsafe { instruction_data.assume_init_ref() },
        };

        invoke_signed_with_bounds::<MAX_CPI_ACCOUNTS>(
            &instruction,
            &account_views[..instruction_account_count],
            signer_seeds,
        )
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

fn build_swap_instruction_data(
    in_amount: u64,
    minimum_out_amount: u64,
) -> MaybeUninit<[u8; SWAP2_DATA_LEN]> {
    let mut instruction_data = MaybeUninit::<[u8; SWAP2_DATA_LEN]>::uninit();

    unsafe {
        let ptr = instruction_data.as_mut_ptr() as *mut u8;
        core::ptr::copy_nonoverlapping(SWAP2_DISCRIMINATOR.as_ptr(), ptr, 8);
        core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
        core::ptr::copy_nonoverlapping(minimum_out_amount.to_le_bytes().as_ptr(), ptr.add(16), 8);
        core::ptr::copy_nonoverlapping(EMPTY_REMAINING_ACCOUNTS_INFO_LEN.as_ptr(), ptr.add(24), 4);
    }

    instruction_data
}

impl<'info> SwapTokenAccounts<'info> for MeteoraDlmm {
    type Accounts = MeteoraDlmmSwapAccounts<'info>;
    type Data = ();

    fn token_accounts(
        ctx: &Self::Accounts,
        _data: &Self::Data,
    ) -> (&'info AccountView, &'info AccountView) {
        (ctx.user_token_in, ctx.user_token_out)
    }
}

fn fixed_instruction_accounts<'a>(
    ctx: &'a MeteoraDlmmSwapAccounts<'a>,
) -> [InstructionAccount<'a>; CPI_FIXED_ACCOUNT_COUNT] {
    [
        InstructionAccount::writable(ctx.lb_pair.address()),
        optional_instruction_account(ctx.bin_array_bitmap_extension),
        InstructionAccount::writable(ctx.reserve_x.address()),
        InstructionAccount::writable(ctx.reserve_y.address()),
        InstructionAccount::writable(ctx.user_token_in.address()),
        InstructionAccount::writable(ctx.user_token_out.address()),
        InstructionAccount::readonly(ctx.token_x_mint.address()),
        InstructionAccount::readonly(ctx.token_y_mint.address()),
        InstructionAccount::writable(ctx.oracle.address()),
        optional_instruction_account(ctx.host_fee_in),
        InstructionAccount::readonly_signer(ctx.user.address()),
        InstructionAccount::readonly(ctx.token_x_program.address()),
        InstructionAccount::readonly(ctx.token_y_program.address()),
        InstructionAccount::readonly(ctx.memo_program.address()),
        InstructionAccount::readonly(ctx.event_authority.address()),
        InstructionAccount::readonly(ctx.program.address()),
    ]
}

fn fixed_account_views<'a>(
    ctx: &'a MeteoraDlmmSwapAccounts<'a>,
) -> [&'a AccountView; CPI_FIXED_ACCOUNT_COUNT] {
    [
        ctx.lb_pair,
        ctx.bin_array_bitmap_extension,
        ctx.reserve_x,
        ctx.reserve_y,
        ctx.user_token_in,
        ctx.user_token_out,
        ctx.token_x_mint,
        ctx.token_y_mint,
        ctx.oracle,
        ctx.host_fee_in,
        ctx.user,
        ctx.token_x_program,
        ctx.token_y_program,
        ctx.memo_program,
        ctx.event_authority,
        ctx.program,
    ]
}
