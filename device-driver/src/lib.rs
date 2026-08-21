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

mod repeats;

pub use buffer::*;
pub use command::*;
pub use fieldset::*;
pub use register::*;

pub use repeats::*;

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
    type CommandAddressType: Address;
    /// The buffer address type
    type BufferAddressType: Address;
    /// The address mode of the registers in this block
    type RegisterAddressMode;

    /// Get a reference to the inner interface.
    /// With it you can do out-of-band operations that aren't defined in the generated code.
    fn interface(&mut self) -> &mut Self::Interface;

    /// Start a bulk-read transaction
    ///
    /// You can chain reads by calling [`register::BulkRegisterOperation::with`].
    /// Once chained, call [`register::BulkRegisterOperation::execute`] to perform the read.
    fn bulk_read(
        &mut self,
    ) -> register::BulkRegisterOperation<
        '_,
        Self,
        <Self::Interface as RegisterInterfaceBase>::AddressType,
        (),
        RO,
    >
    where
        Self::Interface: RegisterInterfaceBase,
        Self::RegisterAddressMode: AddressMode,
    {
        register::BulkRegisterOperation {
            block: self,
            start_address: None,
            next_address: None,
            field_sets: (),
            _phantom: PhantomData,
        }
    }

    /// Start a bulk-write transaction
    ///
    /// You can chain writes by calling [`register::BulkRegisterOperation::with`].
    /// Once chained, call [`register::BulkRegisterOperation::execute`] to perform the write.
    fn bulk_write(
        &mut self,
    ) -> register::BulkRegisterOperation<
        '_,
        Self,
        <Self::Interface as RegisterInterfaceBase>::AddressType,
        (),
        WO,
    >
    where
        Self::Interface: RegisterInterfaceBase,
        Self::RegisterAddressMode: AddressMode,
    {
        register::BulkRegisterOperation {
            block: self,
            start_address: None,
            next_address: None,
            field_sets: (),
            _phantom: PhantomData,
        }
    }

    /// Start a bulk-modify transaction
    ///
    /// You can chain modifies by calling [`register::BulkRegisterOperation::with`].
    /// Once chained, call [`register::BulkRegisterOperation::execute`] to perform the modify.
    fn bulk_modify(
        &mut self,
    ) -> register::BulkRegisterOperation<
        '_,
        Self,
        <Self::Interface as RegisterInterfaceBase>::AddressType,
        (),
        RW,
    >
    where
        Self::Interface: RegisterInterfaceBase,
        Self::RegisterAddressMode: AddressMode,
    {
        register::BulkRegisterOperation {
            block: self,
            start_address: None,
            next_address: None,
            field_sets: (),
            _phantom: PhantomData,
        }
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

trait SealedAddress {}

/// A trait implemented for the types that can be used as an address
#[expect(private_bounds, reason = "sealed on purpose")]
#[cfg(feature = "defmt")]
pub trait Address: Copy + Eq + Display + Debug + defmt::Format + SealedAddress {
    #[doc(hidden)]
    const ZERO: Self;
    #[doc(hidden)]
    fn add(self, val: i32) -> Self;
}
/// A trait implemented for the types that can be used as an address
#[expect(private_bounds, reason = "sealed on purpose")]
#[cfg(not(feature = "defmt"))]
pub trait Address: Copy + Eq + Display + Debug + SealedAddress {
    #[doc(hidden)]
    const ZERO: Self;
    #[doc(hidden)]
    fn add(self, val: i32) -> Self;
}

impl SealedAddress for u8 {}
impl Address for u8 {
    const ZERO: Self = 0;
    fn add(self, val: i32) -> Self {
        (self as i32 + val).try_into().unwrap()
    }
}
impl SealedAddress for u16 {}
impl Address for u16 {
    const ZERO: Self = 0;
    fn add(self, val: i32) -> Self {
        (self as i32 + val).try_into().unwrap()
    }
}
impl SealedAddress for u32 {}
impl Address for u32 {
    const ZERO: Self = 0;
    fn add(self, val: i32) -> Self {
        self.checked_add_signed(val).unwrap()
    }
}
impl SealedAddress for u64 {}
impl Address for u64 {
    const ZERO: Self = 0;
    fn add(self, val: i32) -> Self {
        self.checked_add_signed(val as i64).unwrap()
    }
}
impl SealedAddress for i8 {}
impl Address for i8 {
    const ZERO: Self = 0;
    fn add(self, val: i32) -> Self {
        (self as i32 + val).try_into().unwrap()
    }
}
impl SealedAddress for i16 {}
impl Address for i16 {
    const ZERO: Self = 0;
    fn add(self, val: i32) -> Self {
        (self as i32 + val).try_into().unwrap()
    }
}
impl SealedAddress for i32 {}
impl Address for i32 {
    const ZERO: Self = 0;
    fn add(self, val: i32) -> Self {
        self + val
    }
}
impl SealedAddress for i64 {}
impl Address for i64 {
    const ZERO: Self = 0;
    fn add(self, val: i32) -> Self {
        self + val as i64
    }
}

#[diagnostic::on_unimplemented(
    message = "no `register-address-mode` is specified in the driver, so bulk register operations are not possible",
    label = "not supported for this driver",
    note = "if you are the author of the driver, specify `register-address-mode` in the device config to enable this feature if the device supports it",
    note = "not all devices support this feature"
)]
#[doc(hidden)]
pub trait AddressMode {
    #[doc(hidden)]
    fn next_address<A: Address>(current_address: A, current_size: usize) -> A;
}

#[doc(hidden)]
pub struct MappedAddressMode;
impl AddressMode for MappedAddressMode {
    #[inline]
    fn next_address<A: Address>(current_address: A, current_size: usize) -> A {
        // Current size can be cast to i32 fine because this is the size of a fieldset
        // Fieldsets are limited to 1MB in size

        current_address.add(current_size as i32)
    }
}

#[doc(hidden)]
pub struct IndexedAddressMode;
impl AddressMode for IndexedAddressMode {
    #[inline]
    fn next_address<A: Address>(current_address: A, _current_size: usize) -> A {
        current_address.add(1)
    }
}
