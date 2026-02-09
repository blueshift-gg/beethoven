#![no_std]

mod swap;
#[cfg(feature = "swap-with-parameters")]
mod swap_with_parameters;

pub use swap::*;
#[cfg(feature = "swap-with-parameters")]
pub use swap_with_parameters::*;

use solana_address::Address;

pub const GAMMA_PROGRAM_ID: Address =
    Address::from_str_const("GAMMA7meSFWaBXF25oSUgmGRwaW6sCMFLmBNiMSdbHVT");

pub(crate) const SWAP_DISCRIMINATOR: [u8; 8] = [239, 82, 192, 187, 160, 26, 223, 223];

pub struct Gamma;
