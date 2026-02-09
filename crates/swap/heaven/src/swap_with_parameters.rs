use {
    crate::{Heaven, HEAVEN_PROGRAM_ID, BUY_DISCRIMINATOR, SELL_DISCRIMINATOR},
    beethoven_core::{SwapParameters, SwapWithParameters, token_account_mint},
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::{address_eq, Address},
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub struct HeavenSwapRemaining<'info> {
    pub heaven_program: &'info AccountView,
    pub token_a_owner: &'info AccountView,
    pub token_b_owner: &'info AccountView,
    pub ata_program: &'info AccountView,
    pub system_program: &'info AccountView,
    pub pool_state: &'info AccountView,
    pub token_a_mint: &'info AccountView,
    pub token_b_mint: &'info AccountView,
    pub pool_token_a_account: &'info AccountView,
    pub pool_token_b_account: &'info AccountView,
    pub protocol_config: &'info AccountView,
    pub ix_sysvar: &'info AccountView,
    pub chainlink_id: &'info AccountView,
    pub chainlink_sol_usd_feed: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for HeavenSwapRemaining<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < 14 {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [heaven_program, token_a_owner, token_b_owner, ata_program, system_program, pool_state, token_a_mint, token_b_mint, pool_token_a_account, pool_token_b_account, protocol_config, ix_sysvar, chainlink_id, chainlink_sol_usd_feed, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(HeavenSwapRemaining {
            heaven_program,
            token_a_owner,
            token_b_owner,
            ata_program,
            system_program,
            pool_state,
            token_a_mint,
            token_b_mint,
            pool_token_a_account,
            pool_token_b_account,
            protocol_config,
            ix_sysvar,
            chainlink_id,
            chainlink_sol_usd_feed,
        })
    }
}

pub struct HeavenSwapExtra<'a> {
    pub event: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for HeavenSwapExtra<'a> {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self { event: data })
    }
}

impl<'info> SwapWithParameters<'info> for Heaven {
    type Remaining = HeavenSwapRemaining<'info>;
    type Extra = HeavenSwapExtra<'info>;

    fn try_parse_remaining(
        remaining: &'info [AccountView],
    ) -> Result<Self::Remaining, ProgramError> {
        HeavenSwapRemaining::try_from(remaining)
    }

    fn swap_with_parameters_signed(
        params: &SwapParameters<'info>,
        remaining: &Self::Remaining,
        in_amount: u64,
        minimum_out_amount: u64,
        extra: &Self::Extra,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let in_ata_mint = unsafe { &*(token_account_mint(params.in_ata).as_ptr() as *const Address) };
        let pool_a_mint = unsafe { &*(token_account_mint(remaining.pool_token_a_account).as_ptr() as *const Address) };
        let is_a_in = address_eq(in_ata_mint, pool_a_mint);

        let discriminator = if is_a_in {
            &SELL_DISCRIMINATOR
        } else {
            &BUY_DISCRIMINATOR
        };

        let (user_token_a_account, user_token_b_account) = if is_a_in {
            (params.in_ata, params.out_ata)
        } else {
            (params.out_ata, params.in_ata)
        };

        let accounts = [
            InstructionAccount::readonly(remaining.token_a_owner.address()),
            InstructionAccount::readonly(remaining.token_b_owner.address()),
            InstructionAccount::readonly(remaining.ata_program.address()),
            InstructionAccount::readonly(remaining.system_program.address()),
            InstructionAccount::writable(remaining.pool_state.address()),
            InstructionAccount::readonly_signer(params.user_wallet.address()),
            InstructionAccount::readonly(remaining.token_a_mint.address()),
            InstructionAccount::readonly(remaining.token_b_mint.address()),
            InstructionAccount::writable(user_token_a_account.address()),
            InstructionAccount::writable(user_token_b_account.address()),
            InstructionAccount::writable(remaining.pool_token_a_account.address()),
            InstructionAccount::writable(remaining.pool_token_b_account.address()),
            InstructionAccount::writable(remaining.protocol_config.address()),
            InstructionAccount::readonly(remaining.ix_sysvar.address()),
            InstructionAccount::readonly(remaining.chainlink_id.address()),
            InstructionAccount::readonly(remaining.chainlink_sol_usd_feed.address()),
        ];

        let account_infos = [
            remaining.token_a_owner,
            remaining.token_b_owner,
            remaining.ata_program,
            remaining.system_program,
            remaining.pool_state,
            params.user_wallet,
            remaining.token_a_mint,
            remaining.token_b_mint,
            user_token_a_account,
            user_token_b_account,
            remaining.pool_token_a_account,
            remaining.pool_token_b_account,
            remaining.protocol_config,
            remaining.ix_sysvar,
            remaining.chainlink_id,
            remaining.chainlink_sol_usd_feed,
        ];

        let event_len = extra.event.len();
        let instruction_data_len = 8 + 8 + 8 + 4 + event_len;

        if event_len == 0 {
            let mut instruction_data = MaybeUninit::<[u8; 28]>::uninit();
            unsafe {
                let ptr = instruction_data.as_mut_ptr() as *mut u8;
                core::ptr::copy_nonoverlapping(discriminator.as_ptr(), ptr, 8);
                core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
                core::ptr::copy_nonoverlapping(
                    minimum_out_amount.to_le_bytes().as_ptr(),
                    ptr.add(16),
                    8,
                );
                core::ptr::copy_nonoverlapping(0u32.to_le_bytes().as_ptr(), ptr.add(24), 4);
            }

            let instruction = InstructionView {
                program_id: &HEAVEN_PROGRAM_ID,
                accounts: &accounts,
                data: unsafe {
                    core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 28)
                },
            };

            return invoke_signed(&instruction, &account_infos, signer_seeds);
        }

        const MAX_EVENT_LEN: usize = 256;
        if event_len > MAX_EVENT_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }

        let mut instruction_data = MaybeUninit::<[u8; 28 + MAX_EVENT_LEN]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(discriminator.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(16),
                8,
            );
            core::ptr::copy_nonoverlapping(
                (event_len as u32).to_le_bytes().as_ptr(),
                ptr.add(24),
                4,
            );
            core::ptr::copy_nonoverlapping(extra.event.as_ptr(), ptr.add(28), event_len);
        }

        let instruction = InstructionView {
            program_id: &HEAVEN_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe {
                core::slice::from_raw_parts(
                    instruction_data.as_ptr() as *const u8,
                    instruction_data_len,
                )
            },
        };

        invoke_signed(&instruction, &account_infos, signer_seeds)
    }
}
