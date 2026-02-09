#![no_std]

mod swap;
#[cfg(feature = "swap-with-parameters")]
mod swap_with_parameters;

pub use swap::*;
#[cfg(feature = "swap-with-parameters")]
pub use swap_with_parameters::*;

use solana_address::Address;

pub const MANIFEST_PROGRAM_ID: Address =
    Address::from_str_const("MNFSTqtC93rEfYHB6hF82sKdZpUDFWkViLByLd1k1Ms");

pub(crate) const SWAP_DISCRIMINATOR: u8 = 13;

pub struct Manifest;
