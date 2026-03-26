use {
    beethoven::{try_from_tagged_swap_context, Swap, SwapContext, SwapData, SwapProtocolTag},
    pinocchio::{error::ProgramError, AccountView, ProgramResult},
};

/// Instruction data for Swap
///
/// Layout:
/// [0..8]  - in_amount (u64, little-endian)
/// [8..16] - minimum_out_amount (u64, little-endian)
/// [16]    - protocol tag
/// [17..?] - optional remaining-accounts length for dynamic protocols
/// [..]    - protocol-specific data (parsed via SwapContext::try_from_swap_data)
pub struct SwapInstructionData<'a> {
    pub in_amount: u64,
    pub minimum_out_amount: u64,
    pub extra_data: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for SwapInstructionData<'a> {
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

pub(crate) struct TaggedSwapContext<'a> {
    pub accounts: SwapContext<'a>,
    pub data: SwapData<'a>,
    pub remaining_accounts: &'a [AccountView],
    pub remaining_data: &'a [u8],
}

pub(crate) fn parse_tagged_swap_context_and_data<'a>(
    accounts: &'a [AccountView],
    data: &'a [u8],
) -> Result<TaggedSwapContext<'a>, ProgramError> {
    let (&tag_byte, data) = data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;
    let protocol_tag = SwapProtocolTag::from_byte(tag_byte)?;

    let (remaining_accounts_len, data) = if protocol_tag.uses_remaining_accounts_len() {
        let (&remaining_accounts_len, data) = data
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;
        (remaining_accounts_len as usize, data)
    } else {
        (0, data)
    };

    let (accounts, remaining_accounts) =
        try_from_tagged_swap_context(protocol_tag, accounts, remaining_accounts_len)?;
    let (data, remaining_data) = accounts.try_from_swap_data(data)?;

    Ok(TaggedSwapContext {
        accounts,
        data,
        remaining_accounts,
        remaining_data,
    })
}

pub struct SwapInstruction<'a> {
    pub accounts: SwapContext<'a>,
    pub data: SwapData<'a>,
    pub in_amount: u64,
    pub minimum_out_amount: u64,
}

impl<'a> TryFrom<(&'a [AccountView], &'a [u8])> for SwapInstruction<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a [AccountView], &'a [u8])) -> Result<Self, Self::Error> {
        let instruction_data = SwapInstructionData::try_from(data)?;
        let parsed = parse_tagged_swap_context_and_data(accounts, instruction_data.extra_data)?;

        Ok(Self {
            accounts: parsed.accounts,
            data: parsed.data,
            in_amount: instruction_data.in_amount,
            minimum_out_amount: instruction_data.minimum_out_amount,
        })
    }
}

impl<'a> SwapInstruction<'a> {
    pub fn process(&self) -> ProgramResult {
        SwapContext::swap(
            &self.accounts,
            self.in_amount,
            self.minimum_out_amount,
            &self.data,
        )
    }
}

pub fn process(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    SwapInstruction::try_from((accounts, data))?.process()
}
