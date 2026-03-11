#![no_std]

// Re-export core traits
pub use beethoven_core::{Deposit, Swap};
#[cfg(feature = "jupiter-deposit")]
pub use beethoven_deposit_jupiter as jupiter;
// Re-export protocol crates under feature flags
#[cfg(feature = "kamino-deposit")]
pub use beethoven_deposit_kamino as kamino;
#[cfg(feature = "aldrin-swap")]
pub use beethoven_swap_aldrin as aldrin;
#[cfg(feature = "aldrin_v2-swap")]
pub use beethoven_swap_aldrin_v2 as aldrin_v2;
#[cfg(feature = "futarchy-swap")]
pub use beethoven_swap_futarchy as futarchy;
#[cfg(feature = "gamma-swap")]
pub use beethoven_swap_gamma as gamma;
#[cfg(feature = "heaven-swap")]
pub use beethoven_swap_heaven as heaven;
#[cfg(feature = "manifest-swap")]
pub use beethoven_swap_manifest as manifest;
#[cfg(feature = "omnipair-swap")]
pub use beethoven_swap_omnipair as omnipair;
#[cfg(feature = "perena-swap")]
pub use beethoven_swap_perena as perena;
#[cfg(feature = "scale_amm-swap")]
pub use beethoven_swap_scale_amm as scale_amm;
#[cfg(feature = "scale_vmm-swap")]
pub use beethoven_swap_scale_vmm as scale_vmm;
#[cfg(feature = "solfi-swap")]
pub use beethoven_swap_solfi as solfi;
#[cfg(feature = "solfi_v2-swap")]
pub use beethoven_swap_solfi_v2 as solfi_v2;

// Context enums and convenience functions
mod context;
pub use context::*;
