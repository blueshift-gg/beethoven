#![no_std]

use {
    beethoven_core::Swap,
    solana_account_view::AccountView,
    solana_address::Address,
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const SCALE_AMM_PROGRAM_ID: Address =
    Address::from_str_const("SCALEwAvEK5gtkdHiFzXfPgtk2YwJxPDzaV3aDmR7tA");

const BUY_DISCRIMINATOR: [u8; 8] = [102, 6, 61, 18, 1, 218, 235, 234];
const SELL_DISCRIMINATOR: [u8; 8] = [51, 230, 133, 164, 1, 127, 131, 173];

const FIXED_ACCOUNT_COUNT: usize = 15;
const MAX_BENEFICIARY_ACCOUNTS: usize = 5;

pub struct ScaleAmm;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum ScaleAmmSide {
    Buy = 0,
    Sell = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScaleAmmSwapData {
    pub side: ScaleAmmSide,
}

impl TryFrom<&[u8]> for ScaleAmmSwapData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }

        let side = match data[0] {
            0 => ScaleAmmSide::Buy,
            1 => ScaleAmmSide::Sell,
            _ => return Err(ProgramError::InvalidInstructionData),
        };

        Ok(Self { side })
    }
}

pub struct ScaleAmmSwapAccounts<'info> {
    pub scale_amm_program: &'info AccountView,
    pub pool: &'info AccountView,
    pub user: &'info AccountView,
    pub owner: &'info AccountView,
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
    pub beneficiary_accounts: &'info [AccountView],
}

impl<'info> TryFrom<&'info [AccountView]> for ScaleAmmSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() < FIXED_ACCOUNT_COUNT {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let [scale_amm_program, pool, user, owner, mint_a, mint_b, user_ta_a, user_ta_b, vault_a, vault_b, platform_fee_ta_a, token_program_a, token_program_b, system_program, config, beneficiary_accounts @ ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if beneficiary_accounts.len() > MAX_BENEFICIARY_ACCOUNTS {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            scale_amm_program,
            pool,
            user,
            owner,
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
            beneficiary_accounts,
        })
    }
}

fn build_instruction_data(side: ScaleAmmSide, in_amount: u64, minimum_out_amount: u64) -> [u8; 24] {
    let mut instruction_data = [0u8; 24];

    let discriminator = match side {
        ScaleAmmSide::Buy => BUY_DISCRIMINATOR,
        ScaleAmmSide::Sell => SELL_DISCRIMINATOR,
    };

    instruction_data[0..8].copy_from_slice(&discriminator);
    instruction_data[8..16].copy_from_slice(&in_amount.to_le_bytes());
    instruction_data[16..24].copy_from_slice(&minimum_out_amount.to_le_bytes());

    instruction_data
}

impl<'info> Swap<'info> for ScaleAmm {
    type Accounts = ScaleAmmSwapAccounts<'info>;
    type Data = ScaleAmmSwapData;

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
                    InstructionAccount::writable(ctx.pool.address()),
                    InstructionAccount::writable_signer(ctx.user.address()),
                    InstructionAccount::readonly(ctx.owner.address()),
                    InstructionAccount::readonly(ctx.mint_a.address()),
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
                ],
                [
                    ctx.pool,
                    ctx.user,
                    ctx.owner,
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
                ],
                &instruction_data,
                signer_seeds,
            ),
            [beneficiary_0] => invoke_with_accounts(
                [
                    InstructionAccount::writable(ctx.pool.address()),
                    InstructionAccount::writable_signer(ctx.user.address()),
                    InstructionAccount::readonly(ctx.owner.address()),
                    InstructionAccount::readonly(ctx.mint_a.address()),
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
                    InstructionAccount::writable(beneficiary_0.address()),
                ],
                [
                    ctx.pool,
                    ctx.user,
                    ctx.owner,
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
                    beneficiary_0,
                ],
                &instruction_data,
                signer_seeds,
            ),
            [beneficiary_0, beneficiary_1] => invoke_with_accounts(
                [
                    InstructionAccount::writable(ctx.pool.address()),
                    InstructionAccount::writable_signer(ctx.user.address()),
                    InstructionAccount::readonly(ctx.owner.address()),
                    InstructionAccount::readonly(ctx.mint_a.address()),
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
                    InstructionAccount::writable(beneficiary_0.address()),
                    InstructionAccount::writable(beneficiary_1.address()),
                ],
                [
                    ctx.pool,
                    ctx.user,
                    ctx.owner,
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
                    beneficiary_0,
                    beneficiary_1,
                ],
                &instruction_data,
                signer_seeds,
            ),
            [beneficiary_0, beneficiary_1, beneficiary_2] => invoke_with_accounts(
                [
                    InstructionAccount::writable(ctx.pool.address()),
                    InstructionAccount::writable_signer(ctx.user.address()),
                    InstructionAccount::readonly(ctx.owner.address()),
                    InstructionAccount::readonly(ctx.mint_a.address()),
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
                    InstructionAccount::writable(beneficiary_0.address()),
                    InstructionAccount::writable(beneficiary_1.address()),
                    InstructionAccount::writable(beneficiary_2.address()),
                ],
                [
                    ctx.pool,
                    ctx.user,
                    ctx.owner,
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
                    beneficiary_0,
                    beneficiary_1,
                    beneficiary_2,
                ],
                &instruction_data,
                signer_seeds,
            ),
            [beneficiary_0, beneficiary_1, beneficiary_2, beneficiary_3] => invoke_with_accounts(
                [
                    InstructionAccount::writable(ctx.pool.address()),
                    InstructionAccount::writable_signer(ctx.user.address()),
                    InstructionAccount::readonly(ctx.owner.address()),
                    InstructionAccount::readonly(ctx.mint_a.address()),
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
                    InstructionAccount::writable(beneficiary_0.address()),
                    InstructionAccount::writable(beneficiary_1.address()),
                    InstructionAccount::writable(beneficiary_2.address()),
                    InstructionAccount::writable(beneficiary_3.address()),
                ],
                [
                    ctx.pool,
                    ctx.user,
                    ctx.owner,
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
                        InstructionAccount::writable(ctx.pool.address()),
                        InstructionAccount::writable_signer(ctx.user.address()),
                        InstructionAccount::readonly(ctx.owner.address()),
                        InstructionAccount::readonly(ctx.mint_a.address()),
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
                        InstructionAccount::writable(beneficiary_0.address()),
                        InstructionAccount::writable(beneficiary_1.address()),
                        InstructionAccount::writable(beneficiary_2.address()),
                        InstructionAccount::writable(beneficiary_3.address()),
                        InstructionAccount::writable(beneficiary_4.address()),
                    ],
                    [
                        ctx.pool,
                        ctx.user,
                        ctx.owner,
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
    instruction_data: &[u8; 24],
    signer_seeds: &[Signer],
) -> ProgramResult {
    let instruction = InstructionView {
        program_id: &SCALE_AMM_PROGRAM_ID,
        accounts: &accounts,
        data: instruction_data,
    };

    invoke_signed(&instruction, &account_infos, signer_seeds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_scale_amm_swap_data() {
        let buy = ScaleAmmSwapData::try_from(&[0u8][..]).unwrap();
        assert_eq!(buy.side, ScaleAmmSide::Buy);

        let sell = ScaleAmmSwapData::try_from(&[1u8][..]).unwrap();
        assert_eq!(sell.side, ScaleAmmSide::Sell);

        let empty: &[u8] = &[];

        assert_eq!(
            ScaleAmmSwapData::try_from(empty).unwrap_err(),
            ProgramError::InvalidInstructionData
        );
        assert_eq!(
            ScaleAmmSwapData::try_from(&[2u8][..]).unwrap_err(),
            ProgramError::InvalidInstructionData
        );
    }

    #[test]
    fn test_instruction_data_layout() {
        let amount = 42u64;
        let min_out = 7u64;

        let buy = build_instruction_data(ScaleAmmSide::Buy, amount, min_out);
        assert_eq!(&buy[0..8], &BUY_DISCRIMINATOR);
        assert_eq!(&buy[8..16], &amount.to_le_bytes());
        assert_eq!(&buy[16..24], &min_out.to_le_bytes());

        let sell = build_instruction_data(ScaleAmmSide::Sell, amount, min_out);
        assert_eq!(&sell[0..8], &SELL_DISCRIMINATOR);
        assert_eq!(&sell[8..16], &amount.to_le_bytes());
        assert_eq!(&sell[16..24], &min_out.to_le_bytes());
    }
}
