#![no_std]

use {
    beethoven_core::Deposit,
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed_with_bounds, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const HYLO_PROGRAM_ID: Address =
    Address::from_str_const("HYEXCHtHkBagdStcJCp3xbbb9B7sdMdWXFNj6mdsG4hn");

const MINT_STABLECOIN_DISCRIMINATOR: [u8; 8] = [196, 235, 215, 70, 211, 5, 214, 238];
const MINT_LEVERCOIN_DISCRIMINATOR: [u8; 8] = [91, 156, 221, 157, 151, 186, 223, 231];

// mint_stablecoin: 18 CPI accounts, mint_levercoin: 19 CPI accounts
// (includes trailing program account required by Anchor CPI context)
const STABLECOIN_CPI_ACCOUNTS: usize = 18;
const LEVERCOIN_CPI_ACCOUNTS: usize = 19;
const MAX_CPI_ACCOUNTS: usize = 19;

// instruction data: discriminator(8) + amount(8) + option_tag(1) + slippage(16) = 33 max
const MAX_IX_DATA_LEN: usize = 33;

pub struct Hylo;

/// mint_type: 0 = stablecoin (hyUSD), 1 = levercoin (xSOL)
/// expected_token_out + slippage_tolerance: SlippageConfig fields (both 0 = None)
pub struct HyloDepositData {
    pub mint_type: u8,
    pub expected_token_out: u64,
    pub slippage_tolerance: u64,
}

impl HyloDepositData {
    pub const DATA_LEN: usize = 17; // 1 + 8 + 8
}

impl TryFrom<&[u8]> for HyloDepositData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        let mint_type = data[0];
        if mint_type > 1 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            mint_type,
            expected_token_out: u64::from_le_bytes(data[1..9].try_into().unwrap()),
            slippage_tolerance: u64::from_le_bytes(data[9..17].try_into().unwrap()),
        })
    }
}

pub struct HyloDepositAccounts<'info> {
    pub hylo_program: &'info AccountView,
    pub accounts: &'info [AccountView],
}

impl<'info> TryFrom<&'info [AccountView]> for HyloDepositAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        // Minimum: 1 detection prefix + 17 stablecoin CPI accounts
        if accounts.len() < 1 + STABLECOIN_CPI_ACCOUNTS {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        Ok(Self {
            hylo_program: &accounts[0],
            accounts: &accounts[1..],
        })
    }
}

impl<'info> Deposit<'info> for Hylo {
    type Accounts = HyloDepositAccounts<'info>;
    type Data = HyloDepositData;

    fn deposit_signed(
        ctx: &HyloDepositAccounts<'info>,
        amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let cpi_account_count = if data.mint_type == 0 {
            STABLECOIN_CPI_ACCOUNTS
        } else {
            LEVERCOIN_CPI_ACCOUNTS
        };

        if ctx.accounts.len() < cpi_account_count {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let accs = &ctx.accounts[..cpi_account_count];

        // Build instruction account metas
        let mut account_metas = MaybeUninit::<[InstructionAccount; MAX_CPI_ACCOUNTS]>::uninit();
        let meta_ptr = account_metas.as_mut_ptr() as *mut InstructionAccount;

        unsafe {
            // [0] user: writable, signer
            core::ptr::write(
                meta_ptr,
                InstructionAccount::writable_signer(accs[0].address()),
            );
            // [1] hylo: writable
            core::ptr::write(
                meta_ptr.add(1),
                InstructionAccount::writable(accs[1].address()),
            );
            // [2] fee_auth: readonly
            core::ptr::write(
                meta_ptr.add(2),
                InstructionAccount::readonly(accs[2].address()),
            );
            // [3] vault_auth: readonly
            core::ptr::write(
                meta_ptr.add(3),
                InstructionAccount::readonly(accs[3].address()),
            );
            // [4] coin_auth (stablecoin_auth or levercoin_auth): readonly
            core::ptr::write(
                meta_ptr.add(4),
                InstructionAccount::readonly(accs[4].address()),
            );
            // [5] fee_vault: writable
            core::ptr::write(
                meta_ptr.add(5),
                InstructionAccount::writable(accs[5].address()),
            );
            // [6] lst_vault: writable
            core::ptr::write(
                meta_ptr.add(6),
                InstructionAccount::writable(accs[6].address()),
            );
            // [7] lst_header: readonly
            core::ptr::write(
                meta_ptr.add(7),
                InstructionAccount::readonly(accs[7].address()),
            );
            // [8] user_lst_ta: writable
            core::ptr::write(
                meta_ptr.add(8),
                InstructionAccount::writable(accs[8].address()),
            );
            // [9] user_output_ta: writable
            core::ptr::write(
                meta_ptr.add(9),
                InstructionAccount::writable(accs[9].address()),
            );
            // [10] lst_mint: readonly
            core::ptr::write(
                meta_ptr.add(10),
                InstructionAccount::readonly(accs[10].address()),
            );
            // [11] output_mint: writable
            core::ptr::write(
                meta_ptr.add(11),
                InstructionAccount::writable(accs[11].address()),
            );

            // Remaining accounts differ by mint_type:
            // stablecoin: [12..16] = pyth, token, ata, system, event_auth, program
            // levercoin:  [12] = stablecoin_mint (readonly), then [13..17] = same tail
            for (i, acc) in accs
                .iter()
                .enumerate()
                .skip(12)
                .take(cpi_account_count - 12)
            {
                core::ptr::write(meta_ptr.add(i), InstructionAccount::readonly(acc.address()));
            }
        }

        let account_metas = unsafe { core::slice::from_raw_parts(meta_ptr, cpi_account_count) };

        // Build account infos array (references into the slice)
        let mut account_infos = [&accs[0]; MAX_CPI_ACCOUNTS];
        for i in 1..cpi_account_count {
            account_infos[i] = &accs[i];
        }
        let account_infos = &account_infos[..cpi_account_count];

        // Build instruction data
        let discriminator = if data.mint_type == 0 {
            &MINT_STABLECOIN_DISCRIMINATOR
        } else {
            &MINT_LEVERCOIN_DISCRIMINATOR
        };

        let has_slippage = data.expected_token_out > 0 || data.slippage_tolerance > 0;

        let ix_data_len = if has_slippage { 33 } else { 17 };

        let mut instruction_data = MaybeUninit::<[u8; MAX_IX_DATA_LEN]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            // discriminator (8 bytes)
            core::ptr::copy_nonoverlapping(discriminator.as_ptr(), ptr, 8);
            // amount_lst_to_deposit (8 bytes)
            core::ptr::copy_nonoverlapping(amount.to_le_bytes().as_ptr(), ptr.add(8), 8);

            if has_slippage {
                // Option tag: Some
                *ptr.add(16) = 0x01;
                // expected_token_out (8 bytes)
                core::ptr::copy_nonoverlapping(
                    data.expected_token_out.to_le_bytes().as_ptr(),
                    ptr.add(17),
                    8,
                );
                // slippage_tolerance (8 bytes)
                core::ptr::copy_nonoverlapping(
                    data.slippage_tolerance.to_le_bytes().as_ptr(),
                    ptr.add(25),
                    8,
                );
            } else {
                // Option tag: None
                *ptr.add(16) = 0x00;
            }
        }

        let ix_data = unsafe {
            core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, ix_data_len)
        };

        let deposit_ix = InstructionView {
            program_id: &HYLO_PROGRAM_ID,
            accounts: account_metas,
            data: ix_data,
        };

        invoke_signed_with_bounds::<MAX_CPI_ACCOUNTS>(&deposit_ix, account_infos, signer_seeds)?;

        Ok(())
    }

    fn deposit(ctx: &HyloDepositAccounts<'info>, amount: u64, data: &Self::Data) -> ProgramResult {
        Self::deposit_signed(ctx, amount, data, &[])
    }
}
