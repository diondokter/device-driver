#![allow(async_fn_in_trait)]
#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]
#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

use core::fmt::{Debug, Display};

pub use embedded_io;
pub use embedded_io_async;

mod register;
pub use register::*;
mod command;
pub use command::*;
mod buffer;
pub use buffer::*;

// #[doc(hidden)]
pub mod ops;

#[cfg(feature = "macros")]
pub use device_driver_macros::*;

// #[doc(hidden)]
pub trait Fieldset: Default {
    const METADATA: FieldsetMetadata;

    fn get_inner_buffer(&self) -> &[u8];
    fn get_inner_buffer_mut(&mut self) -> &mut [u8];
}

/// Metadata about fieldsets
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
pub struct FieldsetMetadata {
    /// The byte order of the fieldset
    pub byte_order: ByteOrder,
}

impl FieldsetMetadata {
    /// A default that allow you to construct the metadata
    pub const DEFAULT: Self = Self {
        byte_order: ByteOrder::LE,
    };

    /// Create a new instance with the default value
    pub const fn new() -> Self {
        Self::DEFAULT
    }

    /// Set the byte order
    pub const fn with_byte_order(self, byte_order: ByteOrder) -> Self {
        Self { byte_order, ..self }
    }
}

impl Default for FieldsetMetadata {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Value representing the byte order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ByteOrder {
    /// Little endian
    LE,
    /// Big endian
    BE,
}

/// The error returned by the generated [`TryFrom`]s.
/// It contains the base type of the enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ConversionError<T> {
    /// The value of the thing that was tried to be converted
    pub source: T,
    /// The name of the target type
    pub target: &'static str,
}

impl<T: Display> Display for ConversionError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Could not convert value from `{}` to type `{}`",
            self.source, self.target
        )
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
pub trait ReadCapability {}
#[doc(hidden)]
pub trait WriteCapability {}

impl WriteCapability for WO {}

impl ReadCapability for RO {}

impl WriteCapability for RW {}
impl ReadCapability for RW {}
