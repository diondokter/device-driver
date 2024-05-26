#![allow(async_fn_in_trait)]
#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]
#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

pub use bitvec;
pub use device_driver_macros::*;
pub use funty;
pub use num_enum;

mod register;
pub use register::*;
mod command;
pub use command::*;
