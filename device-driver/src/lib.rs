#![allow(async_fn_in_trait)]
#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]
#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

use core::fmt::{Debug, Display};

pub use bitvec;
pub use embedded_io;
pub use embedded_io_async;

mod register;
pub use register::*;
mod command;
pub use command::*;
mod buffer;
pub use buffer::*;

#[cfg(feature = "macros")]
pub use device_driver_macros::*;

#[doc(hidden)]
pub trait FieldSet {
    /// The inner buffer type
    type BUFFER: From<Self> + Into<Self> + AsMut<[u8]> + AsRef<[u8]>
    where
        Self: Sized;

    /// Create a new instance, loaded with the default value (if any)
    fn new_with_default() -> Self;

    /// Create a new instance, loaded all 0's
    fn new_with_zero() -> Self;
}

/// The error returned by the generated [TryFrom]s.
/// It contains the base type of the enum.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ConversionError<T>(pub T);

impl<T: Display> Display for ConversionError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Could not convert value from `{}`", self.0)
    }
}

impl<T: Display + Debug> core::error::Error for ConversionError<T> {}

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
