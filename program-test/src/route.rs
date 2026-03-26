use {
    crate::swap::parse_tagged_swap_context_and_data,
    beethoven::{Swap, SwapContext},
    pinocchio::{error::ProgramError, AccountView, ProgramResult},
};

const TOKEN_ACCOUNT_AMOUNT_OFFSET: usize = 64;
const TOKEN_ACCOUNT_AMOUNT_END: usize = TOKEN_ACCOUNT_AMOUNT_OFFSET + 8;

/// Route instruction data layout (after discriminator):
///
/// [0..8]   - initial_in_amount (u64, little-endian)
/// [8..16]  - minimum_final_out_amount (u64, little-endian)
/// [16]     - num_legs (u8)
/// Per leg (repeated num_legs times):
///   [protocol_tag: u8]
///   [remaining_accounts_len: u8] only for protocols with dynamic account tails
///   [extra_data: protocol determines length]
pub struct RouteInstructionData<'a> {
    pub initial_in_amount: u64,
    pub minimum_final_out_amount: u64,
    pub num_legs: usize,
    pub extra_data: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for RouteInstructionData<'a> {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() < 17 {
            return Err(ProgramError::InvalidInstructionData);
        }

        Ok(Self {
            initial_in_amount: u64::from_le_bytes(data[0..8].try_into().unwrap()),
            minimum_final_out_amount: u64::from_le_bytes(data[8..16].try_into().unwrap()),
            num_legs: data[16] as usize,
            extra_data: &data[17..],
        })
    }
}

fn read_token_account_amount(account: &AccountView) -> Result<u64, ProgramError> {
    if account.data_len() < TOKEN_ACCOUNT_AMOUNT_END {
        return Err(ProgramError::InvalidAccountData);
    }

    let data = unsafe { account.borrow_unchecked() };
    Ok(u64::from_le_bytes(
        data[TOKEN_ACCOUNT_AMOUNT_OFFSET..TOKEN_ACCOUNT_AMOUNT_END]
            .try_into()
            .unwrap(),
    ))
}

pub fn process(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    let instruction_data = RouteInstructionData::try_from(data)?;

    if instruction_data.num_legs == 0 {
        return Err(ProgramError::InvalidInstructionData);
    }

    let mut remaining_accounts = accounts;
    let mut remaining_data = instruction_data.extra_data;
    let mut carried_in_amount = instruction_data.initial_in_amount;
    let mut previous_output_account: Option<&AccountView> = None;

    for leg_index in 0..instruction_data.num_legs {
        let parsed = parse_tagged_swap_context_and_data(remaining_accounts, remaining_data)?;

        let (input_token_account, output_token_account) =
            parsed.accounts.token_accounts(&parsed.data)?;

        if let Some(previous_output_account) = previous_output_account {
            if previous_output_account.address() != input_token_account.address() {
                return Err(ProgramError::InvalidAccountData);
            }
        }

        let output_before = read_token_account_amount(output_token_account)?;
        let minimum_out_amount = if leg_index + 1 == instruction_data.num_legs {
            instruction_data.minimum_final_out_amount
        } else {
            0
        };

        SwapContext::swap(
            &parsed.accounts,
            carried_in_amount,
            minimum_out_amount,
            &parsed.data,
        )?;

        let output_after = read_token_account_amount(output_token_account)?;
        let output_delta = output_after
            .checked_sub(output_before)
            .ok_or(ProgramError::InvalidAccountData)?;

        carried_in_amount = output_delta;
        previous_output_account = Some(output_token_account);
        remaining_accounts = parsed.remaining_accounts;
        remaining_data = parsed.remaining_data;
    }

    Ok(())
}
