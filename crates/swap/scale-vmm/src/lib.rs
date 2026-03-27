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

pub const SCALE_VMM_PROGRAM_ID: Address =
    Address::from_str_const("SCALEWoRSpVZpMRqHEcDfNvBh3nUSe34jDr9r689gLa");

const BUY_DISCRIMINATOR: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
const SELL_DISCRIMINATOR: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];

const FIXED_ACCOUNT_COUNT: usize = 22;
const MAX_BENEFICIARY_ACCOUNTS: usize = 5;

pub struct ScaleVmm;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ScaleVmmSide {
    Buy = 0,
    Sell = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScaleVmmSwapData {
    pub side: ScaleVmmSide,
}

impl ScaleVmmSwapData {
    pub const DATA_LEN: usize = 1;
}

impl TryFrom<&[u8]> for ScaleVmmSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let side = match data[0] {
            0 => ScaleVmmSide::Buy,
            1 => ScaleVmmSide::Sell,
            _ => return Err(ProgramError::InvalidInstructionData),
        };

        Ok(Self { side })
    }
}

impl ScaleVmmSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = FIXED_ACCOUNT_COUNT;
}

pub struct ScaleVmmSwapAccounts<'info> {
    pub scale_vmm_program: &'info AccountView,
    pub pair: &'info AccountView,
    pub user: &'info AccountView,
    pub mint_a: &'info AccountView,
    pub mint_b: &'info AccountView,
    pub user_ta_a: &'info AccountView,
    pub user_ta_b: &'info AccountView,
    pub vault_a: &'info AccountView,
    pub vault_b: &'info AccountView,
    pub platform_fee_ta_a: &'info AccountView,
    pub token_program_a: &'info AccountView,
    pub token_program_b: &'info AccountView,
    pub system_program: &'info AccountView,
    pub config: &'info AccountView,
    pub amm_program: &'info AccountView,
    pub amm_pool: &'info AccountView,
    pub amm_vault_a: &'info AccountView,
    pub amm_vault_b: &'info AccountView,
    pub amm_config: &'info AccountView,
    pub amm_token_program_a: &'info AccountView,
    pub amm_token_program_b: &'info AccountView,
    pub amm_system_program: &'info AccountView,
    pub beneficiary_accounts: &'info [AccountView],
}

impl<'info> TryFrom<&'info [AccountView]> for ScaleVmmSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [scale_vmm_program, pair, user, mint_a, mint_b, user_ta_a, user_ta_b, vault_a, vault_b, platform_fee_ta_a, token_program_a, token_program_b, system_program, config, amm_program, amm_pool, amm_vault_a, amm_vault_b, amm_config, amm_token_program_a, amm_token_program_b, amm_system_program, beneficiary_accounts @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if beneficiary_accounts.len() > MAX_BENEFICIARY_ACCOUNTS {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            scale_vmm_program,
            pair,
            user,
            mint_a,
            mint_b,
            user_ta_a,
            user_ta_b,
            vault_a,
            vault_b,
            platform_fee_ta_a,
            token_program_a,
            token_program_b,
            system_program,
            config,
            amm_program,
            amm_pool,
            amm_vault_a,
            amm_vault_b,
            amm_config,
            amm_token_program_a,
            amm_token_program_b,
            amm_system_program,
            beneficiary_accounts,
        })
    }
}

fn build_instruction_data(
    side: ScaleVmmSide,
    in_amount: u64,
    minimum_out_amount: u64,
) -> MaybeUninit<[u8; 24]> {
    let discriminator = match side {
        ScaleVmmSide::Buy => &BUY_DISCRIMINATOR,
        ScaleVmmSide::Sell => &SELL_DISCRIMINATOR,
    };

    let mut instruction_data = MaybeUninit::<[u8; 24]>::uninit();
    unsafe {
        let ptr = instruction_data.as_mut_ptr() as *mut u8;
        core::ptr::copy_nonoverlapping(discriminator.as_ptr(), ptr, 8);
        core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
        core::ptr::copy_nonoverlapping(minimum_out_amount.to_le_bytes().as_ptr(), ptr.add(16), 8);
    }

    instruction_data
}

impl<'info> Swap<'info> for ScaleVmm {
    type Accounts = ScaleVmmSwapAccounts<'info>;
    type Data = ScaleVmmSwapData;

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let instruction_data = build_instruction_data(data.side, in_amount, minimum_out_amount);

        match ctx.beneficiary_accounts {
            [] => invoke_with_accounts(
                [
                    InstructionAccount::writable(ctx.pair.address()),
                    InstructionAccount::writable_signer(ctx.user.address()),
                    InstructionAccount::writable(ctx.mint_a.address()),
                    InstructionAccount::readonly(ctx.mint_b.address()),
                    InstructionAccount::writable(ctx.user_ta_a.address()),
                    InstructionAccount::writable(ctx.user_ta_b.address()),
                    InstructionAccount::writable(ctx.vault_a.address()),
                    InstructionAccount::writable(ctx.vault_b.address()),
                    InstructionAccount::writable(ctx.platform_fee_ta_a.address()),
                    InstructionAccount::readonly(ctx.token_program_a.address()),
                    InstructionAccount::readonly(ctx.token_program_b.address()),
                    InstructionAccount::readonly(ctx.system_program.address()),
                    InstructionAccount::readonly(ctx.config.address()),
                    InstructionAccount::readonly(ctx.amm_program.address()),
                    InstructionAccount::writable(ctx.amm_pool.address()),
                    InstructionAccount::writable(ctx.amm_vault_a.address()),
                    InstructionAccount::writable(ctx.amm_vault_b.address()),
                    InstructionAccount::readonly(ctx.amm_config.address()),
                    InstructionAccount::readonly(ctx.amm_token_program_a.address()),
                    InstructionAccount::readonly(ctx.amm_token_program_b.address()),
                    InstructionAccount::readonly(ctx.amm_system_program.address()),
                ],
                [
                    ctx.pair,
                    ctx.user,
                    ctx.mint_a,
                    ctx.mint_b,
                    ctx.user_ta_a,
                    ctx.user_ta_b,
                    ctx.vault_a,
                    ctx.vault_b,
                    ctx.platform_fee_ta_a,
                    ctx.token_program_a,
                    ctx.token_program_b,
                    ctx.system_program,
                    ctx.config,
                    ctx.amm_program,
                    ctx.amm_pool,
                    ctx.amm_vault_a,
                    ctx.amm_vault_b,
                    ctx.amm_config,
                    ctx.amm_token_program_a,
                    ctx.amm_token_program_b,
                    ctx.amm_system_program,
                ],
                &instruction_data,
                signer_seeds,
            ),
            [beneficiary_0] => invoke_with_accounts(
                [
                    InstructionAccount::writable(ctx.pair.address()),
                    InstructionAccount::writable_signer(ctx.user.address()),
                    InstructionAccount::writable(ctx.mint_a.address()),
                    InstructionAccount::readonly(ctx.mint_b.address()),
                    InstructionAccount::writable(ctx.user_ta_a.address()),
                    InstructionAccount::writable(ctx.user_ta_b.address()),
                    InstructionAccount::writable(ctx.vault_a.address()),
                    InstructionAccount::writable(ctx.vault_b.address()),
                    InstructionAccount::writable(ctx.platform_fee_ta_a.address()),
                    InstructionAccount::readonly(ctx.token_program_a.address()),
                    InstructionAccount::readonly(ctx.token_program_b.address()),
                    InstructionAccount::readonly(ctx.system_program.address()),
                    InstructionAccount::readonly(ctx.config.address()),
                    InstructionAccount::readonly(ctx.amm_program.address()),
                    InstructionAccount::writable(ctx.amm_pool.address()),
                    InstructionAccount::writable(ctx.amm_vault_a.address()),
                    InstructionAccount::writable(ctx.amm_vault_b.address()),
                    InstructionAccount::readonly(ctx.amm_config.address()),
                    InstructionAccount::readonly(ctx.amm_token_program_a.address()),
                    InstructionAccount::readonly(ctx.amm_token_program_b.address()),
                    InstructionAccount::readonly(ctx.amm_system_program.address()),
                    InstructionAccount::writable(beneficiary_0.address()),
                ],
                [
                    ctx.pair,
                    ctx.user,
                    ctx.mint_a,
                    ctx.mint_b,
                    ctx.user_ta_a,
                    ctx.user_ta_b,
                    ctx.vault_a,
                    ctx.vault_b,
                    ctx.platform_fee_ta_a,
                    ctx.token_program_a,
                    ctx.token_program_b,
                    ctx.system_program,
                    ctx.config,
                    ctx.amm_program,
                    ctx.amm_pool,
                    ctx.amm_vault_a,
                    ctx.amm_vault_b,
                    ctx.amm_config,
                    ctx.amm_token_program_a,
                    ctx.amm_token_program_b,
                    ctx.amm_system_program,
                    beneficiary_0,
                ],
                &instruction_data,
                signer_seeds,
            ),
            [beneficiary_0, beneficiary_1] => invoke_with_accounts(
                [
                    InstructionAccount::writable(ctx.pair.address()),
                    InstructionAccount::writable_signer(ctx.user.address()),
                    InstructionAccount::writable(ctx.mint_a.address()),
                    InstructionAccount::readonly(ctx.mint_b.address()),
                    InstructionAccount::writable(ctx.user_ta_a.address()),
                    InstructionAccount::writable(ctx.user_ta_b.address()),
                    InstructionAccount::writable(ctx.vault_a.address()),
                    InstructionAccount::writable(ctx.vault_b.address()),
                    InstructionAccount::writable(ctx.platform_fee_ta_a.address()),
                    InstructionAccount::readonly(ctx.token_program_a.address()),
                    InstructionAccount::readonly(ctx.token_program_b.address()),
                    InstructionAccount::readonly(ctx.system_program.address()),
                    InstructionAccount::readonly(ctx.config.address()),
                    InstructionAccount::readonly(ctx.amm_program.address()),
                    InstructionAccount::writable(ctx.amm_pool.address()),
                    InstructionAccount::writable(ctx.amm_vault_a.address()),
                    InstructionAccount::writable(ctx.amm_vault_b.address()),
                    InstructionAccount::readonly(ctx.amm_config.address()),
                    InstructionAccount::readonly(ctx.amm_token_program_a.address()),
                    InstructionAccount::readonly(ctx.amm_token_program_b.address()),
                    InstructionAccount::readonly(ctx.amm_system_program.address()),
                    InstructionAccount::writable(beneficiary_0.address()),
                    InstructionAccount::writable(beneficiary_1.address()),
                ],
                [
                    ctx.pair,
                    ctx.user,
                    ctx.mint_a,
                    ctx.mint_b,
                    ctx.user_ta_a,
                    ctx.user_ta_b,
                    ctx.vault_a,
                    ctx.vault_b,
                    ctx.platform_fee_ta_a,
                    ctx.token_program_a,
                    ctx.token_program_b,
                    ctx.system_program,
                    ctx.config,
                    ctx.amm_program,
                    ctx.amm_pool,
                    ctx.amm_vault_a,
                    ctx.amm_vault_b,
                    ctx.amm_config,
                    ctx.amm_token_program_a,
                    ctx.amm_token_program_b,
                    ctx.amm_system_program,
                    beneficiary_0,
                    beneficiary_1,
                ],
                &instruction_data,
                signer_seeds,
            ),
            [beneficiary_0, beneficiary_1, beneficiary_2] => invoke_with_accounts(
                [
                    InstructionAccount::writable(ctx.pair.address()),
                    InstructionAccount::writable_signer(ctx.user.address()),
                    InstructionAccount::writable(ctx.mint_a.address()),
                    InstructionAccount::readonly(ctx.mint_b.address()),
                    InstructionAccount::writable(ctx.user_ta_a.address()),
                    InstructionAccount::writable(ctx.user_ta_b.address()),
                    InstructionAccount::writable(ctx.vault_a.address()),
                    InstructionAccount::writable(ctx.vault_b.address()),
                    InstructionAccount::writable(ctx.platform_fee_ta_a.address()),
                    InstructionAccount::readonly(ctx.token_program_a.address()),
                    InstructionAccount::readonly(ctx.token_program_b.address()),
                    InstructionAccount::readonly(ctx.system_program.address()),
                    InstructionAccount::readonly(ctx.config.address()),
                    InstructionAccount::readonly(ctx.amm_program.address()),
                    InstructionAccount::writable(ctx.amm_pool.address()),
                    InstructionAccount::writable(ctx.amm_vault_a.address()),
                    InstructionAccount::writable(ctx.amm_vault_b.address()),
                    InstructionAccount::readonly(ctx.amm_config.address()),
                    InstructionAccount::readonly(ctx.amm_token_program_a.address()),
                    InstructionAccount::readonly(ctx.amm_token_program_b.address()),
                    InstructionAccount::readonly(ctx.amm_system_program.address()),
                    InstructionAccount::writable(beneficiary_0.address()),
                    InstructionAccount::writable(beneficiary_1.address()),
                    InstructionAccount::writable(beneficiary_2.address()),
                ],
                [
                    ctx.pair,
                    ctx.user,
                    ctx.mint_a,
                    ctx.mint_b,
                    ctx.user_ta_a,
                    ctx.user_ta_b,
                    ctx.vault_a,
                    ctx.vault_b,
                    ctx.platform_fee_ta_a,
                    ctx.token_program_a,
                    ctx.token_program_b,
                    ctx.system_program,
                    ctx.config,
                    ctx.amm_program,
                    ctx.amm_pool,
                    ctx.amm_vault_a,
                    ctx.amm_vault_b,
                    ctx.amm_config,
                    ctx.amm_token_program_a,
                    ctx.amm_token_program_b,
                    ctx.amm_system_program,
                    beneficiary_0,
                    beneficiary_1,
                    beneficiary_2,
                ],
                &instruction_data,
                signer_seeds,
            ),
            [beneficiary_0, beneficiary_1, beneficiary_2, beneficiary_3] => invoke_with_accounts(
                [
                    InstructionAccount::writable(ctx.pair.address()),
                    InstructionAccount::writable_signer(ctx.user.address()),
                    InstructionAccount::writable(ctx.mint_a.address()),
                    InstructionAccount::readonly(ctx.mint_b.address()),
                    InstructionAccount::writable(ctx.user_ta_a.address()),
                    InstructionAccount::writable(ctx.user_ta_b.address()),
                    InstructionAccount::writable(ctx.vault_a.address()),
                    InstructionAccount::writable(ctx.vault_b.address()),
                    InstructionAccount::writable(ctx.platform_fee_ta_a.address()),
                    InstructionAccount::readonly(ctx.token_program_a.address()),
                    InstructionAccount::readonly(ctx.token_program_b.address()),
                    InstructionAccount::readonly(ctx.system_program.address()),
                    InstructionAccount::readonly(ctx.config.address()),
                    InstructionAccount::readonly(ctx.amm_program.address()),
                    InstructionAccount::writable(ctx.amm_pool.address()),
                    InstructionAccount::writable(ctx.amm_vault_a.address()),
                    InstructionAccount::writable(ctx.amm_vault_b.address()),
                    InstructionAccount::readonly(ctx.amm_config.address()),
                    InstructionAccount::readonly(ctx.amm_token_program_a.address()),
                    InstructionAccount::readonly(ctx.amm_token_program_b.address()),
                    InstructionAccount::readonly(ctx.amm_system_program.address()),
                    InstructionAccount::writable(beneficiary_0.address()),
                    InstructionAccount::writable(beneficiary_1.address()),
                    InstructionAccount::writable(beneficiary_2.address()),
                    InstructionAccount::writable(beneficiary_3.address()),
                ],
                [
                    ctx.pair,
                    ctx.user,
                    ctx.mint_a,
                    ctx.mint_b,
                    ctx.user_ta_a,
                    ctx.user_ta_b,
                    ctx.vault_a,
                    ctx.vault_b,
                    ctx.platform_fee_ta_a,
                    ctx.token_program_a,
                    ctx.token_program_b,
                    ctx.system_program,
                    ctx.config,
                    ctx.amm_program,
                    ctx.amm_pool,
                    ctx.amm_vault_a,
                    ctx.amm_vault_b,
                    ctx.amm_config,
                    ctx.amm_token_program_a,
                    ctx.amm_token_program_b,
                    ctx.amm_system_program,
                    beneficiary_0,
                    beneficiary_1,
                    beneficiary_2,
                    beneficiary_3,
                ],
                &instruction_data,
                signer_seeds,
            ),
            [beneficiary_0, beneficiary_1, beneficiary_2, beneficiary_3, beneficiary_4] => {
                invoke_with_accounts(
                    [
                        InstructionAccount::writable(ctx.pair.address()),
                        InstructionAccount::writable_signer(ctx.user.address()),
                        InstructionAccount::writable(ctx.mint_a.address()),
                        InstructionAccount::readonly(ctx.mint_b.address()),
                        InstructionAccount::writable(ctx.user_ta_a.address()),
                        InstructionAccount::writable(ctx.user_ta_b.address()),
                        InstructionAccount::writable(ctx.vault_a.address()),
                        InstructionAccount::writable(ctx.vault_b.address()),
                        InstructionAccount::writable(ctx.platform_fee_ta_a.address()),
                        InstructionAccount::readonly(ctx.token_program_a.address()),
                        InstructionAccount::readonly(ctx.token_program_b.address()),
                        InstructionAccount::readonly(ctx.system_program.address()),
                        InstructionAccount::readonly(ctx.config.address()),
                        InstructionAccount::readonly(ctx.amm_program.address()),
                        InstructionAccount::writable(ctx.amm_pool.address()),
                        InstructionAccount::writable(ctx.amm_vault_a.address()),
                        InstructionAccount::writable(ctx.amm_vault_b.address()),
                        InstructionAccount::readonly(ctx.amm_config.address()),
                        InstructionAccount::readonly(ctx.amm_token_program_a.address()),
                        InstructionAccount::readonly(ctx.amm_token_program_b.address()),
                        InstructionAccount::readonly(ctx.amm_system_program.address()),
                        InstructionAccount::writable(beneficiary_0.address()),
                        InstructionAccount::writable(beneficiary_1.address()),
                        InstructionAccount::writable(beneficiary_2.address()),
                        InstructionAccount::writable(beneficiary_3.address()),
                        InstructionAccount::writable(beneficiary_4.address()),
                    ],
                    [
                        ctx.pair,
                        ctx.user,
                        ctx.mint_a,
                        ctx.mint_b,
                        ctx.user_ta_a,
                        ctx.user_ta_b,
                        ctx.vault_a,
                        ctx.vault_b,
                        ctx.platform_fee_ta_a,
                        ctx.token_program_a,
                        ctx.token_program_b,
                        ctx.system_program,
                        ctx.config,
                        ctx.amm_program,
                        ctx.amm_pool,
                        ctx.amm_vault_a,
                        ctx.amm_vault_b,
                        ctx.amm_config,
                        ctx.amm_token_program_a,
                        ctx.amm_token_program_b,
                        ctx.amm_system_program,
                        beneficiary_0,
                        beneficiary_1,
                        beneficiary_2,
                        beneficiary_3,
                        beneficiary_4,
                    ],
                    &instruction_data,
                    signer_seeds,
                )
            }
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

fn invoke_with_accounts<const ACCOUNTS: usize>(
    accounts: [InstructionAccount; ACCOUNTS],
    account_infos: [&AccountView; ACCOUNTS],
    instruction_data: &MaybeUninit<[u8; 24]>,
    signer_seeds: &[Signer],
) -> ProgramResult {
    let instruction = InstructionView {
        program_id: &SCALE_VMM_PROGRAM_ID,
        accounts: &accounts,
        data: unsafe { core::slice::from_raw_parts(instruction_data.as_ptr() as *const u8, 24) },
    };

    invoke_signed(&instruction, &account_infos, signer_seeds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scale_vmm_swap_data() {
        let buy = ScaleVmmSwapData::try_from(&[0u8][..]).unwrap();
        assert_eq!(buy.side, ScaleVmmSide::Buy);

        let sell = ScaleVmmSwapData::try_from(&[1u8][..]).unwrap();
        assert_eq!(sell.side, ScaleVmmSide::Sell);

        let empty: &[u8] = &[];

        assert_eq!(
            ScaleVmmSwapData::try_from(empty).unwrap_err(),
            ProgramError::InvalidInstructionData
        );
        assert_eq!(
            ScaleVmmSwapData::try_from(&[2u8][..]).unwrap_err(),
            ProgramError::InvalidInstructionData
        );
    }

    #[test]
    fn test_instruction_data_layout() {
        let amount = 42u64;
        let min_out = 7u64;

        let buy = build_instruction_data(ScaleVmmSide::Buy, amount, min_out);
        let buy = unsafe { &*buy.as_ptr() };
        assert_eq!(&buy[0..8], &BUY_DISCRIMINATOR);
        assert_eq!(&buy[8..16], &amount.to_le_bytes());
        assert_eq!(&buy[16..24], &min_out.to_le_bytes());

        let sell = build_instruction_data(ScaleVmmSide::Sell, amount, min_out);
        let sell = unsafe { &*sell.as_ptr() };
        assert_eq!(&sell[0..8], &SELL_DISCRIMINATOR);
        assert_eq!(&sell[8..16], &amount.to_le_bytes());
        assert_eq!(&sell[16..24], &min_out.to_le_bytes());
    }
}
