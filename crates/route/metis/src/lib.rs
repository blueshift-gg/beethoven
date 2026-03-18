#![no_std]

use {
    beethoven_core::Route,
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

// JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4
pub const JUPITER_PROGRAM_ID: Address = Address::new_from_array([
    4, 121, 213, 91, 242, 49, 192, 110, 238, 116, 197, 110, 206, 104, 21, 7, 253, 177, 178, 222,
    163, 244, 142, 81, 2, 177, 205, 162, 86, 188, 19, 143,
]);
// D8cy77BBepLMngZx6ZukaTff5hCt1HrWyKk3Hnd9oitf
pub const JUPITER_EVENT_AUTHORITY: Address = Address::new_from_array([
    180, 63, 250, 39, 245, 215, 246, 74, 116, 192, 155, 31, 41, 88, 121, 222, 75, 9, 171, 54, 223,
    201, 221, 81, 75, 50, 26, 167, 179, 140, 229, 232,
]);

// Instruction Discriminators
pub const EXACT_OUT_ROUTE_DISCRIMINATOR: [u8; 8] = [208, 51, 239, 151, 123, 43, 237, 92];
pub const ROUTE_DISCRIMINATOR: [u8; 8] = [229, 23, 203, 151, 122, 227, 173, 42];
pub const SHARED_ACCOUNTS_EXACT_OUT_ROUTE_DISCRIMINATOR: [u8; 8] =
    [176, 209, 105, 168, 154, 125, 69, 62];
pub const SHARED_ACCOUNTS_ROUTE_DISCRIMINATOR: [u8; 8] = [193, 32, 155, 51, 65, 214, 156, 129];
const MAX_TOTAL_ACCOUNTS: usize = 255;

pub struct Metis;

pub struct MetisRouteAccounts<'info> {
    pub token_program: &'info AccountView,
    pub token_account_authority: &'info AccountView,
    pub source_token_account: &'info AccountView,
    pub destination_token_account: &'info AccountView,
    pub source_mint: &'info AccountView,
    pub destination_mint: &'info AccountView,
    pub event_authority: &'info AccountView,
    pub jupiter_program: &'info AccountView,
}

impl MetisRouteAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 8;
}

impl<'info> TryFrom<&'info [AccountView]> for MetisRouteAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < MetisRouteAccounts::NUM_ACCOUNTS {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [jupiter_program, token_program, token_account_authority, source_token_account, destination_token_account, source_mint, destination_mint, event_authority, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(MetisRouteAccounts {
            token_program,
            token_account_authority,
            source_token_account,
            destination_token_account,
            source_mint,
            destination_mint,
            event_authority,
            jupiter_program,
        })
    }
}

impl<'info> Route<'info> for Metis {
    type Accounts = MetisRouteAccounts<'info>;

    fn check_amount_and_slippage(
        swap_data: &[u8],
        amount: u64,
        slippage_bps: u16,
    ) -> Result<(), ProgramError> {
        let swap_data_length = swap_data.len();

        if swap_data.len() < 8 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let bps_offset = swap_data_length - size_of::<u16>() - size_of::<u8>();

        if slippage_bps
            != u16::from_le_bytes(
                swap_data[bps_offset..bps_offset + size_of::<u16>()]
                    .try_into()
                    .unwrap(),
            )
        {
            return Err(ProgramError::InvalidInstructionData);
        }

        let amount_offset = bps_offset - size_of::<u64>() - size_of::<u64>();

        if amount
            != u64::from_le_bytes(
                swap_data[amount_offset..amount_offset + size_of::<u64>()]
                    .try_into()
                    .unwrap(),
            )
        {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(())
    }

    fn route_signed(
        ctx: &Self::Accounts,
        swap_data: &[u8],
        remaining_accounts: &[AccountView],
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        if *ctx.jupiter_program.address() != JUPITER_PROGRAM_ID {
            return Err(ProgramError::IncorrectProgramId);
        }

        if *ctx.event_authority.address() != JUPITER_EVENT_AUTHORITY {
            return Err(ProgramError::IncorrectAuthority);
        }

        let mut accounts = MaybeUninit::<[InstructionAccount; MAX_TOTAL_ACCOUNTS]>::uninit();
        let accounts_ptr = accounts.as_mut_ptr() as *mut InstructionAccount;
        let mut account_views = MaybeUninit::<[&AccountView; MAX_TOTAL_ACCOUNTS]>::uninit();
        let account_views_ptr = account_views.as_mut_ptr() as *mut &AccountView;
        let mut account_count = 0usize;
        let mut view_count = 0usize;

        macro_rules! push {
            ($view:expr, $meta:expr) => {{
                unsafe {
                    core::ptr::write(accounts_ptr.add(account_count), $meta);
                    core::ptr::write(account_views_ptr.add(view_count), $view);
                }
                account_count += 1;
                view_count += 1;
            }};
        }

        match swap_data {
            data if data.starts_with(&EXACT_OUT_ROUTE_DISCRIMINATOR) => {
                push!(
                    ctx.token_program,
                    InstructionAccount::readonly(ctx.token_program.address())
                );
                push!(
                    ctx.token_account_authority,
                    InstructionAccount::readonly_signer(ctx.token_account_authority.address())
                );
                push!(
                    ctx.source_token_account,
                    InstructionAccount::writable(ctx.source_token_account.address())
                );
                push!(
                    ctx.destination_token_account,
                    InstructionAccount::writable(ctx.destination_token_account.address())
                );
                push!(
                    ctx.jupiter_program,
                    InstructionAccount::readonly(ctx.jupiter_program.address())
                );
                push!(
                    ctx.source_mint,
                    InstructionAccount::readonly(ctx.source_mint.address())
                );
                push!(
                    ctx.destination_mint,
                    InstructionAccount::readonly(ctx.destination_mint.address())
                );
                push!(
                    ctx.jupiter_program,
                    InstructionAccount::readonly(ctx.jupiter_program.address())
                );
                push!(
                    ctx.jupiter_program,
                    InstructionAccount::readonly(ctx.jupiter_program.address())
                );
                push!(
                    ctx.event_authority,
                    InstructionAccount::readonly(ctx.event_authority.address())
                );
                push!(
                    ctx.jupiter_program,
                    InstructionAccount::readonly(ctx.jupiter_program.address())
                );

                for account in remaining_accounts {
                    let meta = InstructionAccount::from(account);
                    push!(account, meta);
                }
            }
            data if data.starts_with(&ROUTE_DISCRIMINATOR) => {
                push!(
                    ctx.token_program,
                    InstructionAccount::readonly(ctx.token_program.address())
                );
                push!(
                    ctx.token_account_authority,
                    InstructionAccount::readonly_signer(ctx.token_account_authority.address())
                );
                push!(
                    ctx.source_token_account,
                    InstructionAccount::writable(ctx.source_token_account.address())
                );
                push!(
                    ctx.destination_token_account,
                    InstructionAccount::writable(ctx.destination_token_account.address())
                );
                push!(
                    ctx.jupiter_program,
                    InstructionAccount::readonly(ctx.jupiter_program.address())
                );
                push!(
                    ctx.destination_mint,
                    InstructionAccount::readonly(ctx.destination_mint.address())
                );
                push!(
                    ctx.jupiter_program,
                    InstructionAccount::readonly(ctx.jupiter_program.address())
                );
                push!(
                    ctx.event_authority,
                    InstructionAccount::readonly(ctx.event_authority.address())
                );
                push!(
                    ctx.jupiter_program,
                    InstructionAccount::readonly(ctx.jupiter_program.address())
                );

                for account in remaining_accounts {
                    let meta = InstructionAccount::from(account);
                    push!(account, meta);
                }
            }
            data if data.starts_with(&SHARED_ACCOUNTS_EXACT_OUT_ROUTE_DISCRIMINATOR)
                || data.starts_with(&SHARED_ACCOUNTS_ROUTE_DISCRIMINATOR) =>
            {
                let [program_authority, program_source_token_account, program_destination_token_account, ..] =
                    remaining_accounts
                else {
                    return Err(ProgramError::NotEnoughAccountKeys);
                };

                push!(
                    ctx.token_program,
                    InstructionAccount::readonly(ctx.token_program.address())
                );
                push!(
                    program_authority,
                    InstructionAccount::readonly(program_authority.address())
                );
                push!(
                    ctx.token_account_authority,
                    InstructionAccount::readonly_signer(ctx.token_account_authority.address())
                );
                push!(
                    ctx.source_token_account,
                    InstructionAccount::writable(ctx.source_token_account.address())
                );
                push!(
                    program_source_token_account,
                    InstructionAccount::writable(program_source_token_account.address())
                );
                push!(
                    program_destination_token_account,
                    InstructionAccount::writable(program_destination_token_account.address())
                );
                push!(
                    ctx.destination_token_account,
                    InstructionAccount::writable(ctx.destination_token_account.address())
                );
                push!(
                    ctx.source_mint,
                    InstructionAccount::readonly(ctx.source_mint.address())
                );
                push!(
                    ctx.destination_mint,
                    InstructionAccount::readonly(ctx.destination_mint.address())
                );
                push!(
                    ctx.jupiter_program,
                    InstructionAccount::readonly(ctx.jupiter_program.address())
                );
                push!(
                    ctx.jupiter_program,
                    InstructionAccount::readonly(ctx.jupiter_program.address())
                );
                push!(
                    ctx.event_authority,
                    InstructionAccount::readonly(ctx.event_authority.address())
                );
                push!(
                    ctx.jupiter_program,
                    InstructionAccount::readonly(ctx.jupiter_program.address())
                );

                for account in remaining_accounts.iter().skip(3) {
                    let meta = InstructionAccount::from(account);
                    push!(account, meta);
                }
            }
            _ => return Err(ProgramError::InvalidInstructionData),
        }

        let instruction = InstructionView {
            program_id: &JUPITER_PROGRAM_ID,
            accounts: unsafe { core::slice::from_raw_parts(accounts_ptr, account_count) },
            data: swap_data,
        };

        invoke_signed_with_bounds::<MAX_TOTAL_ACCOUNTS>(
            &instruction,
            unsafe { core::slice::from_raw_parts(account_views_ptr, view_count) },
            signer_seeds,
        )?;

        Ok(())
    }

    fn route(
        ctx: &MetisRouteAccounts<'info>,
        swap_data: &[u8],
        remaining_accounts: &[AccountView],
    ) -> ProgramResult {
        Self::route_signed(ctx, swap_data, remaining_accounts, &[])
    }
}
