#![no_std]
#![allow(unexpected_cfgs)]

use pinocchio::{
    error::ProgramError, no_allocator, nostd_panic_handler, AccountView, Address, ProgramResult,
};

mod deposit;
mod swap;

no_allocator!();
nostd_panic_handler!();

#[cfg(target_arch = "bpf")]
pinocchio::program_entrypoint!(process_instruction);

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
