use {
    crate::swap::parse_tagged_swap_context_and_data,
    beethoven::{Swap, SwapContext},
    pinocchio::{error::ProgramError, AccountView, ProgramResult},
};

/// Multi-swap instruction data layout (after discriminator):
///
/// [num_swaps: u8]
/// Per swap (repeated num_swaps times):
///   [in_amount: u64 LE]
///   [min_out_amount: u64 LE]
///   [swap_leg_header: 4 bytes]
///   [extra_data: exact byte length from the header]
///
/// Accounts are a flat concatenation. Each swap consumes its fixed account
/// prefix plus the explicit remaining-account tail from the header.
pub fn process(accounts: &[AccountView], data: &[u8]) -> ProgramResult {
    if data.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }

    let num_swaps = data[0] as usize;
    let mut remaining_data = &data[1..];
    let mut remaining_accounts = accounts;

    for _ in 0..num_swaps {
        if remaining_data.len() < 16 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let in_amount = u64::from_le_bytes(
            remaining_data[..8]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );
        let min_out_amount = u64::from_le_bytes(
            remaining_data[8..16]
                .try_into()
                .map_err(|_| ProgramError::InvalidInstructionData)?,
        );
        remaining_data = &remaining_data[16..];

        let parsed = parse_tagged_swap_context_and_data(remaining_accounts, remaining_data)?;

        SwapContext::swap(&parsed.accounts, in_amount, min_out_amount, &parsed.data)?;

        remaining_accounts = parsed.remaining_accounts;
        remaining_data = parsed.remaining_data;
    }

    if !remaining_accounts.is_empty() || !remaining_data.is_empty() {
        return Err(ProgramError::InvalidInstructionData);
    }

    Ok(())
}
