use {
    beethoven::{try_from_swap_with_parameters_context, swap_with_parameters as do_swap_with_parameters},
    pinocchio::{error::ProgramError, AccountView, ProgramResult},
};

pub struct SwapWithParametersInstructionData<'a> {
    pub in_amount: u64,
    pub minimum_out_amount: u64,
    pub extra_data: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for SwapWithParametersInstructionData<'a> {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() < 16 {
            return Err(ProgramError::InvalidInstructionData);
        }
        Ok(Self {
            in_amount: u64::from_le_bytes(data[0..8].try_into().unwrap()),
            minimum_out_amount: u64::from_le_bytes(data[8..16].try_into().unwrap()),
            extra_data: &data[16..],
        })
    }
}

pub fn process(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let instruction_data = SwapWithParametersInstructionData::try_from(data)?;
    let (_, ctx) = try_from_swap_with_parameters_context(accounts)?;
    let extra = ctx.try_from_extra(instruction_data.extra_data)?;
    do_swap_with_parameters(
        accounts,
        instruction_data.in_amount,
        instruction_data.minimum_out_amount,
        &extra,
    )
}
