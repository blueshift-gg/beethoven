use {
    beethoven::{try_from_tagged_swap_context, Swap, SwapContext, SwapData, SwapProtocolTag},
    pinocchio::{error::ProgramError, AccountView, ProgramResult},
};

const SWAP_LEG_HEADER_LEN: usize = 4;

/// Instruction data for Swap
///
/// Layout:
/// [0..8]  - in_amount (u64, little-endian)
/// [8..16] - minimum_out_amount (u64, little-endian)
/// [16..20] - swap leg header
/// [20..]   - protocol-specific extra data with exact byte length from the header
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

pub(crate) struct SwapLegHeader {
    pub protocol_tag: SwapProtocolTag,
    pub remaining_accounts_len: usize,
    pub extra_data_len: usize,
}

fn split_data_checked(data: &[u8], count: usize) -> Result<(&[u8], &[u8]), ProgramError> {
    data.split_at_checked(count)
        .ok_or(ProgramError::InvalidInstructionData)
}

pub(crate) fn parse_swap_leg_header(data: &[u8]) -> Result<(SwapLegHeader, &[u8]), ProgramError> {
    let (header_bytes, remaining_data) = split_data_checked(data, SWAP_LEG_HEADER_LEN)?;
    let protocol_tag = SwapProtocolTag::from_byte(header_bytes[0])?;
    let remaining_accounts_len = header_bytes[1] as usize;
    let extra_data_len = u16::from_le_bytes([header_bytes[2], header_bytes[3]]) as usize;

    Ok((
        SwapLegHeader {
            protocol_tag,
            remaining_accounts_len,
            extra_data_len,
        },
        remaining_data,
    ))
}

pub(crate) fn parse_tagged_swap_context_and_data<'a>(
    accounts: &'a [AccountView],
    data: &'a [u8],
) -> Result<TaggedSwapContext<'a>, ProgramError> {
    let (header, data) = parse_swap_leg_header(data)?;
    let (extra_data, remaining_data) = split_data_checked(data, header.extra_data_len)?;

    let (accounts, remaining_accounts) =
        try_from_tagged_swap_context(header.protocol_tag, accounts, header.remaining_accounts_len)?;
    let data = accounts.try_from_swap_data_exact(extra_data)?;

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

        if !parsed.remaining_accounts.is_empty() || !parsed.remaining_data.is_empty() {
            return Err(ProgramError::InvalidInstructionData);
        }

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
