#![no_std]

mod swap;
#[cfg(feature = "swap-with-parameters")]
mod swap_with_parameters;

pub use swap::*;
#[cfg(feature = "swap-with-parameters")]
pub use swap_with_parameters::*;

use solana_address::Address;

pub const FUTARCHY_PROGRAM_ID: Address =
    Address::from_str_const("FUTARELBfJfQ8RDGhg1wdhddq1odMAJUePHFuBYfUxKq");

pub(crate) const SWAP_DISCRIMINATOR: [u8; 8] = [167, 97, 12, 231, 237, 78, 166, 251];

pub struct Futarchy;

#[repr(u8)]
pub enum SwapType {
    Buy = 0,
    Sell = 1,
}
