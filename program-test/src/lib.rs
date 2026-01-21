#![cfg_attr(any(target_arch = "bpf", target_os = "solana"), no_std)]
#![allow(unexpected_cfgs)]

use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

mod deposit;
mod swap;

#[cfg(feature = "upstream-bpf")]
pinocchio::nostd_panic_handler!();
#[cfg(feature = "upstream-bpf")]
pinocchio::no_allocator!();
#[cfg(feature = "upstream-bpf")]
pinocchio::program_entrypoint!(process_instruction);

#[cfg(not(feature = "upstream-bpf"))]
pinocchio::nostd_panic_handler!();
#[cfg(not(feature = "upstream-bpf"))]
pinocchio::entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match discriminator {
        0 => deposit::process(accounts, data),
        1 => swap::process(accounts, data),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}
