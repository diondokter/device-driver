#![allow(async_fn_in_trait)]
#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]
#![doc = include_str!(concat!("../", env!("CARGO_PKG_README")))]

use core::fmt::{Debug, Display};
use core::marker::PhantomData;

mod buffer;
mod command;
mod fieldset;
mod register;

pub use buffer::*;
pub use command::*;
pub use fieldset::*;
pub use register::*;

pub use embedded_io;
pub use embedded_io_async;

#[doc(hidden)]
pub mod ops;

#[cfg(feature = "macros")]
pub use device_driver_macros::*;

/// Trait implemented on every generated block/device.
pub trait Block: Sized {
    /// The interface used by the block
    type Interface;
    /// The register address type
    type RegisterAddressType: Address;
    /// The command address type
    type CommandAddressType;
    /// The buffer address type
    type BufferAddressType;

    /// Get a reference to the inner interface.
    /// With it you can do out-of-band operations that aren't defined in the generated code.
    fn interface(&mut self) -> &mut Self::Interface;

    /// Start a multi-read transaction
    ///
    /// You can chain reads by calling [register::MultiRegisterOperation::with].
    /// Once chained, call [register::MultiRegisterOperation::execute] to perform the read.
    fn multi_read(
        &mut self,
    ) -> register::MultiRegisterOperation<'_, Self, Self::RegisterAddressType, (), RO> {
        register::MultiRegisterOperation {
            device: self,
            start_address: None,
            field_sets: (),
            _phantom: PhantomData,
        }
    }

    /// Start a multi-write transaction
    ///
    /// You can chain writes by calling [register::MultiRegisterOperation::with].
    /// Once chained, call [register::MultiRegisterOperation::execute] to perform the read.
    fn multi_write(
        &mut self,
    ) -> register::MultiRegisterOperation<'_, Self, Self::RegisterAddressType, (), WO> {
        register::MultiRegisterOperation {
            device: self,
            start_address: None,
            field_sets: (),
            _phantom: PhantomData,
        }
    }

    /// Start a multi-modify transaction
    ///
    /// You can chain modifies by calling [register::MultiRegisterOperation::with].
    /// Once chained, call [register::MultiRegisterOperation::execute] to perform the read.
    fn multi_modify(
        &mut self,
    ) -> register::MultiRegisterOperation<'_, Self, Self::RegisterAddressType, (), RW> {
        register::MultiRegisterOperation {
            device: self,
            start_address: None,
            field_sets: (),
            _phantom: PhantomData,
        }
    }
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

#[doc(hidden)]
pub trait Address: Copy {
    fn add(self, val: i32) -> Self;
}

impl Address for u8 {
    fn add(self, val: i32) -> Self {
        (self as i32 + val).try_into().unwrap()
    }
}
impl Address for u16 {
    fn add(self, val: i32) -> Self {
        (self as i32 + val).try_into().unwrap()
    }
}
impl Address for u32 {
    fn add(self, val: i32) -> Self {
        self.checked_add_signed(val).unwrap()
    }
}
impl Address for u64 {
    fn add(self, val: i32) -> Self {
        self.checked_add_signed(val as i64).unwrap()
    }
}
impl Address for i8 {
    fn add(self, val: i32) -> Self {
        (self as i32 + val).try_into().unwrap()
    }
}
impl Address for i16 {
    fn add(self, val: i32) -> Self {
        (self as i32 + val).try_into().unwrap()
    }
}
impl Address for i32 {
    fn add(self, val: i32) -> Self {
        self + val
    }
}
impl Address for i64 {
    fn add(self, val: i32) -> Self {
        self + val as i64
    }
}
