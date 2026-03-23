use {
    beethoven::{try_from_deposit_context, Deposit, DepositContext, DepositData},
    pinocchio::{error::ProgramError, AccountView, ProgramResult},
};

/// Instruction data for Deposit
///
/// Layout:
/// [0..8] - amount (u64, little-endian)
/// [8..] - protocol-specific data (parsed via DepositContext::try_from_deposit_data)
pub struct DepositInstructionData<'a> {
    pub amount: u64,
    pub extra_data: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for DepositInstructionData<'a> {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() < 8 {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self {
            amount: u64::from_le_bytes(data[0..8].try_into().unwrap()),
            extra_data: &data[8..],
        })
    }
}

pub struct DepositInstruction<'a> {
    pub accounts: DepositContext<'a>,
    pub data: DepositData,
    pub amount: u64,
}

impl<'a> TryFrom<(&'a [AccountView], &[u8])> for DepositInstruction<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a [AccountView], &[u8])) -> Result<Self, Self::Error> {
        let instruction_data = DepositInstructionData::try_from(data)?;
        let ctx = try_from_deposit_context(accounts)?;
        let (deposit_data, _remaining_data) =
            ctx.try_from_deposit_data(instruction_data.extra_data)?;

        Ok(Self {
            accounts: ctx,
            data: deposit_data,
            amount: instruction_data.amount,
        })
    }
}

impl<'a> DepositInstruction<'a> {
    pub fn process(&self) -> ProgramResult {
        DepositContext::deposit(&self.accounts, self.amount, &self.data)
    }
}

pub fn process(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    DepositInstruction::try_from((accounts, data))?.process()
}
