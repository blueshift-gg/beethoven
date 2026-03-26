#![no_std]

use {
    beethoven_core::{Swap, SwapTokenAccounts},
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const METEORA_DLMM_PROGRAM_ID: Address =
    Address::from_str_const("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");

const SWAP2_DISCRIMINATOR: [u8; 8] = [65, 75, 63, 76, 235, 91, 91, 136];
const FIXED_ACCOUNT_COUNT: usize = 17;
const MAX_BIN_ARRAY_ACCOUNTS: usize = 3;
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

fn is_valid_bin_array_account(account: &AccountView) -> bool {
    account.owned_by(&METEORA_DLMM_PROGRAM_ID) && account.is_writable() && !account.executable()
}

impl<'info> MeteoraDlmmSwapAccounts<'info> {
    fn try_from_parts(
        accounts: &'info [AccountView],
        bin_array_accounts: &'info [AccountView],
    ) -> Result<Self, ProgramError> {
        if bin_array_accounts.len() > MAX_BIN_ARRAY_ACCOUNTS {
            return Err(ProgramError::InvalidAccountData);
        }

        if bin_array_accounts
            .iter()
            .any(|account| !is_valid_bin_array_account(account))
        {
            return Err(ProgramError::InvalidAccountData);
        }

        let fixed_accounts = accounts
            .get(..FIXED_ACCOUNT_COUNT)
            .ok_or(ProgramError::NotEnoughAccountKeys)?;

        let [meteora_dlmm_program, lb_pair, bin_array_bitmap_extension, reserve_x, reserve_y, user_token_in, user_token_out, token_x_mint, token_y_mint, oracle, host_fee_in, user, token_x_program, token_y_program, memo_program, event_authority, program] =
            fixed_accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

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

    pub fn try_from_with_bin_array_len(
        accounts: &'info [AccountView],
        bin_array_accounts_len: usize,
    ) -> Result<Self, ProgramError> {
        if bin_array_accounts_len > MAX_BIN_ARRAY_ACCOUNTS {
            return Err(ProgramError::InvalidAccountData);
        }

        let total_accounts = FIXED_ACCOUNT_COUNT
            .checked_add(bin_array_accounts_len)
            .ok_or(ProgramError::InvalidInstructionData)?;

        if accounts.len() < total_accounts {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let bin_array_accounts = &accounts[FIXED_ACCOUNT_COUNT..total_accounts];
        Self::try_from_parts(accounts, bin_array_accounts)
    }
}

impl<'info> TryFrom<&'info [AccountView]> for MeteoraDlmmSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < FIXED_ACCOUNT_COUNT {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let trailing_accounts = &accounts[FIXED_ACCOUNT_COUNT..];
        let bin_array_accounts_len = trailing_accounts
            .iter()
            .take(MAX_BIN_ARRAY_ACCOUNTS)
            .take_while(|account| is_valid_bin_array_account(account))
            .count();

        let bin_array_accounts =
            &accounts[FIXED_ACCOUNT_COUNT..FIXED_ACCOUNT_COUNT + bin_array_accounts_len];

        Self::try_from_parts(accounts, bin_array_accounts)
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

        match ctx.bin_array_accounts {
            [] => invoke_with_accounts(
                [
                    InstructionAccount::writable(ctx.lb_pair.address()),
                    InstructionAccount::readonly(ctx.bin_array_bitmap_extension.address()),
                    InstructionAccount::writable(ctx.reserve_x.address()),
                    InstructionAccount::writable(ctx.reserve_y.address()),
                    InstructionAccount::writable(ctx.user_token_in.address()),
                    InstructionAccount::writable(ctx.user_token_out.address()),
                    InstructionAccount::readonly(ctx.token_x_mint.address()),
                    InstructionAccount::readonly(ctx.token_y_mint.address()),
                    InstructionAccount::writable(ctx.oracle.address()),
                    InstructionAccount::writable(ctx.host_fee_in.address()),
                    InstructionAccount::readonly_signer(ctx.user.address()),
                    InstructionAccount::readonly(ctx.token_x_program.address()),
                    InstructionAccount::readonly(ctx.token_y_program.address()),
                    InstructionAccount::readonly(ctx.memo_program.address()),
                    InstructionAccount::readonly(ctx.event_authority.address()),
                    InstructionAccount::readonly(ctx.program.address()),
                ],
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
                ],
                unsafe { instruction_data.assume_init_ref() },
                signer_seeds,
            ),
            [bin_array_0] => invoke_with_accounts(
                [
                    InstructionAccount::writable(ctx.lb_pair.address()),
                    InstructionAccount::readonly(ctx.bin_array_bitmap_extension.address()),
                    InstructionAccount::writable(ctx.reserve_x.address()),
                    InstructionAccount::writable(ctx.reserve_y.address()),
                    InstructionAccount::writable(ctx.user_token_in.address()),
                    InstructionAccount::writable(ctx.user_token_out.address()),
                    InstructionAccount::readonly(ctx.token_x_mint.address()),
                    InstructionAccount::readonly(ctx.token_y_mint.address()),
                    InstructionAccount::writable(ctx.oracle.address()),
                    InstructionAccount::writable(ctx.host_fee_in.address()),
                    InstructionAccount::readonly_signer(ctx.user.address()),
                    InstructionAccount::readonly(ctx.token_x_program.address()),
                    InstructionAccount::readonly(ctx.token_y_program.address()),
                    InstructionAccount::readonly(ctx.memo_program.address()),
                    InstructionAccount::readonly(ctx.event_authority.address()),
                    InstructionAccount::readonly(ctx.program.address()),
                    InstructionAccount::writable(bin_array_0.address()),
                ],
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
                    bin_array_0,
                ],
                unsafe { instruction_data.assume_init_ref() },
                signer_seeds,
            ),
            [bin_array_0, bin_array_1] => invoke_with_accounts(
                [
                    InstructionAccount::writable(ctx.lb_pair.address()),
                    InstructionAccount::readonly(ctx.bin_array_bitmap_extension.address()),
                    InstructionAccount::writable(ctx.reserve_x.address()),
                    InstructionAccount::writable(ctx.reserve_y.address()),
                    InstructionAccount::writable(ctx.user_token_in.address()),
                    InstructionAccount::writable(ctx.user_token_out.address()),
                    InstructionAccount::readonly(ctx.token_x_mint.address()),
                    InstructionAccount::readonly(ctx.token_y_mint.address()),
                    InstructionAccount::writable(ctx.oracle.address()),
                    InstructionAccount::writable(ctx.host_fee_in.address()),
                    InstructionAccount::readonly_signer(ctx.user.address()),
                    InstructionAccount::readonly(ctx.token_x_program.address()),
                    InstructionAccount::readonly(ctx.token_y_program.address()),
                    InstructionAccount::readonly(ctx.memo_program.address()),
                    InstructionAccount::readonly(ctx.event_authority.address()),
                    InstructionAccount::readonly(ctx.program.address()),
                    InstructionAccount::writable(bin_array_0.address()),
                    InstructionAccount::writable(bin_array_1.address()),
                ],
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
                    bin_array_0,
                    bin_array_1,
                ],
                unsafe { instruction_data.assume_init_ref() },
                signer_seeds,
            ),
            [bin_array_0, bin_array_1, bin_array_2] => invoke_with_accounts(
                [
                    InstructionAccount::writable(ctx.lb_pair.address()),
                    InstructionAccount::readonly(ctx.bin_array_bitmap_extension.address()),
                    InstructionAccount::writable(ctx.reserve_x.address()),
                    InstructionAccount::writable(ctx.reserve_y.address()),
                    InstructionAccount::writable(ctx.user_token_in.address()),
                    InstructionAccount::writable(ctx.user_token_out.address()),
                    InstructionAccount::readonly(ctx.token_x_mint.address()),
                    InstructionAccount::readonly(ctx.token_y_mint.address()),
                    InstructionAccount::writable(ctx.oracle.address()),
                    InstructionAccount::writable(ctx.host_fee_in.address()),
                    InstructionAccount::readonly_signer(ctx.user.address()),
                    InstructionAccount::readonly(ctx.token_x_program.address()),
                    InstructionAccount::readonly(ctx.token_y_program.address()),
                    InstructionAccount::readonly(ctx.memo_program.address()),
                    InstructionAccount::readonly(ctx.event_authority.address()),
                    InstructionAccount::readonly(ctx.program.address()),
                    InstructionAccount::writable(bin_array_0.address()),
                    InstructionAccount::writable(bin_array_1.address()),
                    InstructionAccount::writable(bin_array_2.address()),
                ],
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
                    bin_array_0,
                    bin_array_1,
                    bin_array_2,
                ],
                unsafe { instruction_data.assume_init_ref() },
                signer_seeds,
            ),
            _ => Err(ProgramError::InvalidAccountData),
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

fn invoke_with_accounts<const N: usize>(
    accounts: [InstructionAccount; N],
    account_infos: [&AccountView; N],
    instruction_data: &[u8; SWAP2_DATA_LEN],
    signer_seeds: &[Signer],
) -> ProgramResult {
    let instruction = InstructionView {
        program_id: &METEORA_DLMM_PROGRAM_ID,
        accounts: &accounts,
        data: instruction_data,
    };

    invoke_signed(&instruction, &account_infos, signer_seeds)
}
