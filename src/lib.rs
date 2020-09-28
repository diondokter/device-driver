//! Crate providing some tools to create better device drivers more easily
//!
//! The best source to see how it works is the examples folder.

#![no_std]
#![forbid(missing_docs)]

pub use bit::Bit;
pub use bitvec;

// #[macro_use]
// pub mod hl;
/// The module with tools for creating the low-level parts of the device driver
#[macro_use]
pub mod ll;

mod bit;
