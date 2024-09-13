#![allow(async_fn_in_trait)]
#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]
#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

pub use bitvec;
pub use device_driver_macros::*;
pub use embedded_io;
pub use embedded_io_async;

mod register;
pub use register::*;
mod command;
pub use command::*;
mod buffer;
pub use buffer::*;

#[doc(hidden)]
pub trait FieldSet {
    /// The inner buffer type
    type BUFFER: From<Self> + Into<Self> + AsMut<[u8]> + AsRef<[u8]>
    where
        Self: Sized;

    /// Create a new instance, loaded with the default value (if any)
    fn new() -> Self;

    /// Create a new instance, loaded all 0's
    fn new_zero() -> Self;
}

#[doc(hidden)]
pub struct WO;
#[doc(hidden)]
pub struct RO;
#[doc(hidden)]
pub struct RW;
#[doc(hidden)]
pub struct RC;
#[doc(hidden)]
pub struct CO;

#[doc(hidden)]
pub trait ReadCapability {}
#[doc(hidden)]
pub trait WriteCapability {}

impl WriteCapability for WO {}

impl ReadCapability for RO {}

impl WriteCapability for RW {}
impl ReadCapability for RW {}
