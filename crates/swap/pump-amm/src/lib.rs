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

pub const PUMP_AMM_PROGRAM_ID: Address = address!("pAMMBay6oceH9fJKBRHGP5D4bD4sWpmSwMn52FMfXEA");

const BUY_DISCRIMINATOR: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
const SELL_DISCRIMINATOR: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];
// Optional associated token account of the UserVolumeAccumulator for Pump AMM program
const MAX_REMAINING_ACCOUNTS: usize = 1;
const MAX_ACCOUNTS: usize = PumpAmmSwapAccounts::MIN_NUM_ACCOUNTS_BUY + MAX_REMAINING_ACCOUNTS;

pub struct PumpAmm;

pub struct PumpAmmSwapData {
    pub track_volume: Option<bool>,
    pub is_buy: bool,
}

impl PumpAmmSwapData {
    // 2 - track_volume
    // 1 - is_buy
    pub const DATA_LEN: usize = 3;
}

impl TryFrom<&[u8]> for PumpAmmSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < Self::DATA_LEN {
            return Err(ProgramError::InvalidInstructionData);
        }
        let (tag, b, is_buy) = (data[0], data[1], data[2]);
        let track_volume = match tag {
            0 if b == 0 => None,
            0 => return Err(ProgramError::InvalidInstructionData),
            1 => Some(match b {
                0 => false,
                1 => true,
                _ => return Err(ProgramError::InvalidInstructionData),
            }),
            _ => return Err(ProgramError::InvalidInstructionData),
        };
        Ok(Self {
            track_volume,
            is_buy: is_buy == 1,
        })
    }
}

impl PumpAmmSwapAccounts<'_> {
    pub const MIN_NUM_ACCOUNTS_BUY: usize = 26;
    pub const MIN_NUM_ACCOUNTS_SELL: usize = 24;
}

pub struct PumpAmmSwapAccounts<'info> {
    pub base: PumpAmmSwapAccountsBase<'info>,
    pub leg: PumpAmmSwapAccountsLeg<'info>,
    pub tail: PumpAmmSwapAccountsTail<'info>,
}

pub struct PumpAmmSwapAccountsBase<'info> {
    pub pump_amm_program: &'info AccountView,
    pub pool: &'info AccountView,
    pub user: &'info AccountView,
    pub global_config: &'info AccountView,
    pub base_mint: &'info AccountView,
    pub quote_mint: &'info AccountView,
    pub user_base_token_account: &'info AccountView,
    pub user_quote_token_account: &'info AccountView,
    pub pool_base_token_account: &'info AccountView,
    pub pool_quote_token_account: &'info AccountView,
    pub protocol_fee_recipient: &'info AccountView,
    pub protocol_fee_recipient_token_account: &'info AccountView,
    pub base_token_program: &'info AccountView,
    pub quote_token_program: &'info AccountView,
    pub system_program: &'info AccountView,
    pub associated_token_program: &'info AccountView,
    pub event_authority: &'info AccountView,
    pub program: &'info AccountView,
    pub coin_creator_vault_ata: &'info AccountView,
    pub coin_creator_vault_authority: &'info AccountView,
}

pub enum PumpAmmSwapAccountsLeg<'info> {
    Buy {
        global_volume_accumulator: &'info AccountView,
        user_volume_accumulator: &'info AccountView,
    },
    Sell,
}

pub struct PumpAmmSwapAccountsTail<'info> {
    pub fee_config: &'info AccountView,
    pub fee_program: &'info AccountView,
    pub pool_v2: &'info AccountView,
    pub remaining_accounts: &'info [AccountView],
    pub fee_recipient: &'info AccountView,
    pub fee_recipient_quote_mint_ata: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for PumpAmmSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        // check if account len is at least the minimum
        if accounts.len() < PumpAmmSwapAccounts::MIN_NUM_ACCOUNTS_SELL {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let i = accounts;

        let base = PumpAmmSwapAccountsBase {
            pump_amm_program: &i[0],
            pool: &i[1],
            user: &i[2],
            global_config: &i[3],
            base_mint: &i[4],
            quote_mint: &i[5],
            user_base_token_account: &i[6],
            user_quote_token_account: &i[7],
            pool_base_token_account: &i[8],
            pool_quote_token_account: &i[9],
            protocol_fee_recipient: &i[10],
            protocol_fee_recipient_token_account: &i[11],
            base_token_program: &i[12],
            quote_token_program: &i[13],
            system_program: &i[14],
            associated_token_program: &i[15],
            event_authority: &i[16],
            program: &i[17],
            coin_creator_vault_ata: &i[18],
            coin_creator_vault_authority: &i[19],
        };

        // fee recipient and fee recipient quote mint ata are the last 2 accounts
        // requires remaining accounts to be strict, with no unused accounts
        let remaining_accounts_len = i.len();
        let fee_recipient_index = remaining_accounts_len - 2;
        let fee_recipient = &i[fee_recipient_index];
        let fee_recipient_quote_mint_ata = &i[fee_recipient_index + 1];

        if accounts.len() >= PumpAmmSwapAccounts::MIN_NUM_ACCOUNTS_BUY {
            Ok(PumpAmmSwapAccounts {
                base,
                leg: PumpAmmSwapAccountsLeg::Buy {
                    global_volume_accumulator: &i[20],
                    user_volume_accumulator: &i[21],
                },
                tail: PumpAmmSwapAccountsTail {
                    fee_config: &i[22],
                    fee_program: &i[23],
                    pool_v2: &i[24],
                    remaining_accounts: &i[25..fee_recipient_index],
                    fee_recipient,
                    fee_recipient_quote_mint_ata,
                },
            })
        } else {
            Ok(PumpAmmSwapAccounts {
                base,
                leg: PumpAmmSwapAccountsLeg::Sell,
                tail: PumpAmmSwapAccountsTail {
                    fee_config: &i[20],
                    fee_program: &i[21],
                    pool_v2: &i[22],
                    remaining_accounts: &i[23..fee_recipient_index],
                    fee_recipient,
                    fee_recipient_quote_mint_ata,
                },
            })
        }
    }
}

impl<'info> Swap<'info> for PumpAmm {
    type Accounts = PumpAmmSwapAccounts<'info>;
    type Data = PumpAmmSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let base_account_len = match &ctx.leg {
            PumpAmmSwapAccountsLeg::Buy { .. } => 25,
            PumpAmmSwapAccountsLeg::Sell => 23,
        };
        let total_accounts = base_account_len + ctx.tail.remaining_accounts.len();

        if total_accounts > MAX_ACCOUNTS {
            // TODO: should be 'Too many accounts' error
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let mut account_metas = MaybeUninit::<[InstructionAccount; MAX_ACCOUNTS]>::uninit();
        let account_metas_ptr = account_metas.as_mut_ptr() as *mut InstructionAccount;

        unsafe {
            core::ptr::write(
                account_metas_ptr,
                InstructionAccount::writable(ctx.base.pool.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(1),
                InstructionAccount::writable_signer(ctx.base.user.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(2),
                InstructionAccount::readonly(ctx.base.global_config.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(3),
                InstructionAccount::readonly(ctx.base.base_mint.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(4),
                InstructionAccount::readonly(ctx.base.quote_mint.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(5),
                InstructionAccount::writable(ctx.base.user_base_token_account.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(6),
                InstructionAccount::writable(ctx.base.user_quote_token_account.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(7),
                InstructionAccount::writable(ctx.base.pool_base_token_account.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(8),
                InstructionAccount::writable(ctx.base.pool_quote_token_account.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(9),
                InstructionAccount::readonly(ctx.base.protocol_fee_recipient.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(10),
                InstructionAccount::writable(
                    ctx.base.protocol_fee_recipient_token_account.address(),
                ),
            );
            core::ptr::write(
                account_metas_ptr.add(11),
                InstructionAccount::readonly(ctx.base.base_token_program.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(12),
                InstructionAccount::readonly(ctx.base.quote_token_program.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(13),
                InstructionAccount::readonly(ctx.base.system_program.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(14),
                InstructionAccount::readonly(ctx.base.associated_token_program.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(15),
                InstructionAccount::readonly(ctx.base.event_authority.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(16),
                InstructionAccount::readonly(ctx.base.program.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(17),
                InstructionAccount::writable(ctx.base.coin_creator_vault_ata.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(18),
                InstructionAccount::readonly(ctx.base.coin_creator_vault_authority.address()),
            );

            let mut index: usize = 19;

            if let PumpAmmSwapAccountsLeg::Buy {
                global_volume_accumulator,
                user_volume_accumulator,
            } = &ctx.leg
            {
                core::ptr::write(
                    account_metas_ptr.add(19),
                    InstructionAccount::readonly(global_volume_accumulator.address()),
                );
                core::ptr::write(
                    account_metas_ptr.add(20),
                    InstructionAccount::writable(user_volume_accumulator.address()),
                );
                index = 21;
            }

            core::ptr::write(
                account_metas_ptr.add(index),
                InstructionAccount::readonly(ctx.tail.fee_config.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(index + 1),
                InstructionAccount::readonly(ctx.tail.fee_program.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(index + 2),
                InstructionAccount::readonly(ctx.tail.pool_v2.address()),
            );
            index += 3;

            for account in ctx.tail.remaining_accounts.iter() {
                core::ptr::write(
                    account_metas_ptr.add(index),
                    InstructionAccount::from(account),
                );
                index += 1;
            }

            core::ptr::write(
                account_metas_ptr.add(index),
                InstructionAccount::readonly(ctx.tail.fee_recipient.address()),
            );
            core::ptr::write(
                account_metas_ptr.add(index + 1),
                InstructionAccount::writable(ctx.tail.fee_recipient_quote_mint_ata.address()),
            );
        }

        let account_metas =
            unsafe { core::slice::from_raw_parts(account_metas_ptr, total_accounts) };

        let mut account_infos = [ctx.base.pool; MAX_ACCOUNTS];
        account_infos[1] = ctx.base.user;
        account_infos[2] = ctx.base.global_config;
        account_infos[3] = ctx.base.base_mint;
        account_infos[4] = ctx.base.quote_mint;
        account_infos[5] = ctx.base.user_base_token_account;
        account_infos[6] = ctx.base.user_quote_token_account;
        account_infos[7] = ctx.base.pool_base_token_account;
        account_infos[8] = ctx.base.pool_quote_token_account;
        account_infos[9] = ctx.base.protocol_fee_recipient;
        account_infos[10] = ctx.base.protocol_fee_recipient_token_account;
        account_infos[11] = ctx.base.base_token_program;
        account_infos[12] = ctx.base.quote_token_program;
        account_infos[13] = ctx.base.system_program;
        account_infos[14] = ctx.base.associated_token_program;
        account_infos[15] = ctx.base.event_authority;
        account_infos[16] = ctx.base.program;
        account_infos[17] = ctx.base.coin_creator_vault_ata;
        account_infos[18] = ctx.base.coin_creator_vault_authority;

        let mut index: usize = 19;

        if let PumpAmmSwapAccountsLeg::Buy {
            global_volume_accumulator,
            user_volume_accumulator,
        } = &ctx.leg
        {
            account_infos[19] = global_volume_accumulator;
            account_infos[20] = user_volume_accumulator;
            index = 21;
        }

        account_infos[index] = ctx.tail.fee_config;
        account_infos[index + 1] = ctx.tail.fee_program;
        account_infos[index + 2] = ctx.tail.pool_v2;
        index += 3;
        for (i, account) in ctx.tail.remaining_accounts.iter().enumerate() {
            account_infos[index + i] = account;
            index += 1;
        }
        account_infos[index] = ctx.tail.fee_recipient;
        account_infos[index + 1] = ctx.tail.fee_recipient_quote_mint_ata;

        let account_infos = &account_infos[..total_accounts];

        let mut instruction_data = MaybeUninit::<[u8; 26]>::uninit();

        let discriminator = match data.is_buy {
            true => BUY_DISCRIMINATOR,
            false => SELL_DISCRIMINATOR,
        };

        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(discriminator.as_ptr(), ptr, 8);

            // for buy, in amount is base amount, out amount is quote amount
            // for sell, in amount is quote amount, out amount is base amount
            let base_amount = match data.is_buy {
                true => in_amount,
                false => minimum_out_amount,
            };
            let quote_amount = match data.is_buy {
                true => minimum_out_amount,
                false => in_amount,
            };

            core::ptr::copy_nonoverlapping(quote_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(base_amount.to_le_bytes().as_ptr(), ptr.add(16), 8);

            let bytes = match data.track_volume {
                None => [0, 0],
                Some(b) => [1, b as u8],
            };

            core::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr.add(24), 2);
        }

        let instruction = InstructionView {
            program_id: &PUMP_AMM_PROGRAM_ID,
            accounts: account_metas,
            data: unsafe {
                core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 26)
            },
        };

        invoke_signed_with_bounds::<MAX_ACCOUNTS, _>(&instruction, account_infos, signer_seeds)?;

        Ok(())
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
