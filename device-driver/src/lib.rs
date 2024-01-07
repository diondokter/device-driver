#![allow(async_fn_in_trait)]
#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]
#![doc = include_str!("../../README.md")]

use core::{
    convert::{TryFrom, TryInto},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub use bitvec;
pub use device_driver_macros::*;
pub use funty;
pub use num_enum;

use bitvec::{array::BitArray, field::BitField};
use funty::Integral;

/// A trait to represent the interface to the device.
///
/// This is called to write to and read from registers.
pub trait RegisterDevice {
    /// The error type
    type Error;
    /// The address type used by this interface. Should likely be an integer.
    type AddressType;

    /// Write the given data to the register.
    ///
    /// The address and the length of the register is descriped with the constant in the `R` [Register] type.
    /// The data can made into a normal slice by calling `as_raw_slice` on it.
    fn write_register<R, const SIZE_BYTES: usize>(
        &mut self,
        data: &BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error>
    where
        R: Register<SIZE_BYTES, AddressType = Self::AddressType>;

    /// Read the data from the register into the given buffer.
    ///
    /// The address and the length of the register is descriped with the constant in the `R` [Register] type.
    /// The data can made into a normal slice by calling `as_raw_slice` on it.
    fn read_register<R, const SIZE_BYTES: usize>(
        &mut self,
        data: &mut BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error>
    where
        R: Register<SIZE_BYTES, AddressType = Self::AddressType>;
}

/// A trait to represent the interface to the device.
///
/// This is called to asynchronously write to and read from registers.
pub trait AsyncRegisterDevice {
    /// The error type
    type Error;
    /// The address type used by this interface. Should likely be an integer.
    type AddressType;

    /// Write the given data to the register asynchronously.
    ///
    /// The address and the length of the register is descriped with the constant in the `R` [Register] type.
    /// The data can made into a normal slice by calling `as_raw_slice` on it.
    async fn write_register<R, const SIZE_BYTES: usize>(
        &mut self,
        data: &BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error>
    where
        R: Register<SIZE_BYTES, AddressType = Self::AddressType>;

    /// Read the data from the register asynchronously into the given buffer.
    ///
    /// The address and the length of the register is descriped with the constant in the `R` [Register] type.
    /// The data can made into a normal slice by calling `as_raw_slice` on it.
    async fn read_register<R, const SIZE_BYTES: usize>(
        &mut self,
        data: &mut BitArray<[u8; SIZE_BYTES]>,
    ) -> Result<(), Self::Error>
    where
        R: Register<SIZE_BYTES, AddressType = Self::AddressType>;
}

/// The abstraction and description of a register
pub trait Register<const SIZE_BYTES: usize> {
    /// The all-zero bits representation of this register
    const ZERO: Self;

    /// The address type of this register abstraction
    type AddressType;
    /// The address value of where this register is stored on the device
    const ADDRESS: Self::AddressType;

    /// Typestate value of how this register can be used.
    ///
    /// Should be one of:
    /// - [WriteOnly]
    /// - [ReadOnly]
    /// - [ClearOnly]
    /// - [ReadWrite]
    /// - [ReadClear]
    type RWType;

    /// The size of the register in bits
    const SIZE_BITS: usize;

    /// The type of the writer `W` type associated with this register
    type WriteFields: From<Self> + Into<Self> + Deref<Target = Self> + DerefMut
    where
        Self: Sized;
    /// The type of the reader `R` type associated with this register
    type ReadFields: From<Self> + Into<Self> + Deref<Target = Self> + DerefMut
    where
        Self: Sized;

    /// Get mutable access to the local representation of the register bits
    fn bits_mut(&mut self) -> &mut BitArray<[u8; SIZE_BYTES]>;
    /// Get access to the local representation of the register bits
    fn bits(&self) -> &BitArray<[u8; SIZE_BYTES]>;

    /// The 'default'/reset value of the register.
    /// 
    /// Optional. Standardly implemented as returning [Self::ZERO].
    fn reset_value() -> Self
    where
        Self: Sized,
    {
        Self::ZERO
    }
}

/// Object that performs actions on the device in the context of a register
pub struct RegisterOperation<'a, D, R, const SIZE_BYTES: usize>
where
    R: Register<SIZE_BYTES>,
{
    device: &'a mut D,
    _phantom: PhantomData<R>,
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    R: Register<SIZE_BYTES>,
{
    #[doc(hidden)]
    pub fn new(device: &'a mut D) -> Self {
        Self {
            device,
            _phantom: PhantomData,
        }
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: RegisterDevice<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWType: WriteCapability,
{
    /// Write to the register.
    ///
    /// The closure is given the write object initialized to the reset value of the register.
    /// If no reset value is specified for this register, this function is the same as [Self::write_with_zero].
    pub fn write(
        &mut self,
        f: impl FnOnce(&mut R::WriteFields) -> &mut R::WriteFields,
    ) -> Result<(), D::Error> {
        let mut register = R::reset_value().into();
        f(&mut register);
        self.device.write_register::<R, SIZE_BYTES>(register.bits())
    }

    /// Write to the register.
    ///
    /// The closure is given the write object initialized to all zero.
    pub fn write_with_zero(
        &mut self,
        f: impl FnOnce(&mut R::WriteFields) -> &mut R::WriteFields,
    ) -> Result<(), D::Error> {
        let mut register = R::ZERO.into();
        f(&mut register);
        self.device.write_register::<R, SIZE_BYTES>(register.bits())
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: RegisterDevice<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWType: ReadCapability,
{
    /// Read the register from the device
    pub fn read(&mut self) -> Result<R::ReadFields, D::Error> {
        let mut register = R::ZERO;
        self.device
            .read_register::<R, SIZE_BYTES>(register.bits_mut())?;
        Ok(register.into())
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: RegisterDevice<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWType: ReadCapability + WriteCapability,
{
    /// Modify the existing register value.
    ///
    /// The register is read, the value is then passed to the closure for making changes.
    /// The result is then written back to the device.
    pub fn modify(
        &mut self,
        f: impl FnOnce(&mut R::WriteFields) -> &mut R::WriteFields,
    ) -> Result<(), D::Error> {
        let mut register = self.read()?.into().into();
        f(&mut register);
        self.device
            .write_register::<R, SIZE_BYTES>(register.into().bits())
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: RegisterDevice<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWType: ClearCapability,
{
    /// Write the reset value (or zero if the reset value is specified) to the register
    pub fn clear(&mut self) -> Result<(), D::Error> {
        self.device
            .write_register::<R, SIZE_BYTES>(R::reset_value().bits())
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: AsyncRegisterDevice<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWType: WriteCapability,
{
    /// Write to the register.
    ///
    /// The closure is given the write object initialized to the reset value of the register.
    /// If no reset value is specified for this register, this function is the same as [Self::write_with_zero].
    pub async fn write_async(
        &mut self,
        f: impl FnOnce(&mut R::WriteFields) -> &mut R::WriteFields,
    ) -> Result<(), D::Error> {
        let mut register = R::reset_value().into();
        f(&mut register);
        self.device
            .write_register::<R, SIZE_BYTES>(register.bits())
            .await
    }

    /// Write to the register.
    ///
    /// The closure is given the write object initialized to all zero.
    pub async fn write_with_zero_async(
        &mut self,
        f: impl FnOnce(&mut R::WriteFields) -> &mut R::WriteFields,
    ) -> Result<(), D::Error> {
        let mut register = R::ZERO.into();
        f(&mut register);
        self.device
            .write_register::<R, SIZE_BYTES>(register.bits())
            .await
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: AsyncRegisterDevice<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWType: ReadCapability,
{
    /// Read the register from the device
    pub async fn read_async(&mut self) -> Result<R, D::Error> {
        let mut register = R::ZERO;
        self.device
            .read_register::<R, SIZE_BYTES>(register.bits_mut())
            .await?;
        Ok(register)
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: AsyncRegisterDevice<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWType: ReadCapability + WriteCapability,
{
    /// Modify the existing register value.
    ///
    /// The register is read, the value is then passed to the closure for making changes.
    /// The result is then written back to the device.
    pub async fn modify_async(&mut self, f: impl FnOnce(&mut R) -> &mut R) -> Result<(), D::Error> {
        let mut register = self.read_async().await?;
        f(&mut register);
        self.device
            .write_register::<R, SIZE_BYTES>(register.bits())
            .await
    }
}

impl<'a, D, R, const SIZE_BYTES: usize> RegisterOperation<'a, D, R, SIZE_BYTES>
where
    D: AsyncRegisterDevice<AddressType = R::AddressType>,
    R: Register<SIZE_BYTES>,
    R::RWType: ClearCapability,
{
    /// Write the reset value (or zero if the reset value is specified) to the register
    pub async fn clear_async(&mut self) -> Result<(), D::Error> {
        self.device
            .write_register::<R, SIZE_BYTES>(R::reset_value().bits())
            .await
    }
}

#[doc(hidden)]
pub fn read_field<
    RD,
    R,
    DATA,
    BACKING,
    const START: usize,
    const END: usize,
    const SIZE_BYTES: usize,
>(
    register: &R,
) -> Result<DATA, <BACKING as TryInto<DATA>>::Error>
where
    RD: Deref<Target = R>,
    R: Register<SIZE_BYTES>,
    DATA: TryFrom<BACKING> + Into<BACKING>,
    BACKING: Integral,
{
    register.bits()[START..END].load_be::<BACKING>().try_into()
}

#[doc(hidden)]
pub fn read_field_no_convert<
    RD,
    R,
    BACKING,
    const START: usize,
    const END: usize,
    const SIZE_BYTES: usize,
>(
    register: &R,
) -> BACKING
where
    RD: Deref<Target = R>,
    R: Register<SIZE_BYTES>,
    BACKING: Integral,
{
    register.bits()[START..END].load_be::<BACKING>()
}

#[doc(hidden)]
pub fn read_field_bool<RD, R, DATA, const START: usize, const SIZE_BYTES: usize>(
    register: &R,
) -> Result<DATA, <bool as TryInto<DATA>>::Error>
where
    RD: Deref<Target = R>,
    R: Register<SIZE_BYTES>,
    DATA: TryFrom<bool> + Into<bool>,
{
    register.bits()[START].try_into()
}

#[doc(hidden)]
pub fn read_field_bool_no_convert<RD, R, const START: usize, const SIZE_BYTES: usize>(
    register: &R,
) -> bool
where
    RD: Deref<Target = R>,
    R: Register<SIZE_BYTES>,
{
    register.bits()[START]
}

#[doc(hidden)]
pub fn write_field<
    RD,
    R,
    DATA,
    BACKING,
    const START: usize,
    const END: usize,
    const SIZE_BYTES: usize,
>(
    register: &mut RD,
    data: DATA,
) -> &mut RD
where
    RD: DerefMut<Target = R>,
    R: Register<SIZE_BYTES>,
    DATA: TryFrom<BACKING> + Into<BACKING>,
    BACKING: Integral,
{
    register.bits_mut()[START..END].store_be(data.into());
    register
}

#[doc(hidden)]
pub fn write_field_no_convert<
    RD,
    R,
    BACKING,
    const START: usize,
    const END: usize,
    const SIZE_BYTES: usize,
>(
    register: &mut RD,
    data: BACKING,
) -> &mut RD
where
    RD: DerefMut<Target = R>,
    R: Register<SIZE_BYTES>,
    BACKING: Integral,
{
    register.bits_mut()[START..END].store_be(data);
    register
}

#[doc(hidden)]
pub fn write_field_bool<RD, R, DATA, const START: usize, const SIZE_BYTES: usize>(
    register: &mut RD,
    data: DATA,
) -> &mut RD
where
    RD: DerefMut<Target = R>,
    R: Register<SIZE_BYTES>,
    DATA: TryFrom<bool> + Into<bool>,
{
    register.bits_mut().set(START, data.into());
    register
}

#[doc(hidden)]
pub fn write_field_bool_no_convert<RD, R, const START: usize, const SIZE_BYTES: usize>(
    register: &mut RD,
    data: bool,
) -> &mut RD
where
    RD: DerefMut<Target = R>,
    R: Register<SIZE_BYTES>,
{
    register.bits_mut().set(START, data);
    register
}

#[doc(hidden)]
pub struct WriteOnly;
#[doc(hidden)]
pub struct ReadOnly;
#[doc(hidden)]
pub struct ReadWrite;
#[doc(hidden)]
pub struct ReadClear;
#[doc(hidden)]
pub struct ClearOnly;

#[doc(hidden)]
pub trait ReadCapability {}
#[doc(hidden)]
pub trait WriteCapability {}
#[doc(hidden)]
pub trait ClearCapability {}

impl WriteCapability for WriteOnly {}
impl ClearCapability for WriteOnly {}

impl ReadCapability for ReadOnly {}

impl WriteCapability for ReadWrite {}
impl ReadCapability for ReadWrite {}
impl ClearCapability for ReadWrite {}

impl ReadCapability for ReadClear {}
impl ClearCapability for ReadClear {}

impl ClearCapability for ClearOnly {}
