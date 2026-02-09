#![no_std]

mod swap;
#[cfg(feature = "swap-with-parameters")]
mod swap_with_parameters;

pub use swap::*;
#[cfg(feature = "swap-with-parameters")]
pub use swap_with_parameters::*;

use solana_address::Address;

pub const ALDRIN_V2_PROGRAM_ID: Address =
    Address::from_str_const("CURVGoZn8zycx6FXwwevgBTB2gVvdbGTEpvMJDbgs2t4");

pub(crate) const SWAP_DISCRIMINATOR: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];

pub struct AldrinV2;

#[repr(u8)]
pub enum Side {
    Bid = 0,
    Ask = 1,
}
