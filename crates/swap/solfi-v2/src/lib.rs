#![no_std]

mod swap;
#[cfg(feature = "swap-with-parameters")]
mod swap_with_parameters;

pub use swap::*;
#[cfg(feature = "swap-with-parameters")]
pub use swap_with_parameters::*;

use solana_address::Address;

pub const SOLFI_V2_PROGRAM_ID: Address =
    Address::from_str_const("SV2EYYJyRz2YhfXwXnhNAevDEui5Q6yrfyo13WtupPF");

pub(crate) const SWAP_DISCRIMINATOR: u8 = 7;

pub struct SolFiV2;
