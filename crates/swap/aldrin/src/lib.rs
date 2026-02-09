#![no_std]

mod swap;
#[cfg(feature = "swap-with-parameters")]
mod swap_with_parameters;

pub use swap::*;
#[cfg(feature = "swap-with-parameters")]
pub use swap_with_parameters::*;

use solana_address::Address;

pub const ALDRIN_PROGRAM_ID: Address =
    Address::from_str_const("AMM55ShdkoGRB5jVYPjWziwk8m5MpwyDgsMWHaMSQWH6");

pub(crate) const SWAP_DISCRIMINATOR: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];

pub struct Aldrin;

#[repr(u8)]
pub enum Side {
    Bid = 0,
    Ask = 1,
}
