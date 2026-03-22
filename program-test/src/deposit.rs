use {
    beethoven::{try_from_deposit_context, Deposit, DepositContext},
    pinocchio::{error::ProgramError, AccountView, ProgramResult},
};

/// Instruction data for Deposit
///
/// Layout:
/// [0..8] - amount (u64, little-endian)
pub struct DepositInstructionData {
    pub amount: u64,
}

impl TryFrom<&[u8]> for DepositInstructionData {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < 8 {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self {
            amount: u64::from_le_bytes(data[0..8].try_into().unwrap()),
        })
    }
}

pub struct DepositInstruction<'a> {
    pub accounts: DepositContext<'a>,
    pub data: DepositInstructionData,
    pub remaining_accounts: &'a [AccountView],
}

impl<'a> TryFrom<(&'a [AccountView], &[u8])> for DepositInstruction<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a [AccountView], &[u8])) -> Result<Self, Self::Error> {
        let (ctx, remaining_accounts) = try_from_deposit_context(accounts)?;
        Ok(Self {
            accounts: ctx,
            data: DepositInstructionData::try_from(data)?,
            remaining_accounts,
        })
    }
}

impl<'a> DepositInstruction<'a> {
    pub fn process(&self) -> ProgramResult {
        DepositContext::deposit(&self.accounts, self.data.amount, self.remaining_accounts)
    }
}

pub fn process(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    DepositInstruction::try_from((accounts, data))?.process()
}
